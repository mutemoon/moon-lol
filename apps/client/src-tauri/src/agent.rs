use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::Local;
use lol_agent_runtime::{
    resolve_credentials, run_orchestrator, AgentConfig, AgentRunResult, CredentialResolver,
    OrchestratorSink, PlatformEnv, ProviderCredentials, ResolvedCredentials,
};
use lol_client::GameClient;
use rig::completion::Message;
use serde_json::Value;
use tauri::{Emitter, Manager};

use crate::state::AppState;
use crate::ws::WsSession;

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
        let creds = provider.map(|p| ProviderCredentials {
            api_key: p.api_key.clone(),
            base_url: p.base_url.clone(),
            api_format: p.api_format.clone(),
        });
        resolve_credentials(agent, creds, env)
    }
}

/// 桌面端副作用出口：emit 对话历史 / 终结事件、写盘历史、停游戏进程。
pub struct DesktopSink {
    app: tauri::AppHandle,
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

        // 保存本局所有的 Agent 对话历史记录
        self.save_histories(results, last_game_time);

        // 终结并关闭游戏
        if let Some(state) = self.app.try_state::<Mutex<AppState>>() {
            let _ = crate::process::stop_game(&state);
        }
    }

    async fn is_running(&self) -> bool {
        self.app
            .try_state::<Mutex<AppState>>()
            .and_then(|m| m.lock().ok().map(|s| s.ws.is_some()))
            .unwrap_or(false)
    }
}

impl DesktopSink {
    fn save_histories(&self, results: &[AgentRunResult], game_duration: f64) {
        let home = self.app.path().home_dir().unwrap_or_default();
        let session_dir = home
            .join(".moon-lol")
            .join("history")
            .join(&self.start_time_str);

        if let Err(e) = fs::create_dir_all(&session_dir) {
            println!("[Agent Orchestrator] 创建历史目录失败: {:?}", e);
            return;
        }

        for r in results {
            let saved = SavedAgentHistory {
                agent_id: r.agent.id.clone(),
                champion: r.agent.champion.clone(),
                team: r.agent.team.clone(),
                prompt: r.agent.prompt.clone(),
                system_prompt: self.system_prompt.clone(),
                history: r.history.clone(),
                game_duration,
                datetime: self.start_time_str.clone(),
            };

            let json_path = session_dir.join(format!("{}.json", r.agent.id));
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

/// 从 `~/.moon-lol/providers.json` 读取供应商列表（失败返回空）。
fn load_providers(app: &tauri::AppHandle) -> Vec<crate::ModelProvider> {
    let home = app.path().home_dir().unwrap_or_default();
    let path = home.join(".moon-lol").join("providers.json");
    let Ok(content) = fs::read_to_string(&path) else {
        return Vec::new();
    };
    serde_json::from_str(&content).unwrap_or_default()
}

/// 从 `~/.moon-lol/games/{scene}.json` 读取场景 agent 配置；任一环节缺失返回空。
fn load_scene_agents(app: &tauri::AppHandle) -> Vec<AgentConfig> {
    let Some(name) = app
        .try_state::<Mutex<AppState>>()
        .and_then(|m| m.lock().ok().and_then(|s| s.active_scene.clone()))
    else {
        return Vec::new();
    };

    let home = app.path().home_dir().unwrap_or_default();
    let json_path = home
        .join(".moon-lol")
        .join("games")
        .join(format!("{}.json", name));
    let Ok(content) = fs::read_to_string(&json_path) else {
        return Vec::new();
    };
    serde_json::from_str(&content).unwrap_or_default()
}

/// 后台协调器循环：在 WARMUP_DURATION 时暂停游戏，运行 Agent 决策，继续游戏 1s，周而复始，在 GAME_END_TIME 时终止并统计。
///
/// 纯编排逻辑由 `lol_agent_runtime::run_orchestrator` 承载，本函数仅负责桌面端的
/// 凭证解析、副作用出口与场景配置加载。
pub async fn run_agent_orchestrator(app: tauri::AppHandle, ws: WsSession) {
    println!("[Agent Orchestrator] 启动 AI Agent 后台生命周期循环");

    let agents = load_scene_agents(&app);
    if agents.is_empty() {
        println!("[Agent Orchestrator] 未检测到自定义场景配置代理，拒绝开启 AI Agent 决策环");
        return;
    }
    println!(
        "[Agent Orchestrator] 成功加载自定义场景配置代理: {:?}",
        agents
    );

    let env = PlatformEnv::from_env();
    let resolver = Arc::new(DesktopCredentialResolver {
        providers: load_providers(&app),
    });
    let start_time_str = Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
    let sink = Arc::new(DesktopSink {
        app: app.clone(),
        start_time_str,
        system_prompt: String::new(),
    });
    let client = GameClient::new(ws);

    run_orchestrator(client, agents, resolver, env, sink).await;
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
