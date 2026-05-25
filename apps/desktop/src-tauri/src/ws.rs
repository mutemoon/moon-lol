use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, oneshot};
use tauri::{AppHandle, Emitter};
use futures_util::{StreamExt, SinkExt};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::Message;
use std::sync::atomic::{AtomicU64, Ordering};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WsRequest {
    pub id: u64,
    #[serde(rename = "type")]
    pub msg_type: String,
    pub cmd: String,
    pub params: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WsResponse {
    pub id: u64,
    #[serde(rename = "type")]
    pub msg_type: String,
    pub ok: bool,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WsEvent {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub event: String,
    pub data: serde_json::Value,
}

#[derive(Clone)]
pub struct WsSession {
    tx: mpsc::Sender<Message>,
    pending: Arc<Mutex<HashMap<u64, oneshot::Sender<WsResponse>>>>,
    next_id: Arc<AtomicU64>,
}

impl WsSession {
    pub async fn send_cmd(&self, cmd: String, params: serde_json::Value) -> Result<WsResponse, String> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);

        let req = WsRequest {
            id,
            msg_type: "cmd".to_string(),
            cmd,
            params,
        };

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

pub async fn start_ws_client(app: AppHandle, port: u16) -> Result<WsSession, String> {
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
    let pending = Arc::new(Mutex::new(HashMap::<u64, oneshot::Sender<WsResponse>>::new()));
    let next_id = Arc::new(AtomicU64::new(1));

    let pending_clone = pending.clone();
    let app_clone = app.clone();

    // 写入循环
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if write.send(msg).await.is_err() {
                break;
            }
        }
    });

    // 读取循环
    tokio::spawn(async move {
        while let Some(msg_res) = read.next().await {
            let Ok(msg) = msg_res else { break; };
            let Message::Text(text) = msg else { continue; };

            if let Ok(resp) = serde_json::from_str::<WsResponse>(&text) {
                if resp.msg_type == "result" {
                    let mut pending_lock = pending_clone.lock().unwrap();
                    if let Some(tx) = pending_lock.remove(&resp.id) {
                        let _ = tx.send(resp);
                    }
                    continue;
                }
            }

            if let Ok(event) = serde_json::from_str::<WsEvent>(&text) {
                if event.msg_type == "event" {
                    let _ = app_clone.emit("ws-event", &event);
                    continue;
                }
            }

            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&text) {
                let _ = app_clone.emit("ws-event", &val);
            }
        }

        // 意外关闭或断开时通知前端
        let close_event = WsEvent {
            msg_type: "event".to_string(),
            event: "game_close".to_string(),
            data: serde_json::Value::Object(serde_json::Map::new()),
        };
        let _ = app_clone.emit("ws-event", &close_event);
    });

    Ok(WsSession {
        tx,
        pending,
        next_id,
    })
}
