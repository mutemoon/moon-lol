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

use crate::rell::buffs::{BuffRellE, BuffRellR, BuffRellW};

#[derive(Default)]
pub struct PluginRell;

impl Plugin for PluginRell {
    fn build(&self, app: &mut App) {
        app.add_observer(on_rell_q);
        app.add_observer(on_rell_w);
        app.add_observer(on_rell_e);
        app.add_observer(on_rell_r);
        app.add_observer(on_rell_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Rell"))]
#[reflect(Component)]
pub struct Rell;

fn on_rell_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_rell: Query<(), With<Rell>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_rell.get(entity).is_err() {
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
    // Q is shattering strike - damage and armor reduction
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 500.0,
                angle: 30.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
        }],
    });
}

fn on_rell_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_rell: Query<(), With<Rell>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_rell.get(entity).is_err() {
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
    // W is crashing blow - damage and knockup
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 400.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
        }],
    });
}

fn on_rell_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_rell: Query<(), With<Rell>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_rell.get(entity).is_err() {
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
    // E is full bind - stun
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

fn on_rell_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_rell: Query<(), With<Rell>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_rell.get(entity).is_err() {
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
    // R is catharsis - AoE damage and slow
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 400.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
        }],
    });
}

fn on_rell_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_rell: Query<(), With<Rell>>,
) {
    let source = trigger.source;
    if q_rell.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // W slows
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffRellW::new(0.5, 1.5));
    // E stuns
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffRellE::new(0.75, 1.0));
    // R slows
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffRellR::new(0.4, 2.0));
}
