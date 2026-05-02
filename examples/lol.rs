use bevy::prelude::*;
use lol_champions::PluginChampions;
use lol_core::PluginCore;
use lol_core::game::PluginGame;
use lol_render::PluginRender;

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
            PluginCore.build().set(PluginGame {
                scenes: vec!["games/classic_fiora.ron".to_owned()],
            }),
            PluginRender,
            PluginChampions,
        ))
        .run();
}
