use std::time::Duration;

use bevy::prelude::*;
use bevy::winit::{UpdateMode, WinitSettings};
use moon_lol::{PluginBarrack, PluginCore, PluginResource};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.build().set(WindowPlugin {
                primary_window: Some(Window {
                    title: "classic 1v1 fiora".to_string(),
                    resolution: (300, 300).into(),
                    position: WindowPosition::At((0, 1000).into()),
                    ..default()
                }),
                ..default()
            }),
            PluginCore
                .build()
                .set(PluginResource {
                    game_config_path: "games/attack.ron".to_owned(),
                })
                .disable::<PluginBarrack>(),
        ))
        .run();
}
