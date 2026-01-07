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
