use std::collections::HashSet;

use bevy::prelude::*;
use league_core::extract::SpellObject;
use lol_base::prop::{HashKey, LoadHashKeyTrait};

use crate::action::damage::{TargetDamage, TargetFilter};
use crate::damage::{CommandDamageCreate, Damage};
use crate::entities::champion::Champion;
use crate::entities::minion::Minion;
use crate::skill::{get_skill_value, Skill, Skills};
use crate::team::Team;

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
    res_assets_spell_object: Res<Assets<SpellObject>>,
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
                dash_damage.hit_entities.insert(target);
            }
        }
    }
}
