use bevy::prelude::*;
use moon_lol::PluginCore;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.build().set(WindowPlugin {
                primary_window: Some(Window {
                    canvas: Some("#lol".to_string()),
                    ..default()
                }),
                ..default()
            }),
            PluginCore.build(),
        ))
        .run();
}
