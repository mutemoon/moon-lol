mod abilities;
mod core;
mod entities;
mod logging;
mod server;

pub use abilities::*;
pub use core::*;
pub use entities::*;
pub use logging::*;
pub use server::*;

use bevy::{
    app::{plugin_group, App},
    DefaultPlugins,
};

plugin_group! {
    pub struct PluginCore {
        :PluginFioraPassive,
        :PluginFioraE,
        :PluginFioraR,

        :PluginBarrack,
        :PluginChampion,
        :PluginCharacter,
        :PluginDebugSphere,
        :PluginFiora,
        :PluginHwei,
        :PluginMinion,

        :PluginAction,
        :PluginAnimation,
        :PluginAttack,
        :PluginAttackAuto,
        :PluginBase,
        :PluginCamera,
        :PluginController,
        :PluginDamage,
        :PluginGame,
        :PluginLife,
        :PluginLifetime,
        :PluginMap,
        :PluginMovement,
        :PluginNavigaton,
        :PluginParticle,
        :PluginResource,
        :PluginRotate,
        :PluginRun,
        :PluginSkill,
        :PluginSkin,
        :PluginState,
        :PluginUI,
    }
}

fn main() {
    // App::new()
    //     .add_plugins((
    //         PluginLogging,
    //         DefaultPlugins
    //             .build()
    //             .disable::<bevy::log::LogPlugin>()
    //             .set(WindowPlugin {
    //                 primary_window: Some(Window {
    //                     title: "moon-lol".to_string(),
    //                     ..default()
    //                 }),
    //                 ..default()
    //             }),
    //         PluginCore.build().set(PluginResource {
    //             game_config_path: "games/attack.ron".to_owned(),
    //         }),
    //     ))
    //     .run();

    let mut app = App::new();
    app.add_plugins(DefaultPlugins);

    app.add_plugins(PluginBarrack);
    app.add_plugins(PluginChampion);
    app.add_plugins(PluginCharacter);
    app.add_plugins(PluginDebugSphere);
    app.add_plugins(PluginFiora);
    app.add_plugins(PluginHwei);
    app.add_plugins(PluginMinion);

    app.add_plugins(PluginAction);
    app.add_plugins(PluginAnimation);
    app.add_plugins(PluginAttack);
    app.add_plugins(PluginAttackAuto);
    app.add_plugins(PluginBase);
    app.add_plugins(PluginCamera);
    app.add_plugins(PluginController);
    app.add_plugins(PluginDamage);
    app.add_plugins(PluginGame);
    app.add_plugins(PluginLife);
    app.add_plugins(PluginLifetime);
    app.add_plugins(PluginMap);
    app.add_plugins(PluginMovement);
    app.add_plugins(PluginNavigaton);
    // app.add_plugins(PluginParticle);
    app.add_plugins(PluginResource {
        game_config_path: "games/null.ron".to_owned(),
    });
    app.add_plugins(PluginRotate);
    app.add_plugins(PluginRun);
    app.add_plugins(PluginSkill);
    app.add_plugins(PluginSkin);
    app.add_plugins(PluginState);
    // app.add_plugins(PluginUI);

    app.run();
}
