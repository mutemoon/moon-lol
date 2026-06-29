//! Rig Agent 决策环：web server 侧对运行中本地对局的 AI 编排。
//!
//! 与 Tauri 后端 `agent.rs` 共享同一套进程内 rmcp 工具层（`lol_client::serve_inprocess`）：
//! - 连接 Bevy 子进程的 WS（经 `lol_client`），建 `GameClient`。
//! - `serve_inprocess` 用内存 duplex 起 rmcp client/server 对，仅暴露 observe / action。
//! - rig agent 通过 `.rmcp_tools(tools, peer)` 注入工具，循环：暂停 → 观测 → 思考 → 恢复。
//!
//! LLM 凭据来自环境变量（ANTHROPIC_API_KEY / ANTHROPIC_BASE_URL / ANTHROPIC_MODEL /
//! ANTHROPIC_PREAMBLE），与 Tauri 编排环保持一致。

use std::collections::HashMap;
use std::env;
use std::time::Duration;

use lol_client::{GameClient, serve_inprocess};
use rig::client::CompletionClient;
use rig::completion::{Chat, Message};
use rig::providers::anthropic;
use rig::providers::anthropic::completion::CompletionModel;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::time::sleep;
use tracing::{info, warn};

/// 暂停阈值（游戏时间 < 此值时不启动 AI 决策，等待 warmup）。
const WARMUP_DURATION: f64 = 40.0;
/// 对局终结时间。
const GAME_END_TIME: f64 = 120.0;

