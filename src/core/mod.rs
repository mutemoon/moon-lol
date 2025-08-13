mod animation;
mod attack;
mod base;
mod camera;
mod command;
mod config;
mod damage;
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
pub use damage::*;
pub use life::*;
pub use map::*;
pub use movement::*;
pub use navigation::*;
pub use resource::*;
pub use ui::*;

pub struct PluginCore;

impl bevy::app::Plugin for PluginCore {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins((
            PluginAnimation,
            PluginAttack,
            PluginCamera,
            PluginDamage,
            PluginLife,
            PluginMap,
            PluginMovement,
            PluginNavigaton,
            PluginResource,
            PluginTarget,
            PluginUI,
        ));
    }
}
