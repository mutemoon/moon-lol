//! 对局 WS 会话服务。
//!
//! 基于 `lol_client::WsSession` / `GameClient` 封装本地对局的调试 / 控制操作：
//! pause / resume / god_mode / toggle_cooldown / reset_position / switch_champion / set_script，
//! 以及事件订阅（用 tokio mpsc 替代 Tauri Channel）。

use std::sync::Arc;

use lol_client::{GameClient, WsSession};
use tokio::sync::mpsc;
use uuid::Uuid;

use super::types::LocalGameState;

/// 对局控制操作结果。
type MatchResult<T> = Result<T, String>;

// ── 辅助：从共享状态取 session ──

fn get_session(state: &LocalGameState, id: &Uuid) -> MatchResult<WsSession> {
    state
        .ws_sessions
        .lock()
        .map_err(|e| format!("锁获取失败: {e}"))?
        .get(id)
        .cloned()
        .ok_or_else(|| "对局 WS 未连接".to_string())
}

/// 通用骨架：解析 id → 取 session → 在全局 tokio runtime 内对 GameClient 执行操作。
/// 所有对局控制命令共用，消除重复样板。
async fn with_session<Fut, T>(
    state: &Arc<LocalGameState>,
    id_str: &str,
    f: impl FnOnce(GameClient) -> Fut + Send + 'static,
) -> MatchResult<T>
where
    Fut: std::future::Future<Output = MatchResult<T>> + Send + 'static,
    T: Send + 'static,
{
    let state = state.clone();
    let id_str = id_str.to_string();
    super::runtime::run_on_tokio(move || async move {
        let id = Uuid::parse_str(&id_str).map_err(|e| format!("无效对局 id: {e}"))?;
        let session = get_session(&state, &id)?;
        f(GameClient::new(session)).await
    })
    .await
}

// ── 控制操作 ──

/// 暂停本地对局（幂等，返回是否实际触发了暂停状态切换）。
pub async fn pause_match(state: &Arc<LocalGameState>, id_str: &str) -> MatchResult<bool> {
    with_session(state, id_str, |client| async move {
        client.pause().await.map_err(|e| format!("暂停失败: {e}"))
    })
    .await
}

/// 恢复本地对局（幂等，返回是否实际触发了暂停状态切换）。
pub async fn resume_match(state: &Arc<LocalGameState>, id_str: &str) -> MatchResult<bool> {
    with_session(state, id_str, |client| async move {
        client.unpause().await.map_err(|e| format!("恢复失败: {e}"))
    })
    .await
}

/// 设置本地对局的上帝模式状态。
pub async fn set_god_mode(
    state: &Arc<LocalGameState>,
    id_str: &str,
    enabled: bool,
) -> MatchResult<()> {
    with_session(state, id_str, move |client| async move {
        client.god_mode(enabled).await.map_err(|e| e.to_string())?;
        Ok(())
    })
    .await
}

/// 设置本地对局的冷却状态。
pub async fn toggle_cooldown(
    state: &Arc<LocalGameState>,
    id_str: &str,
    enabled: bool,
) -> MatchResult<()> {
    with_session(state, id_str, move |client| async move {
        client
            .toggle_cooldown(enabled)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    })
    .await
}

/// 重置本地对局中的英雄位置。
pub async fn reset_position(state: &Arc<LocalGameState>, id_str: &str) -> MatchResult<()> {
    with_session(state, id_str, |client| async move {
        client.reset_position().await.map_err(|e| e.to_string())?;
        Ok(())
    })
    .await
}

/// 切换本地对局中的调试视角英雄。
pub async fn switch_champion(
    state: &Arc<LocalGameState>,
    id_str: &str,
    name: &str,
) -> MatchResult<()> {
    let name = name.to_string();
    with_session(state, id_str, move |client| async move {
        client
            .switch_champion(&name)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    })
    .await
}

/// 设置本地对局中某个角色的运行脚本代码。
pub async fn set_script(
    state: &Arc<LocalGameState>,
    id_str: &str,
    entity_id: u64,
    source: &str,
) -> MatchResult<()> {
    let source = source.to_string();
    with_session(state, id_str, move |client| async move {
        client
            .set_script(entity_id, &source)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    })
    .await
}

// ── 事件订阅 ──

/// 订阅本地对局的实时事件流。
///
/// 返回 `mpsc::Receiver`，调用方可通过它接收 `serde_json::Value` 格式的事件。
/// 已启动的事件转发循环（在 process_service 的 `connect_and_subscribe` 中）会自动
/// 向所有注册的 sender 推送事件。
pub fn subscribe_match_events(
    state: &Arc<LocalGameState>,
    id_str: &str,
) -> MatchResult<mpsc::Receiver<serde_json::Value>> {
    let id = Uuid::parse_str(id_str).map_err(|e| format!("无效对局 id: {e}"))?;
    let (tx, rx) = mpsc::channel::<serde_json::Value>(128);
    let mut channels = state
        .event_channels
        .lock()
        .map_err(|e| format!("锁获取失败: {e}"))?;
    channels.entry(id).or_insert_with(Vec::new).push(tx);
    Ok(rx)
}
