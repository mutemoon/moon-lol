use bevy::prelude::*;
use clap::Parser;
use lol_agent::PluginAgentObserver;
use lol_champions::PluginChampions;
use lol_core::PluginCore;
use lol_core::game::GameScenes;
use lol_core::log::{LogDbPath, create_log_plugin};
use lol_debug::PluginDebug;
use lol_render::PluginRender;
use lol_server::PluginServer;

#[derive(Parser)]
#[command(name = "moon_lol")]
struct Args {
    #[arg(long, default_value = "9001")]
    ws_port: u16,

    #[arg(long, default_value = "sandbox")]
    mode: String,

    #[arg(long, default_value = "Riven")]
    champion: String,

    #[arg(long)]
    scene: Option<String>,
}

fn main() {
    let args = Args::parse();
    let (log_plugin, log_db_path) = create_log_plugin();

    let mut app = App::new();
    app.insert_resource(LogDbPath(log_db_path));

    // Register user_games custom asset source for absolute home dir loading
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_else(|_| ".".to_string());
    let user_games_path = std::path::Path::new(&home)
        .join(".moon-lol")
        .join("games");
    let _ = std::fs::create_dir_all(&user_games_path);
    app.register_asset_source(
        "user_games",
        bevy::asset::io::AssetSourceBuilder::platform_default(
            &user_games_path.to_string_lossy(),
            None,
        ),
    );

    app.add_plugins((
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
        PluginServer {
            ws_port: args.ws_port,
        },
        PluginDebug,
        PluginAgentObserver,
    ));

    let scene_path = args.scene.unwrap_or_else(|| "games/classic_fiora.ron".to_string());
    app.insert_resource(GameScenes::new(vec![scene_path]))
        .run();
}

