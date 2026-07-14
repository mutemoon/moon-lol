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

use crate::rumble::buffs::BuffRumbleW;

#[derive(Default)]
pub struct PluginRumble;

impl Plugin for PluginRumble {
    fn build(&self, app: &mut App) {
        app.add_observer(on_rumble_q);
        app.add_observer(on_rumble_w);
        app.add_observer(on_rumble_e);
        app.add_observer(on_rumble_r);
        app.add_observer(on_rumble_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Rumble"))]
#[reflect(Component)]
pub struct Rumble;

fn on_rumble_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_rumble: Query<(), With<Rumble>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_rumble.get(entity).is_err() {
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
    // Q is electro harpoon - damage over time
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 600.0,
                angle: 30.0,
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

fn on_rumble_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_rumble: Query<(), With<Rumble>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_rumble.get(entity).is_err() {
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
    // W is scrap shield - shield
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffRumbleW::new(50.0, 1.5));
}

fn on_rumble_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_rumble: Query<(), With<Rumble>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_rumble.get(entity).is_err() {
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
    // E is electro harpoon - damage
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 850.0,
                angle: 15.0,
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

fn on_rumble_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_rumble: Query<(), With<Rumble>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_rumble.get(entity).is_err() {
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
    // R is electro fire - large AoE damage
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 900.0 },
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

fn on_rumble_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_rumble: Query<(), With<Rumble>>,
) {
    let source = trigger.source;
    if q_rumble.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // E slows
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffRumbleW::new(50.0, 1.5));
}
