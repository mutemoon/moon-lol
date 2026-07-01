//! Bevy 游戏进程启动的共享纯知识：CLI 参数拼装、默认 RUST_LOG、workspace 根定位、二进制名。
//!
//! 桌面端（`std::process`）与云端（`tokio::process`）的 spawn 机制不同，但参数词汇表、
//! RUST_LOG 默认值、workspace_root 探测、二进制名是共享的，归一于此。各端自行决定
//! `cargo run` 前缀（桌面 `cargo run --`、云端 `cargo run --bin moon_lol --`）与 spawn 方式。

use std::path::PathBuf;

/// Bevy 进程的游戏参数（CLI 标志）。`None` 的字段不产出对应标志，以兼容桌面端
/// （传 mode/champion/scene）与云端 headless（仅 ws-port + headless）两种调用面。
#[derive(Debug, Clone, Default)]
pub struct BevyGameConfig {
    pub mode: Option<String>,
    pub champion: Option<String>,
    /// 场景名（原始名，自动包装成 `user_games://{name}.ron`）。
    pub scene: Option<String>,
    pub headless: bool,
}

/// 拼装 Bevy 进程的 CLI 参数（`--ws-port` 之后的游戏标志，不含 `cargo run --` 前缀）。
pub fn bevy_args(port: u16, cfg: &BevyGameConfig) -> Vec<String> {
    let mut args = vec!["--ws-port".to_string(), port.to_string()];
    if let Some(mode) = &cfg.mode {
        args.push("--mode".into());
        args.push(mode.clone());
    }
    if let Some(champion) = &cfg.champion {
        args.push("--champion".into());
        args.push(champion.clone());
    }
    if let Some(scene) = &cfg.scene {
        args.push("--scene".into());
        args.push(format!("user_games://{}.ron", scene));
    }
    if cfg.headless {
        args.push("--headless".into());
    }
    args
}

/// 默认 RUST_LOG（桌面端 dev/release 两处共用）。
pub fn default_rust_log() -> &'static str {
    "info,lol_core=debug,lol_server=debug,lol_champions=debug,lol_render=debug,moon_lol=debug"
}

/// 打包二进制名（按目标平台）。
pub fn binary_name() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        "lol.exe"
    }
    #[cfg(not(target_os = "windows"))]
    {
        "lol"
    }
}

/// workspace 根目录（开发阶段从 `CARGO_MANIFEST_DIR` 向上遍历寻找含有 rust-toolchain.toml 或 pnpm-workspace.yaml 的目录作为根）。
pub fn workspace_root() -> Option<PathBuf> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").ok()?;
    let mut path = std::path::PathBuf::from(manifest_dir);
    loop {
        if path.join("rust-toolchain.toml").exists() || path.join("pnpm-workspace.yaml").exists() {
            return Some(path);
        }
        if let Some(parent) = path.parent() {
            path = parent.to_path_buf();
        } else {
            break;
        }
    }
    None
}

/// 启动一个 Bevy 进程所需的可配置项。
///
/// 桌面端与云端的差异（dev/release 二进制、`cargo run` 前缀、cwd、RUST_LOG、
/// 是否 headless）全部收敛到这里；[`build_command`] 据此构建配置好的（未 spawn 的）
/// `std::process::Command`，stdio=null、env、cwd、program、前缀、游戏参数只此一份。
/// 桌面端同步 `.spawn()` 得 `std::process::Child`；
/// 云端用 `tokio::process::Command::from(cmd).spawn()` 得 `tokio::process::Child`。
#[derive(Debug, Clone)]
pub struct BevySpawnRequest {
    /// 可执行程序：dev 为 `cargo`，release 为打包二进制路径。
    pub program: String,
    /// 程序名之后的固定前缀（如 `["run", "--"]` 或 `["run", "--bin", "moon_lol", "--"]`）。
    pub prefix_args: Vec<String>,
    pub port: u16,
    pub game_config: BevyGameConfig,
    /// 工作目录；`None` 表示沿用调用进程的 cwd。
    pub cwd: Option<PathBuf>,
    /// RUST_LOG 值；`None` 表示不设置（沿用进程环境）。
    pub rust_log: Option<String>,
    /// 每局日志 SQLite 路径；`None` 时 Bevy 进程沿用默认 `~/.moon-lol/logs/debug.db`。
    pub log_db: Option<PathBuf>,
}

/// 据请求构建配置好的（未 spawn 的）`std::process::Command`。
///
/// 配置：stdout/stderr=null、可选 cwd、可选 RUST_LOG、program、前缀、`bevy_args`。
pub fn build_command(req: &BevySpawnRequest) -> std::process::Command {
    let mut cmd = std::process::Command::new(&req.program);
    cmd.args(&req.prefix_args)
        .args(bevy_args(req.port, &req.game_config))
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    if let Some(cwd) = &req.cwd {
        cmd.current_dir(cwd);
    }
    if let Some(rust_log) = &req.rust_log {
        cmd.env("RUST_LOG", rust_log);
    }
    if let Some(log_db) = &req.log_db {
        cmd.arg("--log-db").arg(log_db);
    }
    cmd
}
