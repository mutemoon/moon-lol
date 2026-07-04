mod agent;
mod error;
mod log;
mod process;
mod state;
mod ws;

use std::fs;
use std::path::PathBuf;

use lol_game_process_manager::{ManagerError, StartGameInput};
use serde::Serialize;
use state::AppState;
use tauri::Manager;
use uuid::Uuid;

fn get_config_dir(app: &tauri::AppHandle) -> Result<PathBuf, error::AppError> {
    let home = app.path().home_dir()?;
    let dir = home.join(".moon-lol");
    if !dir.exists() {
        fs::create_dir_all(&dir)?;
    }
    Ok(dir)
}

/// 模型供应商（桌面端本地存储，与前端 ModelProvider 接口及云端 DTO 对齐，snake_case）。
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct ModelProvider {
    pub id: String,
    pub name: String,
    pub category: String,
    pub preset_type: String,
    pub base_url: String,
    #[serde(default)]
    pub api_key: String,
    pub api_format: String,
    pub models: Vec<lol_agent_runtime::ModelConfig>,
    pub enabled: bool,
    #[serde(default)]
    pub website_url: String,
    #[serde(default)]
    pub api_key_url: String,
    #[serde(default)]
    pub icon: String,
    #[serde(default)]
    pub icon_color: String,
    pub sort_order: i32,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct FrontAgentConfig {
    pub id: Option<String>,
    pub champion: String,
    pub team: String,
    pub prompt: String,
    pub spawn_point: [f32; 2],
    #[serde(default = "default_agent_type")]
    pub agent_type: String,
    pub model: Option<String>,
    pub provider_id: Option<String>,
    pub config_json: Option<serde_json::Value>,
}

fn default_agent_type() -> String {
    "llm".to_string()
}

fn write_scene_ron(
    app: &tauri::AppHandle,
    scene_name: &str,
    agents: &[FrontAgentConfig],
) -> Result<(), error::AppError> {
    let dir = get_config_dir(app)?.join("games");
    if !dir.exists() {
        fs::create_dir_all(&dir)?;
    }

    let ron_path = dir.join(format!("{}.ron", scene_name));

    let mut resolved_agents = Vec::new();
    for (idx, agent) in agents.iter().enumerate() {
        let mut resolved = agent.clone();
        if resolved.id.is_none() {
            let champ_lower = agent.champion.to_lowercase();
            resolved.id = Some(format!("{}_{}", champ_lower, idx));
        }
        resolved_agents.push(resolved);
    }

    // Construct Bevy dynamic scene RON string
    let mut ron_content = String::new();
    ron_content.push_str("(\n    resources: {},\n    entities: {\n");

    for (idx, agent) in resolved_agents.iter().enumerate() {
        let entity_id = 4294967185 + idx as u64;
        let x = agent.spawn_point[0];
        let z = agent.spawn_point[1];
        let y = if agent.champion == "Fiora" { 38.0 } else { 0.0 };
        let team = &agent.team; // "Order" or "Chaos"
        let champ_lower = agent.champion.to_lowercase();
        let agent_id = agent.id.as_ref().unwrap();

        ron_content.push_str(&format!("        {entity_id}: (\n"));
        ron_content.push_str("            components: {\n");
        ron_content.push_str(&format!(
            "                \"bevy_transform::components::transform::Transform\": (\n"
        ));
        ron_content.push_str(&format!(
            "                    translation: ({:.1}, {:.1}, {:.1}),\n",
            x, y, z
        ));
        ron_content.push_str("                    rotation: (0.0, 0.0, 0.0, 1.0),\n");
        ron_content.push_str("                    scale: (1.0, 1.0, 1.0),\n");
        ron_content.push_str("                ),\n");
        ron_content.push_str(&format!(
            "                \"lol_core::team::Team\": {},\n",
            team
        ));
        ron_content.push_str(&format!(
            "                \"lol_champions::{}::{}\": (),\n",
            champ_lower, agent.champion
        ));
        if idx == 0 {
            ron_content.push_str("                \"lol_render::controller::Controller\": (),\n");
            ron_content.push_str("                \"lol_render::controller::SelfPlayer\": (),\n");
            ron_content.push_str("                \"lol_render::camera::Focus\": (),\n");
        }
        ron_content.push_str("                \"lol_core::entities::champion::Champion\": (),\n");
        ron_content.push_str(&format!(
            "                \"lol_core::entities::champion::AgentId\": (\"{}\"),\n",
            agent_id
        ));
        ron_content.push_str(&format!(
            "                \"lol_base::character::ConfigCharacterRecord\": (\n"
        ));
        ron_content.push_str(&format!(
            "                    character_record: Path(\"characters/{}/config.ron\"),\n",
            champ_lower
        ));
        ron_content.push_str("                ),\n");
        ron_content.push_str(&format!(
            "                \"lol_base::character::ConfigSkin\": (\n"
        ));
        ron_content.push_str(&format!(
            "                    skin: Path(\"characters/{}/skins/skin0.ron\"),\n",
            champ_lower
        ));
        ron_content.push_str("                ),\n");
        ron_content.push_str("            },\n");
        ron_content.push_str("        ),\n");
    }

    ron_content.push_str("    },\n)\n");

    fs::write(&ron_path, ron_content)?;
    Ok(())
}

/// 运行中的对局摘要（前端列表用）。
#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct RunningGame {
    id: String,
    port: i32,
    status: String,
}

