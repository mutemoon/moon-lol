//! Rig Agent 决策环：纯编排逻辑，桌面端与云端共用。
//!
//! 与具体存储 / 副作用解耦：
//! - 凭证由 [`CredentialResolver`](crate::resolver::CredentialResolver) 解析。
//! - 副作用（事件推送、历史写盘、停进程）由 [`OrchestratorSink`](crate::sink::OrchestratorSink) 承接。
//!
//! 流程：连接 Bevy WS（调用方传入 `GameClient`）→ `serve_inprocess` 注入 rmcp 工具层
//! （observe + action）→ rig agent 经 `.rmcp_tools` 注入 → 循环：暂停 → 观测 → 思考 → 恢复。

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use lol_client::{GameClient, serve_inprocess};
use rig::agent::Agent;
use rig::client::CompletionClient;
use rig::completion::{Chat, Message};
use rig::providers::anthropic;
use rig::providers::anthropic::completion::CompletionModel;
use serde_json::Value;
use tokio::time::sleep;
use tracing::{info, warn};

use crate::credentials::{AgentConfig, PlatformEnv};
use crate::resolver::CredentialResolver;
use crate::sink::{AgentRunResult, OrchestratorSink};

/// 暂停阈值（游戏时间 < 此值时不启动 AI 决策，等待 warmup）。
const WARMUP_DURATION: f64 = 40.0;
/// 对局终结时间。
const GAME_END_TIME: f64 = 120.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AgentState {
    // 游戏暂停中，AI 思考下一步行动
    Thinking,
    // 游戏继续执行，执行 N 帧后再次暂停
    Playing,
    // 游戏时间 < WARMUP_DURATION，未启动 AI 决策
    Warmup,
    // 游戏时间 >= GAME_END_TIME，终结并展示成绩
    Finished,
}

struct Observation {
    raw_data: Value,
    time: f64,
}

struct Orchestrator {
    hero_entity_ids: HashMap<String, u64>,
    rig_agents: Vec<(AgentConfig, Agent<CompletionModel>, Vec<Message>)>,
    cycle_count: u64,
    state: AgentState,
    last_game_time: f64,
}

impl Orchestrator {
    async fn new(
        client: &GameClient,
        agents: Vec<AgentConfig>,
        resolver: &Arc<dyn CredentialResolver>,
        env: &PlatformEnv,
    ) -> Option<Self> {
        if agents.is_empty() {
            info!("[Orchestrator] 无场景 agent 配置，跳过 AI Agent 决策环");
            return None;
        }

        let hero_entity_ids = wait_and_map_hero_entity_ids(client, agents.len()).await;
        info!(
            "[Orchestrator] 已映射英雄实体 ID 字典: {:?}",
            hero_entity_ids
        );

        // 进程内 rmcp 工具层（observe + action），所有 rig agent 共享同一对。
        let (tools, peer) = match serve_inprocess(client.clone()).await {
            Ok(pair) => pair,
            Err(e) => {
                warn!("[Orchestrator] 启动进程内 rmcp 工具层失败: {}", e);
                return None;
            }
        };

        let mut rig_agents = Vec::new();
        for agent_cfg in &agents {
            let creds = match resolver.resolve(agent_cfg, env).await {
                Some(c) => c,
                None => {
                    warn!(
                        "[Orchestrator] Agent [{}] 无 LLM 凭证（未配供应商且平台 env 为空），跳过",
                        agent_cfg.id
                    );
                    continue;
                }
            };
            let rig_agent = create_agent(
                creds.api_key,
                creds.base_url,
                creds.model,
                agent_cfg.prompt.clone(),
                tools.clone(),
                peer.clone(),
            );
            rig_agents.push((agent_cfg.clone(), rig_agent, Vec::<Message>::new()));
        }

        if rig_agents.is_empty() {
            info!("[Orchestrator] 所有 agent 均无 LLM 凭证，跳过 AI 决策环");
            return None;
        }

        Some(Self {
            hero_entity_ids,
            rig_agents,
            cycle_count: 0,
            state: AgentState::Warmup,
            last_game_time: 0.0,
        })
    }

    async fn step(&mut self, client: &GameClient, sink: &Arc<dyn OrchestratorSink>) -> bool {
        if !sink.is_running().await {
            info!("[Orchestrator] 游戏进程已销毁或连接会话已被清空，退出 AI 决策生命周期控制环。");
            return false;
        }

        let obs = match get_observation(client).await {
            Ok(obs) => obs,
            Err(e) => {
                warn!("[Orchestrator] 获取观测数据异常: {}, 500ms 后重试...", e);
                sleep(Duration::from_millis(500)).await;
                return true;
            }
        };

        let time = obs.time;
        self.last_game_time = time;
        info!(
            "[Orchestrator] 游戏内实时时间: {:.2}s, 当前状态: {:?}",
            time, self.state
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
                if let Err(e) = self.handle_thinking(client, sink).await {
                    warn!("[Orchestrator] Thinking 处理异常: {}", e);
                }
                self.state = AgentState::Playing;
            }
            AgentState::Playing => {
                // 恢复运行 1 秒钟
                sleep(Duration::from_secs(1)).await;
                if time >= GAME_END_TIME {
                    self.state = AgentState::Finished;
                } else {
                    self.state = AgentState::Thinking;
                }
            }
            AgentState::Finished => {
                self.handle_finished(sink, obs.raw_data).await;
                return false;
            }
        }

