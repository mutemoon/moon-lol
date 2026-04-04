use bevy::prelude::*;

#[derive(Debug, Clone)]
pub struct SkillNavigationTo {
    pub target: Vec2,
}

#[derive(Debug, Clone)]
pub struct SkillAutoAttack {
    pub target: Entity,
}
