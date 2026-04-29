use bevy::prelude::*;
use lol_base::map::MapPaths;
use lol_core::game::PluginGame;
use lol_core::navigation::navigation::NavigationDebug;
use moon_lol::PluginCore;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.build().set(WindowPlugin {
                primary_window: Some(Window {
                    title: "classic 1v1 fiora".to_string(),
                    resolution: (100, 100).into(),
                    position: WindowPosition::At((0, 0).into()),
                    ..default()
                }),
                ..default()
            }),
            PluginCore.build().set(PluginGame {
                scenes: vec!["games/test_scene.ron".to_owned()],
            }),
        ))
        .insert_resource(MapPaths::new("test"))
        .insert_resource(NavigationDebug)
        .run();
}
