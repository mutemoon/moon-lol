mod fiora_e;
mod fiora_passive;

pub use fiora_e::*;
pub use fiora_passive::*;

use bevy::app::plugin_group;

plugin_group! {
    pub struct PluginAbilities {
        :PluginFioraPassive,
        :PluginFioraE,
    }
}
