// 协议类型与会话逻辑统一在 Bevy-free 的 lol_client 中，此处仅做 Tauri 事件桥接。
pub use lol_client::{start_ws_client as start_ws_client_inner, WsSession};
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc;

/// 连接游戏 WS 服务端，并把非响应消息（事件）转发给前端 `ws-event` 监听者。
pub async fn start_ws_client(app: AppHandle, port: u16) -> Result<WsSession, String> {
    let (event_tx, mut event_rx) = mpsc::channel::<serde_json::Value>(64);
    let session = start_ws_client_inner(port, Some(event_tx)).await?;

    // 事件转发循环：把 lol_client 推送的原始事件 JSON emit 给前端
    tokio::spawn(async move {
        while let Some(val) = event_rx.recv().await {
            let _ = app.emit("ws-event", &val);
        }
    });

    Ok(session)
}
