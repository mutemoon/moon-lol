use std::time::Duration;

use bevy::prelude::*;
use bevy::winit::{UpdateMode, WinitSettings};
use moon_lol::{PluginBarrack, PluginCore, PluginResource};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.build().set(WindowPlugin {
                primary_window: Some(Window {
                    title: "锐雯技能测试".to_string(),
                    // resolution: (300, 300).into(),
                    // position: WindowPosition::At((0, 1000).into()),
                    ..default()
                }),
                ..default()
            }),
            PluginCore
                .build()
                .set(PluginResource {
                    game_config_path: "games/riven.ron".to_owned(),
                })
                .disable::<PluginBarrack>(),
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
