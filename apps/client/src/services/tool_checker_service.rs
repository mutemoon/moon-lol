//! 环境与二进制工具体检服务：异步探测 Git、Cargo、Worker、Node/NPX、dxbc-compiler、LoL 客户端等。
//!
//! 在 client 提取资源前进行系统环境与外部工具体检，确保工具就绪并提供友好的修复建议。

use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::time::timeout;

use super::assets_path::resolve_assets_dir;
use super::runtime::run_on_tokio;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolCategory {
    /// 基础提取必备（如 Git、提取 Worker、LoL Game 路径）
    Required,
    /// 推荐工具（如 Node.js / NPX 用于地图 Mesh / WebP 优化）
    Recommended,
    /// 着色器提取专用（如 dxbc-compiler.exe）
    ShaderSpecific,
    /// 可选扩展（如 FFmpeg 多媒体工具）
    Optional,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolHealthStatus {
    Checking,
    Passed,
    Warning,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCheckItem {
    pub id: String,
    pub name: String,
    pub category: ToolCategory,
    pub status: ToolHealthStatus,
    pub version_or_path: Option<String>,
    pub description: String,
    pub remedy_hint: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EnvironmentHealthReport {
    pub items: Vec<ToolCheckItem>,
    pub is_checking: bool,
    pub all_required_passed: bool,
    pub check_timestamp: Option<String>,
}

impl EnvironmentHealthReport {
    /// 检查所有必备项是否全部通过
    #[allow(dead_code)]
    pub fn is_ready_for_basic(&self) -> bool {
        self.items
            .iter()
            .filter(|item| item.category == ToolCategory::Required)
            .all(|item| item.status == ToolHealthStatus::Passed)
    }

    /// 统计各项状态数量 (passed, warning, failed)
    pub fn summary_counts(&self) -> (usize, usize, usize) {
        let mut passed = 0;
        let mut warning = 0;
        let mut failed = 0;
        for item in &self.items {
            match item.status {
                ToolHealthStatus::Passed => passed += 1,
                ToolHealthStatus::Warning => warning += 1,
                ToolHealthStatus::Failed => failed += 1,
                ToolHealthStatus::Checking => {}
            }
        }
        (passed, warning, failed)
    }
}

/// 执行命令并提取首行输出版本信息，带超时控制与 Windows cmd 兜底
async fn run_version_cmd(program: &str, args: &[&str]) -> Result<String, String> {
    let timeout_dur = Duration::from_secs(3);

    // 优先直接执行
    let direct_res = timeout(timeout_dur, async {
        let mut cmd = tokio::process::Command::new(program);
        cmd.args(args);
        #[cfg(target_os = "windows")]
        {
            // Windows 下隐藏控制台窗口
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            cmd.creation_flags(CREATE_NO_WINDOW);
        }
        cmd.output().await
    })
    .await;

    if let Ok(Ok(output)) = direct_res {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let first_line = stdout.lines().next().unwrap_or(&stdout).trim().to_string();
            if !first_line.is_empty() {
                return Ok(first_line);
            }
        }
    }

    // Windows 环境下通过 cmd /C 兜底（针对 .cmd/.bat 脚本如 npx）
    #[cfg(target_os = "windows")]
    {
        let cmd_str = format!("{} {}", program, args.join(" "));
        let cmd_res = timeout(timeout_dur, async {
            let mut cmd = tokio::process::Command::new("cmd");
            cmd.args(["/C", &cmd_str]);
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            cmd.creation_flags(CREATE_NO_WINDOW);
            cmd.output().await
        })
        .await;

        if let Ok(Ok(output)) = cmd_res {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
                let first_line = stdout.lines().next().unwrap_or(&stdout).trim().to_string();
                if !first_line.is_empty() {
                    return Ok(first_line);
                }
            }
        }
    }

    Err(format!("无法运行工具: {program}"))
}

/// 体检 1: Git
async fn check_git() -> ToolCheckItem {
    let res = run_version_cmd("git", &["--version"]).await;
    match res {
        Ok(ver) => ToolCheckItem {
            id: "git".into(),
            name: "Git 命令行工具".into(),
            category: ToolCategory::Required,
            status: ToolHealthStatus::Passed,
            version_or_path: Some(ver),
            description: "用于自动拉取与同步 CommunityDragon-Data 哈希字典仓库".into(),
            remedy_hint: None,
        },
        Err(_) => ToolCheckItem {
            id: "git".into(),
            name: "Git 命令行工具".into(),
            category: ToolCategory::Required,
            status: ToolHealthStatus::Failed,
            version_or_path: None,
            description: "用于自动拉取与同步 CommunityDragon-Data 哈希字典仓库".into(),
            remedy_hint: Some("未检测到 git，请安装 Git (https://git-scm.com/) 并配置环境变量 PATH".into()),
        },
    }
}

/// 体检 2: 提取 Worker 二进制 / Cargo
async fn check_worker() -> ToolCheckItem {
    let is_dev = lol_client::launch::is_dev();
    if is_dev {
        // Dev 模式：依赖 cargo 运行 lol_extractor
        let res = run_version_cmd("cargo", &["--version"]).await;
        match res {
            Ok(ver) => ToolCheckItem {
                id: "worker".into(),
                name: "提取 Worker (Dev / Cargo)".into(),
                category: ToolCategory::Required,
                status: ToolHealthStatus::Passed,
                version_or_path: Some(format!("{ver} (开发环境 cargo run -p lol_extractor)")),
                description: "资源提取核心管线后台 Worker 进程".into(),
                remedy_hint: None,
            },
            Err(_) => ToolCheckItem {
                id: "worker".into(),
                name: "提取 Worker (Dev / Cargo)".into(),
                category: ToolCategory::Required,
                status: ToolHealthStatus::Failed,
                version_or_path: None,
                description: "资源提取核心管线后台 Worker 进程".into(),
                remedy_hint: Some("开发模式下未检测到 Cargo，请安装 Rust 工具链 (https://rustup.rs/)".into()),
            },
        }
    } else {
        // Release 模式：依赖独立可执行文件 lol_extractor
        let (prog, _) = lol_client::launch::resolve_executable("lol_extractor", "lol_extractor");
        let path = PathBuf::from(&prog);
        if path.exists() {
            ToolCheckItem {
                id: "worker".into(),
                name: "提取 Worker (Release 二进制)".into(),
                category: ToolCategory::Required,
                status: ToolHealthStatus::Passed,
                version_or_path: Some(path.display().to_string()),
                description: "资源提取核心管线独立可执行文件 lol_extractor".into(),
                remedy_hint: None,
            }
        } else {
            ToolCheckItem {
                id: "worker".into(),
                name: "提取 Worker (Release 二进制)".into(),
                category: ToolCategory::Required,
                status: ToolHealthStatus::Failed,
                version_or_path: Some(prog),
                description: "资源提取核心管线独立可执行文件 lol_extractor".into(),
                remedy_hint: Some("未在客户端同级目录或 Release 目录找到 lol_extractor.exe".into()),
            }
        }
    }
}

/// 体检 3: LoL 游戏 Game 目录
async fn check_game_path(game_path: &str) -> ToolCheckItem {
    let p = Path::new(game_path);
    if game_path.trim().is_empty() {
        return ToolCheckItem {
            id: "game_path".into(),
            name: "英雄联盟 Game 根目录".into(),
            category: ToolCategory::Required,
            status: ToolHealthStatus::Failed,
            version_or_path: None,
            description: "游戏核心原始 WAD / 数据包所在目录".into(),
            remedy_hint: Some("请在配置中指定英雄联盟客户端 Game 目录".into()),
        };
    }

    if !p.exists() {
        return ToolCheckItem {
            id: "game_path".into(),
            name: "英雄联盟 Game 根目录".into(),
            category: ToolCategory::Required,
            status: ToolHealthStatus::Failed,
            version_or_path: Some(game_path.to_string()),
            description: "游戏核心原始 WAD / 数据包所在目录".into(),
            remedy_hint: Some(format!("路径不存在: {}", game_path)),
        };
    }

    // 检查核心标记文件/目录
    let exe_path = p.join(if cfg!(target_os = "windows") {
        "League of Legends.exe"
    } else {
        "League of Legends"
    });
    let data_path = p.join("DATA");
    let has_exe = exe_path.exists();
    let has_data = data_path.exists();

    if has_exe || has_data {
        ToolCheckItem {
            id: "game_path".into(),
            name: "英雄联盟 Game 根目录".into(),
            category: ToolCategory::Required,
            status: ToolHealthStatus::Passed,
            version_or_path: Some(format!("{} (检测到核心游戏数据)", p.display())),
            description: "游戏核心原始 WAD / 数据包所在目录".into(),
            remedy_hint: None,
        }
    } else {
        ToolCheckItem {
            id: "game_path".into(),
            name: "英雄联盟 Game 根目录".into(),
            category: ToolCategory::Required,
            status: ToolHealthStatus::Warning,
            version_or_path: Some(p.display().to_string()),
            description: "游戏核心原始 WAD / 数据包所在目录".into(),
            remedy_hint: Some("目录存在但未发现 League of Legends.exe 或 DATA 目录，提取可能无法找到完整 WAD".into()),
        }
    }
}

/// 体检 4: Node.js & NPX
async fn check_node_npx() -> ToolCheckItem {
    let npx_res = run_version_cmd("npx", &["--version"]).await;
    let node_res = run_version_cmd("node", &["--version"]).await;

    match (node_res, npx_res) {
        (Ok(node_ver), Ok(npx_ver)) => ToolCheckItem {
            id: "node_npx".into(),
            name: "Node.js & NPX".into(),
            category: ToolCategory::Recommended,
            status: ToolHealthStatus::Passed,
            version_or_path: Some(format!("Node: {}, NPX: {}", node_ver, npx_ver)),
            description: "用于调用 gltf-transform / gltf-pipeline 执行地图与 3D 模型 WebP 压缩后处理".into(),
            remedy_hint: None,
        },
        (Ok(node_ver), Err(_)) => ToolCheckItem {
            id: "node_npx".into(),
            name: "Node.js & NPX".into(),
            category: ToolCategory::Recommended,
            status: ToolHealthStatus::Warning,
            version_or_path: Some(format!("Node: {} (未检测到 NPX)", node_ver)),
            description: "用于调用 gltf-transform / gltf-pipeline 执行地图与 3D 模型 WebP 压缩后处理".into(),
            remedy_hint: Some("NPX 未就绪，若未开启 Fast Mode，地图后处理可能会被跳过".into()),
        },
        _ => ToolCheckItem {
            id: "node_npx".into(),
            name: "Node.js & NPX".into(),
            category: ToolCategory::Recommended,
            status: ToolHealthStatus::Warning,
            version_or_path: None,
            description: "用于调用 gltf-transform / gltf-pipeline 执行地图与 3D 模型 WebP 压缩后处理".into(),
            remedy_hint: Some("未检测到 Node.js / NPX。提取仍可进行，但建议安装 Node.js (https://nodejs.org/) 以支持模型 WebP 纹理优化".into()),
        },
    }
}

/// 体检 5: DXBC Compiler (Shader 编译工具)
async fn check_dxbc_compiler() -> ToolCheckItem {
    let assets_dir = resolve_assets_dir();
    let bin_name = if cfg!(target_os = "windows") {
        "dxbc-compiler.exe"
    } else {
        "dxbc-compiler"
    };
    let compiler_path = assets_dir.join("tools").join(bin_name);

    if compiler_path.exists() {
        ToolCheckItem {
            id: "dxbc_compiler".into(),
            name: "DXBC to SPIR-V Compiler".into(),
            category: ToolCategory::ShaderSpecific,
            status: ToolHealthStatus::Passed,
            version_or_path: Some(compiler_path.display().to_string()),
            description: "用于将 ShaderCache 中的 DXBC 字节码转译编译为 SPIR-V 着色器".into(),
            remedy_hint: None,
        }
    } else {
        ToolCheckItem {
            id: "dxbc_compiler".into(),
            name: "DXBC to SPIR-V Compiler".into(),
            category: ToolCategory::ShaderSpecific,
            status: ToolHealthStatus::Warning,
            version_or_path: Some(compiler_path.display().to_string()),
            description: "用于将 ShaderCache 中的 DXBC 字节码转译编译为 SPIR-V 着色器".into(),
            remedy_hint: Some(format!("未在 {} 找到 dxbc-compiler，若提取 Shader 则需放置该工具", compiler_path.display())),
        }
    }
}

/// 体检 6: FFmpeg (多媒体扩展)
async fn check_ffmpeg() -> ToolCheckItem {
    let res = run_version_cmd("ffmpeg", &["-version"]).await;
    match res {
        Ok(ver) => ToolCheckItem {
            id: "ffmpeg".into(),
            name: "FFmpeg 视频转码工具".into(),
            category: ToolCategory::Optional,
            status: ToolHealthStatus::Passed,
            version_or_path: Some(ver),
            description: "用于录制测试技能视频与渲染转码 (可选)".into(),
            remedy_hint: None,
        },
        Err(_) => ToolCheckItem {
            id: "ffmpeg".into(),
            name: "FFmpeg 视频转码工具".into(),
            category: ToolCategory::Optional,
            status: ToolHealthStatus::Passed, // 可选工具缺失不标红
            version_or_path: Some("未检测到 (可选)".into()),
            description: "用于录制测试技能视频与渲染转码 (可选)".into(),
            remedy_hint: None,
        },
    }
}

/// 执行全部二进制工具与环境体检（通过 run_on_tokio 桥接，兼容 gpui executor 上下文）
pub async fn run_environment_health_check(game_path: &str) -> EnvironmentHealthReport {
    let path = game_path.to_string();
    let res = run_on_tokio(move || async move {
        // 并发运行各项探针
        let (git, worker, game, node, dxbc, ffmpeg) = tokio::join!(
            check_git(),
            check_worker(),
            check_game_path(&path),
            check_node_npx(),
            check_dxbc_compiler(),
            check_ffmpeg()
        );

        let items = vec![git, worker, game, node, dxbc, ffmpeg];
        let all_required_passed = items
            .iter()
            .filter(|i| i.category == ToolCategory::Required)
            .all(|i| i.status == ToolHealthStatus::Passed);

        let timestamp = {
            let now = std::time::SystemTime::now();
            if let Ok(duration) = now.duration_since(std::time::UNIX_EPOCH) {
                let secs = duration.as_secs();
                let hour = (secs / 3600 + 8) % 24; // UTC+8
                let min = (secs % 3600) / 60;
                let sec = secs % 60;
                format!("{:02}:{:02}:{:02}", hour, min, sec)
            } else {
                "刚刚".to_string()
            }
        };

        Ok(EnvironmentHealthReport {
            items,
            is_checking: false,
            all_required_passed,
            check_timestamp: Some(timestamp),
        })
    })
    .await;

    res.unwrap_or_default()
}

/// 提取前校验：依据当前勾选的提取选项判定是否满足启动条件
pub fn validate_before_extraction(
    report: &EnvironmentHealthReport,
    extract_shaders: bool,
    _skip_map_geo: bool,
) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    for item in &report.items {
        // 1. 必备项未通过直接报错
        if item.category == ToolCategory::Required && item.status == ToolHealthStatus::Failed {
            let hint = item.remedy_hint.clone().unwrap_or_else(|| "工具缺失".into());
            errors.push(format!("【{}】未就绪: {}", item.name, hint));
        }

        // 2. 勾选了 Shader 提取但 dxbc-compiler 缺失
        if extract_shaders && item.id == "dxbc_compiler" && item.status != ToolHealthStatus::Passed {
            errors.push("【Shader 提取】需 dxbc-compiler 工具，请在 assets/tools 中放置 dxbc-compiler.exe".into());
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_report_summary_and_validation() {
        let mut report = EnvironmentHealthReport {
            items: vec![
                ToolCheckItem {
                    id: "git".into(),
                    name: "Git".into(),
                    category: ToolCategory::Required,
                    status: ToolHealthStatus::Passed,
                    version_or_path: Some("git version 2.40.0".into()),
                    description: "Git 工具".into(),
                    remedy_hint: None,
                },
                ToolCheckItem {
                    id: "worker".into(),
                    name: "Worker".into(),
                    category: ToolCategory::Required,
                    status: ToolHealthStatus::Passed,
                    version_or_path: Some("cargo 1.80.0".into()),
                    description: "提取 Worker".into(),
                    remedy_hint: None,
                },
                ToolCheckItem {
                    id: "game_path".into(),
                    name: "Game Path".into(),
                    category: ToolCategory::Required,
                    status: ToolHealthStatus::Passed,
                    version_or_path: Some("D:\\Games\\LoL".into()),
                    description: "游戏目录".into(),
                    remedy_hint: None,
                },
                ToolCheckItem {
                    id: "dxbc_compiler".into(),
                    name: "dxbc-compiler".into(),
                    category: ToolCategory::ShaderSpecific,
                    status: ToolHealthStatus::Warning,
                    version_or_path: None,
                    description: "Shader 编译工具".into(),
                    remedy_hint: Some("未找到 dxbc-compiler.exe".into()),
                },
            ],
            is_checking: false,
            all_required_passed: true,
            check_timestamp: Some("12:00:00".into()),
        };

        let (passed, warn, failed) = report.summary_counts();
        assert_eq!(passed, 3);
        assert_eq!(warn, 1);
        assert_eq!(failed, 0);

        // 未勾选 shader 提取时校验应通过
        let res_no_shader = validate_before_extraction(&report, false, false);
        assert!(res_no_shader.is_ok());

        // 勾选 shader 提取时由于 dxbc-compiler 处于 Warning/缺失，应当拦截并报错
        let res_with_shader = validate_before_extraction(&report, true, false);
        assert!(res_with_shader.is_err());

        // 当必需项失败时，无论如何都应拦截
        report.items[0].status = ToolHealthStatus::Failed;
        let res_failed_git = validate_before_extraction(&report, false, false);
        assert!(res_failed_git.is_err());
    }
}

