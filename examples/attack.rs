use bevy::prelude::*;
use lol_core::game::PluginGame;
use lol_core::log::create_log_plugin;
use lol_core::navigation::navigation::NavigationDebug;
use moon_lol::PluginCore;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .build()
                .set(create_log_plugin())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "classic 1v1 fiora".to_string(),
                        resolution: (1000, 1000).into(),
                        position: WindowPosition::At((0, 0).into()),
                        ..default()
                    }),
                    ..default()
                }),
            PluginCore.build().set(PluginGame {
                scenes: vec!["games/attack.ron".to_owned()],
            }),
        ))
        .insert_resource(NavigationDebug)
        .run();
}
