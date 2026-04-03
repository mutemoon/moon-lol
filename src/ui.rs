pub mod animation;
pub mod bind;
pub mod button;
pub mod damage;
pub mod element;
pub mod health_bar;
pub mod player;
pub mod skill;
use bevy::prelude::*;

#[derive(Default)]
pub struct PluginUI;

impl Plugin for PluginUI {
    fn build(&self, app: &mut App) {
        app.add_plugins(PluginUIElement);
        app.add_plugins(PluginUIHealthBar);
        app.add_plugins(PluginUIBind);
        app.add_plugins(PluginUIButton);
        app.add_plugins(PluginUISkill);
        app.add_plugins(PluginUIPlayer);
        app.add_plugins(PluginUIDamage);
        app.add_plugins(PluginUIAnimation);
    }
}
