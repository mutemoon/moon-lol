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

use crate::belveth::buffs::{BuffBelvethPassive, BuffBelvethW};

#[derive(Default)]
pub struct PluginBelveth;

impl Plugin for PluginBelveth {
    fn build(&self, app: &mut App) {
        app.add_observer(on_belveth_q);
        app.add_observer(on_belveth_w);
        app.add_observer(on_belveth_e);
        app.add_observer(on_belveth_r);
        app.add_observer(on_belveth_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Belveth"))]
#[reflect(Component)]
pub struct Belveth;

fn on_belveth_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_belveth: Query<(), With<Belveth>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_belveth.get(entity).is_err() {
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
            shape: DamageShape::Circle { radius: 400.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Physical,
                ..Default::default()
            }],
            ..Default::default()
        }],
    });
}

fn on_belveth_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_belveth: Query<(), With<Belveth>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_belveth.get(entity).is_err() {
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
            shape: DamageShape::Sector {
                radius: 660.0,
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

fn on_belveth_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_belveth: Query<(), With<Belveth>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_belveth.get(entity).is_err() {
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
            shape: DamageShape::Circle { radius: 500.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Physical,
                ..Default::default()
            }],
            ..Default::default()
        }],
    });
}

fn on_belveth_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_belveth: Query<(), With<Belveth>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_belveth.get(entity).is_err() {
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

fn on_belveth_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_belveth: Query<(), With<Belveth>>,
) {
    let source = trigger.source;
    if q_belveth.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    commands
        .entity(target)
        .with_related::<BuffOf>(BuffBelvethPassive::new(2, 0.1, 5.0));
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffBelvethW::new(0.5, 2.0));
}
