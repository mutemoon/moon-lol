use std::collections::HashSet;

use bevy::prelude::*;
use lol_base::spell::Spell;

use crate::action::damage::{TargetDamage, TargetFilter};
use crate::damage::{CommandDamageCreate, Damage};
use crate::entities::champion::Champion;
use crate::entities::minion::Minion;
use crate::movement::{CommandMovement, MovementAction, MovementWay};
use crate::skill::{Skill, Skills, get_skill_value};
use crate::team::Team;

#[derive(Debug, Clone, EntityEvent)]
pub struct ActionDash {
    pub entity: Entity,
    pub move_type: DashMoveType,
    pub damage: Option<DashDamage>,
    pub speed: f32,
    pub skill: Handle<Spell>,
    pub point: Vec2,
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
    pub skill: Handle<Spell>,
    pub hit_entities: HashSet<Entity>,
}

pub fn on_action_dash(
    trigger: On<ActionDash>,
    mut commands: Commands,
    q_transform: Query<&Transform>,
) {
    let entity = trigger.event_target();

    let Ok(transform) = q_transform.get(entity) else {
        debug!(
            "on_action_dash: entity {:?} has no Transform, skipping",
            entity
        );
        return;
    };

    let current_pos = transform.translation;
    let vector = trigger.point - current_pos.xz();
    let distance = vector.length();

    debug!(
        "on_action_dash: entity {:?} at ({:.2}, {:.2}, {:.2}) -> point ({:.2}, {:.2}), distance: {:.2}, speed: {:.2}",
        entity,
        current_pos.x,
        current_pos.y,
        current_pos.z,
        trigger.point.x,
        trigger.point.y,
        distance,
        trigger.speed
    );

    let (destination, move_type_desc) = match trigger.move_type {
        DashMoveType::Fixed(fixed_distance) => {
            let direction = if distance < 0.001 {
                transform.forward().xz().normalize()
            } else {
                vector.normalize()
            };
            let dest = current_pos.xz() + direction * fixed_distance;
            debug!(
                "on_action_dash: DashMoveType::Fixed distance {:.2}, direction ({:.2}, {:.2}), destination ({:.2}, {:.2})",
                fixed_distance, direction.x, direction.y, dest.x, dest.y
            );
            (dest, format!("Fixed({:.2})", fixed_distance))
        }
        DashMoveType::Pointer { max } => {
            let dest = if distance < max {
                debug!(
                    "on_action_dash: DashMoveType::Pointer distance {:.2} < max {:.2}, going to point",
                    distance, max
                );
                trigger.point
            } else {
                let direction = vector.normalize();
                let dest = current_pos.xz() + direction * max;
                debug!(
                    "on_action_dash: DashMoveType::Pointer distance {:.2} >= max {:.2}, clamping to max",
                    distance, max
                );
                dest
            };
            (dest, format!("Pointer(max: {:.2})", max))
        }
    };

    if let Some(damage) = &trigger.damage {
        debug!(
            "on_action_dash: adding DashDamageComponent with radius_end {:.2}, filter: {:?}",
            damage.radius_end, damage.damage.filter
        );
        commands.entity(entity).insert(DashDamageComponent {
            start_pos: current_pos,
            target_pos: Vec3::new(destination.x, current_pos.y, destination.y),
            damage: damage.clone(),
            skill: trigger.skill.clone(),
            hit_entities: std::collections::HashSet::default(),
        });
    } else {
        debug!("on_action_dash: no damage component to add");
    }

    debug!(
        "on_action_dash: triggering CommandMovement for entity {:?}, destination ({:.2}, {:.2}, {:.2}), move_type: {}",
        entity, destination.x, current_pos.y, destination.y, move_type_desc
    );
    commands.trigger(CommandMovement {
        entity,
        priority: 100,
        action: MovementAction::Start {
            way: MovementWay::Path(vec![Vec3::new(destination.x, current_pos.y, destination.y)]),
            speed: Some(trigger.speed),
            source: "Dash".to_string(),
        },
    });
}

pub fn on_dash_end(
    trigger: On<crate::movement::EventMovementEnd>,
    mut commands: Commands,
    q: Query<&DashDamageComponent>,
) {
    let entity = trigger.event_target();
    if q.get(entity).is_ok() {
        commands.entity(entity).remove::<DashDamageComponent>();
    }
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
    res_assets_spell_object: Res<Assets<Spell>>,
) {
    for (entity, dasher_transform, mut dash_damage, team) in q_dasher.iter_mut() {
        let Some(skill_object) = res_assets_spell_object.get(&dash_damage.skill) else {
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
                &dash_damage.damage.damage.amount,
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
                dash_damage.hit_entities.insert(target);
            }
        }
    }
}
