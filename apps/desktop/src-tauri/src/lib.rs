mod agent;
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

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct AiConfig {
    pub api_key: String,
    pub base_url: String,
    pub preamble: String,
}

fn get_config_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let home = app.path().home_dir().map_err(|e| e.to_string())?;
    let dir = home.join(".moon-lol");
    if !dir.exists() {
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    }
    Ok(dir.join(".env"))
}

#[tauri::command]
fn get_ai_config(app: tauri::AppHandle) -> Result<AiConfig, String> {
    let path = get_config_path(&app)?;
    if !path.exists() {
        return Ok(AiConfig {
            api_key: String::new(),
            base_url: String::new(),
            preamble: String::new(),
        });
    }

    let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
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

    Ok(AiConfig { api_key, base_url, preamble })
}

#[tauri::command]
fn set_ai_config(app: tauri::AppHandle, config: AiConfig) -> Result<(), String> {
    let path = get_config_path(&app)?;
    let escaped_preamble = config.preamble.replace("\n", "\\n");
    let content = format!(
        "ANTHROPIC_API_KEY=\"{}\"\nANTHROPIC_BASE_URL=\"{}\"\nANTHROPIC_PREAMBLE=\"{}\"\n",
        config.api_key.trim(),
        config.base_url.trim(),
        escaped_preamble.trim()
    );
    fs::write(&path, content).map_err(|e| e.to_string())?;

    std::env::set_var("ANTHROPIC_API_KEY", config.api_key.trim());
    std::env::set_var("ANTHROPIC_BASE_URL", config.base_url.trim());
    std::env::set_var("ANTHROPIC_PREAMBLE", config.preamble.trim());

    Ok(())
}

#[tauri::command]
fn start_game(
    app: tauri::AppHandle,
    state: tauri::State<'_, Mutex<AppState>>,
    config: process::GameConfig,
) -> Result<(), String> {
    process::start_game(&state, &app, config)
}

#[tauri::command]
fn stop_game(state: tauri::State<'_, Mutex<AppState>>) -> Result<(), String> {
    process::stop_game(&state)
}

#[tauri::command]
async fn connect_ws(
    app: tauri::AppHandle,
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<(), String> {
    let port = {
        let s = state.lock().map_err(|e| e.to_string())?;
        let Some(ref bevy) = s.bevy else {
            return Err("游戏未启动，无法获取端口".to_string());
        };
        bevy.port
    };

    let session = ws::start_ws_client(app.clone(), port).await?;
    let mut s = state.lock().map_err(|e| e.to_string())?;
    s.ws = Some(session.clone());

    // 启动 AI Agent 自动化观测决策流
    tokio::spawn(agent::run_agent_orchestrator(app, session));

    Ok(())
}

#[tauri::command]
fn disconnect_ws(state: tauri::State<'_, Mutex<AppState>>) -> Result<(), String> {
    let mut s = state.lock().map_err(|e| e.to_string())?;
    s.ws = None;
    Ok(())
}

#[tauri::command]
async fn send_ws_cmd(
    state: tauri::State<'_, Mutex<AppState>>,
    cmd: String,
    params: serde_json::Value,
) -> Result<serde_json::Value, String> {
    let session = {
        let s = state.lock().map_err(|e| e.to_string())?;
        let Some(ref ws) = s.ws else {
            return Err("WS 未连接".to_string());
        };
        ws.clone()
    };

    let resp = session.send_cmd(cmd, params).await?;
    if !resp.ok {
        return Err(resp.error.unwrap_or_else(|| "未知错误".to_string()));
    }
    Ok(resp.data.unwrap_or(serde_json::Value::Null))
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
            log::query_logs,
            log::query_log_entities,
            log::query_log_categories,
            log::clear_logs,
            connect_ws,
            disconnect_ws,
            send_ws_cmd
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

