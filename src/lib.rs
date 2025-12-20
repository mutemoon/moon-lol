mod abilities;
mod core;
mod entities;
mod server;

pub use core::*;

pub use abilities::*;
use bevy::app::plugin_group;
pub use entities::*;
pub use server::*;

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
        // :PluginUI,
    }
}
