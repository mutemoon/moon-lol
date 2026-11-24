//! Bevy 游戏进程启动的共享纯知识：CLI 参数拼装、默认 RUST_LOG、workspace 根定位、二进制名。
//!
//! 桌面端（`std::process`）与云端（`tokio::process`）的 spawn 机制不同，但参数词汇表、
//! RUST_LOG 默认值、workspace_root 探测、二进制名是共享的，归一于此。各端自行决定
//! `cargo run` 前缀（桌面 `cargo run --`、云端 `cargo run --bin moon_lol --`）与 spawn 方式。

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Bevy 进程的游戏参数（CLI 标志）。`None` 的字段不产出对应标志，以兼容桌面端
/// （传 mode/champion/scene）与云端 headless（仅 ws-port + headless）两种调用面。
#[derive(Debug, Clone, Default)]
pub struct BevyGameConfig {
    pub mode: Option<String>,
    pub champion: Option<String>,
    /// 场景名（原始名，自动包装成 `user_games://{name}.ron`）。
    pub scene: Option<String>,
    pub headless: bool,
    /// 房间配置档路径：游戏进程读它的 `pause_at` 执行暂停计划。
    pub profile: Option<PathBuf>,
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
        args.push(format!("user_games://{scene}.ron"));
    }
    if let Some(profile) = &cfg.profile {
        args.push("--profile".into());
        args.push(profile.display().to_string());
    }
    if cfg.headless {
        args.push("--headless".into());
    }
    args
}

/// 房间配置档：由 `lol room start` 写到 `~/.moon-lol/rooms/<id>/config.ron`，
/// 游戏进程以 `--profile` 读回。只承载启动期决定的东西。
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct RoomProfile {
    /// 游戏内时间（秒）到点自动暂停，便于稳定截图/取快照。
    #[serde(default)]
    pub pause_at: Vec<f64>,
}

impl RoomProfile {
    /// 读取配置档；文件缺失或损坏时视为「无计划」。
    pub fn load(path: &Path) -> Self {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|text| ron::from_str(&text).ok())
            .unwrap_or_default()
    }

    /// 写配置档（父目录自动创建）。
    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let text = ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())
            .map_err(std::io::Error::other)?;
        std::fs::write(path, text)
    }
}

/// 默认 RUST_LOG（桌面端 dev/release 两处共用）。
pub fn default_rust_log() -> &'static str {
    "info,lol_core=debug,lol_server=debug,lol_champions=debug,lol_render=debug,moon_lol=debug"
}

/// 运行环境探测：dev（`cargo run`/`cargo test` 会设置 `CARGO`）与否。
pub fn is_dev() -> bool {
    std::env::var("CARGO").is_ok() || workspace_root().is_some()
}

/// 安装根目录：dev = workspace 根；release = 当前可执行文件所在目录。
///
/// 子进程默认 cwd，以及 release 打包后与 client 同目录的兄弟二进制查找都以此为准。
pub fn install_root() -> Option<PathBuf> {
    if is_dev() {
        workspace_root()
    } else {
        std::env::current_exe().ok()?.parent().map(PathBuf::from)
    }
}

/// 平台可执行文件名（Windows 追加 `.exe`）。
fn executable_name(bin: &str) -> String {
    if cfg!(target_os = "windows") {
        format!("{bin}.exe")
    } else {
        bin.to_string()
    }
}

/// 解析一个工作区子进程为「程序 + 前缀参数」，dev/release 通用。
///
/// - dev：`cargo run -p <pkg> --bin <bin> --`
/// - release：优先当前可执行文件同级目录（打包布局，client 与各 worker 同目录），
///   其次 workspace `target/release/`，最后 PATH。
pub fn resolve_executable(pkg: &str, bin: &str) -> (String, Vec<String>) {
    if is_dev() {
        return (
            "cargo".to_string(),
            vec![
                "run".into(),
                "-p".into(),
                pkg.into(),
                "--bin".into(),
                bin.into(),
                "--".into(),
            ],
        );
    }
    let exe_name = executable_name(bin);
    if let Some(dir) = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(PathBuf::from))
    {
        let cand = dir.join(&exe_name);
        if cand.exists() {
            return (cand.display().to_string(), vec![]);
        }
    }
    if let Some(ws) = workspace_root() {
        let cand = ws.join("target").join("release").join(&exe_name);
        if cand.exists() {
            return (cand.display().to_string(), vec![]);
        }
    }
    (bin.to_string(), vec![])
}

/// workspace 根目录：优先 `CARGO_MANIFEST_DIR`（`cargo run`/`cargo test` 自动设置），
/// 否则从当前可执行文件所在的 `target/<profile>/` 向上定位。
pub fn workspace_root() -> Option<PathBuf> {
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        if let Some(root) = find_workspace_root(&PathBuf::from(manifest_dir)) {
            return Some(root);
        }
    }
    let exe_dir = std::env::current_exe().ok()?.parent().map(PathBuf::from)?;
    find_workspace_root(&exe_dir)
}

/// 从 `start` 逐级向上找含 `rust-toolchain.toml`/`pnpm-workspace.yaml` 的目录。
fn find_workspace_root(start: &std::path::Path) -> Option<PathBuf> {
    let mut path = start.to_path_buf();
    loop {
        if path.join("rust-toolchain.toml").exists() || path.join("pnpm-workspace.yaml").exists() {
            return Some(path);
        }
        path = path.parent()?.to_path_buf();
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_roundtrip_through_disk() {
        let dir = std::env::temp_dir().join(format!("lol-room-profile-{}", std::process::id()));
        let path = dir.join("config.ron");
        RoomProfile {
            pause_at: vec![10.0, 13.5],
        }
        .save(&path)
        .unwrap();
        assert_eq!(RoomProfile::load(&path).pause_at, vec![10.0, 13.5]);
        assert!(
            RoomProfile::load(&dir.join("missing.ron"))
                .pause_at
                .is_empty()
        );
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn bevy_args_include_profile() {
        let args = bevy_args(
            9100,
            &BevyGameConfig {
                champion: Some("riven".into()),
                profile: Some(PathBuf::from("C:/tmp/config.ron")),
                ..Default::default()
            },
        );
        assert_eq!(
            args,
            vec![
                "--ws-port",
                "9100",
                "--champion",
                "riven",
                "--profile",
                "C:/tmp/config.ron"
            ]
        );
    }
}
