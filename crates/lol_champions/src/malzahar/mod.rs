pub mod buffs;

use bevy::prelude::*;
use lol_base::animation_names::{ANIM_SPELL1, ANIM_SPELL2, ANIM_SPELL3, ANIM_SPELL4};
use lol_base::render_cmd::CommandAnimationPlay;
use lol_core::action::damage::{
    ActionDamage, ActionDamageEffect, DamageShape, TargetDamage, TargetFilter,
};
use lol_core::base::buff::BuffOf;
use lol_core::buffs::cc_debuffs::DebuffSilence;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{EventSkillCast, Skill, SkillSlot};

use crate::malzahar::buffs::BuffMalzaharE;

#[derive(Default)]
pub struct PluginMalzahar;

impl Plugin for PluginMalzahar {
    fn build(&self, app: &mut App) {
        app.add_observer(on_malzahar_q);
        app.add_observer(on_malzahar_w);
        app.add_observer(on_malzahar_e);
        app.add_observer(on_malzahar_r);
        app.add_observer(on_malzahar_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Malzahar"))]
#[reflect(Component)]
pub struct Malzahar;

fn on_malzahar_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_malzahar: Query<(), With<Malzahar>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_malzahar.get(entity).is_err() {
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
    // Q opens void gates and silences
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 900.0,
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

fn on_malzahar_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_malzahar: Query<(), With<Malzahar>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_malzahar.get(entity).is_err() {
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
    // W summons voidlings
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 150.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
        }],
    });
}

fn on_malzahar_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_malzahar: Query<(), With<Malzahar>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_malzahar.get(entity).is_err() {
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
    // E infects target
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

fn on_malzahar_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_malzahar: Query<(), With<Malzahar>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_malzahar.get(entity).is_err() {
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
    // R suppresses target
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

fn on_malzahar_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_malzahar: Query<(), With<Malzahar>>,
) {
    let source = trigger.source;
    if q_malzahar.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Q silences
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffSilence::new(1.5));

    // E applies infection
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffMalzaharE::new(50.0, 4.0));
}
