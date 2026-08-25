use bevy::asset::io::AssetSourceBuilder;
use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use bevy::world_serialization::WorldSerializationPlugin;
use clap::Parser;
use lol_agent::PluginAgentObserver;
use lol_base::map::MapPaths;
use lol_champions::PluginChampions;
use lol_core::PluginCore;
use lol_core::game::GameScenes;
use lol_core::log::create_log_plugin;
use lol_core::skill::{GodMode, NoCooldown};
use lol_debug::PluginDebug;
use lol_particle::PluginParticle;
use lol_render::PluginRender;
use lol_server::PluginServer;

mod player_champion;
use lol_share::paths::games_dir;
use player_champion::{PlayerChampion, PluginPlayerChampion};

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
    map: Option<String>,

    #[arg(long)]
    scene: Option<String>,

    #[arg(long)]
    headless: bool,

    /// 每局日志 SQLite 路径；缺省沿用 ~/.moon-lol/logs/debug.db。
    #[arg(long)]
    log_db: Option<std::path::PathBuf>,

    #[arg(long)]
    no_cooldown: bool,

    #[arg(long)]
    god: bool,
}

fn main() {
    let args = Args::parse();
    let log_plugin = create_log_plugin(args.log_db);

    let mut app = App::new();

    // Register user_games custom asset source for absolute home dir loading
    let user_games_path = games_dir();
    let _ = std::fs::create_dir_all(&user_games_path);
    app.register_asset_source(
        "user_games",
        AssetSourceBuilder::platform_default(&user_games_path.to_string_lossy(), None),
    );

    if args.headless {
        app.insert_resource(TimeUpdateStrategy::FixedTimesteps(1));
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            WorldSerializationPlugin,
            log_plugin,
            PluginCore,
            PluginChampions,
            PluginServer {
                ws_port: args.ws_port,
            },
            PluginAgentObserver,
        ));
    } else {
        app.add_plugins((
            DefaultPlugins.build().set(log_plugin).set(WindowPlugin {
                primary_window: Some(Window {
                    title: format!("classic 1v1 · {}", args.champion),
                    resolution: (300, 300).into(),
                    position: WindowPosition::At((0, 1000).into()),
                    ..default()
                }),
                ..default()
            }),
            PluginCore,
            PluginRender,
            PluginParticle,
            PluginChampions,
            PluginServer {
                ws_port: args.ws_port,
            },
            PluginDebug,
            PluginAgentObserver,
        ));
    }

    let scene_path = args
        .scene
        .unwrap_or_else(|| "games/classic.ron".to_string());

    if let Some(map) = args.map {
        app.insert_resource(MapPaths::new(&map));
    }

    app.insert_resource(GodMode(args.god));
    app.insert_resource(NoCooldown(args.no_cooldown || args.god));
    app.insert_resource(PlayerChampion(args.champion.to_lowercase()));
    app.add_plugins(PluginPlayerChampion);
    app.insert_resource(GameScenes::new(vec![scene_path]));

    app.run();
}
