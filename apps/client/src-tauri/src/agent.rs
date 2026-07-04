use std::pin::Pin;
use std::sync::{Arc, Weak};

use async_trait::async_trait;
use chrono::Local;
use lol_agent_runtime::{
    resolve_credentials, run_orchestrator, AgentConfig, AgentRunResult, CredentialResolver,
    OrchestratorSink, PlatformEnv, ProviderCredentials, ResolvedCredentials,
};
use lol_client::{start_ws_client, GameClient};
use lol_game_process_manager::{AgentRunner, GameProcessManager};
use rig::completion::Message;
use serde_json::Value;
use tauri::{Emitter, Manager};
use tracing::{debug, error, info, warn};

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct SavedAgentHistory {
    pub agent_id: String,
    pub champion: String,
    pub team: String,
    pub prompt: String,
    pub system_prompt: String,
    pub history: Vec<Message>,
    pub game_duration: f64,
    pub datetime: String,
}

#[derive(serde::Serialize, Clone)]
struct AgentHistoryPayload {
    agent_id: String,
    champion: String,
    history: Vec<Message>,
}

#[derive(serde::Serialize, Clone)]
struct AgentFinishedPayload {
    minion_kills: u64,
    gold: f64,
}

/// 桌面端凭证解析：按 `provider_id` 在 `~/.moon-lol/providers.json` 里查供应商。
pub struct DesktopCredentialResolver {
    providers: Vec<crate::ModelProvider>,
}

#[async_trait]
impl CredentialResolver for DesktopCredentialResolver {
    async fn resolve(&self, agent: &AgentConfig, env: &PlatformEnv) -> Option<ResolvedCredentials> {
        info!(
            "开始解析凭证: agent_id={}, champion={}, provider_id={:?}, model={:?}",
            agent.id, agent.champion, agent.provider_id, agent.model
        );
        let provider = agent.provider_id.as_deref().and_then(|pid| {
            let found = self.providers.iter().find(|p| p.id == pid);
            if found.is_none() {
                warn!("未能找到 provider_id 为 {:?} 的 model provider", pid);
            }
            found
        });
        let creds = provider.map(|p| {
            let max_tokens = agent.model.as_deref().and_then(|m| {
                p.models
                    .iter()
                    .find(|model| model.name == m)
                    .map(|model| model.max_tokens)
            });
            debug!(
                "匹配到 provider ({}): base_url={:?}, api_format={:?}, max_tokens={:?}",
                p.name, p.base_url, p.api_format, max_tokens
            );
            ProviderCredentials {
                api_key: p.api_key.clone(),
                base_url: p.base_url.clone(),
                api_format: p.api_format.clone(),
                max_tokens,
            }
        });
        let res = resolve_credentials(agent, creds, env);
        if let Some(ref r) = res {
            info!(
                "凭证解析成功 (agent_id={}): base_url={}, has_api_key={}",
                agent.id,
                r.base_url,
                !r.api_key.is_empty()
            );
        } else {
            warn!("凭证解析未成功 (agent_id={})", agent.id);
        }
        res
    }
}

/// 桌面端副作用出口：emit 对话历史 / 终结事件、写盘历史、停游戏进程（按 port 委托 manager）。
pub struct DesktopSink {
    app: tauri::AppHandle,
    port: i32,
    weak: Weak<GameProcessManager>,
    start_time_str: String,
    system_prompt: String,
}

#[async_trait]
impl OrchestratorSink for DesktopSink {
    async fn on_agent_thought(&self, agent: &AgentConfig, history: &[Message], _cycle: u64) {
        let _ = self.app.emit(
            "agent-history-updated",
            AgentHistoryPayload {
                agent_id: agent.id.clone(),
                champion: agent.champion.clone(),
                history: history.to_vec(),
            },
        );
    }

    async fn on_finished(
        &self,
        final_observation: &Value,
        last_game_time: f64,
        results: &[AgentRunResult],
    ) {
        info!("游戏时间已达 2 分钟，正在终结并进行成绩统计...");

        let minion_kills = final_observation
            .get("myself")
            .and_then(|m| m.get("minion_kills"))
            .and_then(|k| k.as_u64())
            .unwrap_or(0);
        let gold = final_observation
            .get("myself")
            .and_then(|m| m.get("gold"))
            .and_then(|g| g.as_f64())
            .unwrap_or(0.0);

        // 广播统计数据至前端展示
        let _ = self.app.emit(
            "agent-finished",
            AgentFinishedPayload { minion_kills, gold },
        );

        let histories: Vec<SavedAgentHistory> = results
            .iter()
            .map(|r| SavedAgentHistory {
                agent_id: r.agent.id.clone(),
                champion: r.agent.champion.clone(),
                team: r.agent.team.clone(),
                prompt: r.agent.prompt.clone(),
                system_prompt: self.system_prompt.clone(),
                history: r.history.clone(),
                game_duration: last_game_time,
                datetime: self.start_time_str.clone(),
            })
            .collect();

        let _ = self.app.emit("match-history-ready", histories);

        // 终结并关闭游戏（按 port 委托 manager）
        if let Some(m) = self.weak.upgrade() {
            let _ = m.stop_by_port(self.port).await;
        }
    }

    async fn is_running(&self) -> bool {
        let Some(m) = self.weak.upgrade() else {
            return false;
        };
        m.find_by_port(self.port).await.is_ok()
    }
}

/// 构造桌面端 AI 决策环启动器（注入 GameProcessManager）。
///
/// runner 闭包在 start 时由 manager 调用：自建一条独立 WS（不转发事件，避免与调试会话重复），
/// 解析本地 providers.json 凭证，跑 `run_orchestrator`；sink 通过 weak manager 引用在终结时停进程。
pub fn make_desktop_runner(app: tauri::AppHandle, weak: Weak<GameProcessManager>) -> AgentRunner {
    Arc::new(move |port: i32, agents: Vec<AgentConfig>| {
        let app = app.clone();
        let weak = weak.clone();
        Box::pin(async move {
            info!("启动 AI Agent 后台生命周期循环 port={port}");
            info!(
                "参与游戏编排的 Agents: {:?}",
                agents.iter().map(|a| &a.champion).collect::<Vec<_>>()
            );

            let ws = match start_ws_client(port as u16, None).await {
                Ok(ws) => {
                    info!("成功连接 Bevy WS (port={port})");
                    ws
                }
                Err(e) => {
                    error!("连接 Bevy WS 失败: {e}");
                    return;
                }
            };

            let env = PlatformEnv::from_env();
            let providers = if let Some(state) = app.try_state::<crate::state::AppState>() {
                let providers = state.model_providers.lock().unwrap().clone();
                info!(
                    "成功从 AppState 获取 model providers, 数量: {}",
                    providers.len()
                );
                providers
            } else {
                warn!("警告: 未能获取 AppState, model providers 列表为空");
                Vec::new()
            };
            let resolver = Arc::new(DesktopCredentialResolver { providers });
            let start_time_str = Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
            let sink = Arc::new(DesktopSink {
                app: app.clone(),
                port,
                weak,
                start_time_str,
                system_prompt: String::new(),
            });
            let client = GameClient::new(ws);

            info!("启动 Bevy 游戏客户端，开始运行决策环 run_orchestrator...");
            run_orchestrator(client, agents, resolver, env, sink).await;
            info!("决策环 run_orchestrator 已结束");
        }) as Pin<Box<dyn std::future::Future<Output = ()> + Send>>
    })
}
