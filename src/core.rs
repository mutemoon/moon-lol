mod action;
mod animation;
mod attack;
mod attack_auto;
mod base;
mod camera;
mod config;
mod controller;
mod damage;
mod effect;
mod game;
mod life;
mod lifetime;
mod map;
mod movement;
mod navigation;
mod particle;
mod resource;
mod rotate;
mod run;
mod skill;
mod spawn;
mod ui;
mod utils;

pub use action::*;
pub use animation::*;
pub use attack::*;
pub use attack_auto::*;
pub use base::*;
pub use camera::*;
pub use config::*;
pub use controller::*;
pub use damage::*;
pub use effect::*;
pub use game::*;
pub use life::*;
pub use lifetime::*;
pub use map::*;
pub use movement::*;
pub use navigation::*;
pub use particle::*;
pub use resource::*;
pub use rotate::*;
pub use run::*;
pub use skill::*;
pub use spawn::*;
pub use ui::*;
pub use utils::*;

use bevy::app::plugin_group;

plugin_group! {
    pub struct PluginCore {
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
