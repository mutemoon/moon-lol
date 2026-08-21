//! 粒子 WS 客户端 + 资产读取服务。
//!
//! 粒子渲染 server（lol_particle，默认 9002）只负责「播放」：唯一输入是一段
//! `ConfigVfxSystemDefinition` 的 RON 字符串。英雄列表与英雄粒子从本地资产读取
//! （assets/characters/*/skins/skin0_vfx.ron），WS 层只传 play/stop 控制命令。
//!
//! WS 协议 JSON RPC：请求 `{id, cmd, params}`，响应 `{id, type:"result", ok, data, error}`。

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use lol_share::ConfigVfxSystemDefinition;
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, oneshot};

use super::runtime::tokio_runtime;
use super::ws_bridge::{run_auto_reconnect_loop, SendOutcome};
use crate::components::sidebar::AppSidebar;

pub const DEFAULT_PARTICLE_SERVER_URL: &str = "ws://127.0.0.1:9002";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(8);

// ── 公开类型 ──

#[derive(Clone, Debug)]
pub struct ParticleSystemDef {
    pub hash: u32,
    pub name: String,
    pub def_ron: String,
    pub def: ConfigVfxSystemDefinition,
}

#[derive(Clone, Debug)]
pub enum ParticleWsEvent {
    Connected,
    Disconnected { error: Option<String> },
}

/// WS 客户端句柄（Clone），用于从 UI 发送 play/stop 命令。
#[derive(Clone)]
pub struct ParticleWsHandle {
    cmd_tx: mpsc::UnboundedSender<WsCmd>,
    is_connected: Arc<AtomicBool>,
}

impl ParticleWsHandle {
    /// 是否当前已建立 WebSocket 连接。
    pub fn is_connected(&self) -> bool {
        self.is_connected.load(Ordering::SeqCst)
    }

    /// 播放粒子：把 RON 定义发给 server。在 cx.spawn 内 await。
    pub async fn play_particle(&self, def_ron: &str) -> Result<(), String> {
        if !self.is_connected() {
            return Err("粒子渲染服务未连接 (ws://127.0.0.1:9002)".to_string());
        }
        let cmd_tx = self.cmd_tx.clone();
        let def_ron = def_ron.to_string();
        // request_via 内部依赖 tokio::time::timeout，必须在全局 tokio runtime 内跑。
        super::runtime::run_on_tokio(move || async move {
            request_via(
                &cmd_tx,
                "play_particle",
                serde_json::json!({"def": def_ron}),
            )
            .await?;
            Ok(())
        })
        .await
    }

    /// 停止粒子播放。在 cx.spawn 内 await。
    pub async fn stop_particle(&self) -> Result<(), String> {
        if !self.is_connected() {
            return Err("粒子渲染服务未连接 (ws://127.0.0.1:9002)".to_string());
        }
        let cmd_tx = self.cmd_tx.clone();
        super::runtime::run_on_tokio(move || async move {
            request_via(&cmd_tx, "stop_particle", serde_json::json!({})).await?;
            Ok(())
        })
        .await
    }
}

// ── 内部类型 ──

type WrappedResult = Result<serde_json::Value, String>;

enum WsCmd {
    Request {
        cmd: String,
        params: serde_json::Value,
        reply: oneshot::Sender<WrappedResult>,
    },
}

#[derive(Serialize)]
struct WsRequest {
    id: u64,
    cmd: String,
    params: serde_json::Value,
}

#[derive(Deserialize, Debug)]
struct WsResponse {
    id: u64,
    #[serde(rename = "type")]
    resp_type: String,
    ok: bool,
    data: Option<serde_json::Value>,
    error: Option<String>,
}

// ── 公开 API ──

