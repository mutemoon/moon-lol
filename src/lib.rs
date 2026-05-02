use bevy::app::Plugin;
use lol_champions::PluginChampions;
use lol_core::PluginCore;
use lol_render::PluginRender;

pub struct PluginLOL;

impl Plugin for PluginLOL {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(PluginCore);
        app.add_plugins(PluginRender);
        app.add_plugins(PluginChampions);
    }
}
