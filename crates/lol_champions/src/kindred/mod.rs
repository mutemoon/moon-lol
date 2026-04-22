pub mod buffs;

use bevy::prelude::*;
use league_utils::hash_bin;
use lol_base::spell::Spell;
use lol_core::action::damage::{DamageShape, TargetDamage, TargetFilter};
use lol_core::base::buff::BuffOf;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{
    EventSkillCast, Skill, SkillSlot, play_skill_animation, skill_damage, spawn_skill_particle,
};

use crate::kindred::buffs::{BuffKindredE, BuffKindredW};

#[derive(Default)]
pub struct PluginKindred;

impl Plugin for PluginKindred {
    fn build(&self, app: &mut App) {
        app.add_observer(on_kindred_skill_cast);
        app.add_observer(on_kindred_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Kindred"))]
#[reflect(Component)]
pub struct Kindred;

fn on_kindred_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_kindred: Query<(), With<Kindred>>,
    q_transform: Query<&Transform>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_kindred.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.key_spell_object.clone();

    match skill.slot {
        SkillSlot::Q => cast_kindred_q(
            &mut commands,
            &q_transform,
            entity,
            trigger.point,
            skill_spell,
        ),
        SkillSlot::W => cast_kindred_w(&mut commands, entity, skill_spell),
        SkillSlot::E => cast_kindred_e(&mut commands, entity, skill_spell),
        SkillSlot::R => cast_kindred_r(&mut commands, entity, skill_spell),
        _ => {}
    }
}

fn cast_kindred_q(
    commands: &mut Commands,
    _q_transform: &Query<&Transform>,
    entity: Entity,
    _point: Vec2,
    skill_spell: Handle<Spell>,
) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Kindred_Q_Cast"));

    // Q is a dash that shoots arrows
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Circle { radius: 300.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Kindred_Q_Hit")),
    );
}

fn cast_kindred_w(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Kindred_W_Cast"));

    // W marks an area where Wolf attacks
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffKindredW::new(50.0, 8.5));

    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Circle { radius: 500.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Kindred_W_Hit")),
    );
}

fn cast_kindred_e(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Kindred_E_Cast"));

    // E marks and slows, 3 marks = Wolf attacks
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Sector {
            radius: 500.0,
            angle: 30.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Kindred_E_Hit")),
    );
}

fn cast_kindred_r(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Kindred_R_Cast"));

    // R creates a protective zone
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Circle { radius: 535.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Kindred_R_Hit")),
    );
}

fn on_kindred_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_kindred: Query<(), With<Kindred>>,
) {
    let source = trigger.source;
    if q_kindred.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Apply slow and mark
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffKindredE::new(1, 0.3, 2.0));
}
