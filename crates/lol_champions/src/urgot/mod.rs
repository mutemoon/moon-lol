pub mod buffs;

use bevy::prelude::*;
use lol_base::animation_names::{ANIM_SPELL1, ANIM_SPELL2, ANIM_SPELL3, ANIM_SPELL4};
use lol_base::render_cmd::CommandAnimationPlay;
use lol_core::action::damage::{
    ActionDamage, ActionDamageEffect, DamageShape, TargetDamage, TargetFilter,
};
use lol_core::action::dash::{ActionDash, DashMoveType};
use lol_core::base::buff::BuffOf;
use lol_core::buffs::cc_debuffs::DebuffSlow;
use lol_core::buffs::shield_white::BuffShieldWhite;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{EventSkillCast, Skill, SkillSlot};

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
        app.add_observer(on_urgot_q);
        app.add_observer(on_urgot_w);
        app.add_observer(on_urgot_e);
        app.add_observer(on_urgot_r);
        app.add_observer(on_urgot_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Urgot"))]
#[reflect(Component)]
pub struct Urgot;

fn on_urgot_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_urgot: Query<(), With<Urgot>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_urgot.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::Q) {
        return;
    }

    let skill_spell = skill.spell.clone();
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL1.to_string(),
        repeat: false,
        duration: None,
    });
    // Q is a mortar shot that damages and slows enemies in area
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 200.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Physical,
            }],
        }],
    });
}

fn on_urgot_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_urgot: Query<(), With<Urgot>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_urgot.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::W) {
        return;
    }

    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL2.to_string(),
        repeat: false,
        duration: None,
    });
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

fn on_urgot_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_urgot: Query<(), With<Urgot>>,
    _q_transform: Query<&Transform>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_urgot.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::E) {
        return;
    }

    let point = trigger.point;
    let _skill_spell = skill.spell.clone();
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL3.to_string(),
        repeat: false,
        duration: None,
    });
    // E is a dash that provides shield and knocks back enemies
    commands.trigger(ActionDash {
        entity,
        point: point,
        move_type: DashMoveType::Pointer { max: 300.0 }, // E doesn't deal damage directly but knockback
        speed: 700.0,
    });
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffShieldWhite::new(100.0));
}

fn on_urgot_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_urgot: Query<(), With<Urgot>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_urgot.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::R) {
        return;
    }

    let skill_spell = skill.spell.clone();
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL4.to_string(),
        repeat: false,
        duration: None,
    });
    // R is a long-range targeted ability that executes and marks enemy
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Nearest {
                max_distance: 500.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::Champion,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Physical,
            }],
        }],
    });
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
