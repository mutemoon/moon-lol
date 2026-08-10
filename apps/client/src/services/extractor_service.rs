//! 提取任务服务：spawn `lol_extractor` worker 子进程，流式解析 stdio JSON 进度行。
//!
//! worker 协议：stdout 每行一个 `{"step": u8, "kind": "log"|"status", "msg": string}`，
//! step 与本文件 `ExtractionStep` 对齐。退出码 0 = 成功，非 0 = 失败（错误写 stderr）。

use std::process::Stdio;

use serde::Deserialize;
use tokio::io::AsyncBufReadExt;
use tokio::sync::mpsc;

use super::assets_path::resolve_assets_dir;
use super::runtime::run_on_tokio;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtractionStep {
    GitSync = 0,
    BaseAndUi = 1,
    Audio = 2,
    Shaders = 3,
    Complete = 4,
}

#[derive(Debug, Clone)]
pub struct ExtractionConfig {
    pub game_path: String,
    pub extract_base_and_ui: bool,
    pub extract_shaders: bool,
    pub extract_audio: bool,
    pub skip_map_geo: bool,
}

impl Default for ExtractionConfig {
    fn default() -> Self {
        Self {
            game_path: r"D:\WeGameApps\英雄联盟\Game".to_string(),
            extract_base_and_ui: true,
            extract_shaders: false,
            extract_audio: false,
            skip_map_geo: false,
        }
    }
}

#[derive(Deserialize)]
struct WorkerProgress {
    step: u8,
    kind: String,
    msg: String,
}

fn step_from_u8(v: u8) -> ExtractionStep {
    match v {
        0 => ExtractionStep::GitSync,
        1 => ExtractionStep::BaseAndUi,
        2 => ExtractionStep::Audio,
        3 => ExtractionStep::Shaders,
        _ => ExtractionStep::Complete,
    }
}

/// 据运行环境决定提取 worker 程序与前缀：dev `cargo run -p lol_extractor`，release 解析兄弟二进制。
fn extractor_command() -> (String, Vec<String>) {
    lol_client::launch::resolve_executable("lol_extractor", "lol_extractor")
}

/// 运行提取任务：spawn worker 子进程，把 stdout JSON 进度解析后推送给 UI 通道。
pub async fn run_extraction_task(
    config: ExtractionConfig,
    log_tx: mpsc::UnboundedSender<(ExtractionStep, String)>,
    step_tx: mpsc::UnboundedSender<(ExtractionStep, String)>,
) -> Result<(), String> {
    let assets_dir = resolve_assets_dir();
    let (program, prefix_args) = extractor_command();

    let mut cmd = std::process::Command::new(&program);
    cmd.args(&prefix_args)
        .arg("--game-path")
        .arg(&config.game_path)
        .arg("--assets-dir")
        .arg(&assets_dir);
    // bool 标志位：仅在勾选时传递（未勾选保持 worker 默认 false）
    if config.skip_map_geo {
        cmd.arg("--skip-map-geo");
    }
    if config.extract_base_and_ui {
        cmd.arg("--extract-base");
    }
    if config.extract_audio {
        cmd.arg("--extract-audio");
    }
    if config.extract_shaders {
        cmd.arg("--extract-shaders");
    }
    if let Some(root) = lol_client::launch::install_root() {
        cmd.current_dir(root);
    }
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

    run_on_tokio(move || async move {
        let mut child = tokio::process::Command::from(cmd)
            .spawn()
            .map_err(|e| format!("启动提取 worker 失败: {e}"))?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "无法读取提取 worker 输出".to_string())?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| "无法读取提取 worker 错误输出".to_string())?;

        // 排空 stderr（worker 内部库的 println!/eprintln! 可能很多，不读会阻塞）
        let stderr_drain = tokio::spawn(async move {
            let mut lines = tokio::io::BufReader::new(stderr).lines();
            let mut err = Vec::new();
            while let Ok(Some(line)) = lines.next_line().await {
                err.push(line);
            }
            err
        });

        let mut lines = tokio::io::BufReader::new(stdout).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            if let Ok(prog) = serde_json::from_str::<WorkerProgress>(&line) {
                let step = step_from_u8(prog.step);
                match prog.kind.as_str() {
                    "status" => {
                        let _ = step_tx.send((step, prog.msg));
                    }
                    _ => {
                        let _ = log_tx.send((step, prog.msg));
                    }
                }
            }
        }

        let status = child
            .wait()
            .await
            .map_err(|e| format!("等待提取 worker 退出失败: {e}"))?;
        if !status.success() {
            let err_detail = stderr_drain.await.unwrap_or_default().join("\n");
            return Err(format!(
                "提取 worker 退出码: {:?}\n{}",
                status.code(),
                err_detail
            ));
        }
        Ok(())
    })
    .await
}
