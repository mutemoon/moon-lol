use std::net::TcpListener;

use tokio::process::{Child, Command};

/// Find an available port on localhost.
pub fn pick_free_port() -> Option<u16> {
    TcpListener::bind("127.0.0.1:0")
        .ok()
        .and_then(|l| l.local_addr().ok())
        .map(|a| a.port())
}

/// Spawn the visual environment subprocess.
/// Returns the child process handle and the port it listens on.
pub async fn spawn_visual_env(checkpoint_path: &str) -> anyhow::Result<(Child, u16)> {
    let port = pick_free_port().ok_or_else(|| anyhow::anyhow!("没有可用端口"))?;

    let mut cmd = Command::new("cargo");
    cmd.args(["run", "-p", "lol_rl_visual", "--"])
        .arg("--port")
        .arg(port.to_string())
        .arg("--checkpoint")
        .arg(checkpoint_path);

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    let child = cmd.kill_on_drop(true).spawn()?;

    Ok((child, port))
}
