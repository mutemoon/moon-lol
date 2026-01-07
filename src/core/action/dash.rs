use std::collections::HashSet;

use bevy::prelude::*;
use bevy_behave::prelude::{BehaveCtx, BehaveTrigger};
use league_core::SpellObject;
use lol_config::{ConfigNavigationGrid, HashKey, LoadHashKeyTrait};
use lol_core::Team;

use crate::{
    get_skill_value, Champion, CommandDamageCreate, CommandMovement, Damage, EventMovementEnd,
    Minion, MovementAction, MovementWay, ResourceGrid, Skill, SkillEffectContext, Skills,
    TargetDamage, TargetFilter,
};

#[derive(Debug, Clone)]
pub struct ActionDash {
    pub move_type: DashMoveType,
    pub damage: Option<DashDamage>,
    pub speed: f32,
    pub skill: HashKey<SpellObject>,
}

#[derive(Debug, Clone)]
pub enum DashMoveType {
    Fixed(f32),
    Pointer { max: f32 },
}

#[derive(Debug, Clone)]
pub struct DashDamage {
    pub radius_end: f32,
    pub damage: TargetDamage,
}

#[derive(Component)]
pub struct DashDamageComponent {
    pub start_pos: Vec3,
    pub target_pos: Vec3,
    pub damage: DashDamage,
    pub skill: HashKey<SpellObject>,
    pub hit_entities: HashSet<Entity>,
}

#[derive(Component)]
pub struct DashBehaveCtx(pub BehaveCtx);

pub fn on_action_dash(
    trigger: On<BehaveTrigger<ActionDash>>,
    mut commands: Commands,
    q_transform: Query<&Transform>,
    q_skill_effect_ctx: Query<&SkillEffectContext>,
    res_grid: Res<ResourceGrid>,
    assets_grid: Res<Assets<ConfigNavigationGrid>>,
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

    let destination = if let Some(grid) = assets_grid.get(&res_grid.0) {
        let grid_pos = grid.get_cell_xy_by_position(&destination);
        if let Some(new_grid_pos) =
            crate::core::navigation::find_nearest_walkable_cell(grid, grid_pos)
        {
            grid.get_cell_center_position_by_xy(new_grid_pos).xz()
        } else {
            destination
        }
    } else {
        destination
    };

    if let Some(damage) = &event.damage {
        commands.entity(entity).insert(DashDamageComponent {
            start_pos: transform.translation,
            target_pos: Vec3::new(destination.x, transform.translation.y, destination.y),
            damage: damage.clone(),
            skill: event.skill,
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
    q_target: Query<(
        Entity,
        &Transform,
        &Team,
        Option<&Champion>,
        Option<&Minion>,
    )>,
    q_skills: Query<&Skills>,
    q_skill: Query<&Skill>,
    q_damage: Query<&Damage>,
    res_assets_spell_object: Res<Assets<SpellObject>>,
    // TODO: Get entity radius
) {
    for (entity, dasher_transform, mut dash_damage, team) in q_dasher.iter_mut() {
        let Some(skill_object) = res_assets_spell_object.load_hash(dash_damage.skill) else {
            return;
        };
        let Ok(skills) = q_skills.get(entity) else {
            return;
        };
        let skill = skills
            .iter()
            .map(|v| q_skill.get(v))
            .find_map(|v| v.ok())
            .unwrap();

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

        for (target, target_transform, target_team, champion, minion) in q_target.iter() {
            if team == target_team {
                continue;
            }

            if dash_damage.hit_entities.contains(&target) {
                continue;
            }

            let apply = match dash_damage.damage.damage.filter {
                TargetFilter::All => true,
                TargetFilter::Champion => champion.is_some(),
                TargetFilter::Minion => minion.is_some(),
            };

            if !apply {
                continue;
            }

            let damage_amount = get_skill_value(
                &skill_object,
                dash_damage.damage.damage.amount,
                skill.level,
                |stat| {
                    if stat == 2 {
                        if let Ok(damage) = q_damage.get(entity) {
                            return damage.0;
                        }
                    }
                    0.0
                },
            )
            .unwrap();

            if dasher_transform
                .translation
                .distance(target_transform.translation)
                <= current_radius
            {
                commands.trigger(CommandDamageCreate {
                    entity: target,
                    source: entity,
                    damage_type: dash_damage.damage.damage.damage_type,
                    amount: damage_amount,
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
