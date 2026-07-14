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

#[derive(Default)]
pub struct PluginIvern;

impl Plugin for PluginIvern {
    fn build(&self, app: &mut App) {
        app.add_observer(on_ivern_q);
        app.add_observer(on_ivern_w);
        app.add_observer(on_ivern_e);
        app.add_observer(on_ivern_r);
        app.add_observer(on_ivern_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Ivern"))]
#[reflect(Component)]
pub struct Ivern;

fn on_ivern_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_ivern: Query<(), With<Ivern>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_ivern.get(entity).is_err() {
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
    // Q roots enemies
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 1150.0,
                angle: 20.0,
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

fn on_ivern_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_ivern: Query<(), With<Ivern>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_ivern.get(entity).is_err() {
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
    // W creates brush;
}

fn on_ivern_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_ivern: Query<(), With<Ivern>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_ivern.get(entity).is_err() {
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
    // E is a shield that explodes
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 750.0 },
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

fn on_ivern_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_ivern: Query<(), With<Ivern>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_ivern.get(entity).is_err() {
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
    // R summons Daisy;
}

fn on_ivern_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_ivern: Query<(), With<Ivern>>,
) {
    let source = trigger.source;
    if q_ivern.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Q roots
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffSlow::new(0.5, 2.0));
}
