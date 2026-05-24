use rig::completion::ToolDefinition;
use rig::tool::Tool;

#[derive(serde::Deserialize)]
pub struct BashArgs {
    pub cmd: String,
}

pub struct BashTool;

impl Tool for BashTool {
    const NAME: &'static str = "bash";
    type Error = std::convert::Infallible;
    type Args = BashArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "bash".to_string(),
            description: "运行本地命令行指令。例如使用 `cargo run --bin lol-cli -- obs` 获取局势观测，或者 `cargo run --bin lol-cli -- act move --x 7500 --y 7500` 移动坐标。".to_string(),
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
        if !cmd_trimmed.starts_with("lol_cli")
            && !cmd_trimmed.starts_with("lol-cli")
            && !cmd_trimmed.starts_with("cargo run --bin lol_cli")
            && !cmd_trimmed.starts_with("cargo run --bin lol-cli")
        {
            return Ok("错误: 安全策略限制，只允许执行以 lol_cli 开头的命令。".to_string());
        }

        let output = {
            #[cfg(target_os = "windows")]
            {
                std::process::Command::new("cmd")
                    .args(["/C", &args.cmd])
                    .output()
            }
            #[cfg(not(target_os = "windows"))]
            {
                std::process::Command::new("sh")
                    .args(["-c", &args.cmd])
                    .output()
            }
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
