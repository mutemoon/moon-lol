use bevy::prelude::*;

/// 召唤物组件，用于标识安妮的提伯斯、婕拉的植物等特殊单位
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Pet;

#[derive(Default)]
pub struct PluginPet;

impl Plugin for PluginPet {
    fn build(&self, _app: &mut App) {}
}
