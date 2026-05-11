use bevy::prelude::*;
use bevy::winit::WinitPlugin;
use lol_champions::PluginChampions;
use lol_core::PluginCore;
use lol_core::game::GameScenes;
use lol_render::PluginRender;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.build().disable::<WinitPlugin>(),
            PluginCore,
            PluginRender,
            PluginChampions,
        ))
        .insert_resource(GameScenes::new(vec!["games/test.ron".to_owned()]))
        .run();
}
