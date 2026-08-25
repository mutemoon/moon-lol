use std::time::Duration;

use lol_rl_protocol::{VisualInFrame, VisualOutFrame};
use tokio::sync::mpsc;
use tokio::time::sleep;

use super::runtime::tokio_runtime;
use super::ws_bridge::{run_frame_connection, SendOutcome};

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

    // 复用全局 tokio runtime 跑视觉 WS 后台任务（不阻塞 gpui 主线程）
    tokio_runtime().spawn(async move {
        let ws_url = format!("ws://127.0.0.1:{port}");

        // 子进程（cargo run + Bevy 初始化 + 资产加载）需要时间启动，失败后重试直到连上；
        // 会话结束（如子进程退出）则不再重连。
        loop {
            let connected = run_frame_connection(
                &ws_url,
                &mut cmd_rx,
                event_tx.clone(),
                || {
                    let _ = event_tx.send(VisualWsEvent::Connected);
                },
                |cmd: VisualInFrame| match bincode::serialize(&cmd) {
                    Ok(bytes) => SendOutcome::Send(bytes),
                    Err(_) => SendOutcome::Skip,
                },
                |bytes: &[u8]| {
                    match bincode::deserialize::<VisualOutFrame>(bytes) {
                        Ok(f) => Some(VisualWsEvent::Frame(f)),
                        Err(e) => {
                            eprintln!(">>> [Visual WS] 反序列化 VisualOutFrame 失败: {e}");
                            None
                        }
                    }
                },
            )
            .await;
            let _ = event_tx.send(VisualWsEvent::Disconnected);
            if connected {
                break;
            }
            sleep(Duration::from_millis(500)).await;
        }
    });

    cmd_tx
}
