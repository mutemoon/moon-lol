pub mod q;
pub mod w;

use bevy::prelude::*;
use lol_base::animation_names::{ANIM_SPELL1, ANIM_SPELL2, ANIM_SPELL3, ANIM_SPELL4};
use lol_base::render_cmd::CommandAnimationPlay;
use lol_core::action::damage::{
    ActionDamage, ActionDamageEffect, DamageShape, TargetDamage, TargetFilter,
};
use lol_core::base::buff::BuffOf;
use lol_core::damage::DamageType;
use lol_core::entities::champion::Champion;
use lol_core::skill::{EventSkillCast, Skill, SkillSlot};

use crate::garen::q::{BuffGarenQ, BuffGarenQAttack};
use crate::garen::w::BuffGarenW;

// Garen Q parameters
const GAREN_Q_MOVE_SPEED_BONUS: f32 = 0.3; // 30% move speed bonus
const GAREN_Q_DURATION: f32 = 1.5; // 1.5 seconds
const GAREN_Q_SILENCE_DURATION: f32 = 1.0; // 1 second silence

// Garen W parameters
const GAREN_W_TENACITY: f32 = 0.3; // 30% tenacity
const GAREN_W_DAMAGE_REDUCTION: f32 = 0.3; // 30% damage reduction
const GAREN_W_SHIELD: f32 = 100.0; // shield amount
const GAREN_W_DURATION: f32 = 2.0; // 2 seconds duration

#[derive(Default)]
pub struct PluginGaren;

impl Plugin for PluginGaren {
    fn build(&self, app: &mut App) {
        app.add_observer(on_garen_q);
        app.add_observer(on_garen_w);
        app.add_observer(on_garen_e);
        app.add_observer(on_garen_r);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Garen"))]
#[reflect(Component)]
pub struct Garen;

fn on_garen_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_garen: Query<(), With<Garen>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_garen.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::Q) {
        return;
    }

    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL1.to_string(),
        repeat: false,
        duration: None,
    });
    // Q provides movement speed buff and enhanced next attack
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffGarenQ::new(GAREN_Q_MOVE_SPEED_BONUS, GAREN_Q_DURATION));

    // Add the enhanced attack buff (silence on hit)
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffGarenQAttack::new(GAREN_Q_SILENCE_DURATION));

    debug!(
        "{:?} 释放了 {} 技能，获得 {}% 移速加成和沉默效果",
        entity,
        "Garen Q",
        (GAREN_Q_MOVE_SPEED_BONUS * 100.0) as i32
    );
}

fn on_garen_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_garen: Query<(), With<Garen>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_garen.get(entity).is_err() {
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
    // W provides tenacity, damage reduction, and a shield
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffGarenW::new(
            GAREN_W_TENACITY,
            GAREN_W_DAMAGE_REDUCTION,
            GAREN_W_SHIELD,
            GAREN_W_DURATION,
        ));

    debug!(
        "{:?} 释放了 {} 技能，获得 {}% 韧性、{}% 伤害减免和 {} 护盾",
        entity,
        "Garen W",
        (GAREN_W_TENACITY * 100.0) as i32,
        (GAREN_W_DAMAGE_REDUCTION * 100.0) as i32,
        GAREN_W_SHIELD as i32
    );
}

fn on_garen_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_garen: Query<(), With<Garen>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_garen.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::E) {
        return;
    }

    let skill_spell = skill.spell.clone();
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL3.to_string(),
        repeat: false,
        duration: None,
    });
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

fn on_garen_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_garen: Query<(), With<Garen>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_garen.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::R) {
        return;
    }

    let _point = trigger.point;
    let skill_spell = skill.spell.clone();
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL4.to_string(),
        repeat: false,
        duration: None,
    });
    // R is a targeted execute ability - use Nearest shape for single target
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Nearest {
                max_distance: 400.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::Champion,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Physical,
            }],
        }],
    });
}
