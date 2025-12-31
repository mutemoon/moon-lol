use std::collections::HashSet;

use bevy::prelude::*;
use bevy_behave::prelude::{BehaveCtx, BehaveTrigger};
use lol_core::Team;

use crate::{
    CommandDamageCreate, CommandMovement, DamageType, EventMovementEnd, MovementAction,
    MovementWay, SkillEffectContext,
};

#[derive(Debug, Clone)]
pub struct ActionDash {
    pub move_type: DashMoveType,
    pub damage: Option<DashDamage>,
    pub speed: f32,
}

#[derive(Debug, Clone)]
pub enum DashMoveType {
    Fixed(f32),
    Pointer { max: f32 },
}

#[derive(Debug, Clone)]
pub struct DashDamage {
    pub amount: f32,
    pub radius_end: f32,
    pub damage_type: DamageType,
}

#[derive(Component)]
pub struct DashDamageComponent {
    pub start_pos: Vec3,
    pub target_pos: Vec3,
    pub damage: DashDamage,
    pub hit_entities: HashSet<Entity>,
}

#[derive(Component)]
pub struct DashBehaveCtx(pub BehaveCtx);

pub fn on_action_dash(
    trigger: On<BehaveTrigger<ActionDash>>,
    mut commands: Commands,
    q_transform: Query<&Transform>,
    q_skill_effect_ctx: Query<&SkillEffectContext>,
) {
    let ctx = trigger.ctx();
    let entity = ctx.target_entity();
    let event = trigger.inner();
    let behave_entity = ctx.behave_entity();
    let skill_effect_ctx = q_skill_effect_ctx.get(behave_entity).ok();
    let skill_effect_ctx = skill_effect_ctx.unwrap();
    let transform = q_transform.get(entity).unwrap();
    let vector = skill_effect_ctx.point - transform.translation.xz();
    let distance = vector.length();

    let destination = match event.move_type {
        DashMoveType::Fixed(fixed_distance) => {
            let direction = if distance < 0.001 {
                transform.forward().xz().normalize()
            } else {
                vector.normalize()
            };
            transform.translation.xz() + direction * fixed_distance
        }
        DashMoveType::Pointer { max } => {
            if distance < max {
                skill_effect_ctx.point
            } else {
                let direction = vector.normalize();
                transform.translation.xz() + direction * max
            }
        }
    };

    if let Some(damage) = &event.damage {
        commands.entity(entity).insert(DashDamageComponent {
            start_pos: transform.translation,
            target_pos: Vec3::new(destination.x, transform.translation.y, destination.y),
            damage: damage.clone(),
            hit_entities: HashSet::default(),
        });
    }

    commands.entity(entity).insert(DashBehaveCtx(ctx.clone()));
    commands.trigger(CommandMovement {
        entity,
        priority: 100,
        action: MovementAction::Start {
            way: MovementWay::Path(vec![Vec3::new(
                destination.x,
                transform.translation.y,
                destination.y,
            )]),
            speed: Some(event.speed),
            source: "Dash".to_string(),
        },
    });
}

pub fn on_action_dash_end(
    trigger: On<EventMovementEnd>,
    mut commands: Commands,
    q: Query<&DashBehaveCtx>,
) {
    let entity = trigger.event_target();
    let Ok(DashBehaveCtx(ctx)) = q.get(entity) else {
        return;
    };
    commands.entity(entity).remove::<DashDamageComponent>();
    commands.trigger(ctx.success());
}

pub fn update_dash_damage(
    mut commands: Commands,
    mut q_dasher: Query<(Entity, &Transform, &mut DashDamageComponent, &Team)>,
    q_target: Query<(Entity, &Transform, &Team)>,
    // TODO: Get entity radius
) {
    for (dasher, dasher_transform, mut dash_damage, team) in q_dasher.iter_mut() {
        let start_pos = dash_damage.start_pos;
        let target_pos = dash_damage.target_pos;
        let current_pos = dasher_transform.translation;

        let total_dist = start_pos.distance(target_pos);
        if total_dist < 0.001 {
            continue;
        }
        let current_dist = start_pos.distance(current_pos);
        let progress = (current_dist / total_dist).clamp(0.0, 1.0);

        // TODO: Get actual radius from component
        let radius_start = 65.0;
        let current_radius =
            radius_start + (dash_damage.damage.radius_end - radius_start) * progress;

        for (target, target_transform, target_team) in q_target.iter() {
            if team == target_team {
                continue;
            }

            if dash_damage.hit_entities.contains(&target) {
                continue;
            }

            if dasher_transform
                .translation
                .distance(target_transform.translation)
                <= current_radius
            {
                commands.trigger(CommandDamageCreate {
                    entity: target,
                    source: dasher,
                    damage_type: dash_damage.damage.damage_type,
                    amount: dash_damage.damage.amount,
                });
                // Optional: Spawn hit effect
                // commands.trigger(CommandSkinParticleSpawn {
                //     entity: target,
                //     hash: hash_bin("HitEffect"),
                // });
                dash_damage.hit_entities.insert(target);
            }
        }
    }
}