fn map_manager_error(e: ManagerError) -> error::AppError {
    use ManagerError as E;
    match e {
        E::NotFound => error::AppError::Generic("对局不存在".into()),
        E::Conflict(msg) => error::AppError::Generic(msg),
        E::Validation(msg) => error::AppError::Generic(msg),
        E::Internal(msg) => error::AppError::Generic(msg),
    }
}

/// 启动一局本地游戏：分配端口、spawn Bevy 进程、登记进程表；有场景 agent 则自动起 AI 决策环。
#[tauri::command]
async fn start_game(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    config: process::GameConfig,
) -> Result<RunningGame, error::AppError> {
    let id = Uuid::new_v4();
    let log_db = process::log_db_path_for(&app, id).map_err(error::AppError::Generic)?;

    if let Some(providers) = &config.providers {
        *state.model_providers.lock().unwrap() = providers.clone();
    }

    let mut scenario_agents = config.agents.clone().unwrap_or_default();
    for (idx, agent) in scenario_agents.iter_mut().enumerate() {
        if agent.id.is_none() {
            let champ_lower = agent.champion.to_lowercase();
            agent.id = Some(format!("{}_{}", champ_lower, idx));
        }
    }

    if let Some(scene_name) = &config.scene_name {
        if !scenario_agents.is_empty() {
            write_scene_ron(&app, scene_name, &scenario_agents)?;
        }
    }

    let scenario_agents_runtime: Vec<lol_agent_runtime::AgentConfig> = scenario_agents
        .iter()
        .map(|a| lol_agent_runtime::AgentConfig {
            id: a.id.clone().unwrap_or_default(),
            champion: a.champion.clone(),
            team: a.team.clone(),
            prompt: a.prompt.clone(),
            model: a.model.clone(),
            provider_id: a.provider_id.clone(),
        })
        .collect();

    let spawn = lol_client::launch::BevySpawnRequest {
        program: String::new(), // 由 launcher 覆写
        prefix_args: vec![],
        port: 0, // 由 manager 覆写
        game_config: process::game_config(&config),
        cwd: process::workspace_root(),
        rust_log: Some(process::rust_log()),
        log_db: Some(log_db),
    };
    let input = StartGameInput {
        id,
        spawn,
        scenario_agents: scenario_agents_runtime,
    };
    let (_proc_id, port) = state
        .manager
        .start(input)
        .await
        .map_err(map_manager_error)?;
    println!("[tauri] 游戏进程启动 id={id} port={port}");

    // 自动建立与游戏进程的 WS 连接并登记会话
    let session = ws::start_ws_client(state.event_channels.clone(), id, port as u16)
        .await
        .map_err(error::AppError::Generic)?;
    state.ws_sessions.lock().unwrap().insert(id, session);

    Ok(RunningGame {
        id: id.to_string(),
        port,
        status: "running".into(),
    })
}

