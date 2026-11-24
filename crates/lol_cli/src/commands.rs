//! `lol room` / `lol game` 的命令实现：只负责编排，进程与文件细节在 `room` / `paths`。

use std::path::PathBuf;

use lol_client::GameClient;
use serde_json::Value;

use crate::error::{self, Error, Result};
use crate::paths;
use crate::room::{self, RoomConfig, RoomState, RoomStatus, RunningRoom};

/// `lol room create`：新建房间目录并写入默认配置。
pub fn create(id: &str, champion: Option<String>, scene: Option<String>) -> Result<()> {
    let dir = paths::room_dir(id);
    if dir.exists() {
        return Err(Error::User(format!("房间 {id} 已存在：{}", dir.display())));
    }
    let config = RoomConfig {
        champion: champion.unwrap_or_else(|| RoomConfig::default().champion),
        scene,
        ..Default::default()
    };
    config.save(&paths::room_config_file(id))?;
    println!("房间已创建 id: {id}");
    println!("配置: {}", paths::room_config_file(id).display());
    Ok(())
}

/// `lol room config`：改暂停计划 / 英雄 / 场景。
pub fn configure(
    id: &str,
    pause_at: Option<Vec<f64>>,
    champion: Option<String>,
    scene: Option<String>,
) -> Result<()> {
    ensure_room(id)?;
    let path = paths::room_config_file(id);
    let mut config = RoomConfig::load(&path);
    if let Some(pause_at) = pause_at {
        config.pause_at = pause_at;
    }
    if let Some(champion) = champion {
        config.champion = champion;
    }
    if scene.is_some() {
        config.scene = scene;
    }
    config.save(&path)?;

    println!("房间 {id} 配置已更新");
    println!("  champion: {}", config.champion);
    println!(
        "  scene: {}",
        config.scene.as_deref().unwrap_or("<默认 classic>")
    );
    println!("  pause_at: {}", format_pause_at(&config.pause_at));
    Ok(())
}

/// `lol room start`：spawn 游戏进程，等控制面就绪后返回（进程继续在后台运行）。
pub async fn start(id: &str) -> Result<()> {
    ensure_room(id)?;
    let config = RoomConfig::load(&paths::room_config_file(id));

    // 同一房间只允许一个存活进程；「启动中被中断」留下的进程也要挡掉。
    if let Ok(state) = RoomState::load(id)
        && (state.is_alive() || (state.status == RoomStatus::Starting && connecting(state.port)))
    {
        return Err(Error::User(format!(
            "房间 {id} 已有存活进程 pid={}，先 `lol room --id {id} kill`",
            state.pid
        )));
    }

    let (program, prefix) = lol_client::launch::resolve_executable("moon_lol", "moon_lol");
    let mut running = RunningRoom::spawn(id, &config, &program, &prefix)?;
    let port = running.state.port;
    println!("游戏进程已创建 pid: {} 控制端口: {port}", running.state.pid);
    println!("日志: {}", running.state.log.display());

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(180);
    let client = loop {
        if let Some(code) = running.exited()? {
            return Err(Error::User(format!(
                "游戏进程启动失败（退出码 {code:?}），见日志 {}",
                running.state.log.display()
            )));
        }
        if std::time::Instant::now() > deadline {
            return Err(Error::User(format!(
                "等待游戏控制面超时（180s），见日志 {}",
                running.state.log.display()
            )));
        }
        if let Ok(session) = lol_client::start_ws_client(port, None).await {
            break GameClient::new(session);
        }
    };

    running.set_status(RoomStatus::Running)?;
    // 游戏 Startup 后可能仍处于暂停（CLI 场景），统一恢复为运行态。
    let _ = client.unpause().await;
    println!("游戏已就绪 id: {id}");
    println!("房间目录: {}", paths::room_dir(id).display());
    Ok(())
}

