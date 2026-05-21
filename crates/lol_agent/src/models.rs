use bevy::prelude::*;
use lol_champions::fiora::passive::Vital;
use lol_core::attack::AttackState;
use serde::{Deserialize, Serialize};

#[derive(Component)]
pub struct AttackTarget;

#[derive(Clone, Serialize, Deserialize)]
pub struct ObserveEnemyHero {
    pub entity: Entity,
    pub position: Vec2,
    pub health: f32,
    pub max_health: f32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Observe {
    pub time: f32,
    pub myself: ObserveMyself,
    pub minions: Vec<ObserveMinion>,
    pub enemy_hero: Option<ObserveEnemyHero>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ObserveMinion {
    pub entity: Entity,
    pub position: Vec2,
    pub health: f32,
    pub distance: f32,
    pub vital: Option<Vital>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ObserveSkill {
    pub index: usize,
    pub level: usize,
    /// None=可用, Some(剩余秒数)=冷却中
    pub cooldown_remaining: Option<f32>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ObserveMyself {
    pub position: Vec2,
    pub attack_state: Option<AttackState>,
    pub run_target: Option<Vec2>,
    pub health: f32,
    pub max_health: f32,
    pub level: u32,
    pub ability_resource: Option<(f32, f32)>,
    pub attack_damage: f32,
    pub attack_range: f32,
    pub attack_speed: f32,
    pub armor: f32,
    pub skill_points: u32,
    pub skills: Vec<ObserveSkill>,
    pub gold: f32,
    pub kills: u32,
    pub deaths: u32,
    pub assists: u32,
    pub minion_kills: u32,
}
