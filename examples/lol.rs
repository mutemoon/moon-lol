use bevy::prelude::*;
use clap::Parser;
use lol_champions::PluginChampions;
use lol_core::PluginCore;
use lol_core::game::PluginGame;
use lol_core::log::create_log_plugin;
use lol_debug::PluginDebugPanel;
use lol_render::PluginRender;

#[derive(Parser)]
#[command(name = "moon_lol")]
struct Args {
    #[arg(long, default_value = "9001")]
    ws_port: u16,

    #[arg(long, default_value = "sandbox")]
    mode: String,

    #[arg(long, default_value = "Riven")]
    champion: String,
}

fn main() {
    let args = Args::parse();
    let (log_plugin, log_rx) = create_log_plugin();

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
            PluginCore.build().set(PluginGame {
                scenes: vec!["games/classic_fiora.ron".to_owned()],
            }),
            PluginRender,
            PluginChampions,
            PluginDebugPanel {
                ws_port: args.ws_port,
                log_receiver: log_rx,
            },
        ))
        .run();
}
