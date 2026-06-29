use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use futures_util::{SinkExt, StreamExt};
use tokio::sync::{mpsc, oneshot};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::Message;

use crate::protocol::{WsRequest, WsResponse};

/// 与游戏 WebSocket 服务端的会话。Clone 后共享同一连接与挂起请求表。
#[derive(Clone)]
pub struct WsSession {
    tx: mpsc::Sender<Message>,
    pending: Arc<Mutex<HashMap<u64, oneshot::Sender<WsResponse>>>>,
    next_id: Arc<AtomicU64>,
}

impl WsSession {
    /// 发送指令并等待对应 id 的响应（5s 超时）。
    pub async fn send_cmd(
        &self,
        cmd: String,
        params: serde_json::Value,
    ) -> Result<WsResponse, String> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);

        let req = WsRequest { id, cmd, params };

        let req_str = serde_json::to_string(&req).map_err(|e| e.to_string())?;
        let (rx_tx, rx) = oneshot::channel();
        {
            let mut pending_lock = self.pending.lock().unwrap();
            pending_lock.insert(id, rx_tx);
        }

        if let Err(e) = self.tx.send(Message::Text(req_str.into())).await {
            let mut pending_lock = self.pending.lock().unwrap();
            pending_lock.remove(&id);
            return Err(format!("发送 WS 写入任务失败: {}", e));
        }

        match tokio::time::timeout(std::time::Duration::from_secs(5), rx).await {
            Ok(Ok(res)) => Ok(res),
            Ok(Err(_)) => Err("通道已关闭".to_string()),
            Err(_) => {
                let mut pending_lock = self.pending.lock().unwrap();
                pending_lock.remove(&id);
                Err("指令执行超时".to_string())
            }
        }
    }
}

/// 连接游戏 WS 服务端。
///
/// `event_tx` 为可选事件通道：非响应（`result`）的消息会以原始 JSON 推入该通道，
/// 连接断开时会推送一条 `game_close` 事件。CLI / MCP 等不关心事件的消费方可传 `None`。
pub async fn start_ws_client(
    port: u16,
    event_tx: Option<mpsc::Sender<serde_json::Value>>,
) -> Result<WsSession, String> {
    let url = format!("ws://127.0.0.1:{}", port);
    let mut ws_stream = None;
    let start_time = std::time::Instant::now();

    while start_time.elapsed() < std::time::Duration::from_secs(30) {
        if let Ok((stream, _)) = connect_async(&url).await {
            ws_stream = Some(stream);
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
    }

    let Some(stream) = ws_stream else {
        return Err("无法连接至 Bevy WebSocket 服务端".to_string());
    };

    let (mut write, mut read) = stream.split();
    let (tx, mut rx) = mpsc::channel::<Message>(32);
    let pending = Arc::new(Mutex::new(
        HashMap::<u64, oneshot::Sender<WsResponse>>::new(),
    ));
    let next_id = Arc::new(AtomicU64::new(1));

    let pending_clone = pending.clone();

    // 写入循环
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if write.send(msg).await.is_err() {
                break;
            }
        }
    });

    // 读取循环：响应归还挂起调用者，其余作为事件推入 event_tx
    tokio::spawn(async move {
        while let Some(msg_res) = read.next().await {
            let Ok(msg) = msg_res else {
                break;
            };
            let Message::Text(text) = msg else {
                continue;
            };

            if let Ok(resp) = serde_json::from_str::<WsResponse>(&text) {
                if resp.msg_type == "result" {
                    let mut pending_lock = pending_clone.lock().unwrap();
                    if let Some(tx) = pending_lock.remove(&resp.id) {
                        let _ = tx.send(resp);
                    }
                    continue;
                }
            }

            if let Some(event_tx) = event_tx.as_ref() {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&text) {
                    let _ = event_tx.try_send(val);
                }
            }
        }

        // 连接断开时推送 game_close 事件
        if let Some(event_tx) = event_tx.as_ref() {
            let close = serde_json::json!({
                "type": "event",
                "event": "game_close",
                "data": {"reason": "connection closed"}
            });
            let _ = event_tx.try_send(close);
        }
    });

    Ok(WsSession {
        tx,
        pending,
        next_id,
    })
}
