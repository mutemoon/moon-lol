use std::sync::Mutex;
use tauri::State;
use rusqlite::{Connection, OpenFlags, ToSql, params_from_iter};
use crate::state::AppState;

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct LogRow {
    pub id: i64,
    pub timestamp: i64,
    pub level: String,
    pub file: Option<String>,
    pub line: Option<i64>,
    pub entity_id: Option<i64>,
    pub entity_name: Option<String>,
    pub category: Option<String>,
    pub message: String,
}

#[derive(serde::Serialize)]
pub struct QueryLogsResult {
    pub rows: Vec<LogRow>,
    pub total_count: i64,
}

#[derive(serde::Serialize)]
pub struct EntityOption {
    pub entity_id: Option<i64>,
    pub entity_name: Option<String>,
}

#[derive(serde::Serialize)]
pub struct CategoryOption {
    pub category: Option<String>,
}

#[tauri::command]
pub fn query_logs(
    state: State<'_, Mutex<AppState>>,
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

    let conn = Connection::open_with_flags(
        &proc.log_db_path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|e| e.to_string())?;

    let limit = limit.clamp(0, 1000);

    let mut sql_base = String::from("FROM logs WHERE 1=1");
    let mut params: Vec<Box<dyn ToSql>> = vec![];

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
        .query_row(&sql_count, params_from_iter(params.iter().map(|p| p.as_ref())), |r| r.get(0))
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
        .query_map(params_from_iter(data_params.iter().map(|p| p.as_ref())), |row| {
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

#[tauri::command]
pub fn query_log_entities(
    state: State<'_, Mutex<AppState>>,
) -> Result<Vec<EntityOption>, String> {
    let s = state.lock().map_err(|e| e.to_string())?;
    let Some(ref proc) = s.bevy else {
        return Ok(vec![]);
    };
    if !proc.log_db_path.exists() {
        return Ok(vec![]);
    }
    let conn = Connection::open_with_flags(
        &proc.log_db_path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
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
pub fn query_log_categories(
    state: State<'_, Mutex<AppState>>,
) -> Result<Vec<CategoryOption>, String> {
    let s = state.lock().map_err(|e| e.to_string())?;
    let Some(ref proc) = s.bevy else {
        return Ok(vec![]);
    };
    if !proc.log_db_path.exists() {
        return Ok(vec![]);
    }
    let conn = Connection::open_with_flags(
        &proc.log_db_path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
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
pub fn clear_logs(state: State<'_, Mutex<AppState>>) -> Result<(), String> {
    let s = state.lock().map_err(|e| e.to_string())?;
    let Some(ref proc) = s.bevy else {
        return Ok(());
    };
    if !proc.log_db_path.exists() {
        return Ok(());
    }
    let conn = Connection::open(&proc.log_db_path)
        .map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM logs", [])
        .map_err(|e| e.to_string())?;
    Ok(())
}
