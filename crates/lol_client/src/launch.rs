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

/// workspace 根目录（基于编译期 `CARGO_MANIFEST_DIR`；`lol_client` 位于 `crates/lol_client`，parent×2 即根）。
pub fn workspace_root() -> Option<PathBuf> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").ok()?;
    std::path::Path::new(&manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.to_path_buf())
}
