use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;
use std::{env, fs};

use chrono::Local;
use rig::agent::Agent;
use rig::client::CompletionClient;
use rig::completion::{Chat, Message};
use rig::providers::anthropic;
use rig::providers::anthropic::completion::CompletionModel;
use serde_json::Value;
use tauri::{Emitter, Manager};
use tokio::time::sleep;

use crate::state::AppState;
use crate::tools::BashTool;
use crate::ws::WsSession;

static WARMUP_DURATION: f64 = 40.0;
static GAME_END_TIME: f64 = 120.0;

#[derive(serde::Deserialize, Clone, Debug)]
pub struct AgentConfig {
    pub id: String,
    pub champion: String,
    pub team: String,
    pub prompt: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentState {
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

async fn wait_and_map_hero_entity_ids(
    ws: &WsSession,
    expected_count: usize,
) -> HashMap<String, u64> {
    let mut hero_entity_ids = HashMap::new();
    if expected_count == 0 {
        return hero_entity_ids;
    }

    for _ in 0..20 {
        let Ok(resp) = ws.send_cmd("get_agents".to_string(), Value::Null).await else {
            sleep(Duration::from_millis(500)).await;
            continue;
        };

        if !resp.ok {
            sleep(Duration::from_millis(500)).await;
            continue;
        }

        let Some(ref data) = resp.data else {
            sleep(Duration::from_millis(500)).await;
            continue;
        };

        let Some(arr) = data.as_array() else {
            sleep(Duration::from_millis(500)).await;
            continue;
        };

        if arr.is_empty() {
            sleep(Duration::from_millis(500)).await;
            continue;
        }

        for item in arr {
            let id = item
                .get("entity_id")
                .and_then(|id| id.as_u64())
                .unwrap_or(0);
            let agent_id = item.get("agent_id").and_then(|a| a.as_str()).unwrap_or("");
            if id > 0 && !agent_id.is_empty() {
                hero_entity_ids.insert(agent_id.to_string(), id);
            }
        }

        if hero_entity_ids.len() >= expected_count {
            break;
        }

        sleep(Duration::from_millis(500)).await;
    }

    hero_entity_ids
}

pub fn create_agent(
    api_key: String,
    base_url: String,
    model: String,
    preamble: String,
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
        .tool(BashTool)
        .build()
}

pub async fn on_thinking(
    agent: &Agent<CompletionModel>,
    local_chat_history: &mut Vec<Message>,
    cycle: u64,
    entity_id: u64,
) -> Result<String, String> {
    let prompt = format!(
        "开始第 {} 轮决策，使用 -e {} 来指定当前代理的 ID。",
        cycle, entity_id
    );
    let response = agent
        .chat(&prompt, local_chat_history)
        .await
        .map_err(|e| e.to_string())?;
    Ok(response)
}

struct Orchestrator {
    app: tauri::AppHandle,
    ws: WsSession,
    hero_entity_ids: HashMap<String, u64>,
    rig_agents: Vec<(AgentConfig, Agent<CompletionModel>, Vec<Message>)>,
    cycle_count: u64,
    state: AgentState,
    last_game_time: f64,
    start_time_str: String,
    system_prompt: String,
}

impl Orchestrator {
    async fn new(app: tauri::AppHandle, ws: WsSession) -> Option<Self> {
        let api_key = env::var("ANTHROPIC_API_KEY").unwrap_or_default();
        let base_url = env::var("ANTHROPIC_BASE_URL").unwrap_or_default();
        let model = env::var("ANTHROPIC_MODEL").unwrap_or_else(|_| "deepseek-v4-flash".to_string());
        let preamble = env::var("ANTHROPIC_PREAMBLE").unwrap_or_default();

        if api_key.is_empty() {
            println!("[Agent Orchestrator] 未检测到 ANTHROPIC_API_KEY，无法开启 AI Agent 决策环");
            return None;
        }

        // 检测当前是否有自定义 of AI 代理场景配置
        let active_scene = app
            .try_state::<Mutex<AppState>>()
            .and_then(|m| m.lock().ok().and_then(|s| s.active_scene.clone()));

        let mut custom_agents: Vec<AgentConfig> = Vec::new();
        if let Some(ref name) = active_scene {
            let home = app.path().home_dir().unwrap_or_default();
            let json_path = home
                .join(".moon-lol")
                .join("games")
                .join(format!("{}.json", name));
            if json_path.exists() {
                if let Ok(content) = fs::read_to_string(&json_path) {
                    if let Ok(parsed) = serde_json::from_str::<Vec<AgentConfig>>(&content) {
                        custom_agents = parsed;
                        println!(
                            "[Agent Orchestrator] 成功加载自定义场景配置代理: {:?}",
                            custom_agents
                        );
                    }
                }
            }
        }

        if custom_agents.is_empty() {
            println!("[Agent Orchestrator] 未检测到自定义场景配置代理，拒绝开启 AI Agent 决策环");
            return None;
        }

        let hero_entity_ids = wait_and_map_hero_entity_ids(&ws, custom_agents.len()).await;
        println!(
            "[Agent Orchestrator] 已映射英雄实体 ID 字典: {:?}",
            hero_entity_ids
        );

        // 实例化对应的 Rig Agents
        let mut rig_agents = Vec::new();
        for agent_cfg in &custom_agents {
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
            );
            rig_agents.push((agent_cfg.clone(), rig_agent, Vec::<Message>::new()));
        }

        let start_time_str = Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
        Some(Self {
            app,
            ws,
            hero_entity_ids,
            rig_agents,
            cycle_count: 0,
            state: AgentState::Warmup,
            last_game_time: 0.0,
            start_time_str,
            system_prompt: preamble,
        })
    }

    async fn step(&mut self) -> bool {
        // Step 1: 检查 WebSocket 状态，如果连接已被主动关闭或游戏已终止，则立即退出控制环
        let is_running = self
            .app
            .try_state::<Mutex<AppState>>()
            .and_then(|m| m.lock().ok().map(|s| s.ws.is_some()))
            .unwrap_or(false);

        if !is_running {
            println!(
                "[Agent Orchestrator] 游戏进程已销毁或连接会话已被清空，退出 AI 决策生命周期控制环。"
            );
            return false;
        }

        // Step 2: 获取游戏实时观测数据
        let obs = match self.get_observation().await {
            Ok(obs) => obs,
            Err(e) => {
                println!(
                    "[Agent Orchestrator] 获取观测数据异常: {}, 500ms 后重试...",
                    e
                );
                sleep(Duration::from_millis(500)).await;
                return true;
            }
        };

        let time = obs.time;
        self.last_game_time = time;
        println!(
            "[Agent Orchestrator] 游戏内实时时间: {:.2}s, 当前状态: {:?}",
            time, self.state
        );

        // Step 3: 根据当前状态和游戏时间更新状态机
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
                if let Err(e) = self.handle_thinking().await {
                    println!("[Agent Orchestrator] Thinking 处理异常: {}", e);
                }
                self.state = AgentState::Playing;
            }
            AgentState::Playing => {
                // 恢复运行 1秒钟
                sleep(Duration::from_secs(1)).await;
                if time >= GAME_END_TIME {
                    self.state = AgentState::Finished;
                } else {
                    self.state = AgentState::Thinking;
                }
            }
            AgentState::Finished => {
                if let Err(e) = self.handle_finished(obs.raw_data).await {
                    println!("[Agent Orchestrator] Finished 处理异常: {}", e);
                }
                return false;
            }
        }

