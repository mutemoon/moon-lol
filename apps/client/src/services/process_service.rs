//! 本地对局进程托管服务。
//!
//! 复用 `lol_game_process_manager::GameProcessManager` 做进程生命周期（端口池、spawn/kill），
//! 复用 `lol_client` 做 WS 会话连接与会话存储。
//!
//! 桌面端本地进程启动器实现 `ProcessLauncher` trait：
//! - dev 模式（`CARGO` 环境变量存在）：`cargo run --` + 预编译
//! - release 模式（打包发布）：工作区 target 下的二进制

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use lol_client::launch::{build_command, install_root, resolve_executable, BevySpawnRequest};
use lol_game_process_manager::{GameProcessManager, ManagerError, ProcessLauncher, StartGameInput};
use lol_web_protocol::{FrontAgentConfig, GameConfig, RunningGame};
use tokio::sync::{mpsc, Mutex};
use uuid::Uuid;

use super::types::LocalGameState;

// ── 辅助函数 ──

/// 桌面端配置目录：`~/.moon-lol/`，不存在时自动创建。
fn config_dir() -> Result<std::path::PathBuf, String> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|e| format!("无法获取 HOME 目录: {e}"))?;
    let dir = std::path::PathBuf::from(home).join(".moon-lol");
    std::fs::create_dir_all(&dir).map_err(|e| format!("创建配置目录失败: {e}"))?;
    Ok(dir)
}

/// 把 `GameConfig` 转成 `BevyGameConfig`（非 headless，带场景）。
fn bevy_game_config(config: &GameConfig) -> lol_client::launch::BevyGameConfig {
    lol_client::launch::BevyGameConfig {
        mode: Some(config.mode.clone()),
        champion: Some(config.champion.clone()),
        scene: config.scene_name.clone(),
        headless: false,
    }
}

/// 写入 Bevy 动态场景 RON 文件到 `~/.moon-lol/games/{scene_name}.ron`。
pub fn write_scene_ron(scene_name: &str, agents: &[FrontAgentConfig]) -> Result<(), String> {
    let dir = config_dir()?.join("games");
    std::fs::create_dir_all(&dir).map_err(|e| format!("创建 games 目录失败: {e}"))?;

    let ron_path = dir.join(format!("{}.ron", scene_name));

    let mut resolved_agents = Vec::new();
    for (idx, agent) in agents.iter().enumerate() {
        let mut resolved = agent.clone();
        if resolved.id.is_none() {
            let champ_lower = agent.champion.to_lowercase();
            resolved.id = Some(format!("{}_{}", champ_lower, idx));
        }
        resolved_agents.push(resolved);
    }

    let mut ron_content = String::new();
    ron_content.push_str("(\n    resources: {},\n    entities: {\n");

    for (idx, agent) in resolved_agents.iter().enumerate() {
        let entity_id = 4294967185 + idx as u64;
        let x = agent.spawn_point[0];
        let z = agent.spawn_point[1];
        let y = if agent.champion == "Fiora" { 38.0 } else { 0.0 };
        let team = &agent.team;
        let champ_lower = agent.champion.to_lowercase();
        let agent_id = agent.id.as_ref().unwrap();

        ron_content.push_str(&format!("        {entity_id}: (\n"));
        ron_content.push_str("            components: {\n");
        ron_content
            .push_str("                \"bevy_transform::components::transform::Transform\": (\n");
        ron_content.push_str(&format!(
            "                    translation: ({:.1}, {:.1}, {:.1}),\n",
            x, y, z
        ));
        ron_content.push_str("                    rotation: (0.0, 0.0, 0.0, 1.0),\n");
        ron_content.push_str("                    scale: (1.0, 1.0, 1.0),\n");
        ron_content.push_str("                ),\n");
        ron_content.push_str(&format!(
            "                \"lol_core::team::Team\": {},\n",
            team
        ));
        ron_content.push_str(&format!(
            "                \"lol_champions::{}::{}\": (),\n",
            champ_lower, agent.champion
        ));
        if idx == 0 {
            ron_content.push_str("                \"lol_render::controller::Controller\": (),\n");
            ron_content.push_str("                \"lol_render::controller::SelfPlayer\": (),\n");
            ron_content.push_str("                \"lol_base_render::camera::Focus\": (),\n");
        }
        ron_content.push_str("                \"lol_core::entities::champion::Champion\": (),\n");
        ron_content.push_str(&format!(
            "                \"lol_core::entities::champion::AgentId\": (\"{}\"),\n",
            agent_id
        ));
        ron_content.push_str("                \"lol_base::character::ConfigCharacterRecord\": (\n");
        ron_content.push_str(&format!(
            "                    character_record: Path(\"characters/{}/config.ron\"),\n",
            champ_lower
        ));
        ron_content.push_str("                ),\n");
        ron_content.push_str("                \"lol_base::character::ConfigSkin\": (\n");
        ron_content.push_str(&format!(
            "                    skin: Path(\"characters/{}/skins/skin0.ron\"),\n",
            champ_lower
        ));
        ron_content.push_str(&format!(
            "                    vfx: Path(\"characters/{}/skins/skin0_vfx.ron\"),\n",
            champ_lower
        ));
        ron_content.push_str("                ),\n");
        ron_content.push_str("            },\n");
        ron_content.push_str("        ),\n");
    }

    ron_content.push_str("    },\n)\n");

    std::fs::write(&ron_path, ron_content).map_err(|e| format!("写入场景 RON 失败: {e}"))?;
    Ok(())
}

