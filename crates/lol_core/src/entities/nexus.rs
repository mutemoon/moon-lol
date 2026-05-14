use bevy::prelude::*;

#[derive(Default)]
pub struct PluginNexus;
impl Plugin for PluginNexus {
    fn build(&self, app: &mut App) {}
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Nexus;
