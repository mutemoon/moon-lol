use std::path::PathBuf;

use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{Row, SqlitePool};
use tauri::Manager;


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

/// 按对局 id 取出该局的日志 DB 路径（`~/.moon-lol/logs/{game_id}.db`）。
/// 文件不存在返回 None（对局可能尚未写入或已停止）。
fn log_db_path(app: &tauri::AppHandle, game_id: &str) -> Option<PathBuf> {
    let home = app.path().home_dir().ok()?;
    let path = home
        .join(".moon-lol")
        .join("logs")
        .join(format!("{game_id}.db"));
    path.exists().then_some(path)
}

/// 以只读模式打开 SQLite 文件，返回一个临时连接池。
async fn open_read_only(path: &PathBuf) -> Result<SqlitePool, String> {
    let opts = SqliteConnectOptions::new()
        .filename(path)
        .read_only(true)
        .immutable(true);
    SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(opts)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn query_logs(
    app: tauri::AppHandle,
    game_id: String,
    offset: i64,
    limit: i64,
    levels: Option<Vec<String>>,
    entity_id: Option<i64>,
    category: Option<String>,
    search_text: Option<String>,
) -> Result<QueryLogsResult, String> {
    let db_path = match log_db_path(&app, &game_id) {
        Some(p) => p,
        None => {
            return Ok(QueryLogsResult {
                rows: vec![],
                total_count: 0,
            })
        }
    };

    let pool = open_read_only(&db_path).await?;
    let limit = limit.clamp(0, 1000);

    // ── 构造 WHERE 子句与绑定参数（全部走 bind，不内联，防注入）──
    let mut where_clause = String::from("WHERE 1=1");
    let mut levels_args: Vec<String> = vec![];
    let mut entity_id_arg: Option<i64> = None;
    let mut category_arg: Option<String> = None;
    let mut search_arg: Option<String> = None;

    if let Some(ref lvl) = levels {
        if !lvl.is_empty() {
            let placeholders: Vec<&str> = lvl.iter().map(|_| "?").collect();
            where_clause.push_str(&format!(" AND level IN ({})", placeholders.join(",")));
            levels_args = lvl.clone();
        }
    }
    if let Some(eid) = entity_id {
        where_clause.push_str(" AND entity_id = ?");
        entity_id_arg = Some(eid);
    }
    if let Some(ref cat) = category {
        where_clause.push_str(" AND category = ?");
        category_arg = Some(cat.clone());
    }
    if let Some(ref search) = search_text {
        let search = search.trim();
        if !search.is_empty() {
            where_clause.push_str(" AND message LIKE ?");
            search_arg = Some(format!("%{search}%"));
        }
    }

    // 1. COUNT
    let sql_count = format!("SELECT COUNT(*) FROM logs {where_clause}");
    let total_count: i64 = {
        let mut q = sqlx::query_scalar::<_, i64>(&sql_count);
        for l in &levels_args {
            q = q.bind(l);
        }
        if let Some(eid) = entity_id_arg {
            q = q.bind(eid);
        }
        if let Some(ref cat) = category_arg {
            q = q.bind(cat);
        }
        if let Some(ref search) = search_arg {
            q = q.bind(search);
        }
        q.fetch_one(&pool).await.map_err(|e| e.to_string())?
    };

    // 2. 负 offset 自动算最后一页
    let mut real_offset = offset;
    if real_offset < 0 {
        real_offset = std::cmp::max(0, total_count - limit);
    }

    // 3. 数据查询
    let sql_data = format!(
        "SELECT id, timestamp, level, file, line, entity_id, entity_name, category, message \
         FROM logs {where_clause} ORDER BY id ASC LIMIT ? OFFSET ?"
    );
    let mut q = sqlx::query(&sql_data);
    for l in &levels_args {
        q = q.bind(l);
    }
    if let Some(eid) = entity_id_arg {
        q = q.bind(eid);
    }
    if let Some(ref cat) = category_arg {
        q = q.bind(cat);
    }
    if let Some(ref search) = search_arg {
        q = q.bind(search);
    }
    q = q.bind(limit);
    q = q.bind(real_offset);

    let rows = q.fetch_all(&pool).await.map_err(|e| e.to_string())?;
    let mut result = Vec::with_capacity(rows.len());
    for row in rows {
        result.push(LogRow {
            id: row.try_get(0).map_err(|e| e.to_string())?,
            timestamp: row.try_get(1).map_err(|e| e.to_string())?,
            level: row.try_get(2).map_err(|e| e.to_string())?,
            file: row.try_get(3).map_err(|e| e.to_string())?,
            line: row.try_get(4).map_err(|e| e.to_string())?,
            entity_id: row.try_get(5).map_err(|e| e.to_string())?,
            entity_name: row.try_get(6).map_err(|e| e.to_string())?,
            category: row.try_get(7).map_err(|e| e.to_string())?,
            message: row.try_get(8).map_err(|e| e.to_string())?,
        });
    }

    Ok(QueryLogsResult {
        rows: result,
        total_count,
    })
}

#[tauri::command]
pub async fn query_log_entities(
    app: tauri::AppHandle,
    game_id: String,
) -> Result<Vec<EntityOption>, String> {
    let db_path = match log_db_path(&app, &game_id) {
        Some(p) => p,
        None => return Ok(vec![]),
    };
    let pool = open_read_only(&db_path).await?;

    let rows = sqlx::query(
        "SELECT DISTINCT entity_id, entity_name FROM logs \
         WHERE entity_id IS NOT NULL ORDER BY entity_id",
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let mut result = Vec::with_capacity(rows.len());
    for row in rows {
        result.push(EntityOption {
            entity_id: row.try_get(0).map_err(|e| e.to_string())?,
            entity_name: row.try_get(1).map_err(|e| e.to_string())?,
        });
    }
    Ok(result)
}

#[tauri::command]
pub async fn query_log_categories(
    app: tauri::AppHandle,
    game_id: String,
) -> Result<Vec<CategoryOption>, String> {
    let db_path = match log_db_path(&app, &game_id) {
        Some(p) => p,
        None => return Ok(vec![]),
    };
    let pool = open_read_only(&db_path).await?;

    let rows = sqlx::query(
        "SELECT DISTINCT category FROM logs WHERE category IS NOT NULL ORDER BY category",
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let mut result = Vec::with_capacity(rows.len());
    for row in rows {
        result.push(CategoryOption {
            category: row.try_get(0).map_err(|e| e.to_string())?,
        });
    }
    Ok(result)
}

#[tauri::command]
pub async fn clear_logs(app: tauri::AppHandle, game_id: String) -> Result<(), String> {
    let db_path = match log_db_path(&app, &game_id) {
        Some(p) => p,
        None => return Ok(()),
    };
    let opts = SqliteConnectOptions::new().filename(&db_path);
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(opts)
        .await
        .map_err(|e| e.to_string())?;
    sqlx::query("DELETE FROM logs")
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;
    pool.close().await;
    Ok(())
}
