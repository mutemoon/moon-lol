pub mod buffs;
pub mod e;
pub mod q;

#[cfg(test)]
mod e_tests;
#[cfg(test)]
mod tests;

use bevy::prelude::*;
use lol_base::animation_names::{ANIM_SPELL2, ANIM_SPELL4};
use lol_base::render_cmd::CommandAnimationPlay;
use lol_base::spell::Spell;
use lol_core::action::damage::{
    ActionDamage, ActionDamageEffect, DamageShape, TargetDamage, TargetFilter,
};
use lol_core::attack::CommandAttackReset;
use lol_core::base::buff::BuffOf;
use lol_core::buffs::cc_debuffs::DebuffSlow;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{EventSkillCast, Skill, SkillSlot};
use lol_core::team::Team;

use crate::darius::buffs::BuffDariusBleed;
use crate::darius::e::cast_darius_e as execute_darius_e;
use crate::darius::q::cast_darius_q as execute_darius_q;

#[derive(Default)]
pub struct PluginDarius;

impl Plugin for PluginDarius {
    fn build(&self, app: &mut App) {
        app.add_observer(on_darius_q);
        app.add_observer(on_darius_w);
        app.add_observer(on_darius_e);
        app.add_observer(on_darius_r);
        app.add_observer(on_darius_damage_hit);
    }
}

#[derive(Component, Reflect, Default)]
#[require(Champion, Name = Name::new("Darius"))]
#[reflect(Component)]
pub struct Darius;

fn on_darius_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_darius: Query<(), With<Darius>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_darius.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::Q) {
        return;
    }

    cast_darius_q(&mut commands, entity, skill.spell.clone());
}

fn on_darius_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_darius: Query<(), With<Darius>>,
    q_skill: Query<&Skill>,
    q_transform: Query<&Transform>,
    q_team: Query<&Team>,
    q_enemies: Query<(Entity, &Transform), With<Champion>>,
) {
    let entity = trigger.event_target();
    if q_darius.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::W) {
        return;
    }

    cast_darius_w(
        &mut commands,
        entity,
        skill.spell.clone(),
        &q_transform,
        &q_team,
        &q_enemies,
    );
}

fn on_darius_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_darius: Query<(), With<Darius>>,
    q_skill: Query<&Skill>,
    q_transform: Query<&Transform>,
    q_team: Query<&Team>,
    q_enemies: Query<(Entity, &Transform), With<Champion>>,
) {
    let entity = trigger.event_target();
    if q_darius.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::E) {
        return;
    }

    execute_darius_e(
        &mut commands,
        entity,
        skill.spell.clone(),
        trigger.point,
        &q_transform,
        &q_team,
        &q_enemies,
    );
}

fn on_darius_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_darius: Query<(), With<Darius>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_darius.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::R) {
        return;
    }

    cast_darius_r(&mut commands, entity, skill.spell.clone());
}

fn cast_darius_q(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    // Q damage values at max level (level 5):
    // Outer blade: 150 + 0.9 AD
    // Inner blade: 75 + 0.45 AD (half of outer)
    let outer_damage = 150.0;
    let inner_damage = 75.0;

    execute_darius_q(
        commands,
        entity,
        skill_spell,
        inner_damage,
        outer_damage,
        true,
    );
}

fn cast_darius_w(
    commands: &mut Commands,
    entity: Entity,
    skill_spell: Handle<Spell>,
    q_transform: &Query<&Transform>,
    q_team: &Query<&Team>,
    q_enemies: &Query<(Entity, &Transform), With<Champion>>,
) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL2.to_string(),
        repeat: false,
        duration: None,
    });
    // W is an empowered auto attack that resets attack, deals damage, and applies slow
    commands.trigger(CommandAttackReset { entity });

    // Find W's target (nearest enemy within 300 range)
    let Ok(transform) = q_transform.get(entity) else {
        return;
    };
    let Ok(team) = q_team.get(entity) else {
        return;
    };

    let mut nearest_enemy: Option<Entity> = None;
    let mut min_dist = 300.0f32;

    for (enemy_entity, enemy_transform) in q_enemies.iter() {
        if let Ok(enemy_team) = q_team.get(enemy_entity) {
            if enemy_team != team {
                let dist = transform.translation.distance(enemy_transform.translation);
                if dist < min_dist {
                    min_dist = dist;
                    nearest_enemy = Some(enemy_entity);
                }
            }
        }
    }

    // W deals damage to the target
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Nearest {
                max_distance: 300.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "empowered_attack_damage".to_string(),
                damage_type: DamageType::Physical,
            }],
        }],
    });

    // Apply slow to the target directly (W only, not all damage sources)
    if let Some(target) = nearest_enemy {
        commands
            .entity(target)
            .with_related::<BuffOf>(DebuffSlow::new(0.5, 1.0));
    }

    debug!("Darius W: 致残打击，攻击重置 + 额外伤害 + 减速");
}

fn cast_darius_r(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL4.to_string(),
        repeat: false,
        duration: None,
    });
    // R is a targeted execute ability
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Nearest {
                max_distance: 400.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::Champion,
                amount: "damage".to_string(),
                damage_type: DamageType::Physical,
            }],
        }],
    });
}

/// 监听 Darius 造成的伤害，给目标叠加出血
fn on_darius_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_darius: Query<(), With<Darius>>,
) {
    let source = trigger.source;
    if q_darius.get(source).is_err() {
        return;
    }
    let target = trigger.event_target();
    // 所有 Darius 造成的伤害都给目标叠出血（减速现在只在 W 技能中单独处理）
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffDariusBleed::new(1, 5.0));
}
