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

#[derive(serde::Serialize, Clone)]
struct LogRow {
    id: i64,
    timestamp: i64,
    level: String,
    file: Option<String>,
    line: Option<i64>,
    entity_id: Option<i64>,
    entity_name: Option<String>,
    category: Option<String>,
    message: String,
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

#[derive(serde::Serialize)]
struct QueryLogsResult {
    rows: Vec<LogRow>,
    total_count: i64,
}

#[tauri::command]
fn query_logs(
    state: tauri::State<'_, Mutex<AppState>>,
    offset: i64,
    limit: i64,
    levels: Option<Vec<String>>,
    entity_id: Option<i64>,
    category: Option<String>,
    search_text: Option<String>,
) -> Result<QueryLogsResult, String> {
    let s = state.lock().map_err(|e| e.to_string())?;
    let Some(ref proc) = s.bevy else {
        return Ok(QueryLogsResult { rows: vec![], total_count: 0 });
    };

    if !proc.log_db_path.exists() {
        return Ok(QueryLogsResult { rows: vec![], total_count: 0 });
    }

    let conn = rusqlite::Connection::open_with_flags(
        &proc.log_db_path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|e| e.to_string())?;

    let limit = limit.clamp(0, 1000);

    let mut sql_base = String::from("FROM logs WHERE 1=1");
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![];

    if let Some(ref lvl) = levels {
        if !lvl.is_empty() {
            let placeholders: Vec<&str> = lvl.iter().map(|_| "?").collect();
            sql_base = format!("{} AND level IN ({})", sql_base, placeholders.join(","));
            for l in lvl {
                params.push(Box::new(l.clone()));
            }
        }
    }
    if let Some(eid) = entity_id {
        sql_base.push_str(" AND entity_id = ?");
        params.push(Box::new(eid));
    }
    if let Some(ref cat) = category {
        sql_base.push_str(" AND category = ?");
        params.push(Box::new(cat.clone()));
    }
    if let Some(ref search) = search_text {
        let search = search.trim();
        if !search.is_empty() {
            sql_base.push_str(" AND message LIKE ?");
            params.push(Box::new(format!("%{search}%")));
        }
    }

    // 1. 查询符合过滤条件的总记录数
    let sql_count = format!("SELECT COUNT(*) {}", sql_base);
    let total_count: i64 = conn
        .query_row(&sql_count, rusqlite::params_from_iter(params.iter().map(|p| p.as_ref())), |r| r.get(0))
        .map_err(|e| e.to_string())?;

    // 2. 如果 offset < 0，自动计算为最后一页
    let mut real_offset = offset;
    if real_offset < 0 {
        real_offset = std::cmp::max(0, total_count - limit);
    }

    // 3. 查询具体的当前页数据
    let sql_data = format!(
        "SELECT id, timestamp, level, file, line, entity_id, entity_name, category, message {} ORDER BY id ASC LIMIT ? OFFSET ?",
        sql_base
    );
    let mut data_params = params;
    data_params.push(Box::new(limit));
    data_params.push(Box::new(real_offset));

    let mut stmt = conn.prepare(&sql_data).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(rusqlite::params_from_iter(data_params.iter().map(|p| p.as_ref())), |row| {
            Ok(LogRow {
                id: row.get(0)?,
                timestamp: row.get(1)?,
                level: row.get(2)?,
                file: row.get(3)?,
                line: row.get(4)?,
                entity_id: row.get(5)?,
                entity_name: row.get(6)?,
                category: row.get(7)?,
                message: row.get(8)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut result = Vec::new();
    for row in rows {
        result.push(row.map_err(|e| e.to_string())?);
    }
    Ok(QueryLogsResult { rows: result, total_count })
}

#[derive(serde::Serialize)]
struct EntityOption {
    entity_id: Option<i64>,
    entity_name: Option<String>,
}

#[derive(serde::Serialize)]
struct CategoryOption {
    category: Option<String>,
}

#[tauri::command]
fn query_log_entities(
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<EntityOption>, String> {
    let s = state.lock().map_err(|e| e.to_string())?;
    let Some(ref proc) = s.bevy else {
        return Ok(vec![]);
    };
    if !proc.log_db_path.exists() {
        return Ok(vec![]);
    }
    let conn = rusqlite::Connection::open_with_flags(
        &proc.log_db_path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT DISTINCT entity_id, entity_name FROM logs WHERE entity_id IS NOT NULL ORDER BY entity_id")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(EntityOption {
                entity_id: row.get(0)?,
                entity_name: row.get(1)?,
            })
        })
        .map_err(|e| e.to_string())?;
    let mut result = Vec::new();
    for row in rows {
        result.push(row.map_err(|e| e.to_string())?);
    }
    Ok(result)
}

#[tauri::command]
fn query_log_categories(
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<Vec<CategoryOption>, String> {
    let s = state.lock().map_err(|e| e.to_string())?;
    let Some(ref proc) = s.bevy else {
        return Ok(vec![]);
    };
    if !proc.log_db_path.exists() {
        return Ok(vec![]);
    }
    let conn = rusqlite::Connection::open_with_flags(
        &proc.log_db_path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT DISTINCT category FROM logs WHERE category IS NOT NULL ORDER BY category")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(CategoryOption {
                category: row.get(0)?,
            })
        })
        .map_err(|e| e.to_string())?;
    let mut result = Vec::new();
    for row in rows {
        result.push(row.map_err(|e| e.to_string())?);
    }
    Ok(result)
}

#[tauri::command]
fn clear_logs(state: tauri::State<'_, Mutex<AppState>>) -> Result<(), String> {
    let s = state.lock().map_err(|e| e.to_string())?;
    let Some(ref proc) = s.bevy else {
        return Ok(());
    };
    if !proc.log_db_path.exists() {
        return Ok(());
    }
    let conn = rusqlite::Connection::open(&proc.log_db_path)
        .map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM logs", [])
        .map_err(|e| e.to_string())?;
    Ok(())
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
            set_ai_config,
            query_logs,
            query_log_entities,
            query_log_categories,
            clear_logs
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
