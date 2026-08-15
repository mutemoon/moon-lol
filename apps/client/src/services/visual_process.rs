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
pub async fn spawn_visual_env(
    checkpoint_path: &str,
    env_name: &str,
) -> anyhow::Result<(Child, u16)> {
    let port = pick_free_port().ok_or_else(|| anyhow::anyhow!("没有可用端口"))?;

    println!(
        ">>> 从客户端启动 lol_rl_visual (Port: {port}, Env: {env_name}, Checkpoint: {checkpoint_path})..."
    );

    let (program, prefix_args) = lol_client::launch::resolve_executable("lol_rl", "lol_rl_visual");
    let mut cmd = Command::new(&program);
    cmd.args(&prefix_args)
        .arg("--port")
        .arg(port.to_string())
        .arg("--checkpoint")
        .arg(checkpoint_path)
        .arg("--env")
        .arg(env_name)
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit());
    if let Some(root) = lol_client::launch::install_root() {
        cmd.current_dir(root);
    }

    let child = cmd.kill_on_drop(true).spawn()?;

    Ok((child, port))
}
