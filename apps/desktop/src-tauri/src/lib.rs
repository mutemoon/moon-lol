mod agent;
mod error;
mod log;
mod process;
mod state;
mod tools;
mod ws;

use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use state::AppState;
use tauri::Manager;

use rig::tool::Tool;
use tools::{BashArgs, BashTool};

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct AiConfig {
    pub api_key: String,
    pub base_url: String,
    pub preamble: String,
}

fn get_config_dir(app: &tauri::AppHandle) -> Result<PathBuf, error::AppError> {
    let home = app.path().home_dir()?;
    let dir = home.join(".moon-lol");
    if !dir.exists() {
        fs::create_dir_all(&dir)?;
    }
    Ok(dir)
}

fn parse_legacy_env(content: &str) -> AiConfig {
    let mut api_key = String::new();
    let mut base_url = String::new();
    let mut preamble = String::new();

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let Some((key, val)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        let val = val.trim().trim_matches('"').trim_matches('\'').trim();
        if key == "ANTHROPIC_API_KEY" {
            api_key = val.to_string();
        } else if key == "ANTHROPIC_BASE_URL" {
            base_url = val.to_string();
        } else if key == "ANTHROPIC_PREAMBLE" {
            preamble = val.replace("\\n", "\n");
        }
    }

    AiConfig {
        api_key,
        base_url,
        preamble,
    }
}

#[tauri::command]
fn get_ai_config(app: tauri::AppHandle) -> Result<AiConfig, error::AppError> {
    let dir = get_config_dir(&app)?;
    let json_path = dir.join("config.json");
    let env_path = dir.join(".env");

    if json_path.exists() {
        let content = fs::read_to_string(&json_path)?;
        let config: AiConfig = serde_json::from_str(&content)
            .map_err(|e| error::AppError::Generic(format!("JSON 解析失败: {e}")))?;
        return Ok(config);
    }

    if env_path.exists() {
        let content = fs::read_to_string(&env_path)?;
        let config = parse_legacy_env(&content);
        // 自动迁移保存为新格式
        let content_json = serde_json::to_string_pretty(&config)
            .map_err(|e| error::AppError::Generic(format!("JSON 序列化失败: {e}")))?;
        fs::write(&json_path, content_json)?;
        return Ok(config);
    }

    Ok(AiConfig {
        api_key: String::new(),
        base_url: String::new(),
        preamble: String::new(),
    })
}

#[tauri::command]
fn set_ai_config(app: tauri::AppHandle, config: AiConfig) -> Result<(), error::AppError> {
    let dir = get_config_dir(&app)?;
    let json_path = dir.join("config.json");

    let content_json = serde_json::to_string_pretty(&config)
        .map_err(|e| error::AppError::Generic(format!("JSON 序列化失败: {e}")))?;
    fs::write(&json_path, content_json)?;

    std::env::set_var("ANTHROPIC_API_KEY", config.api_key.trim());
    std::env::set_var("ANTHROPIC_BASE_URL", config.base_url.trim());
    std::env::set_var("ANTHROPIC_PREAMBLE", config.preamble.trim());

    Ok(())
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct FrontAgentConfig {
    pub id: Option<String>,
    pub champion: String,
    pub team: String,
    pub prompt: String,
    pub spawn_point: [f32; 2],
}

#[tauri::command]
fn save_custom_scenario(
    app: tauri::AppHandle,
    scene_name: String,
    agents: Vec<FrontAgentConfig>,
) -> Result<(), error::AppError> {
    let dir = get_config_dir(&app)?.join("games");
    if !dir.exists() {
        fs::create_dir_all(&dir)?;
    }

    let mut resolved_agents = Vec::new();
    for (idx, agent) in agents.into_iter().enumerate() {
        let mut resolved = agent.clone();
        if resolved.id.is_none() {
            let champ_lower = agent.champion.to_lowercase();
            resolved.id = Some(format!("{}_{}", champ_lower, idx));
        }
        resolved_agents.push(resolved);
    }

    // Save JSON
    let json_path = dir.join(format!("{}.json", scene_name));
    let json_content = serde_json::to_string_pretty(&resolved_agents)
        .map_err(|e| error::AppError::Generic(format!("JSON 序列化失败: {e}")))?;
    fs::write(&json_path, json_content)?;

    // Save RON dynamic scene
    let ron_path = dir.join(format!("{}.ron", scene_name));

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

#[tauri::command]
fn delete_custom_scenario(
    app: tauri::AppHandle,
    scene_name: String,
) -> Result<(), error::AppError> {
    let dir = get_config_dir(&app)?.join("games");
    if !dir.exists() {
        return Ok(());
    }

    let json_path = dir.join(format!("{}.json", scene_name));
    let ron_path = dir.join(format!("{}.ron", scene_name));

    if json_path.exists() {
        fs::remove_file(json_path)?;
    }
    if ron_path.exists() {
        fs::remove_file(ron_path)?;
    }

    Ok(())
}

#[tauri::command]
fn list_custom_scenarios(app: tauri::AppHandle) -> Result<Vec<String>, error::AppError> {
    let dir = get_config_dir(&app)?.join("games");
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut scenarios = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().is_some_and(|ext| ext == "json") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    scenarios.push(stem.to_string());
                }
            }
        }
    }
    Ok(scenarios)
}

