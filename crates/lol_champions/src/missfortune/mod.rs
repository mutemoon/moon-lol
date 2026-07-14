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

use crate::missfortune::buffs::BuffMissFortuneW;

#[derive(Default)]
pub struct PluginMissFortune;

impl Plugin for PluginMissFortune {
    fn build(&self, app: &mut App) {
        app.add_observer(on_missfortune_q);
        app.add_observer(on_missfortune_w);
        app.add_observer(on_missfortune_e);
        app.add_observer(on_missfortune_r);
        app.add_observer(on_missfortune_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("MissFortune"))]
#[reflect(Component)]
pub struct MissFortune;

fn on_missfortune_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_missfortune: Query<(), With<MissFortune>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_missfortune.get(entity).is_err() {
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
    // Q bounces to second target
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 550.0,
                angle: 10.0,
            },
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

fn on_missfortune_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_missfortune: Query<(), With<MissFortune>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_missfortune.get(entity).is_err() {
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
    // W grants movespeed and attackspeed
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffMissFortuneW::new(0.6, 1.0, 4.0));
}

fn on_missfortune_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_missfortune: Query<(), With<MissFortune>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_missfortune.get(entity).is_err() {
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
    // E is a zone that slows
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 1000.0 },
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

fn on_missfortune_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_missfortune: Query<(), With<MissFortune>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_missfortune.get(entity).is_err() {
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
    // R is a cone of bullets
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 1450.0,
                angle: 45.0,
            },
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

fn on_missfortune_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_missfortune: Query<(), With<MissFortune>>,
) {
    let source = trigger.source;
    if q_missfortune.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // E slows
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffSlow::new(0.4, 2.0));
}
