use bevy::prelude::*;
use bevy_behave::prelude::BehaveTrigger;
use lol_core::Team;

use crate::{Champion, CommandDamageCreate, CommandSkinParticleSpawn, DamageType, Minion};

#[derive(Debug, Clone)]
pub enum DamageShape {
    Circle {
        radius: f32,
    },
    Sector {
        radius: f32,
        angle: f32,
    },
    Annular {
        inner_radius: f32,
        outer_radius: f32,
    },
    Nearest {
        max_distance: f32,
    },
}

impl Default for DamageShape {
    fn default() -> Self {
        Self::Circle { radius: 300.0 }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TargetFilter {
    All,
    Champion,
    Minion,
}

#[derive(Debug, Clone)]
pub struct TargetDamage {
    pub filter: TargetFilter,
    pub amount: f32,
    pub damage_type: DamageType,
}

#[derive(Debug, Clone)]
pub struct ActionDamageEffect {
    pub shape: DamageShape,
    pub damage_list: Vec<TargetDamage>,
    pub particle: Option<u32>,
}

#[derive(Debug, Clone, Default)]
pub struct ActionDamage {
    pub effects: Vec<ActionDamageEffect>,
}

pub fn on_attack_damage(
    trigger: On<BehaveTrigger<ActionDamage>>,
    mut commands: Commands,
    q_transform: Query<&Transform>,
    q_target: Query<(
        Entity,
        &Team,
        Option<&Champion>,
        Option<&Minion>,
        &Transform,
    )>,
    q_team: Query<&Team>,
) {
    let ctx = trigger.ctx();
    let entity = ctx.target_entity();
    let action = trigger.inner();

    let Ok(team) = q_team.get(entity) else {
        commands.trigger(ctx.failure());
        return;
    };
    let Ok(transform) = q_transform.get(entity) else {
        commands.trigger(ctx.failure());
        return;
    };

    let forward = transform.forward().xz();

    for effect in &action.effects {
        let mut targets = Vec::new();

        match effect.shape {
            DamageShape::Circle { radius } => {
                for (target, target_team, _c, _m, target_transform) in q_target.iter() {
                    if target_team == team {
                        continue;
                    }
                    if target_transform.translation.distance(transform.translation) <= radius {
                        targets.push(target);
                    }
                }
            }
            DamageShape::Sector { radius, angle } => {
                let half_angle = angle.to_radians() / 2.0;
                for (target, target_team, _c, _m, target_transform) in q_target.iter() {
                    if target_team == team {
                        continue;
                    }
                    let diff = (target_transform.translation - transform.translation).xz();
                    let distance = diff.length();
                    if distance <= radius {
                        let target_dir = diff.normalize();
                        if forward.dot(target_dir).acos() <= half_angle {
                            targets.push(target);
                        }
                    }
                }
            }
            DamageShape::Annular {
                inner_radius,
                outer_radius,
            } => {
                for (target, target_team, _c, _m, target_transform) in q_target.iter() {
                    if target_team == team {
                        continue;
                    }
                    let distance = target_transform.translation.distance(transform.translation);
                    if distance >= inner_radius && distance <= outer_radius {
                        targets.push(target);
                    }
                }
            }
            DamageShape::Nearest { max_distance } => {
                let mut min_dist = max_distance;
                let mut nearest = None;
                for (target, target_team, _c, _m, target_transform) in q_target.iter() {
                    if target_team == team {
                        continue;
                    }
                    let distance = target_transform.translation.distance(transform.translation);
                    if distance < min_dist {
                        min_dist = distance;
                        nearest = Some(target);
                    }
                }
                if let Some(target) = nearest {
                    targets.push(target);
                }
            }
        }

        for target_entity in targets {
            let Ok((_, _, champion, minion, _)) = q_target.get(target_entity) else {
                continue;
            };

            for damage in &effect.damage_list {
                let apply = match damage.filter {
                    TargetFilter::All => true,
                    TargetFilter::Champion => champion.is_some(),
                    TargetFilter::Minion => minion.is_some(),
                };

                if apply {
                    commands.trigger(CommandDamageCreate {
                        entity: target_entity,
                        source: entity,
                        damage_type: damage.damage_type,
                        amount: damage.amount,
                    });
                }
            }
        }

        if let Some(hash) = effect.particle {
            commands.trigger(CommandSkinParticleSpawn { entity, hash });
        }
    }

    commands.trigger(ctx.success());
}