/// 每局日志 SQLite 路径：`~/.moon-lol/logs/{id}.db`，确保父目录存在。
fn log_db_path_for(id: Uuid) -> Result<std::path::PathBuf, String> {
    let dir = config_dir()?.join("logs");
    std::fs::create_dir_all(&dir).map_err(|e| format!("创建 logs 目录失败: {e}"))?;
    Ok(dir.join(format!("{id}.db")))
}

/// 默认 RUST_LOG（dev/release 共用）。
fn rust_log() -> String {
    std::env::var("RUST_LOG").unwrap_or_else(|_| lol_client::launch::default_rust_log().to_string())
}

/// 据环境决定程序与前缀：dev `cargo run -p moon_lol`，release 解析兄弟二进制。
fn program_and_prefix() -> (String, Vec<String>) {
    resolve_executable("moon_lol", "moon_lol")
}

// ── GPUI 端进程启动器 ──

/// 桌面端进程启动实现（GPUI 版，不依赖 Tauri）：
/// dev `cargo run --`（含预编译）/ release 二进制、tokio spawn、按 port 维护子进程表。
pub struct GpuiProcessLauncher {
    processes: Mutex<HashMap<i32, tokio::process::Child>>,
}

impl GpuiProcessLauncher {
    pub fn new() -> Self {
        Self {
            processes: Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl ProcessLauncher for GpuiProcessLauncher {
    async fn launch(&self, port: i32, req: &BevySpawnRequest) -> Result<(), ManagerError> {
        let (program, prefix_args) = program_and_prefix();
        let mut req = req.clone();
        req.program = program;
        req.prefix_args = prefix_args;
        req.port = port as u16;

        // cargo 运行前预编译
        if req.program == "cargo" {
            let build_cwd = req.cwd.clone().or_else(lol_client::launch::workspace_root);
            let mut build_cmd = tokio::process::Command::new("cargo");
            build_cmd.args(["build", "--bin", "moon_lol"]);
            if let Some(cwd) = build_cwd {
                build_cmd.current_dir(cwd);
            }

            tracing::info!("开发模式：预编译 Bevy 游戏服务端 cargo build --bin moon_lol");
            let status = build_cmd
                .status()
                .await
                .map_err(|e| ManagerError::Internal(format!("执行 cargo build 失败: {e}")))?;
            if !status.success() {
                return Err(ManagerError::Internal(format!(
                    "cargo 编译失败，无法启动对局进程。退出码: {:?}",
                    status.code()
                )));
            }
            tracing::info!("cargo 编译完成，准备启动对局进程");
        }

        let child = tokio::process::Command::from(build_command(&req))
            .spawn()
            .map_err(|e| ManagerError::Internal(format!("启动游戏进程失败: {e}")))?;

        let mut procs = self.processes.lock().await;
        procs.insert(port, child);
        Ok(())
    }

    async fn kill(&self, port: i32) -> Result<(), ManagerError> {
        let mut procs = self.processes.lock().await;
        if let Some(mut child) = procs.remove(&port) {
            let _ = child.kill().await;
        }
        Ok(())
    }
}

// ── 进程服务 ──

/// 本地对局进程服务：进程托管 + WS 会话管理。
///
/// 创建时通过 `GpuiProcessLauncher` 连接 `GameProcessManager`；
/// 不含 AI 决策环（`AgentRunner`），后续页面接线时按需注入。
pub struct ProcessService {
    pub manager: Arc<GameProcessManager>,
    pub state: Arc<LocalGameState>,
}

impl ProcessService {
    pub fn new() -> Self {
        let launcher = Arc::new(GpuiProcessLauncher::new());
        let manager = Arc::new(GameProcessManager::new(launcher, None));
        Self {
            manager,
            state: Arc::new(LocalGameState::new()),
        }
    }

    fn map_manager_error(e: ManagerError) -> String {
        use ManagerError as E;
        match e {
            E::NotFound => "对局不存在".into(),
            E::Conflict(msg) => msg,
            E::Validation(msg) => msg,
            E::Internal(msg) => msg,
        }
    }

    /// 启动一局本地游戏：分配端口、spawn Bevy 进程、登记进程表。
    /// 有场景 agent 时自动写入场景 RON，自动建立 WS 连接并登记会话。
    pub async fn start(&self, config: GameConfig) -> Result<RunningGame, String> {
        let manager = self.manager.clone();
        let state = self.state.clone();
        super::runtime::run_on_tokio(move || async move {
            let id = Uuid::new_v4();
            let log_db = log_db_path_for(id)?;

            let mut scenario_agents = config.agents.clone().unwrap_or_default();
            for (idx, agent) in scenario_agents.iter_mut().enumerate() {
                if agent.id.is_none() {
                    let champ_lower = agent.champion.to_lowercase();
                    agent.id = Some(format!("{}_{}", champ_lower, idx));
                }
            }

            if let Some(scene_name) = &config.scene_name {
                if !scenario_agents.is_empty() {
                    write_scene_ron(scene_name, &scenario_agents)?;
                }
            }

            let scenario_agents_runtime: Vec<lol_agent_runtime::AgentConfig> = scenario_agents
                .iter()
                .map(|a| lol_agent_runtime::AgentConfig {
                    id: a.id.clone().unwrap_or_default(),
                    champion: a.champion.clone(),
                    team: a.team.clone(),
                    prompt: a.prompt.clone(),
                    model: a.model.clone(),
                    provider_id: a.provider_id.clone(),
                })
                .collect();

            let spawn = BevySpawnRequest {
                program: String::new(), // 由 launcher 覆写
                prefix_args: vec![],
                port: 0, // 由 manager 覆写
                game_config: bevy_game_config(&config),
                cwd: install_root(),
                rust_log: Some(rust_log()),
                log_db: Some(log_db),
            };
            let input = StartGameInput {
                id,
                spawn,
                scenario_agents: scenario_agents_runtime,
            };
            let (_proc_id, port) = manager
                .start(input)
                .await
                .map_err(Self::map_manager_error)?;
            tracing::info!("[client] 游戏进程启动 id={id} port={port}");

            // 自动建立与游戏进程的 WS 连接并登记会话
            let session = connect_and_subscribe(state.clone(), id, port as u16).await?;
            state
                .ws_sessions
                .lock()
                .map_err(|e| format!("锁获取失败: {e}"))?
                .insert(id, session);

            Ok(RunningGame {
                id: id.to_string(),
                port,
                status: "running".into(),
            })
        })
        .await
    }

    /// 按 id 停止对局：kill 进程 + 释放端口 + 清理该端口的 WS 会话。
    pub async fn stop(&self, id_str: &str) -> Result<(), String> {
        let manager = self.manager.clone();
        let state = self.state.clone();
        let id_str = id_str.to_string();
        super::runtime::run_on_tokio(move || async move {
            let id = Uuid::parse_str(&id_str).map_err(|e| format!("无效对局 id: {e}"))?;
            manager.stop(id).await.map_err(Self::map_manager_error)?;
            state
                .ws_sessions
                .lock()
                .map_err(|e| format!("锁获取失败: {e}"))?
                .remove(&id);
            state
                .event_channels
                .lock()
                .map_err(|e| format!("锁获取失败: {e}"))?
                .remove(&id);
            Ok(())
        })
        .await
    }

    /// 列出所有运行中的本地对局。
    pub async fn list(&self) -> Result<Vec<RunningGame>, String> {
        let manager = self.manager.clone();
        super::runtime::run_on_tokio(move || async move {
            let procs = manager
                .list_processes()
                .await
                .map_err(Self::map_manager_error)?;
            Ok(procs
                .into_iter()
                .map(|p| RunningGame {
                    id: p.id.to_string(),
                    port: p.port,
                    status: p.status.as_str().to_string(),
                })
                .collect())
        })
        .await
    }

    /// 按 id 查询单个运行中对局。
    pub async fn get(&self, id_str: &str) -> Result<Option<RunningGame>, String> {
        let manager = self.manager.clone();
        let id_str = id_str.to_string();
        super::runtime::run_on_tokio(move || async move {
            let id = Uuid::parse_str(&id_str).map_err(|e| format!("无效对局 id: {e}"))?;
            let procs = manager
                .list_processes()
                .await
                .map_err(Self::map_manager_error)?;
            Ok(procs.into_iter().find(|p| p.id == id).map(|p| RunningGame {
                id: p.id.to_string(),
                port: p.port,
                status: p.status.as_str().to_string(),
            }))
        })
        .await
    }
}

// ── WS 连接辅助 ──

/// 连接游戏 WS 服务端，启动事件转发循环：把 `lol_client::start_ws_client` 推送的事件
/// 转发给所有订阅该对局的 `mpsc::Sender`。
async fn connect_and_subscribe(
    state: Arc<LocalGameState>,
    match_id: Uuid,
    port: u16,
) -> Result<lol_client::WsSession, String> {
    let (event_tx, mut event_rx) = mpsc::channel::<serde_json::Value>(128);
    let session = lol_client::start_ws_client(port, Some(event_tx)).await?;

    // 事件转发循环：先收集 sender 列表再发送，避免 MutexGuard 跨 await 导致 !Send
    let channels = state.event_channels.clone();
    tokio::spawn(async move {
        while let Some(val) = event_rx.recv().await {
            let senders: Vec<mpsc::Sender<serde_json::Value>> = {
                let lock = channels.lock().unwrap();
                lock.get(&match_id).cloned().unwrap_or_default()
            };

            let mut keep = Vec::new();
            for tx in &senders {
                if tx.send(val.clone()).await.is_ok() {
                    keep.push(tx.clone());
                }
            }

            let mut lock = channels.lock().unwrap();
            if let Some(subs) = lock.get_mut(&match_id) {
                *subs = keep;
            }
        }
    });

    Ok(session)
}
