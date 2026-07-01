// 协议类型与会话逻辑统一在 Bevy-free 的 lol_client 中，此处仅做 Tauri 事件桥接。
pub use lol_client::{start_ws_client as start_ws_client_inner, WsSession};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::ipc::Channel;
use tokio::sync::mpsc;
use uuid::Uuid;

type ChannelMap = Arc<Mutex<HashMap<Uuid, Vec<Channel<serde_json::Value>>>>>;

/// 连接游戏 WS 服务端，并把事件通过 state 中的 channels 列表推送给订阅的组件。
pub async fn start_ws_client(
    event_channels: ChannelMap,
    match_id: Uuid,
    port: u16,
) -> Result<WsSession, String> {
    let (event_tx, mut event_rx) = mpsc::channel::<serde_json::Value>(128);
    let session = start_ws_client_inner(port, Some(event_tx)).await?;

    // 事件转发循环：把 lol_client 推送的事件转发给订阅的 tauri::ipc::Channel
    tokio::spawn(async move {
        while let Some(val) = event_rx.recv().await {
            let mut channels_to_keep = Vec::new();
            let mut channels_lock = event_channels.lock().unwrap();
            if let Some(channels) = channels_lock.get_mut(&match_id) {
                for chan in channels.iter() {
                    if chan.send(val.clone()).is_ok() {
                        channels_to_keep.push(chan.clone());
                    }
                }
                *channels = channels_to_keep;
            }
        }
    });

    Ok(session)
}
