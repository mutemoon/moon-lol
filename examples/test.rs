use bevy::prelude::*;
use lol_base::map::MapPaths;
use lol_champions::PluginChampions;
use lol_core::PluginCore;
use lol_core::game::GameScenes;
use lol_core::log::create_log_plugin;
use lol_core::navigation::navigation::NavigationDebug;
use lol_render::PluginRender;

fn main() {
    let (log_plugin, _) = create_log_plugin();
    App::new()
        .add_plugins((
            DefaultPlugins.build().set(log_plugin).set(WindowPlugin {
                primary_window: Some(Window {
                    title: "classic 1v1 fiora".to_string(),
                    resolution: (300, 300).into(),
                    position: WindowPosition::At((0, 1000).into()),
                    ..default()
                }),
                ..default()
            }),
            PluginCore,
            PluginRender,
            PluginChampions,
        ))
        .insert_resource(GameScenes::new(vec!["games/test.ron".to_owned()]))
        .insert_resource(MapPaths::new("test"))
        .insert_resource(NavigationDebug)
        .run();
}
