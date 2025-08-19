mod animation;
mod attack;
mod base;
mod camera;
mod command;
mod config;
mod controller;
mod damage;
mod game;
mod life;
mod map;
mod movement;
mod navigation;
mod resource;
mod ui;


pub use animation::*;
pub use attack::*;
pub use base::*;
pub use camera::*;
pub use command::*;
pub use config::*;
pub use controller::*;
pub use damage::*;
pub use game::*;
pub use life::*;
pub use map::*;
pub use movement::*;
pub use navigation::*;
pub use resource::*;
pub use ui::*;

use bevy::app::plugin_group;

plugin_group! {
    pub struct PluginCore {
        :PluginAnimation,
        :PluginAttack,
        :PluginCamera,
        :PluginController,
        :PluginDamage,
        :PluginGame,
        :PluginLife,
        :PluginMap,
        :PluginMovement,
        :PluginNavigaton,
        :PluginResource,
        :PluginTarget,
        :PluginUI,
    }
}