/// 场景中的单个 agent 定义（前端契约结构，对齐 Tauri `AgentConfig`）。
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SceneAgentConfig {
    pub id: String,
    pub champion: String,
    pub team: String,
    pub prompt: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AgentState {
    Thinking,
    Playing,
    Warmup,
    Finished,
}

struct Orchestrator {
    ws_port: i32,
    hero_entity_ids: HashMap<String, u64>,
    rig_agents: Vec<(
        SceneAgentConfig,
        rig::agent::Agent<CompletionModel>,
        Vec<Message>,
    )>,
    cycle_count: u64,
    state: AgentState,
}

impl Orchestrator {
    async fn new(ws_port: i32, agents: Vec<SceneAgentConfig>, client: GameClient) -> Option<Self> {
        let api_key = env::var("ANTHROPIC_API_KEY").unwrap_or_default();
        let base_url = env::var("ANTHROPIC_BASE_URL").unwrap_or_default();
        let model = env::var("ANTHROPIC_MODEL").unwrap_or_else(|_| "deepseek-v4-flash".into());
        let preamble = env::var("ANTHROPIC_PREAMBLE").unwrap_or_default();

        if api_key.is_empty() {
            info!("[Web Orchestrator] 未检测到 ANTHROPIC_API_KEY，跳过 AI Agent 决策环");
            return None;
        }
        if agents.is_empty() {
            info!("[Web Orchestrator] 无场景 agent 配置，跳过 AI Agent 决策环");
            return None;
        }

        let hero_entity_ids = wait_and_map_hero_entity_ids(&client, agents.len()).await;
        info!(
            "[Web Orchestrator] 已映射英雄实体 ID 字典: {:?}",
            hero_entity_ids
        );

        // 进程内 rmcp 工具层（observe + action），所有 rig agent 共享同一对。
        let (tools, peer) = match serve_inprocess(client.clone()).await {
            Ok(pair) => pair,
            Err(e) => {
                warn!("[Web Orchestrator] 启动进程内 rmcp 工具层失败: {}", e);
                return None;
            }
        };

        let mut rig_agents = Vec::new();
        for agent_cfg in &agents {
            let combined_prompt = if preamble.is_empty() {
                agent_cfg.prompt.clone()
            } else {
                format!("{}\n{}", preamble, agent_cfg.prompt)
            };
            let rig_agent = create_agent(
                api_key.clone(),
                base_url.clone(),
                model.clone(),
                combined_prompt,
                tools.clone(),
                peer.clone(),
            );
            rig_agents.push((agent_cfg.clone(), rig_agent, Vec::<Message>::new()));
        }

        Some(Self {
            ws_port,
            hero_entity_ids,
            rig_agents,
            cycle_count: 0,
            state: AgentState::Warmup,
        })
    }

    async fn step(&mut self, client: &GameClient) -> bool {
        let time = match get_observation_time(client).await {
            Ok(t) => t,
            Err(e) => {
                warn!(
                    "[Web Orchestrator] ws={} 获取观测数据异常: {}, 500ms 后重试...",
                    self.ws_port, e
                );
                sleep(Duration::from_millis(500)).await;
                return true;
            }
        };

        info!(
            "[Web Orchestrator] ws={} 游戏内实时时间: {:.2}s, 当前状态: {:?}",
            self.ws_port, time, self.state
        );

        match self.state {
            AgentState::Warmup => {
                if time >= GAME_END_TIME {
                    self.state = AgentState::Finished;
                } else if time >= WARMUP_DURATION {
                    self.state = AgentState::Thinking;
                } else {
                    sleep(Duration::from_millis(500)).await;
                }
            }
            AgentState::Thinking => {
                if let Err(e) = self.handle_thinking(client).await {
                    warn!("[Web Orchestrator] Thinking 处理异常: {}", e);
                }
                self.state = AgentState::Playing;
            }
            AgentState::Playing => {
                sleep(Duration::from_secs(1)).await;
                if time >= GAME_END_TIME {
                    self.state = AgentState::Finished;
                } else {
                    self.state = AgentState::Thinking;
                }
            }
            AgentState::Finished => {
                info!(
                    "[Web Orchestrator] ws={} 游戏时间已达 {:.0}s，AI 决策环结束",
                    self.ws_port, GAME_END_TIME
                );
                return false;
            }
        }

        true
    }

    async fn handle_thinking(&mut self, client: &GameClient) -> Result<(), String> {
        info!(
            "[Web Orchestrator] ws={} 游戏时间 >= {}s，暂停游戏准备 AI 观测与行动...",
            self.ws_port, WARMUP_DURATION
        );
        client.pause().await?;

        self.cycle_count += 1;
        info!(
            "[Web Orchestrator] ws={} 触发第 {} 次 AI 思考决策环...",
            self.ws_port, self.cycle_count
        );

        for (agent_cfg, rig_agent, history) in &mut self.rig_agents {
            let entity_id = self
                .hero_entity_ids
                .get(&agent_cfg.id)
                .copied()
                .ok_or_else(|| format!("未找到代理 {} 的 entity_id", agent_cfg.id))?;

            info!(
                "[Web Orchestrator] AI Agent [Champion: {}, Team: {}, ID: {}] 正在思考决策...",
                agent_cfg.champion, agent_cfg.team, entity_id
            );
            let prompt = format!(
                "开始第 {} 轮决策，你的英雄实体 ID 为 {}。使用 observe 工具观测局势，使用 action 工具下达动作。",
                self.cycle_count, entity_id
            );
            match rig_agent.chat(&prompt, history).await {
                Ok(reply) => {
                    info!(
                        "[Web Orchestrator] Agent [{}, {}] 决策回复:\n{}",
                        agent_cfg.champion, agent_cfg.team, reply
                    );
                }
                Err(e) => {
                    warn!(
                        "[Web Orchestrator] Agent [{}, {}] 决策执行失败: {}",
                        agent_cfg.champion, agent_cfg.team, e
                    );
                }
            }
        }

        info!("[Web Orchestrator] AI 决策执行完毕，继续运行游戏 1s 后再次决策...");
        client.unpause().await?;
        Ok(())
    }
}

fn create_agent(
    api_key: String,
    base_url: String,
    model: String,
    preamble: String,
    tools: Vec<rmcp::model::Tool>,
    peer: rmcp::service::ServerSink,
) -> rig::agent::Agent<CompletionModel> {
    let client = anthropic::Client::builder()
        .api_key(&api_key)
        .base_url(&base_url)
        .build()
        .expect("初始化 rig Anthropic 客户端失败");

    client
        .agent(&model)
        .max_tokens(200 * 1000)
        .default_max_turns(20)
        .preamble(&preamble)
        .rmcp_tools(tools, peer)
        .build()
}

async fn wait_and_map_hero_entity_ids(
    client: &GameClient,
    expected: usize,
) -> HashMap<String, u64> {
    let mut ids = HashMap::new();
    if expected == 0 {
        return ids;
    }

    for _ in 0..20 {
        let Ok(resp) = client.agents().await else {
            sleep(Duration::from_millis(500)).await;
            continue;
        };
        let Some(data) = resp.data else {
            sleep(Duration::from_millis(500)).await;
            continue;
        };
        let Some(arr) = data.as_array() else {
            sleep(Duration::from_millis(500)).await;
            continue;
        };

        for item in arr {
            let id = item.get("entity_id").and_then(|v| v.as_u64()).unwrap_or(0);
            let agent_id = item.get("agent_id").and_then(|v| v.as_str()).unwrap_or("");
            if id > 0 && !agent_id.is_empty() {
                ids.insert(agent_id.to_string(), id);
            }
        }

        if ids.len() >= expected {
            break;
        }
        sleep(Duration::from_millis(500)).await;
    }

    ids
}

async fn get_observation_time(client: &GameClient) -> Result<f64, String> {
    let resp = client
        .session()
        .send_cmd("get_observe".to_string(), Value::Null)
        .await
        .map_err(|e| e.to_string())?;
    if !resp.ok {
        return Err("获取观测失败：返回结果为 false".into());
    }
    let data = resp
        .data
        .ok_or_else(|| "获取观测失败：未包含数据".to_string())?;
    data.get("time")
        .and_then(|t| t.as_f64())
        .ok_or_else(|| "获取观测失败：时间字段缺失".into())
}

/// 启动 AI Agent 决策环：连接 ws_port 的 Bevy 进程，注入 rmcp tools 并循环决策。
///
/// 无 LLM 凭据或无 agent 配置时静默返回（不报错），与 Tauri 行为一致。
/// `agents` 通常来自场景定义（`Scenario::agents` JSON）。
pub async fn run_agent_orchestrator(ws_port: i32, agents: Vec<SceneAgentConfig>) {
    info!(
        "[Web Orchestrator] 启动 AI Agent 后台生命周期循环 (ws={})",
        ws_port
    );

    let session = match lol_client::start_ws_client(ws_port as u16, None).await {
        Ok(s) => s,
        Err(e) => {
            warn!("[Web Orchestrator] ws={} 连接 Bevy WS 失败: {}", ws_port, e);
            return;
        }
    };
    let client = GameClient::new(session);

    let Some(mut orchestrator) = Orchestrator::new(ws_port, agents, client.clone()).await else {
        return;
    };

    while orchestrator.step(&client).await {}
    info!("[Web Orchestrator] ws={} 决策环退出", orchestrator.ws_port);
}
