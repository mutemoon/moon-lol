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
        let provider = agent
            .provider_id
            .as_deref()
            .and_then(|pid| self.providers.iter().find(|p| p.id == pid));
        let creds = provider.map(|p| {
            let max_tokens = agent.model.as_deref().and_then(|m| {
                p.models.iter().find(|model| model.name == m).map(|model| model.max_tokens)
            });
            ProviderCredentials {
                api_key: p.api_key.clone(),
                base_url: p.base_url.clone(),
                api_format: p.api_format.clone(),
                max_tokens,
            }
        });
        resolve_credentials(agent, creds, env)
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
        println!("[Agent Orchestrator] 游戏时间已达 2 分钟，正在终结并进行成绩统计...");

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
            println!("[Agent Orchestrator] 启动 AI Agent 后台生命周期循环 port={port}");

            let ws = match start_ws_client(port as u16, None).await {
                Ok(ws) => ws,
                Err(e) => {
                    println!("[Agent Orchestrator] 连接 Bevy WS 失败: {e}");
                    return;
                }
            };

            let env = PlatformEnv::from_env();
            let providers = if let Some(state) = app.try_state::<crate::state::AppState>() {
                state.model_providers.lock().unwrap().clone()
            } else {
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

            run_orchestrator(client, agents, resolver, env, sink).await;
        }) as Pin<Box<dyn std::future::Future<Output = ()> + Send>>
    })
}
