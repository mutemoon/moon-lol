pub mod buffs;

use bevy::prelude::*;
use league_utils::hash_bin;
use lol_base::spell::Spell;
use lol_core::action::damage::{DamageShape, TargetDamage, TargetFilter};
use lol_core::action::dash::{ActionDash, DashMoveType};
use lol_core::base::buff::BuffOf;
use lol_core::buffs::cc_debuffs::DebuffSlow;
use lol_core::buffs::shield_white::BuffShieldWhite;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{
    CoolDown, EventSkillCast, Skill, SkillSlot, play_skill_animation, skill_damage, skill_dash,
    spawn_skill_particle,
};

use crate::urgot::buffs::BuffUrgotW;

// Urgot Q parameters
const URGOT_Q_SLOW_PERCENT: f32 = 0.45;
const URGOT_Q_SLOW_DURATION: f32 = 1.25;

// Urgot W parameters
const URGOT_W_ATTACK_INTERVAL: f32 = 0.5; // Attack every 0.5 seconds
const URGOT_W_MOVE_SPEED_REDUCTION: f32 = 0.25; // 25% move speed reduction
const URGOT_W_MAX_RANGE: f32 = 300.0; // Max attack range

#[derive(Default)]
pub struct PluginUrgot;

impl Plugin for PluginUrgot {
    fn build(&self, app: &mut App) {
        app.add_observer(on_urgot_skill_cast);
        app.add_observer(on_urgot_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Urgot"))]
#[reflect(Component)]
pub struct Urgot;

fn on_urgot_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_urgot: Query<(), With<Urgot>>,
    q_transform: Query<&Transform>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_urgot.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.key_spell_object.clone();

    match skill.slot {
        SkillSlot::Q => cast_urgot_q(&mut commands, entity, skill_spell),
        SkillSlot::W => cast_urgot_w(&mut commands, entity),
        SkillSlot::E => cast_urgot_e(
            &mut commands,
            &q_transform,
            entity,
            trigger.point,
            skill_spell,
        ),
        SkillSlot::R => cast_urgot_r(&mut commands, entity, skill_spell),
        _ => {}
    }
}

fn cast_urgot_q(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Urgot_Q_Cast"));
    // Q is a mortar shot that damages and slows enemies in area
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Circle { radius: 200.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Urgot_Q_Hit")),
    );
}

fn cast_urgot_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Urgot_W_Cast"));

    // W is a toggle that makes Urgot fire at nearby enemies with reduced move speed
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffUrgotW::new(
            URGOT_W_ATTACK_INTERVAL,
            URGOT_W_MOVE_SPEED_REDUCTION,
            URGOT_W_MAX_RANGE,
        ));

    debug!(
        "{:?} 释放了 {} 技能，自动攻击周围敌人，移速降低 {}%",
        entity,
        "Urgot W",
        (URGOT_W_MOVE_SPEED_REDUCTION * 100.0) as i32
    );
}

fn cast_urgot_e(
    commands: &mut Commands,
    q_transform: &Query<&Transform>,
    entity: Entity,
    point: Vec2,
    skill_spell: Handle<Spell>,
) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Urgot_E_Cast"));
    // E is a dash that provides shield and knocks back enemies
    skill_dash(
        commands,
        q_transform,
        entity,
        point,
        &ActionDash {
            skill: skill_spell,
            move_type: DashMoveType::Pointer { max: 300.0 },
            damage: None, // E doesn't deal damage directly but knockback
            speed: 700.0,
        },
    );
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffShieldWhite::new(100.0));
}

fn cast_urgot_r(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Urgot_R_Cast"));
    // R is a long-range targeted ability that executes and marks enemy
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Nearest {
            max_distance: 500.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::Champion,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Urgot_R_Hit")),
    );
    debug!("{:?} R 发射钻头，低血量可处决", entity);
}

/// 厄加特Q命中时减速，R命中时挂斩杀标记
fn on_urgot_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_urgot: Query<(), With<Urgot>>,
) {
    let source = trigger.source;
    if q_urgot.get(source).is_err() {
        return;
    }
    let target = trigger.event_target();
    // Q applies slow
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffSlow::new(URGOT_Q_SLOW_PERCENT, URGOT_Q_SLOW_DURATION));
}
