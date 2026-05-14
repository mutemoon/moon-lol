use bevy::prelude::*;

#[derive(Default)]
pub struct PluginInhibitor;
impl Plugin for PluginInhibitor {
    fn build(&self, app: &mut App) {}
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Inhibitor;