#[tauri::command]
fn load_custom_scenario(
    app: tauri::AppHandle,
    scene_name: String,
) -> Result<Vec<FrontAgentConfig>, error::AppError> {
    let path = get_config_dir(&app)?
        .join("games")
        .join(format!("{}.json", scene_name));

    if !path.exists() {
        return Err(error::AppError::Generic(format!(
            "场景配置文件不存在: {}",
            scene_name
        )));
    }

    let content = fs::read_to_string(path)?;
    let agents: Vec<FrontAgentConfig> = serde_json::from_str(&content)
        .map_err(|e| error::AppError::Generic(format!("JSON 解析失败: {e}")))?;

    Ok(agents)
}

#[tauri::command]
fn start_game(
    app: tauri::AppHandle,
    state: tauri::State<'_, Mutex<AppState>>,
    config: process::GameConfig,
) -> Result<(), error::AppError> {
    if let Ok(mut s) = state.lock() {
        s.active_scene = config.scene_name.clone();
    }
    process::start_game(&state, &app, config).map_err(error::AppError::Generic)
}

#[tauri::command]
fn stop_game(state: tauri::State<'_, Mutex<AppState>>) -> Result<(), error::AppError> {
    process::stop_game(&state).map_err(error::AppError::Generic)
}

#[tauri::command]
async fn connect_ws(
    app: tauri::AppHandle,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<(), error::AppError> {
    let port = {
        let s = state.lock().map_err(|_| error::AppError::LockError)?;
        let Some(ref bevy) = s.bevy else {
            return Err(error::AppError::StateError(
                "游戏未启动，无法获取端口".to_string(),
            ));
        };
        bevy.port
    };

    let session = ws::start_ws_client(app.clone(), port)
        .await
        .map_err(error::AppError::Generic)?;
    let mut s = state.lock().map_err(|_| error::AppError::LockError)?;
    s.ws = Some(session.clone());

    // 启动 AI Agent 自动化观测决策流
    tokio::spawn(agent::run_agent_orchestrator(app, session));

    Ok(())
}

#[tauri::command]
fn disconnect_ws(state: tauri::State<'_, Mutex<AppState>>) -> Result<(), error::AppError> {
    let mut s = state.lock().map_err(|_| error::AppError::LockError)?;
    s.ws = None;
    Ok(())
}

#[tauri::command]
async fn send_ws_cmd(
    state: tauri::State<'_, Mutex<AppState>>,
    cmd: String,
    params: serde_json::Value,
) -> Result<serde_json::Value, error::AppError> {
    let session = {
        let s = state.lock().map_err(|_| error::AppError::LockError)?;
        let Some(ref ws) = s.ws else {
            return Err(error::AppError::Generic("WS 未连接".to_string()));
        };
        ws.clone()
    };

    let resp = session
        .send_cmd(cmd, params)
        .await
        .map_err(error::AppError::Generic)?;
    if !resp.ok {
        return Err(error::AppError::Generic(
            resp.error.unwrap_or_else(|| "未知错误".to_string()),
        ));
    }
    Ok(resp.data.unwrap_or(serde_json::Value::Null))
}

#[tauri::command]
async fn run_bash_tool(cmd: String) -> Result<String, error::AppError> {
    let tool = BashTool;
    let args = BashArgs { cmd };
    let output = tool.call(args).await.unwrap();
    Ok(output)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(Mutex::new(AppState::new()))
        .setup(|app| {
            if let Ok(config) = get_ai_config(app.handle().clone()) {
                if !config.api_key.is_empty() {
                    std::env::set_var("ANTHROPIC_API_KEY", &config.api_key);
                }
                if !config.base_url.is_empty() {
                    std::env::set_var("ANTHROPIC_BASE_URL", &config.base_url);
                }
                if !config.preamble.is_empty() {
                    std::env::set_var("ANTHROPIC_PREAMBLE", &config.preamble);
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            start_game,
            stop_game,
            get_ai_config,
            set_ai_config,
            save_custom_scenario,
            delete_custom_scenario,
            list_custom_scenarios,
            load_custom_scenario,
            log::query_logs,
            log::query_log_entities,
            log::query_log_categories,
            log::clear_logs,
            connect_ws,
            disconnect_ws,
            send_ws_cmd,
            agent::list_game_histories,
            agent::get_game_history_detail,
            agent::delete_game_history,
            run_bash_tool
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
