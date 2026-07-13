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

use crate::pyke::buffs::{BuffPykeE, BuffPykeQ};

#[derive(Default)]
pub struct PluginPyke;

impl Plugin for PluginPyke {
    fn build(&self, app: &mut App) {
        app.add_observer(on_pyke_q);
        app.add_observer(on_pyke_w);
        app.add_observer(on_pyke_e);
        app.add_observer(on_pyke_r);
        app.add_observer(on_pyke_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Pyke"))]
#[reflect(Component)]
pub struct Pyke;

fn on_pyke_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_pyke: Query<(), With<Pyke>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_pyke.get(entity).is_err() {
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
    // Q is bone skewer - damage and pull
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 1100.0,
                angle: 10.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Physical,
            }],
        }],
    });
}

fn on_pyke_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_pyke: Query<(), With<Pyke>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_pyke.get(entity).is_err() {
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
    // W is ghostwater dive - invisibility and movespeed;
}

fn on_pyke_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_pyke: Query<(), With<Pyke>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_pyke.get(entity).is_err() {
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
    // E is phantom undertow - dash and stun on return
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 550.0,
                angle: 20.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Physical,
            }],
        }],
    });
}

fn on_pyke_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_pyke: Query<(), With<Pyke>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_pyke.get(entity).is_err() {
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
    // R is death from below - execute damage in AoE
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 750.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Physical,
            }],
        }],
    });
}

fn on_pyke_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_pyke: Query<(), With<Pyke>>,
) {
    let source = trigger.source;
    if q_pyke.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Q slows
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffPykeQ::new(0.9, 1.0));
    // E stuns
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffPykeE::new(1.25, 1.5));
}