/// `lol room list`
pub fn list() -> Result<()> {
    let ids = room::list_rooms();
    if ids.is_empty() {
        println!("还没有房间，先 `lol room create`");
        return Ok(());
    }
    println!("{:<24} {:<10} {:<8} {}", "id", "状态", "端口", "pid");
    for id in ids {
        match RoomState::load(&id) {
            Ok(state) => {
                let status = if state.is_alive() {
                    state.status.label().to_string()
                } else {
                    "exited".to_string()
                };
                println!("{:<24} {:<10} {:<8} {}", id, status, state.port, state.pid);
            }
            Err(_) => println!("{:<24} {:<10}", id, "created"),
        }
    }
    Ok(())
}

/// `lol room kill`：停掉房间进程。
pub fn kill(id: &str) -> Result<()> {
    let mut state = RoomState::load(id)?;
    if state.is_alive() {
        room::kill_tree(state.pid)?;
        println!("房间 {id} 进程 pid={} 已终止", state.pid);
    } else {
        println!("房间 {id} 进程已退出");
    }
    state.status = RoomStatus::Stopped;
    state.save()?;
    Ok(())
}

/// `lol game screenshot`：截主窗口到 `~/.moon-lol/rooms/<id>/shots/`。
pub async fn screenshot(
    client: &GameClient,
    state: &RoomState,
    out: Option<PathBuf>,
) -> Result<PathBuf> {
    let dir = state.shots_dir();
    std::fs::create_dir_all(&dir)?;
    let path = absolute(out.unwrap_or_else(|| dir.join(format!("{}.png", room::timestamp()))))?;
    client
        .screenshot(&path)
        .await
        .map_err(|e| error::control_error(state.port, &e))?;
    Ok(path)
}

/// `lol game snapshot`：全量 ECS 组件导出，落 JSON 文件。
pub async fn snapshot(
    client: &GameClient,
    state: &RoomState,
    component: Option<String>,
    value_contains: Option<String>,
    out: Option<PathBuf>,
) -> Result<(PathBuf, usize)> {
    let mut params = serde_json::json!({});
    if let Some(component) = component {
        params["component"] = Value::String(component);
    }
    if let Some(value_contains) = value_contains {
        params["value_contains"] = Value::String(value_contains);
    }

    let resp = client
        .snapshot(params)
        .await
        .map_err(|e| error::control_error(state.port, &e))?;
    if !resp.ok {
        return Err(Error::User(
            resp.error.unwrap_or_else(|| "快照失败".to_string()),
        ));
    }
    let data = resp.data.unwrap_or(Value::Null);
    let count = data
        .get("entity_count")
        .and_then(Value::as_u64)
        .unwrap_or(0) as usize;

    let dir = state.snaps_dir();
    std::fs::create_dir_all(&dir)?;
    let path = absolute(out.unwrap_or_else(|| dir.join(format!("{}.json", room::timestamp()))))?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(
        &path,
        serde_json::to_vec_pretty(&data).map_err(|e| Error::Serialize(e.to_string()))?,
    )?;
    Ok((path, count))
}

/// 房间是否还活着（供 game 子命令前置检查）。
pub fn live_state(id: &str) -> Result<RoomState> {
    let state = RoomState::load(id)?;
    if !state.is_alive() {
        return Err(Error::Dead(id.to_string()));
    }
    Ok(state)
}

pub fn format_pause_at(pause_at: &[f64]) -> String {
    if pause_at.is_empty() {
        return "<无>".to_string();
    }
    pause_at
        .iter()
        .map(|t| format!("{t:.2}s"))
        .collect::<Vec<_>>()
        .join(", ")
}

/// 房间目录存在即视为「已创建」。
fn ensure_room(id: &str) -> Result<()> {
    if paths::room_dir(id).exists() {
        Ok(())
    } else {
        Err(Error::User(format!(
            "房间 {id} 不存在，先 `lol room create` 或 `lol room create --id {id}`"
        )))
    }
}

/// 相对路径按当前工作目录补全，保证游戏进程（不同 cwd）能写对位置。
fn absolute(path: PathBuf) -> Result<PathBuf> {
    if path.is_absolute() {
        Ok(path)
    } else {
        Ok(std::env::current_dir()?.join(path))
    }
}

fn connecting(port: u16) -> bool {
    std::net::TcpStream::connect_timeout(
        &std::net::SocketAddr::from(([127, 0, 0, 1], port)),
        std::time::Duration::from_millis(50),
    )
    .is_ok()
}
