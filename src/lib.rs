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

use bevy::app::plugin_group;

plugin_group! {
    pub struct PluginCore {
        :PluginFioraPassive,
        :PluginFioraE,
        :PluginFioraR,

        :PluginBarrack,
        :PluginChampion,
        :PluginCharacter,
        :PluginDebugSphere,
        :PluginMinion,
        :PluginTurret,

        :PluginFiora,
        :PluginHwei,
        :PluginRiven,

        :PluginAction,
        :PluginAnimation,
        :PluginAttack,
        :PluginAttackAuto,
        :PluginAggro,
        :PluginBase,
        :PluginCamera,
        :PluginController,
        :PluginCooldown,
        :PluginDamage,
        :PluginGame,
        :PluginLife,
        :PluginLifetime,
        :PluginMap,
        :PluginMissile,
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