        true
    }

    async fn handle_thinking(
        &mut self,
        client: &GameClient,
        sink: &Arc<dyn OrchestratorSink>,
    ) -> Result<(), String> {
        info!(
            "[Orchestrator] 游戏时间 >= {}s，暂停游戏准备 AI 观测与行动...",
            WARMUP_DURATION
        );
        client.pause().await?;

        self.cycle_count += 1;
        info!(
            "[Orchestrator] 触发第 {} 次 AI 思考决策环...",
            self.cycle_count
        );

        for (agent_cfg, rig_agent, history) in &mut self.rig_agents {
            let entity_id = self
                .hero_entity_ids
                .get(&agent_cfg.id)
                .copied()
                .ok_or_else(|| format!("未找到代理 {} 的 entity_id", agent_cfg.id))?;

            info!(
                "[Orchestrator] AI Agent [Champion: {}, Team: {}, ID: {}] 正在思考决策...",
                agent_cfg.champion, agent_cfg.team, entity_id
            );
            let prompt = format!(
                "开始第 {} 轮决策，你的英雄实体 ID 为 {}。使用 observe 工具观测局势，使用 action 工具下达动作。",
                self.cycle_count, entity_id
            );
            match rig_agent.chat(&prompt, history).await {
                Ok(reply) => {
                    info!(
                        "[Orchestrator] Agent [{}, {}] 决策回复:\n{}",
                        agent_cfg.champion, agent_cfg.team, reply
                    );
                }
                Err(e) => {
                    warn!(
                        "[Orchestrator] Agent [{}, {}] 决策执行失败: {}",
                        agent_cfg.champion, agent_cfg.team, e
                    );
                }
            }

            sink.on_agent_thought(agent_cfg, history, self.cycle_count)
                .await;
        }

        info!("[Orchestrator] AI 决策执行完毕，继续运行游戏 1s 后再次决策...");
        client.unpause().await?;
        Ok(())
    }

    async fn handle_finished(&self, sink: &Arc<dyn OrchestratorSink>, raw_data: Value) {
        info!(
            "[Orchestrator] 游戏时间已达 {:.0}s，终结 AI 决策环",
            GAME_END_TIME
        );

        let results: Vec<AgentRunResult> = self
            .rig_agents
            .iter()
            .map(|(cfg, _, history)| AgentRunResult {
                agent: cfg.clone(),
                history: history.clone(),
            })
            .collect();

        sink.on_finished(&raw_data, self.last_game_time, &results)
            .await;
    }
}

fn create_agent(
    api_key: String,
    base_url: String,
    model: String,
    preamble: String,
    tools: Vec<rmcp::model::Tool>,
    peer: rmcp::service::ServerSink,
) -> Agent<CompletionModel> {
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

async fn get_observation(client: &GameClient) -> Result<Observation, String> {
    let resp = client
        .session()
        .send_cmd("get_observe".to_string(), Value::Null)
        .await
        .map_err(|e| e.to_string())?;
    if !resp.ok {
        return Err("获取观测失败：返回结果为 false".into());
    }
    let raw_data = resp
        .data
        .ok_or_else(|| "获取观测失败：未包含数据".to_string())?;
    let time = raw_data
        .get("time")
        .and_then(|t| t.as_f64())
        .ok_or_else(|| "获取观测失败：时间字段缺失".to_string())?;
    Ok(Observation { raw_data, time })
}

/// 启动 AI Agent 决策环。
///
/// `client` 已连接目标 Bevy 进程的 WS。无 LLM 凭据或无 agent 配置时静默返回（不报错）。
/// `resolver` 按每个 agent 的 `provider_id` 解析凭证；`env` 为平台网关回退；
/// `sink` 承接事件推送 / 历史写盘 / 停进程等副作用。
pub async fn run_orchestrator(
    client: GameClient,
    agents: Vec<AgentConfig>,
    resolver: Arc<dyn CredentialResolver>,
    env: PlatformEnv,
    sink: Arc<dyn OrchestratorSink>,
) {
    info!("[Orchestrator] 启动 AI Agent 后台生命周期循环");

    let Some(mut orchestrator) = Orchestrator::new(&client, agents, &resolver, &env).await else {
        return;
    };

    while orchestrator.step(&client, &sink).await {}

    info!("[Orchestrator] 决策环退出");
}
