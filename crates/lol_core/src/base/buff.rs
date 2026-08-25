use bevy::prelude::*;

#[derive(Component, Debug)]
pub struct Buff {
    pub name: &'static str,
}

#[derive(Component, Debug)]
#[relationship(relationship_target = Buffs)]
pub struct BuffOf(pub Entity);

#[derive(Component, Debug)]
#[relationship_target(relationship = BuffOf, linked_spawn)]
pub struct Buffs(Vec<Entity>);

impl std::ops::Deref for Buffs {
    type Target = Vec<Entity>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Default)]
pub struct PluginBuff;

impl Plugin for PluginBuff {
    fn build(&self, app: &mut App) {
        app.add_observer(on_reset_buffs);
    }
}

pub fn on_reset_buffs(
    _trigger: On<crate::action::EventReset>,
    mut commands: Commands,
    q_buffs: Query<Entity, With<BuffOf>>,
) {
    for entity in q_buffs.iter() {
        commands.entity(entity).despawn();
    }
}
