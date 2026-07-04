use std::collections::HashMap;
use std::path::PathBuf;

use async_trait::async_trait;
use lol_client::launch::{
    binary_name, build_command, default_rust_log, BevyGameConfig, BevySpawnRequest,
};
use lol_game_process_manager::{ManagerError, ProcessLauncher};
use serde::Deserialize;
use tauri::Manager;
use tokio::sync::Mutex;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GameConfig {
    pub mode: String,
    pub champion: String,
    pub scene_name: Option<String>,
    pub agents: Option<Vec<crate::FrontAgentConfig>>,
    pub providers: Option<Vec<crate::ModelProvider>>,
}

/// 把桌面端 GameConfig 转成共享的 BevyGameConfig（非 headless，带场景）。
pub fn game_config(config: &GameConfig) -> BevyGameConfig {
    BevyGameConfig {
        mode: Some(config.mode.clone()),
        champion: Some(config.champion.clone()),
        scene: config.scene_name.clone(),
        headless: false,
    }
}

/// 默认 RUST_LOG（桌面端 dev/release 共用）。
pub fn rust_log() -> String {
    std::env::var("RUST_LOG").unwrap_or_else(|_| default_rust_log().to_string())
}

/// workspace 根目录（复用 lol_client::launch::workspace_root）。
pub fn workspace_root() -> Option<PathBuf> {
    lol_client::launch::workspace_root()
}

/// 检查是否处于 dev 模式（无打包二进制可用）。
fn is_dev(app: &tauri::AppHandle) -> bool {
    let resource_dir = app
        .path()
        .resource_dir()
        .map(|p| p.join("bin").join(binary_name()));
    !resource_dir.is_ok_and(|p| p.exists())
}

/// 据环境决定程序与前缀：dev 用 `cargo run --`，release 用打包二进制。
fn program_and_prefix(app: &tauri::AppHandle) -> (String, Vec<String>) {
    if is_dev(app) {
        ("cargo".to_string(), vec!["run".into(), "--".into()])
    } else {
        let binary = app
            .path()
            .resource_dir()
            .map(|p| p.join("bin").join(binary_name()))
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| binary_name().to_string());
        (binary, Vec::new())
    }
}

/// 桌面端进程启动实现：dev `cargo run --` / release 打包二进制、非 headless、
/// tokio spawn、按 port 维护子进程表供 kill。spawn 命令构建复用 `lol_client::launch::build_command`。
pub struct DesktopProcessLauncher {
    app: tauri::AppHandle,
    processes: Mutex<HashMap<i32, tokio::process::Child>>,
}

impl DesktopProcessLauncher {
    pub fn new(app: tauri::AppHandle) -> Self {
        Self {
            app,
            processes: Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl ProcessLauncher for DesktopProcessLauncher {
    async fn launch(&self, port: i32, req: &BevySpawnRequest) -> Result<(), ManagerError> {
        let (program, prefix_args) = program_and_prefix(&self.app);
        let mut req = req.clone();
        req.program = program;
        req.prefix_args = prefix_args;
        req.port = port as u16;

        if req.program == "cargo" {
            let mut build_args = vec!["build".to_string()];
            if req.prefix_args.contains(&"--release".to_string()) {
                build_args.push("--release".to_string());
            }
            build_args.push("--bin".to_string());
            build_args.push("moon_lol".to_string());

            let mut build_cmd = tokio::process::Command::new("cargo");
            build_cmd.args(&build_args);

            let build_cwd = req.cwd.clone().or_else(lol_client::launch::workspace_root);
            if let Some(cwd) = build_cwd {
                build_cmd.current_dir(cwd);
            }

            tracing::info!(
                "开发模式检测到 cargo 运行，正在预编译 Bevy 游戏服务端: cargo {:?}",
                build_args
            );
            match build_cmd.status().await {
                Ok(status) if status.success() => {
                    tracing::info!("cargo 编译完成，准备启动对局进程");
                }
                Ok(status) => {
                    return Err(ManagerError::Internal(format!(
                        "cargo 编译失败，无法启动对局进程。退出码: {:?}",
                        status.code()
                    )));
                }
                Err(e) => {
                    return Err(ManagerError::Internal(format!(
                        "执行 cargo build 失败: {e}"
                    )));
                }
            }
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

/// 每局日志 SQLite 路径：`~/.moon-lol/logs/{id}.db`，确保父目录存在。
pub fn log_db_path_for(app: &tauri::AppHandle, id: uuid::Uuid) -> Result<PathBuf, String> {
    let home = app.path().home_dir().map_err(|e| e.to_string())?;
    let dir = home.join(".moon-lol").join("logs");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.join(format!("{id}.db")))
}
