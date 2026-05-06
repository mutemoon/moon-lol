use bevy::prelude::*;
use lol_champions::PluginChampions;
use lol_core::PluginCore;
use lol_core::game::GameScenes;
use lol_core::log::create_log_plugin;
use lol_core::navigation::navigation::NavigationDebug;
use lol_render::PluginRender;

fn main() {
    let (log_plugin, _log_rx) = create_log_plugin();

    App::new()
        .add_plugins((
            DefaultPlugins.build().set(log_plugin).set(WindowPlugin {
                primary_window: Some(Window {
                    title: "classic 1v1 fiora".to_string(),
                    resolution: (1000, 1000).into(),
                    position: WindowPosition::At((0, 0).into()),
                    ..default()
                }),
                ..default()
            }),
            PluginCore,
            PluginRender,
            PluginChampions,
        ))
        .insert_resource(GameScenes::new(vec!["games/attack.ron".to_owned()]))
        .insert_resource(NavigationDebug)
        .run();
}
