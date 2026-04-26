pub mod buffs;

use bevy::prelude::*;
use league_utils::hash_bin;
use lol_base::spell::Spell;
use lol_core::action::damage::{DamageShape, TargetDamage, TargetFilter};
use lol_core::action::dash::{ActionDash, DashDamage, DashMoveType};
use lol_core::base::buff::BuffOf;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{
    CoolDown, EventSkillCast, Skill, SkillSlot, play_skill_animation, skill_damage, skill_dash,
    spawn_skill_particle,
};

use crate::ezreal::buffs::BuffEzrealPassive;

#[derive(Default)]
pub struct PluginEzreal;

impl Plugin for PluginEzreal {
    fn build(&self, app: &mut App) {
        app.add_observer(on_ezreal_skill_cast);
        app.add_observer(on_ezreal_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Ezreal"))]
#[reflect(Component)]
pub struct Ezreal;

fn on_ezreal_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_ezreal: Query<(), With<Ezreal>>,
    q_transform: Query<&Transform>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_ezreal.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.spell.clone();

    match skill.slot {
        SkillSlot::Q => cast_ezreal_q(&mut commands, entity, skill_spell),
        SkillSlot::W => cast_ezreal_w(&mut commands, entity, skill_spell),
        SkillSlot::E => cast_ezreal_e(
            &mut commands,
            &q_transform,
            entity,
            trigger.point,
            skill_spell,
        ),
        SkillSlot::R => cast_ezreal_r(&mut commands, entity, skill_spell),
        _ => {}
    }
}

fn cast_ezreal_q(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Ezreal_Q_Cast"));

    // Q is a long range skillshot
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Sector {
            radius: 1200.0,
            angle: 15.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Ezreal_Q_Hit")),
    );
}

fn cast_ezreal_w(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Ezreal_W_Cast"));

    // W marks target
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Sector {
            radius: 1200.0,
            angle: 20.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Ezreal_W_Hit")),
    );
}

fn cast_ezreal_e(
    commands: &mut Commands,
    q_transform: &Query<&Transform>,
    entity: Entity,
    point: Vec2,
    skill_spell: Handle<Spell>,
) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Ezreal_E_Cast"));

    // E is a blink/dash
    skill_dash(
        commands,
        q_transform,
        entity,
        point,
        &ActionDash {
            skill: skill_spell,
            move_type: DashMoveType::Pointer { max: 475.0 },
            damage: Some(DashDamage {
                radius_end: 100.0,
                damage: TargetDamage {
                    filter: TargetFilter::All,
                    amount: hash_bin("TotalDamage"),
                    damage_type: DamageType::Magic,
                },
            }),
            speed: 800.0,
        },
    );
}

fn cast_ezreal_r(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Ezreal_R_Cast"));

    // R is global AoE
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Sector {
            radius: 20000.0,
            angle: 30.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Ezreal_R_Hit")),
    );
}

fn on_ezreal_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_ezreal: Query<(), With<Ezreal>>,
) {
    let source = trigger.source;
    if q_ezreal.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Apply passive stacks
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffEzrealPassive::new());
}
