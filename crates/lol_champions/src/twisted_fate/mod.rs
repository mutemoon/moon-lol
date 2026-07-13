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

use crate::twisted_fate::buffs::{BuffTwistedFateWSlow, BuffTwistedFateWStun};

#[derive(Default)]
pub struct PluginTwistedFate;

impl Plugin for PluginTwistedFate {
    fn build(&self, app: &mut App) {
        app.add_observer(on_twisted_fate_q);
        app.add_observer(on_twisted_fate_w);
        app.add_observer(on_twisted_fate_e);
        app.add_observer(on_twisted_fate_r);
        app.add_observer(on_twisted_fate_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("TwistedFate"))]
#[reflect(Component)]
pub struct TwistedFate;

fn on_twisted_fate_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_twisted_fate: Query<(), With<TwistedFate>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_twisted_fate.get(entity).is_err() {
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
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 1450.0,
                angle: 25.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
        }],
    });
}

fn on_twisted_fate_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_twisted_fate: Query<(), With<TwistedFate>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_twisted_fate.get(entity).is_err() {
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
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 325.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
        }],
    });
}

fn on_twisted_fate_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_twisted_fate: Query<(), With<TwistedFate>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_twisted_fate.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::E) {
        return;
    }

    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL3.to_string(),
        repeat: false,
        duration: None,
    });
}

fn on_twisted_fate_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_twisted_fate: Query<(), With<TwistedFate>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_twisted_fate.get(entity).is_err() {
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
}

fn on_twisted_fate_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_twisted_fate: Query<(), With<TwistedFate>>,
) {
    let source = trigger.source;
    if q_twisted_fate.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    commands
        .entity(target)
        .with_related::<BuffOf>(BuffTwistedFateWSlow::new(0.35, 2.0));
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffTwistedFateWStun::new(1.5));
}