        true
    }

    async fn get_observation(&self) -> Result<Observation, String> {
        let obs_res = self
            .ws
            .send_cmd("get_observe".to_string(), Value::Null)
            .await
            .map_err(|e| e.to_string())?;

        if !obs_res.ok {
            return Err("获取观测失败：返回结果为 false".to_string());
        }

        let raw_data = obs_res
            .data
            .ok_or_else(|| "获取观测失败：未包含数据".to_string())?;
        let time = raw_data
            .get("time")
            .and_then(|t| t.as_f64())
            .ok_or_else(|| "获取观测失败：时间字段缺失".to_string())?;

        Ok(Observation { raw_data, time })
    }

    async fn handle_thinking(&mut self) -> Result<(), String> {
        println!(
            "[Agent Orchestrator] 游戏时间 >= {}s，正在暂停游戏准备 AI 观测与行动...",
            WARMUP_DURATION
        );
        self.pause_game().await?;

        self.cycle_count += 1;
        println!(
            "[Agent Orchestrator] 触发第 {} 次 AI 思考决策环...",
            self.cycle_count
        );

        for (agent_cfg, rig_agent, history) in &mut self.rig_agents {
            let entity_id = self
                .hero_entity_ids
                .get(&agent_cfg.id)
                .copied()
                .ok_or_else(|| format!("未找到代理 {} 的 entity_id", agent_cfg.id))?;

            println!(
                "[Agent Orchestrator] AI Agent [Champion: {}, Team: {}, ID: {}] 正在思考决策...",
                agent_cfg.champion, agent_cfg.team, entity_id
            );
            match on_thinking(rig_agent, history, self.cycle_count, entity_id).await {
                Ok(reply) => {
                    println!(
                        "[Agent Orchestrator] Agent [{}, {}] 决策回复内容:\n{}",
                        agent_cfg.champion, agent_cfg.team, reply
                    );
                }
                Err(e) => {
                    println!(
                        "[Agent Orchestrator] Agent [{}, {}] 决策执行失败: {}",
                        agent_cfg.champion, agent_cfg.team, e
                    );
                }
            }

            let _ = self.app.emit(
                "agent-history-updated",
                AgentHistoryPayload {
                    agent_id: agent_cfg.id.clone(),
                    champion: agent_cfg.champion.clone(),
                    history: history.clone(),
                },
            );
        }

        // 继续/恢复游戏
        println!("[Agent Orchestrator] AI 决策执行完毕，继续运行游戏 1s 后再次进行决策...");
        self.resume_game().await?;

        Ok(())
    }

    async fn handle_finished(&self, raw_data: Value) -> Result<(), String> {
        println!("[Agent Orchestrator] 游戏时间已达 2 分钟，正在终结并进行成绩统计...");

        let minion_kills = raw_data
            .get("myself")
            .and_then(|m| m.get("minion_kills"))
            .and_then(|k| k.as_u64())
            .unwrap_or(0);

        let gold = raw_data
            .get("myself")
            .and_then(|m| m.get("gold"))
            .and_then(|g| g.as_f64())
            .unwrap_or(0.0);

        #[derive(serde::Serialize, Clone)]
        struct AgentFinishedPayload {
            minion_kills: u64,
            gold: f64,
        }

        // 广播统计数据至前端展示
        let _ = self.app.emit(
            "agent-finished",
            AgentFinishedPayload { minion_kills, gold },
        );

        // 终结并关闭游戏
        if let Some(state) = self.app.try_state::<Mutex<AppState>>() {
            let _ = crate::process::stop_game(&state);
        }

        Ok(())
    }

    async fn pause_game(&self) -> Result<(), String> {
        let state_resp = self
            .ws
            .send_cmd("get_state".to_string(), Value::Null)
            .await
            .map_err(|e| e.to_string())?;

        let is_paused = state_resp
            .data
            .and_then(|sd| sd.get("paused").and_then(|p| p.as_bool()))
            .unwrap_or(false);

        if !is_paused {
            let _ = self
                .ws
                .send_cmd("toggle_pause".to_string(), Value::Null)
                .await;
        }
        Ok(())
    }

    async fn resume_game(&self) -> Result<(), String> {
        let state_resp = self
            .ws
            .send_cmd("get_state".to_string(), Value::Null)
            .await
            .map_err(|e| e.to_string())?;

        let is_paused = state_resp
            .data
            .and_then(|sd| sd.get("paused").and_then(|p| p.as_bool()))
            .unwrap_or(false);

        if is_paused {
            let _ = self
                .ws
                .send_cmd("toggle_pause".to_string(), Value::Null)
                .await;
        }
        Ok(())
    }

    fn save_all_agent_history(&self) {
        let home = self.app.path().home_dir().unwrap_or_default();
        let session_dir = home
            .join(".moon-lol")
            .join("history")
            .join(&self.start_time_str);

        if let Err(e) = fs::create_dir_all(&session_dir) {
            println!("[Agent Orchestrator] 创建历史目录失败: {:?}", e);
            return;
        }

        for (agent_cfg, _, history) in &self.rig_agents {
            let saved = SavedAgentHistory {
                agent_id: agent_cfg.id.clone(),
                champion: agent_cfg.champion.clone(),
                team: agent_cfg.team.clone(),
                prompt: agent_cfg.prompt.clone(),
                system_prompt: self.system_prompt.clone(),
                history: history.clone(),
                game_duration: self.last_game_time,
                datetime: self.start_time_str.clone(),
            };

            let json_path = session_dir.join(format!("{}.json", agent_cfg.id));
            let json_content = match serde_json::to_string_pretty(&saved) {
                Ok(content) => content,
                Err(e) => {
                    println!("[Agent Orchestrator] 序列化历史记录失败: {:?}", e);
                    continue;
                }
            };

            if let Err(e) = fs::write(&json_path, json_content) {
                println!("[Agent Orchestrator] 写入历史记录文件失败: {:?}", e);
            }
        }
        println!(
            "[Agent Orchestrator] 成功保存对局历史对话到目录: {:?}",
            session_dir
        );
    }
}

