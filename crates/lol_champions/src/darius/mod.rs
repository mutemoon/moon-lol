pub mod buffs;
pub mod q;

use bevy::prelude::*;
use league_utils::hash_bin;
use lol_base::spell::Spell;
use lol_core::action::damage::{DamageShape, TargetDamage, TargetFilter};
use lol_core::base::buff::BuffOf;
use lol_core::buffs::cc_debuffs::DebuffSlow;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{
    EventSkillCast, Skill, SkillSlot, play_skill_animation, reset_skill_attack, skill_damage,
    spawn_skill_particle,
};

use crate::darius::buffs::BuffDariusBleed;
use crate::darius::q::cast_darius_q as execute_darius_q;

#[derive(Default)]
pub struct PluginDarius;

impl Plugin for PluginDarius {
    fn build(&self, app: &mut App) {
        app.add_observer(on_darius_skill_cast);
        app.add_observer(on_darius_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Darius"))]
#[reflect(Component)]
pub struct Darius;

fn on_darius_skill_cast(
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

    let skill_spell = skill.spell.clone();

    match skill.slot {
        SkillSlot::Q => cast_darius_q(&mut commands, entity, skill_spell),
        SkillSlot::W => cast_darius_w(&mut commands, entity),
        SkillSlot::E => cast_darius_e(&mut commands, entity, skill_spell),
        SkillSlot::R => cast_darius_r(&mut commands, entity, skill_spell),
        _ => {}
    }
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

fn cast_darius_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, "spell2".to_string());
    spawn_skill_particle(commands, entity, hash_bin("Darius_W_Cast"));
    // W is an empowered auto attack that applies slow
    reset_skill_attack(commands, entity);
}

fn cast_darius_e(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, "spell3".to_string());
    spawn_skill_particle(commands, entity, hash_bin("Darius_E_Cast"));
    // E is a cone pull
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Sector {
            radius: 300.0,
            angle: 90.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Darius_E_Hit")),
    );
}

fn cast_darius_r(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, "spell4".to_string());
    spawn_skill_particle(commands, entity, hash_bin("Darius_R_Cast"));
    // R is a targeted execute ability
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Nearest {
            max_distance: 400.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::Champion,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Darius_R_Hit")),
    );
}

/// 监听 Darius 造成的伤害，给目标叠加出血和减速
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
    // 所有 Darius 造成的伤害都给目标叠出血
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffDariusBleed::new(1, 5.0));
    // W 命中施加减速
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffSlow::new(0.5, 1.0));
}
