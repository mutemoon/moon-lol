use bevy::prelude::*;

#[derive(Component, Debug)]
#[relationship(relationship_target = Buffs)]
pub struct BuffOf(pub Entity);

#[derive(Component, Debug)]
#[relationship_target(relationship = BuffOf)]
pub struct Buffs(Vec<Entity>);