/// 后台协调器循环：在 WARMUP_DURATION 时暂停游戏，运行 Agent 决策，继续游戏 1s，周而复始，在 GAME_END_TIME 时终止并统计
pub async fn run_agent_orchestrator(app: tauri::AppHandle, ws: WsSession) {
    println!("[Agent Orchestrator] 启动 AI Agent 后台生命周期循环");

    let Some(mut orchestrator) = Orchestrator::new(app, ws).await else {
        return;
    };

    while orchestrator.step().await {}

    // 保存本局所有的 Agent 对话历史记录
    orchestrator.save_all_agent_history();
}

#[derive(serde::Serialize, Clone, Debug)]
pub struct AgentSummary {
    pub agent_id: String,
    pub champion: String,
    pub team: String,
}

#[derive(serde::Serialize, Clone, Debug)]
pub struct GameHistorySummary {
    pub datetime: String,
    pub duration: f64,
    pub agents: Vec<AgentSummary>,
}

fn read_agents_from_dir(dir_path: &PathBuf) -> Option<(Vec<AgentSummary>, f64)> {
    let files = fs::read_dir(dir_path).ok()?;
    let mut agents = Vec::new();
    let mut game_duration = 0.0;

    for sub_entry in files.flatten() {
        let sub_path = sub_entry.path();
        if !sub_path.is_file() || !sub_path.extension().is_some_and(|ext| ext == "json") {
            continue;
        }
        let content = fs::read_to_string(&sub_path).ok()?;
        let parsed: SavedAgentHistory = serde_json::from_str(&content).ok()?;
        agents.push(AgentSummary {
            agent_id: parsed.agent_id,
            champion: parsed.champion,
            team: parsed.team,
        });
        game_duration = parsed.game_duration;
    }

    if agents.is_empty() {
        return None;
    }
    Some((agents, game_duration))
}

