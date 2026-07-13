pub mod buffs;

use bevy::prelude::*;
use lol_base::animation_names::{ANIM_SPELL1, ANIM_SPELL2, ANIM_SPELL3, ANIM_SPELL4};
use lol_base::render_cmd::CommandAnimationPlay;
use lol_core::action::damage::{
    ActionDamage, ActionDamageEffect, DamageShape, TargetDamage, TargetFilter,
};
use lol_core::base::buff::BuffOf;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{EventSkillCast, Skill, SkillSlot};

use crate::nasus::buffs::BuffNasusW;

#[derive(Default)]
pub struct PluginNasus;

impl Plugin for PluginNasus {
    fn build(&self, app: &mut App) {
        app.add_observer(on_nasus_q);
        app.add_observer(on_nasus_w);
        app.add_observer(on_nasus_e);
        app.add_observer(on_nasus_r);
        app.add_observer(on_nasus_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Nasus"))]
#[reflect(Component)]
pub struct Nasus;

fn on_nasus_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_nasus: Query<(), With<Nasus>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_nasus.get(entity).is_err() {
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
    // Q is a siphoning strike;
}

fn on_nasus_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_nasus: Query<(), With<Nasus>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_nasus.get(entity).is_err() {
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
    // W is a slow
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 700.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
        }],
    });
}

fn on_nasus_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_nasus: Query<(), With<Nasus>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_nasus.get(entity).is_err() {
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
    // E is an area damage and armor reduction
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 650.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
        }],
    });
}

fn on_nasus_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_nasus: Query<(), With<Nasus>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_nasus.get(entity).is_err() {
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
    // R transforms Nasus
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 500.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
        }],
    });
}

fn on_nasus_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_nasus: Query<(), With<Nasus>>,
) {
    let source = trigger.source;
    if q_nasus.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // W slows
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffNasusW::new(0.5, 5.0));
}