/// 按 id 停止对局：kill 进程 + 释放端口 + 清理该端口的 WS 会话。
#[tauri::command]
async fn stop_game(state: tauri::State<'_, AppState>, id: String) -> Result<(), error::AppError> {
    let id =
        Uuid::parse_str(&id).map_err(|e| error::AppError::Generic(format!("无效对局 id: {e}")))?;
    state.manager.stop(id).await.map_err(map_manager_error)?;
    state.ws_sessions.lock().unwrap().remove(&id);
    state.event_channels.lock().unwrap().remove(&id);
    Ok(())
}

/// 订阅本地对局的实时事件流（使用 Tauri 2.x Channel 进行单播）。
#[tauri::command]
fn subscribe_match_events(
    state: tauri::State<'_, AppState>,
    id: String,
    channel: tauri::ipc::Channel<serde_json::Value>,
) -> Result<(), error::AppError> {
    let id =
        Uuid::parse_str(&id).map_err(|e| error::AppError::Generic(format!("无效对局 id: {e}")))?;
    let mut event_channels = state.event_channels.lock().unwrap();
    event_channels
        .entry(id)
        .or_insert_with(Vec::new)
        .push(channel);
    Ok(())
}

/// 暂停本地对局（幂等，返回是否触发了暂停状态切换）。
#[tauri::command]
async fn pause_match(
    state: tauri::State<'_, AppState>,
    id: String,
) -> Result<bool, error::AppError> {
    let id =
        Uuid::parse_str(&id).map_err(|e| error::AppError::Generic(format!("无效对局 id: {e}")))?;
    let session = {
        let sessions = state.ws_sessions.lock().unwrap();
        sessions
            .get(&id)
            .cloned()
            .ok_or_else(|| error::AppError::Generic("对局 WS 未连接".to_string()))?
    };
    let client = lol_client::GameClient::new(session);
    let triggered = client.pause().await.map_err(error::AppError::Generic)?;
    Ok(triggered)
}

/// 恢复本地对局（幂等，返回是否触发了暂停状态切换）。
#[tauri::command]
async fn resume_match(
    state: tauri::State<'_, AppState>,
    id: String,
) -> Result<bool, error::AppError> {
    let id =
        Uuid::parse_str(&id).map_err(|e| error::AppError::Generic(format!("无效对局 id: {e}")))?;
    let session = {
        let sessions = state.ws_sessions.lock().unwrap();
        sessions
            .get(&id)
            .cloned()
            .ok_or_else(|| error::AppError::Generic("对局 WS 未连接".to_string()))?
    };
    let client = lol_client::GameClient::new(session);
    let triggered = client.unpause().await.map_err(error::AppError::Generic)?;
    Ok(triggered)
}

/// 设置本地对局的上帝模式状态。
#[tauri::command]
async fn set_god_mode(
    state: tauri::State<'_, AppState>,
    id: String,
    enabled: bool,
) -> Result<(), error::AppError> {
    let id =
        Uuid::parse_str(&id).map_err(|e| error::AppError::Generic(format!("无效对局 id: {e}")))?;
    let session = {
        let sessions = state.ws_sessions.lock().unwrap();
        sessions
            .get(&id)
            .cloned()
            .ok_or_else(|| error::AppError::Generic("对局 WS 未连接".to_string()))?
    };
    let client = lol_client::GameClient::new(session);
    client
        .god_mode(enabled)
        .await
        .map_err(error::AppError::Generic)?;
    Ok(())
}

/// 设置本地对局的冷却状态。
#[tauri::command]
async fn toggle_cooldown(
    state: tauri::State<'_, AppState>,
    id: String,
    enabled: bool,
) -> Result<(), error::AppError> {
    let id =
        Uuid::parse_str(&id).map_err(|e| error::AppError::Generic(format!("无效对局 id: {e}")))?;
    let session = {
        let sessions = state.ws_sessions.lock().unwrap();
        sessions
            .get(&id)
            .cloned()
            .ok_or_else(|| error::AppError::Generic("对局 WS 未连接".to_string()))?
    };
    let client = lol_client::GameClient::new(session);
    client
        .toggle_cooldown(enabled)
        .await
        .map_err(error::AppError::Generic)?;
    Ok(())
}

