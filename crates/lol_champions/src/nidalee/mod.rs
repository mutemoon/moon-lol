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

use crate::nidalee::buffs::{BuffNidaleeE, BuffNidaleeQ};

#[derive(Default)]
pub struct PluginNidalee;

impl Plugin for PluginNidalee {
    fn build(&self, app: &mut App) {
        app.add_observer(on_nidalee_q);
        app.add_observer(on_nidalee_w);
        app.add_observer(on_nidalee_e);
        app.add_observer(on_nidalee_r);
        app.add_observer(on_nidalee_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Nidalee"))]
#[reflect(Component)]
pub struct Nidalee;

fn on_nidalee_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_nidalee: Query<(), With<Nidalee>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_nidalee.get(entity).is_err() {
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
    // Q is a spear (human form)
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 1500.0,
                angle: 10.0,
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

fn on_nidalee_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_nidalee: Query<(), With<Nidalee>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_nidalee.get(entity).is_err() {
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
    // W is a trap (human form) or pounce (cougar form)
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

fn on_nidalee_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_nidalee: Query<(), With<Nidalee>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_nidalee.get(entity).is_err() {
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
    // E is a heal (human form) or swipe (cougar form)
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffNidaleeE::new(100.0, 0.7, 7.0));

    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 300.0 },
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

fn on_nidalee_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_nidalee: Query<(), With<Nidalee>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_nidalee.get(entity).is_err() {
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
    // R transforms between human and cougar forms;
}

fn on_nidalee_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_nidalee: Query<(), With<Nidalee>>,
) {
    let source = trigger.source;
    if q_nidalee.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Q marks target
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffNidaleeQ::new(100.0, 4.0));
}