/// 启动粒子 WS 后台长效自动重连服务，并绑定 UI 状态通知。
pub fn spawn_particle_service(
    entity_weak: gpui::WeakEntity<AppSidebar>,
    cx: &mut gpui::Context<AppSidebar>,
) -> ParticleWsHandle {
    let (event_tx, mut event_rx) = mpsc::unbounded_channel::<ParticleWsEvent>();
    let (cmd_tx, mut cmd_rx) = mpsc::unbounded_channel::<WsCmd>();
    let is_connected = Arc::new(AtomicBool::new(false));

    // 1. GPUI 线程监听事件并驱动 AppSidebar 状态
    let entity_weak_ui = entity_weak.clone();
    cx.spawn(move |_, cx: &mut gpui::AsyncApp| {
        let mut cx = cx.clone();
        async move {
            while let Some(event) = event_rx.recv().await {
                let _ = entity_weak_ui.update(&mut cx, |sidebar, cx| {
                    match event {
                        ParticleWsEvent::Connected => {
                            sidebar.particles.connected = true;
                            sidebar.particles.error = None;
                        }
                        ParticleWsEvent::Disconnected { error } => {
                            sidebar.particles.connected = false;
                            if let Some(e) = error {
                                sidebar.particles.error = Some(e);
                            }
                        }
                    }
                    cx.notify();
                });
            }
        }
    })
    .detach();

    // 2. Tokio 后台任务：无限自动重连
    let is_connected_bg = is_connected.clone();
    tokio_runtime().spawn(async move {
        let next_id = AtomicU64::new(1);
        let pending: Arc<Mutex<HashMap<u64, oneshot::Sender<WrappedResult>>>> =
            Arc::new(Mutex::new(HashMap::new()));

        let encode = {
            let pending = pending.clone();
            move |cmd: WsCmd| match cmd {
                WsCmd::Request { cmd, params, reply } => {
                    let id = next_id.fetch_add(1, Ordering::SeqCst);
                    pending.lock().unwrap().insert(id, reply);
                    let req = WsRequest { id, cmd, params };
                    match serde_json::to_string(&req) {
                        Ok(s) => SendOutcome::SendText(s),
                        Err(e) => {
                            if let Some(reply) = pending.lock().unwrap().remove(&id) {
                                let _ = reply.send(Err(format!("序列化失败: {e}")));
                            }
                            SendOutcome::Skip
                        }
                    }
                }
            }
        };

        let decode = {
            let pending = pending.clone();
            move |bytes: &[u8]| {
                let resp: WsResponse = match serde_json::from_slice(bytes) {
                    Ok(r) => r,
                    Err(_) => return None,
                };
                if resp.resp_type != "result" {
                    return None;
                }
                if let Some(reply) = pending.lock().unwrap().remove(&resp.id) {
                    if resp.ok {
                        let _ = reply.send(Ok(resp.data.unwrap_or(serde_json::Value::Null)));
                    } else {
                        let _ =
                            reply.send(Err(resp.error.unwrap_or_else(|| "请求失败".to_string())));
                    }
                }
                None
            }
        };

        let is_conn_1 = is_connected_bg.clone();
        let is_conn_2 = is_connected_bg.clone();
        let pending_disconn = pending.clone();
        let ev_tx_conn = event_tx.clone();
        let ev_tx_disconn = event_tx.clone();

        run_auto_reconnect_loop(
            || DEFAULT_PARTICLE_SERVER_URL.to_string(),
            &mut cmd_rx,
            event_tx,
            Duration::from_secs(2),
            move || {
                is_conn_1.store(true, Ordering::SeqCst);
                let _ = ev_tx_conn.send(ParticleWsEvent::Connected);
            },
            move || {
                is_conn_2.store(false, Ordering::SeqCst);
                for (_, reply) in pending_disconn.lock().unwrap().drain() {
                    let _ = reply.send(Err("粒子服务连接断开".to_string()));
                }
                let _ = ev_tx_disconn.send(ParticleWsEvent::Disconnected { error: None });
            },
            encode,
            decode,
        )
        .await;
    });

    ParticleWsHandle {
        cmd_tx,
        is_connected,
    }
}

/// 通过 cmd_tx 发送一条 RPC 请求并等待响应。
async fn request_via(
    cmd_tx: &mpsc::UnboundedSender<WsCmd>,
    cmd: &str,
    params: serde_json::Value,
) -> Result<(), String> {
    let (tx, rx) = oneshot::channel();
    cmd_tx
        .send(WsCmd::Request {
            cmd: cmd.to_string(),
            params,
            reply: tx,
        })
        .map_err(|_| "WS 通道已关闭".to_string())?;

    let result = tokio::time::timeout(REQUEST_TIMEOUT, rx)
        .await
        .map_err(|_| format!("命令 {cmd} 超时"))?
        .map_err(|_| "WS 已关闭".to_string())?;

    result.map(|_| ())
}

// ── 资产读取（同步） ──

pub fn characters_dir() -> Result<PathBuf, String> {
    Ok(super::assets_path::resolve_assets_dir().join("characters"))
}

/// 将 ConfigVfxSystemDefinition 序列化为 RON 字符串。
pub fn serialize_vfx_system(def: &ConfigVfxSystemDefinition) -> Result<String, String> {
    ron::ser::to_string(def).map_err(|e| format!("序列化 system 失败: {e}"))
}
