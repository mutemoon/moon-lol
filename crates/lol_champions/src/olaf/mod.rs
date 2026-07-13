pub mod buffs;

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

use crate::olaf::buffs::{BuffOlafR, BuffOlafW};

// Olaf W parameters
const OLAF_W_ATTACK_SPEED_BONUS: f32 = 0.4; // 40% attack speed
const OLAF_W_SHIELD: f32 = 80.0; // shield amount
const OLAF_W_DURATION: f32 = 5.0; // 5 seconds

// Olaf R parameters
const OLAF_R_DURATION: f32 = 6.0; // 6 seconds CC immunity

#[derive(Default)]
pub struct PluginOlaf;

impl Plugin for PluginOlaf {
    fn build(&self, app: &mut App) {
        app.add_observer(on_olaf_q);
        app.add_observer(on_olaf_w);
        app.add_observer(on_olaf_e);
        app.add_observer(on_olaf_r);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Olaf"))]
#[reflect(Component)]
pub struct Olaf;

fn on_olaf_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_olaf: Query<(), With<Olaf>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_olaf.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::Q) {
        return;
    }

    let _point = trigger.point;
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL1.to_string(),
        repeat: false,
        duration: None,
    });
    // Q is a linear axe throw - could add dash damage or just particle;
}

fn on_olaf_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_olaf: Query<(), With<Olaf>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_olaf.get(entity).is_err() {
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
    // W provides attack speed buff and shield
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffOlafW::new(
            OLAF_W_ATTACK_SPEED_BONUS,
            OLAF_W_SHIELD,
            OLAF_W_DURATION,
        ));

    debug!(
        "{:?} 释放了 {} 技能，获得 {}% 攻速加成和 {} 护盾",
        entity,
        "Olaf W",
        (OLAF_W_ATTACK_SPEED_BONUS * 100.0) as i32,
        OLAF_W_SHIELD as i32
    );
}

fn on_olaf_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_olaf: Query<(), With<Olaf>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_olaf.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::E) {
        return;
    }

    let _point = trigger.point;
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
            shape: DamageShape::Nearest {
                max_distance: 200.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::True,
            }],
        }],
    });
    // E also deals self-damage;
}

fn on_olaf_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_olaf: Query<(), With<Olaf>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_olaf.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::R) {
        return;
    }

    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL4.to_string(),
        repeat: false,
        duration: None,
    });
    // R provides CC immunity
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffOlafR::new(OLAF_R_DURATION));

    debug!(
        "{:?} 释放了 {} 技能，免疫控制效果持续 {} 秒",
        entity, "Olaf R", OLAF_R_DURATION
    );
}
