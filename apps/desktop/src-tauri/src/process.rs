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
    pub scene_name: Option<String>,
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

/// Returns the log database path at ~/.moon-lol/logs/debug.db
fn log_db_path(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    let base = app.path().home_dir().map_err(|e| e.to_string())?;
    let path = base.join(".moon-lol").join("logs").join("debug.db");
    // Ensure parent directories exist
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    Ok(path)
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

    // If the previous process died on its own, clear the stale handle.
    if let Some(ref mut proc) = s.bevy {
        match proc.child.try_wait() {
            Ok(Some(_)) => {
                println!("[tauri] Previous Bevy process already exited, clearing stale state");
                s.bevy = None;
            }
            Ok(None) => return Err("game already running".into()),
            Err(e) => {
                println!("[tauri] Error checking Bevy process status: {e}, clearing state");
                s.bevy = None;
            }
        }
    }

    let port: u16 = 9001;
    let root = workspace_root()?;
    let log_db_path = log_db_path(app)?;

    let child = if is_dev(app) {
        start_dev(&root, port, &config)?
    } else {
        start_release(app, &root, port, &config)?
    };

    s.bevy = Some(BevyProcess {
        child,
        port,
        log_db_path,
    });
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
        "[tauri] Dev: cargo run -- --ws-port {} --mode {} --champion {}",
        port, config.mode, config.champion
    );

    let rust_log = std::env::var("RUST_LOG").unwrap_or_else(|_| {
        "info,lol_core=debug,lol_server=debug,lol_champions=debug,lol_render=debug,moon_lol=debug".to_string()
    });

    let mut cmd = Command::new("cargo");
    cmd.current_dir(root)
        .args(["run", "--"])
        .arg("--ws-port")
        .arg(port.to_string())
        .arg("--mode")
        .arg(&config.mode)
        .arg("--champion")
        .arg(&config.champion);

    if let Some(ref scene) = config.scene_name {
        cmd.arg("--scene").arg(format!("user_games://{}.ron", scene));
    }

    cmd.env("RUST_LOG", &rust_log)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
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

    let rust_log = std::env::var("RUST_LOG").unwrap_or_else(|_| {
        "info,lol_core=debug,lol_server=debug,lol_champions=debug,lol_render=debug,moon_lol=debug".to_string()
    });

    let mut cmd = Command::new(&binary);
    cmd.current_dir(root)
        .arg("--ws-port")
        .arg(port.to_string())
        .arg("--mode")
        .arg(&config.mode)
        .arg("--champion")
        .arg(&config.champion);

    if let Some(ref scene) = config.scene_name {
        cmd.arg("--scene").arg(format!("user_games://{}.ron", scene));
    }

    cmd.env("RUST_LOG", &rust_log)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
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

    s.ws = None;

    Ok(())
}

