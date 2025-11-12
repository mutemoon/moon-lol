use bevy::prelude::*;

use moon_lol::{PluginCore, PluginLogging};

fn main() {
    App::new()
        .add_plugins((
            PluginLogging,
            DefaultPlugins
                .build()
                .disable::<bevy::log::LogPlugin>()
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "moon-lol".to_string(),
                        ..default()
                    }),
                    ..default()
                }),
            PluginCore.build(),
        ))
        .run();
}
