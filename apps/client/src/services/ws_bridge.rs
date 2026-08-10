//! WS 帧流客户端共享骨架：建连 + 读/写双循环 + 断开清理。
//!
//! RL WS（`ws.rs`）与视觉 WS（`visual_ws.rs`）是二进制帧流协议，
//! 粒子 WS（`particle_service.rs`）是 JSON-RPC 文本协议——三条链路的连接管理
//! （connect → split → 读写双端 select → 断开）完全一致，统一收敛到这里，
//! 各调用方只需提供编码/解码函数。

use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

/// 写端命令处理结果。
pub enum SendOutcome {
    /// 编码成功，发送二进制帧。
    Send(Vec<u8>),
    /// 编码成功，发送文本帧（JSON-RPC 等文本协议）。
    SendText(String),
    /// 编码失败/无需发送，跳过该命令。
    Skip,
    /// 终止连接（如显式 Disconnect）。
    Close,
}

/// 完成一次「建连 → 会话」。
///
/// - 连接失败返回 `false`（不推送任何事件，重试策略由调用方决定）。
/// - 连接成功调用 `on_connected`，随后运行读/写双循环直至任一端结束，返回 `true`。
/// - 读循环把每个帧的载荷交给 `decode`，返回的事件经 `event_tx` 推给 UI。
/// - 写循环从 `cmd_rx` 取命令，经 `encode` 编码后发送。
pub async fn run_frame_connection<Cmd, Ev>(
    url: &str,
    cmd_rx: &mut mpsc::UnboundedReceiver<Cmd>,
    event_tx: mpsc::UnboundedSender<Ev>,
    on_connected: impl Fn(),
    encode: impl Fn(Cmd) -> SendOutcome,
    decode: impl Fn(&[u8]) -> Option<Ev>,
) -> bool {
    let (mut write, mut read) = match connect_async(url).await {
        Ok((ws, _)) => ws.split(),
        Err(e) => {
            tracing::warn!("WS 连接失败: {e}");
            return false;
        }
    };
    on_connected();

    // 读循环：二进制/文本帧载荷统一转 bytes 交给 decode
    let read_loop = async {
        while let Some(msg) = read.next().await {
            let data = match msg {
                Ok(Message::Binary(d)) => d.to_vec(),
                Ok(Message::Text(t)) => t.as_bytes().to_vec(),
                Ok(Message::Close(_)) => break,
                Err(e) => {
                    tracing::warn!("WS 读取错误: {e}");
                    break;
                }
                _ => continue,
            };
            if let Some(ev) = decode(&data) {
                if event_tx.send(ev).is_err() {
                    break;
                }
            }
        }
    };

    // 写循环：命令经 encode 编码发送
    let write_loop = async {
        while let Some(cmd) = cmd_rx.recv().await {
            match encode(cmd) {
                SendOutcome::Send(bytes) => {
                    if write.send(Message::Binary(bytes.into())).await.is_err() {
                        break;
                    }
                }
                SendOutcome::SendText(text) => {
                    if write.send(Message::Text(text.into())).await.is_err() {
                        break;
                    }
                }
                SendOutcome::Skip => {}
                SendOutcome::Close => break,
            }
        }
    };

    tokio::select! {
        _ = read_loop => {}
        _ = write_loop => {}
    }
    true
}
