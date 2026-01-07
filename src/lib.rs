mod buffs;
mod core;
mod entities;
mod server;
mod ui;

pub use core::*;

use bevy::app::plugin_group;
pub use buffs::*;
pub use entities::*;
pub use server::*;
pub use ui::*;

plugin_group! {
    pub struct PluginCore {
        :PluginDamageReduction,
        :PluginFioraPassive,
        :PluginFioraE,
        :PluginFioraR,
        :PluginRivenPassive,
        :PluginRivenQ,
        :PluginShieldWhite,
        :PluginShieldMagic,

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
