use std::time::Duration;

use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::render::{
    settings::{Backends, RenderCreation, WgpuSettings},
    RenderPlugin,
};
use bevy::winit::{UpdateMode, WinitSettings};

use moon_lol::{PluginBarrack, PluginCore, PluginResource};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.build().set(WindowPlugin {
                primary_window: Some(Window {
                    title: "classic 1v1 fiora".to_string(),
                    resolution: (300.0, 300.0).into(),
                    position: WindowPosition::At((0, 1000).into()),
                    ..default()
                }),
                ..default()
            }), // .set(LogPlugin {
                //     filter: "moon_lol=debug".to_string(),
                //     ..default()
                // })
            PluginCore
                .build()
                .set(PluginResource {
                    game_config_path: "games/attack.ron".to_owned(),
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
        // .add_systems(
        //     Update,
        //     |mut q_camera_state: Query<&mut CameraState, Added<CameraState>>| {
        //         for mut state in q_camera_state.iter_mut() {
        //             state.scale = 0.1;
        //         }
        //     },
        // )
        .run();
}
