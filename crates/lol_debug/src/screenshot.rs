//! 截图命令：主窗口 → PNG 落盘。

use std::path::PathBuf;

use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured, save_to_disk};
use lol_rpc::{CommandWsRequest, RpcAppExt};
use serde::Deserialize;

#[derive(Deserialize, Default)]
pub struct ScreenshotParams {
    /// 目标 PNG 路径。
    pub path: PathBuf,
}

pub struct PluginScreenshot;

impl Plugin for PluginScreenshot {
    fn build(&self, app: &mut App) {
        app.register_rpc::<ScreenshotParams>("screenshot");
        app.add_observer(on_screenshot);
    }
}

/// 截图异步完成：响应在落盘观察者里回填，CLI 收到 `ok` 时文件已写完。
fn on_screenshot(event: On<CommandWsRequest<ScreenshotParams>>, mut commands: Commands) {
    let path = event.params.path.clone();
    if let Some(parent) = path.parent()
        && let Err(e) = std::fs::create_dir_all(parent)
    {
        lol_rpc::respond(&event, Err(format!("创建截图目录失败: {e}")));
        return;
    }

    let response = event.response.clone();
    let id = event.id;
    let saved = path.clone();
    commands.spawn(Screenshot::primary_window()).observe(
        move |captured: On<ScreenshotCaptured>| {
            save_to_disk(&saved)(captured);
            if let Ok(mut lock) = response.lock() {
                *lock = Some(lol_client::protocol::WsResponse::ok_with_data(
                    id,
                    serde_json::json!({ "path": saved.display().to_string() }),
                ));
            }
        },
    );
}
