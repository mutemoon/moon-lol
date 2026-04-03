mod animation;
mod bind;
mod button;
mod damage;
mod element;
mod health_bar;
mod player;
mod skill;

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