/// 重置本地对局中的英雄位置。
#[tauri::command]
async fn reset_position(
    state: tauri::State<'_, AppState>,
    id: String,
) -> Result<(), error::AppError> {
    let id =
        Uuid::parse_str(&id).map_err(|e| error::AppError::Generic(format!("无效对局 id: {e}")))?;
    let session = {
        let sessions = state.ws_sessions.lock().unwrap();
        sessions
            .get(&id)
            .cloned()
            .ok_or_else(|| error::AppError::Generic("对局 WS 未连接".to_string()))?
    };
    let client = lol_client::GameClient::new(session);
    client
        .reset_position()
        .await
        .map_err(error::AppError::Generic)?;
    Ok(())
}

/// 切换本地对局中的调试视角英雄。
#[tauri::command]
async fn switch_champion(
    state: tauri::State<'_, AppState>,
    id: String,
    name: String,
) -> Result<(), error::AppError> {
    let id =
        Uuid::parse_str(&id).map_err(|e| error::AppError::Generic(format!("无效对局 id: {e}")))?;
    let session = {
        let sessions = state.ws_sessions.lock().unwrap();
        sessions
            .get(&id)
            .cloned()
            .ok_or_else(|| error::AppError::Generic("对局 WS 未连接".to_string()))?
    };
    let client = lol_client::GameClient::new(session);
    client
        .switch_champion(&name)
        .await
        .map_err(error::AppError::Generic)?;
    Ok(())
}

/// 设置本地对局中某个角色的运行脚本代码。
#[tauri::command]
async fn set_script(
    state: tauri::State<'_, AppState>,
    id: String,
    entity_id: u64,
    source: String,
) -> Result<(), error::AppError> {
    let id =
        Uuid::parse_str(&id).map_err(|e| error::AppError::Generic(format!("无效对局 id: {e}")))?;
    let session = {
        let sessions = state.ws_sessions.lock().unwrap();
        sessions
            .get(&id)
            .cloned()
            .ok_or_else(|| error::AppError::Generic("对局 WS 未连接".to_string()))?
    };
    let client = lol_client::GameClient::new(session);
    client
        .set_script(entity_id, &source)
        .await
        .map_err(error::AppError::Generic)?;
    Ok(())
}

/// 列出所有运行中的本地对局（前端侧栏/列表页用）。
#[tauri::command]
async fn list_running_games(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<RunningGame>, error::AppError> {
    let procs = state
        .manager
        .list_processes()
        .await
        .map_err(map_manager_error)?;
    Ok(procs
        .into_iter()
        .map(|p| RunningGame {
            id: p.id.to_string(),
            port: p.port,
            status: p.status.as_str().to_string(),
        })
        .collect())
}

/// 按 id 查询单个运行中对局（调试页确认端口用）。
#[tauri::command]
async fn get_running_game(
    state: tauri::State<'_, AppState>,
    id: String,
) -> Result<Option<RunningGame>, error::AppError> {
    let id =
        Uuid::parse_str(&id).map_err(|e| error::AppError::Generic(format!("无效对局 id: {e}")))?;
    let procs = state
        .manager
        .list_processes()
        .await
        .map_err(map_manager_error)?;
    Ok(procs.into_iter().find(|p| p.id == id).map(|p| RunningGame {
        id: p.id.to_string(),
        port: p.port,
        status: p.status.as_str().to_string(),
    }))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            app.manage(AppState::new(app.handle().clone()));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            start_game,
            stop_game,
            list_running_games,
            get_running_game,
            log::query_logs,
            log::query_log_entities,
            log::query_log_categories,
            log::clear_logs,
            subscribe_match_events,
            pause_match,
            resume_match,
            set_god_mode,
            toggle_cooldown,
            reset_position,
            switch_champion,
            set_script
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
