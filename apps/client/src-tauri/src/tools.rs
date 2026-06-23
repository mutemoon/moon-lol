use rig::completion::ToolDefinition;
use rig::tool::Tool;

#[derive(serde::Deserialize)]
pub struct BashArgs {
    pub cmd: String,
}

pub struct BashTool;

fn is_dev() -> bool {
    cfg!(debug_assertions)
}

fn workspace_root() -> Option<std::path::PathBuf> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").ok()?;
    std::path::Path::new(&manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .map(|p| p.to_path_buf())
}

impl Tool for BashTool {
    const NAME: &'static str = "bash";
    type Error = std::convert::Infallible;
    type Args = BashArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "bash".to_string(),
            description: "运行本地命令行指令。".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "cmd": {
                        "type": "string",
                        "description": "要执行的命令行指令"
                    }
                },
                "required": ["cmd"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let cmd_trimmed = args.cmd.trim();
        if !cmd_trimmed.starts_with("lol_cli") {
            return Ok("错误: 安全策略限制，只允许执行以 lol_cli 开头的命令。".to_string());
        }

        let mut final_cmd = args.cmd.clone();
        if is_dev() {
            if cmd_trimmed.starts_with("lol_cli") {
                final_cmd = final_cmd.replacen("lol_cli", "cargo run -p lol_cli --", 1);
            }
        }

        let output = {
            let mut cmd = {
                #[cfg(target_os = "windows")]
                {
                    let mut c = std::process::Command::new("cmd");
                    c.args(["/C", &final_cmd]);
                    c
                }
                #[cfg(not(target_os = "windows"))]
                {
                    let mut c = std::process::Command::new("sh");
                    c.args(["-c", &final_cmd]);
                    c
                }
            };

            if is_dev() {
                if let Some(root) = workspace_root() {
                    cmd.current_dir(root);
                }
            }
            cmd.output()
        };

        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                let stderr = String::from_utf8_lossy(&out.stderr).to_string();
                if out.status.success() {
                    Ok(stdout)
                } else {
                    Ok(format!(
                        "指令执行失败 (code={:?}):\nstdout: {}\nstderr: {}",
                        out.status.code(),
                        stdout,
                        stderr
                    ))
                }
            }
            Err(e) => Ok(format!("执行命令行时发生系统错误: {}", e)),
        }
    }
}
