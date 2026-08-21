//! WS 帧流客户端共享骨架：建连 + 读/写双循环 + 断开清理。
//!
//! RL WS（`ws.rs`）与视觉 WS（`visual_ws.rs`）是二进制帧流协议，
//! 粒子 WS（`particle_service.rs`）是 JSON-RPC 文本协议——三条链路的连接管理
//! （connect → split → 读写双端 select → 断开）完全一致，统一收敛到这里，
//! 各调用方只需提供编码/解码函数。

use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio::time::sleep;
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
    #[allow(dead_code)]
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
    mut on_connected: impl FnMut(),
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

/// 自动重连长连接循环：在连接断开或建连失败后按设定间隔自动重试。
///
/// - `url_fn`: 动态获取目标 WS URL 的闭包或字符串提供者。
/// - `cmd_rx`: 外部输入命令接收通道。
/// - `event_tx`: 向 UI 发送事件的通道。
/// - `retry_interval`: 重试间隔。
/// - `on_connected`: 每次建连成功的回调（例如通知 UI 已连接、发送初始查询帧等）。
/// - `on_disconnected`: 每次连接断开或建连失败时的回调（例如通知 UI 未连接、清理 pending RPC 等）。
/// - `encode`: 将业务命令转换为 `SendOutcome`。
/// - `decode`: 将接收到的字节转换为业务事件 `Ev`。
pub async fn run_auto_reconnect_loop<Cmd, Ev, FUrl, FConn, FDisconn, FEnc, FDec>(
    url_fn: FUrl,
    cmd_rx: &mut mpsc::UnboundedReceiver<Cmd>,
    event_tx: mpsc::UnboundedSender<Ev>,
    retry_interval: Duration,
    mut on_connected: FConn,
    mut on_disconnected: FDisconn,
    encode: FEnc,
    decode: FDec,
) where
    FUrl: Fn() -> String,
    FConn: FnMut(),
    FDisconn: FnMut(),
    FEnc: Fn(Cmd) -> SendOutcome,
    FDec: Fn(&[u8]) -> Option<Ev>,
{
    loop {
        let url = url_fn();
        let _ = run_frame_connection(
            &url,
            cmd_rx,
            event_tx.clone(),
            &mut on_connected,
            &encode,
            &decode,
        )
        .await;

        on_disconnected();
        sleep(retry_interval).await;
    }
}
