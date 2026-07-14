pub mod buffs;

use bevy::prelude::*;
use lol_base::animation_names::{ANIM_SPELL1, ANIM_SPELL2, ANIM_SPELL3, ANIM_SPELL4};
use lol_base::render_cmd::CommandAnimationPlay;
use lol_core::action::damage::{
    ActionDamage, ActionDamageEffect, DamageShape, TargetDamage, TargetFilter,
};
use lol_core::base::buff::BuffOf;
use lol_core::buffs::cc_debuffs::DebuffSlow;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{EventSkillCast, Skill, SkillSlot};

use crate::maokai::buffs::BuffMaokaiW;

#[derive(Default)]
pub struct PluginMaokai;

impl Plugin for PluginMaokai {
    fn build(&self, app: &mut App) {
        app.add_observer(on_maokai_q);
        app.add_observer(on_maokai_w);
        app.add_observer(on_maokai_e);
        app.add_observer(on_maokai_r);
        app.add_observer(on_maokai_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Maokai"))]
#[reflect(Component)]
pub struct Maokai;

fn on_maokai_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_maokai: Query<(), With<Maokai>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_maokai.get(entity).is_err() {
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
    // Q is a knockback
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 350.0,
                angle: 60.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
                ..Default::default()
            }],
            ..Default::default()
        }],
    });
}

fn on_maokai_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_maokai: Query<(), With<Maokai>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_maokai.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::W) {
        return;
    }

    let skill_spell = skill.spell.clone();
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL2.to_string(),
        repeat: false,
        duration: None,
    });
    // W is a dash that roots
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 525.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
                ..Default::default()
            }],
            ..Default::default()
        }],
    });
}

fn on_maokai_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_maokai: Query<(), With<Maokai>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_maokai.get(entity).is_err() {
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
    // E throws sapling that slows
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 1100.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
                ..Default::default()
            }],
            ..Default::default()
        }],
    });
}

fn on_maokai_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_maokai: Query<(), With<Maokai>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_maokai.get(entity).is_err() {
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
    // R is a global knockup
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 3000.0,
                angle: 45.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
                ..Default::default()
            }],
            ..Default::default()
        }],
    });
}

fn on_maokai_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_maokai: Query<(), With<Maokai>>,
) {
    let source = trigger.source;
    if q_maokai.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // W roots
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffMaokaiW::new(2.0, 2.0));

    // E slows
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffSlow::new(0.35, 3.0));
}
