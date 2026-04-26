use bevy::prelude::*;
use lol_core::resource::PluginResource;
use moon_lol::PluginCore;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.build().set(WindowPlugin {
                primary_window: Some(Window {
                    title: "classic 1v1 fiora".to_string(),
                    resolution: (1000, 1000).into(),
                    position: WindowPosition::At((0, 0).into()),
                    ..default()
                }),
                ..default()
            }),
            PluginCore.build().set(PluginResource {
                game_config_path: "games/attack.ron".to_owned(),
            }),
            // .disable::<PluginBarrack>(),
        ))
        .run();
}
