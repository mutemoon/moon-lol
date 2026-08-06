//! 粒子 WS 客户端 + 资产读取服务。
//!
//! 粒子渲染 server（lol_particle，默认 9002）只负责「播放」：唯一输入是一段
//! `ConfigVfxSystemDefinition` 的 RON 字符串。英雄列表与英雄粒子从本地资产读取
//! （assets/characters/*/skins/skin0_vfx.ron），WS 层只传 play/stop 控制命令。
//!
//! WS 协议 JSON RPC：请求 `{id, cmd, params}`，响应 `{id, type:"result", ok, data, error}`。

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use lol_client::launch::workspace_root;
use lol_share::{ConfigVfx, ConfigVfxSystemDefinition};
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, oneshot};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

pub const DEFAULT_WS_URL: &str = "ws://127.0.0.1:9002";
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
}

impl ParticleWsHandle {
    /// 播放粒子：把 RON 定义发给 server。在 cx.spawn 内 await。
    pub async fn play_particle(&self, def_ron: &str) -> Result<(), String> {
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
        let cmd_tx = self.cmd_tx.clone();
        super::runtime::run_on_tokio(move || async move {
            request_via(&cmd_tx, "stop_particle", serde_json::json!({})).await?;
            Ok(())
        })
        .await
    }

    /// 主动断开 WS（发送 Disconnect 命令让后台任务退出）。
    pub fn disconnect(&self) {
        let _ = self.cmd_tx.send(WsCmd::Disconnect);
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
    Disconnect,
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

/// 启动粒子 WS 连接。
///
/// 后台启动独立线程管理 WS 连接生命周期。
/// 返回客户端句柄（发送 play/stop 命令）和事件接收器（Connected / Disconnected）。
pub fn connect_to_particle_server(
    url: &str,
) -> (ParticleWsHandle, mpsc::UnboundedReceiver<ParticleWsEvent>) {
    let (cmd_tx, cmd_rx) = mpsc::unbounded_channel::<WsCmd>();
    let (event_tx, event_rx) = mpsc::unbounded_channel::<ParticleWsEvent>();
    let url = url.to_string();

    std::thread::spawn(move || {
        let rt = match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            Ok(rt) => rt,
            Err(e) => {
                let _ = event_tx.send(ParticleWsEvent::Disconnected {
                    error: Some(format!("无法创建 tokio runtime: {e}")),
                });
                return;
            }
        };
        rt.block_on(run_ws(&url, cmd_rx, event_tx));
    });

    (ParticleWsHandle { cmd_tx }, event_rx)
}

// ── WS 后台主循环 ──

async fn run_ws(
    url: &str,
    mut cmd_rx: mpsc::UnboundedReceiver<WsCmd>,
    event_tx: mpsc::UnboundedSender<ParticleWsEvent>,
) {
    let (mut ws_write, mut ws_read) = match connect_async(url).await {
        Ok((ws, _)) => {
            let _ = event_tx.send(ParticleWsEvent::Connected);
            ws.split()
        }
        Err(e) => {
            let _ = event_tx.send(ParticleWsEvent::Disconnected {
                error: Some(format!("无法连接到粒子渲染 server: {e}")),
            });
            drain_commands(cmd_rx).await;
            return;
        }
    };

    let next_id = AtomicU64::new(1);
    let pending: Arc<Mutex<HashMap<u64, oneshot::Sender<WrappedResult>>>> =
        Arc::new(Mutex::new(HashMap::new()));

    // 读任务：解析 server 响应，匹配 pending map
    let pending_read = pending.clone();
    let pending_cleanup = pending.clone();
    let read_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = ws_read.next().await {
            let text = match msg {
                Message::Text(t) => t,
                Message::Close(_) => break,
                _ => continue,
            };
            let resp: WsResponse = match serde_json::from_str(&text) {
                Ok(r) => r,
                Err(_) => continue,
            };
            if resp.resp_type != "result" {
                continue;
            }
            if let Some(reply) = pending_read.lock().unwrap().remove(&resp.id) {
                if resp.ok {
                    let _ = reply.send(Ok(resp.data.unwrap_or(serde_json::Value::Null)));
                } else {
                    let _ = reply.send(Err(resp.error.unwrap_or_else(|| "请求失败".to_string())));
                }
            }
        }
    });

    // 写任务：从 cmd_rx 取命令，序列化为 JSON 发送
    let write_task = tokio::spawn(async move {
        while let Some(cmd) = cmd_rx.recv().await {
            match cmd {
                WsCmd::Disconnect => break,
                WsCmd::Request { cmd, params, reply } => {
                    let id = next_id.fetch_add(1, Ordering::SeqCst);
                    pending.lock().unwrap().insert(id, reply);

                    let req = WsRequest { id, cmd, params };
                    let payload = match serde_json::to_string(&req) {
                        Ok(s) => s,
                        Err(e) => {
                            if let Some(reply) = pending.lock().unwrap().remove(&id) {
                                let _ = reply.send(Err(format!("序列化失败: {e}")));
                            }
                            continue;
                        }
                    };
                    if ws_write.send(Message::Text(payload.into())).await.is_err() {
                        break;
                    }
                }
            }
        }
    });

    tokio::select! {
        _ = read_task => {}
        _ = write_task => {}
    }

    // 清理未完成的请求
    for (_, reply) in pending_cleanup.lock().unwrap().drain() {
        let _ = reply.send(Err("连接已关闭".to_string()));
    }

    let _ = event_tx.send(ParticleWsEvent::Disconnected { error: None });
}

