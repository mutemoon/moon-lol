use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use lol_rl_protocol::{VisualInFrame, VisualOutFrame};
use tokio::sync::mpsc;
use tokio::time::sleep;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

/// Events pushed from the visual WS connection to the UI.
pub enum VisualWsEvent {
    Connected,
    Disconnected,
    Frame(VisualOutFrame),
}

/// Spawn a background task that connects to the visual WS server and forwards frames.
/// Returns a sender for sending `VisualInFrame` commands to the visual process.
pub fn spawn_visual_ws(
    port: u16,
    event_tx: mpsc::UnboundedSender<VisualWsEvent>,
) -> mpsc::UnboundedSender<VisualInFrame> {
    let (cmd_tx, mut cmd_rx) = mpsc::unbounded_channel::<VisualInFrame>();

    std::thread::spawn(move || {
        let rt = match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            Ok(rt) => rt,
            Err(_) => return,
        };

        rt.block_on(async move {
            let ws_url = format!("ws://127.0.0.1:{port}");

            // 子进程（cargo run + Bevy 初始化 + 资产加载）需要时间启动，失败后重试
            let (mut write, mut read) = loop {
                match connect_async(&ws_url).await {
                    Ok((ws, _)) => {
                        let _ = event_tx.send(VisualWsEvent::Connected);
                        break ws.split();
                    }
                    Err(_) => {
                        let _ = event_tx.send(VisualWsEvent::Disconnected);
                        sleep(Duration::from_millis(500)).await;
                    }
                }
            };

            let read_handle = {
                let event_tx = event_tx.clone();
                tokio::spawn(async move {
                    while let Some(Ok(msg)) = read.next().await {
                        if let Message::Binary(data) = msg {
                            if let Ok(frame) = bincode::deserialize::<VisualOutFrame>(&data) {
                                if event_tx.send(VisualWsEvent::Frame(frame)).is_err() {
                                    break;
                                }
                            }
                        }
                    }
                })
            };

            let write_handle = tokio::spawn(async move {
                while let Some(cmd) = cmd_rx.recv().await {
                    if let Ok(data) = bincode::serialize(&cmd) {
                        if write.send(Message::Binary(data.into())).await.is_err() {
                            break;
                        }
                    }
                }
            });

            tokio::select! {
                _ = read_handle => {},
                _ = write_handle => {},
            }

            let _ = event_tx.send(VisualWsEvent::Disconnected);
        });
    });

    cmd_tx
}
