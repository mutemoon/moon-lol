pub mod buffs;

use bevy::asset::Handle;
use bevy::prelude::*;
use league_utils::hash_bin;
use lol_base::spell::Spell;
use lol_core::action::damage::{DamageShape, TargetDamage, TargetFilter};
use lol_core::action::dash::{ActionDash, DashMoveType};
use lol_core::base::buff::BuffOf;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{
    CoolDown, EventSkillCast, Skill, SkillSlot, play_skill_animation, reset_skill_attack,
    skill_damage, skill_dash, spawn_skill_particle,
};

use crate::lucian::buffs::{BuffLucianPassive, BuffLucianW};

#[derive(Default)]
pub struct PluginLucian;

impl Plugin for PluginLucian {
    fn build(&self, app: &mut App) {
        app.add_observer(on_lucian_skill_cast);
        app.add_observer(on_lucian_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Lucian"))]
#[reflect(Component)]
pub struct Lucian;

fn on_lucian_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_lucian: Query<(), With<Lucian>>,
    q_transform: Query<&Transform>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_lucian.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.spell.clone();

    match skill.slot {
        SkillSlot::Q => cast_lucian_q(&mut commands, entity, skill_spell),
        SkillSlot::W => cast_lucian_w(&mut commands, entity, skill_spell),
        SkillSlot::E => cast_lucian_e(
            &mut commands,
            &q_transform,
            entity,
            trigger.point,
            skill_spell,
        ),
        SkillSlot::R => cast_lucian_r(&mut commands, entity, skill_spell),
        _ => {}
    }
}

fn cast_lucian_q(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Lucian_Q_Cast"));

    // Q is a piercing light beam
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Sector {
            radius: 1000.0,
            angle: 10.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Lucian_Q_Hit")),
    );
}

fn cast_lucian_w(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Lucian_W_Cast"));

    // W marks enemies and grants movespeed to Lucian
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffLucianW::new(60.0, 6.0));

    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Sector {
            radius: 900.0,
            angle: 15.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Lucian_W_Hit")),
    );
}

fn cast_lucian_e(
    commands: &mut Commands,
    q_transform: &Query<&Transform>,
    entity: Entity,
    point: Vec2,
    skill_spell: Handle<Spell>,
) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Lucian_E_Cast"));

    // E is a dash
    reset_skill_attack(commands, entity);

    skill_dash(
        commands,
        q_transform,
        entity,
        point,
        &ActionDash {
            skill: skill_spell,
            move_type: DashMoveType::Pointer { max: 425.0 },
            damage: None,
            speed: 1000.0,
        },
    );
}

fn cast_lucian_r(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Lucian_R_Cast"));

    // R is a barrage of shots
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Sector {
            radius: 1200.0,
            angle: 30.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Lucian_R_Hit")),
    );
}

fn on_lucian_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_lucian: Query<(), With<Lucian>>,
) {
    let source = trigger.source;
    if q_lucian.get(source).is_err() {
        return;
    }

    let _target = trigger.event_target();

    // Passive procs after abilities
    commands
        .entity(source)
        .with_related::<BuffOf>(BuffLucianPassive::new(50.0, 1.0));
}
