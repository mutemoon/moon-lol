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

use crate::trundle::buffs::BuffTrundleQ;

#[derive(Default)]
pub struct PluginTrundle;

impl Plugin for PluginTrundle {
    fn build(&self, app: &mut App) {
        app.add_observer(on_trundle_q);
        app.add_observer(on_trundle_w);
        app.add_observer(on_trundle_e);
        app.add_observer(on_trundle_r);
        app.add_observer(on_trundle_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Trundle"))]
#[reflect(Component)]
pub struct Trundle;

fn on_trundle_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_trundle: Query<(), With<Trundle>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_trundle.get(entity).is_err() {
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
}

fn on_trundle_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_trundle: Query<(), With<Trundle>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_trundle.get(entity).is_err() {
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
}

fn on_trundle_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_trundle: Query<(), With<Trundle>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_trundle.get(entity).is_err() {
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
            shape: DamageShape::Circle { radius: 400.0 },
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

fn on_trundle_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_trundle: Query<(), With<Trundle>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_trundle.get(entity).is_err() {
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
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Nearest {
                max_distance: 650.0,
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

fn on_trundle_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_trundle: Query<(), With<Trundle>>,
) {
    let source = trigger.source;
    if q_trundle.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    commands
        .entity(target)
        .with_related::<BuffOf>(BuffTrundleQ::new(0.4, 2.0));
}
