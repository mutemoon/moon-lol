use std::time::Duration;

use bevy::prelude::*;
use bevy::winit::{UpdateMode, WinitSettings};
use lol_champions::riven::PluginRiven;
use lol_core::PluginCore;
use lol_core::game::PluginGame;
use lol_core::log::create_log_plugin;
use lol_render::PluginRender;

fn main() {
    let (log_plugin, _log_rx) = create_log_plugin();

    App::new()
        .add_plugins((
            DefaultPlugins.build().set(log_plugin).set(WindowPlugin {
                primary_window: Some(Window {
                    title: "锐雯技能测试".to_string(),
                    // resolution: (300, 300).into(),
                    // position: WindowPosition::At((0, 1000).into()),
                    ..default()
                }),
                ..default()
            }),
            PluginCore.set(PluginGame {
                scenes: vec!["games/riven.ron".to_owned()],
            }),
            PluginRender,
            PluginRiven,
        ))
        .insert_resource(WinitSettings {
            focused_mode: UpdateMode::Reactive {
                wait: Duration::MAX,           // 不超时，只靠你触发
                react_to_device_events: false, // 不理会输入设备事件
                react_to_user_events: false,
                react_to_window_events: false,
            },
            unfocused_mode: UpdateMode::Reactive {
                wait: Duration::MAX,
                react_to_device_events: false, // 不理会输入设备事件
                react_to_user_events: false,
                react_to_window_events: false,
            },
        })
        .run();
}
