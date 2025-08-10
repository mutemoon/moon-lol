use bevy::prelude::*;
use bevy::render::{
    settings::{Backends, RenderCreation, WgpuSettings},
    RenderPlugin,
};
use moon_lol::{
    combat::PluginCombat, entities::PluginEntities, league::LeagueLoader, logging::PluginLogging,
    render::PluginRender,
};

fn main() {
    #[cfg(unix)]
    let loader = LeagueLoader::new(
        r"/mnt/c/Program Files (x86)/WeGameApps/英雄联盟/game",
        r"DATA/FINAL/Maps/Shipping/Map11.wad.client",
        r"data/maps/mapgeometry/map11/bloom.mapgeo",
    )
    .unwrap();
    #[cfg(windows)]
    let loader = LeagueLoader::new(
        r"C:\Program Files (x86)\WeGameApps\英雄联盟\game",
        r"DATA\FINAL\Maps\Shipping\Map11.wad.client",
        r"data/maps/mapgeometry/map11/bloom.mapgeo",
    )
    .unwrap();

    loader.to_configs();

    // app
    // App::new()
    //     .add_plugins((
    //         PluginLogging,
    //         DefaultPlugins
    //             .build()
    //             .disable::<bevy::log::LogPlugin>()
    //             .set(WindowPlugin {
    //                 primary_window: Some(Window {
    //                     title: "classic 1v1 fiora".to_string(),
    //                     // resolution: (300.0, 300.0).into(),
    //                     // position: WindowPosition::At((0, 1920).into()),
    //                     ..default()
    //                 }),
    //                 ..default()
    //             })
    //             .set(RenderPlugin {
    //                 render_creation: RenderCreation::Automatic(WgpuSettings {
    //                     backends: Some(Backends::VULKAN),
    //                     // limits: WgpuLimits::downlevel_webgl2_defaults(),
    //                     ..default()
    //                 }),
    //                 ..default()
    //             }),
    //         PluginCombat,
    //         PluginEntities,
    //         PluginRender,
    //     ))
    //     .run();
}
