mod process;
mod state;

use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::Manager;

use state::AppState;

#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct AiConfig {
    api_key: String,
    base_url: String,
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
        });
    }

    let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let mut api_key = String::new();
    let mut base_url = String::new();

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let Some((key, val)) = line.split_once('=') else { continue };
        let key = key.trim();
        let val = val.trim().trim_matches('"').trim_matches('\'').trim();
        if key == "ANTHROPIC_API_KEY" {
            api_key = val.to_string();
        } else if key == "ANTHROPIC_BASE_URL" {
            base_url = val.to_string();
        }
    }

    Ok(AiConfig { api_key, base_url })
}

#[tauri::command]
fn set_ai_config(app: tauri::AppHandle, config: AiConfig) -> Result<(), String> {
    let path = get_config_path(&app)?;
    let content = format!(
        "ANTHROPIC_API_KEY=\"{}\"\nANTHROPIC_BASE_URL=\"{}\"\n",
        config.api_key.trim(),
        config.base_url.trim()
    );
    fs::write(&path, content).map_err(|e| e.to_string())?;

    std::env::set_var("ANTHROPIC_API_KEY", config.api_key.trim());
    std::env::set_var("ANTHROPIC_BASE_URL", config.base_url.trim());

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
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            start_game,
            stop_game,
            get_ai_config,
            set_ai_config
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
