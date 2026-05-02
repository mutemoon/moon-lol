use std::process::{Child, Command, Stdio};
use std::sync::Mutex;

use serde::Deserialize;
use tauri::Manager;

use crate::state::{AppState, BevyProcess};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GameConfig {
    pub mode: String,
    pub champion: String,
}

/// Find the workspace root directory.
fn workspace_root() -> Result<std::path::PathBuf, String> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default();
    // apps/desktop/src-tauri → apps/desktop → apps → workspace root
    std::path::Path::new(&manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .map(|p| p.to_path_buf())
        .ok_or("cannot find workspace root".into())
}

/// Check if we're in dev mode (no bundled binary available).
fn is_dev(app: &tauri::AppHandle) -> bool {
    #[cfg(target_os = "windows")]
    let binary_name = "lol.exe";
    #[cfg(not(target_os = "windows"))]
    let binary_name = "lol";

    let resource_dir = app
        .path()
        .resource_dir()
        .map(|p| p.join("bin").join(binary_name));
    !resource_dir.is_ok_and(|p| p.exists())
}

/// Start the Bevy game process.
pub fn start_game(
    state: &Mutex<AppState>,
    app: &tauri::AppHandle,
    config: GameConfig,
) -> Result<(), String> {
    let mut s = state.lock().map_err(|e| e.to_string())?;

    if s.bevy.is_some() {
        return Err("game already running".into());
    }

    let port: u16 = 9001;
    let root = workspace_root()?;

    let child = if is_dev(app) {
        start_dev(&root, port, &config)?
    } else {
        start_release(app, &root, port, &config)?
    };

    s.bevy = Some(BevyProcess { child, port });
    println!("[tauri] Bevy process started on port {}", port);
    Ok(())
}

/// Dev mode: use `cargo run` to handle all DLL/PATH automatically.
fn start_dev(
    root: &std::path::Path,
    port: u16,
    config: &GameConfig,
) -> Result<Child, String> {
    println!(
        "[tauri] Dev: cargo run --example lol -- --ws-port {} --mode {} --champion {}",
        port, config.mode, config.champion
    );

    Command::new("cargo")
        .current_dir(root)
        .args(["run", "--example", "lol", "--"])
        .arg("--ws-port")
        .arg(port.to_string())
        .arg("--mode")
        .arg(&config.mode)
        .arg("--champion")
        .arg(&config.champion)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("failed to start game: {e}"))
}

/// Release mode: run the bundled binary directly.
fn start_release(
    app: &tauri::AppHandle,
    root: &std::path::Path,
    port: u16,
    config: &GameConfig,
) -> Result<Child, String> {
    #[cfg(target_os = "windows")]
    let binary_name = "lol.exe";
    #[cfg(not(target_os = "windows"))]
    let binary_name = "lol";

    let binary = app
        .path()
        .resource_dir()
        .map_err(|e| e.to_string())?
        .join("bin")
        .join(binary_name);

    println!(
        "[tauri] Release: {} --ws-port {} --mode {} --champion {}",
        binary.display(),
        port,
        config.mode,
        config.champion
    );

    Command::new(&binary)
        .current_dir(root)
        .arg("--ws-port")
        .arg(port.to_string())
        .arg("--mode")
        .arg(&config.mode)
        .arg("--champion")
        .arg(&config.champion)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("failed to start game: {e}"))
}

/// Stop the Bevy game process.
pub fn stop_game(state: &Mutex<AppState>) -> Result<(), String> {
    let mut s = state.lock().map_err(|e| e.to_string())?;

    if let Some(mut proc) = s.bevy.take() {
        println!("[tauri] Stopping Bevy process");
        let _ = proc.child.kill();
    }

    Ok(())
}