#[tauri::command]
pub fn list_game_histories(
    app: tauri::AppHandle,
) -> Result<Vec<GameHistorySummary>, crate::error::AppError> {
    let home = app
        .path()
        .home_dir()
        .map_err(|e| crate::error::AppError::Generic(e.to_string()))?;
    let history_dir = home.join(".moon-lol").join("history");
    if !history_dir.exists() {
        return Ok(Vec::new());
    }

    let mut summaries = Vec::new();
    let entries =
        fs::read_dir(history_dir).map_err(|e| crate::error::AppError::Generic(e.to_string()))?;

    for entry in entries.flatten() {
        let path = entry.path();
        let Some(datetime_str) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };

        if !path.is_dir() {
            continue;
        }

        let Some((agents, game_duration)) = read_agents_from_dir(&path) else {
            continue;
        };

        summaries.push(GameHistorySummary {
            datetime: datetime_str.to_string(),
            duration: game_duration,
            agents,
        });
    }

    summaries.sort_by(|a, b| b.datetime.cmp(&a.datetime));
    Ok(summaries)
}

#[tauri::command]
pub fn get_game_history_detail(
    app: tauri::AppHandle,
    datetime: String,
) -> Result<Vec<SavedAgentHistory>, crate::error::AppError> {
    let home = app
        .path()
        .home_dir()
        .map_err(|e| crate::error::AppError::Generic(e.to_string()))?;
    let session_dir = home.join(".moon-lol").join("history").join(datetime);
    if !session_dir.exists() {
        return Err(crate::error::AppError::Generic(
            "指定的游戏历史记录不存在".to_string(),
        ));
    }

    let mut details = Vec::new();
    let entries =
        fs::read_dir(session_dir).map_err(|e| crate::error::AppError::Generic(e.to_string()))?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() && path.extension().is_some_and(|ext| ext == "json") {
            if let Ok(content) = fs::read_to_string(path) {
                if let Ok(parsed) = serde_json::from_str::<SavedAgentHistory>(&content) {
                    details.push(parsed);
                }
            }
        }
    }

    Ok(details)
}

#[tauri::command]
pub fn delete_game_history(
    app: tauri::AppHandle,
    datetime: String,
) -> Result<(), crate::error::AppError> {
    let home = app
        .path()
        .home_dir()
        .map_err(|e| crate::error::AppError::Generic(e.to_string()))?;
    let session_dir = home.join(".moon-lol").join("history").join(datetime);
    if session_dir.exists() {
        fs::remove_dir_all(session_dir)
            .map_err(|e| crate::error::AppError::Generic(e.to_string()))?;
    }
    Ok(())
}