/// 连接失败或关闭后清空命令队列，对所有命令回复错误。
async fn drain_commands(mut cmd_rx: mpsc::UnboundedReceiver<WsCmd>) {
    while let Some(cmd) = cmd_rx.recv().await {
        if let WsCmd::Request { reply, .. } = cmd {
            let _ = reply.send(Err("未连接".to_string()));
        }
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

fn characters_dir() -> Result<PathBuf, String> {
    let root = workspace_root().ok_or_else(|| "找不到 workspace 根目录".to_string())?;
    Ok(root.join("assets").join("characters"))
}

/// 列出所有带 skin0_vfx.ron 的英雄（名称升序）。
pub fn list_particle_heroes() -> Result<Vec<String>, String> {
    let base = characters_dir()?;
    let read_dir =
        std::fs::read_dir(&base).map_err(|e| format!("读取 {} 失败: {e}", base.display()))?;

    let mut out = Vec::new();
    for entry in read_dir.flatten() {
        if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if base
            .join(&name)
            .join("skins")
            .join("skin0_vfx.ron")
            .is_file()
        {
            out.push(name);
        }
    }
    out.sort();
    Ok(out)
}

/// 加载某英雄的 skin0_vfx.ron，解析为 ConfigVfx，返回每个 system 的 RON 字符串。
pub fn load_hero_particles(hero: &str) -> Result<Vec<ParticleSystemDef>, String> {
    let vfx_path = characters_dir()?
        .join(hero)
        .join("skins")
        .join("skin0_vfx.ron");
    if !vfx_path.is_file() {
        return Err(format!("英雄 {hero} 不存在 skin0_vfx.ron"));
    }

    let content = std::fs::read_to_string(&vfx_path)
        .map_err(|e| format!("读取 {} 失败: {e}", vfx_path.display()))?;
    let config: ConfigVfx =
        ron::from_str(&content).map_err(|e| format!("解析 {hero} 的 ConfigVfx 失败: {e}"))?;

    let mut systems = Vec::with_capacity(config.systems.len());
    for (&hash, def) in &config.systems {
        let def_ron =
            ron::ser::to_string(def).map_err(|e| format!("序列化 system {hash:08x} 失败: {e}"))?;
        systems.push(ParticleSystemDef {
            hash,
            name: def.particle_name.clone(),
            def_ron,
            def: def.clone(),
        });
    }
    systems.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(systems)
}

/// 将 ConfigVfxSystemDefinition 序列化为 RON 字符串。
pub fn serialize_vfx_system(def: &ConfigVfxSystemDefinition) -> Result<String, String> {
    ron::ser::to_string(def).map_err(|e| format!("序列化 system 失败: {e}"))
}
