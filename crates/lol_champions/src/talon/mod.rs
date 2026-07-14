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

use crate::talon::buffs::BuffTalonW;

#[derive(Default)]
pub struct PluginTalon;

impl Plugin for PluginTalon {
    fn build(&self, app: &mut App) {
        app.add_observer(on_talon_q);
        app.add_observer(on_talon_w);
        app.add_observer(on_talon_e);
        app.add_observer(on_talon_r);
        app.add_observer(on_talon_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Talon"))]
#[reflect(Component)]
pub struct Talon;

fn on_talon_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_talon: Query<(), With<Talon>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_talon.get(entity).is_err() {
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
            shape: DamageShape::Nearest {
                max_distance: 600.0,
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

fn on_talon_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_talon: Query<(), With<Talon>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_talon.get(entity).is_err() {
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
                radius: 600.0,
                angle: 30.0,
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

fn on_talon_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_talon: Query<(), With<Talon>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_talon.get(entity).is_err() {
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

fn on_talon_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_talon: Query<(), With<Talon>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_talon.get(entity).is_err() {
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
            shape: DamageShape::Circle { radius: 600.0 },
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

fn on_talon_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_talon: Query<(), With<Talon>>,
) {
    let source = trigger.source;
    if q_talon.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    commands
        .entity(target)
        .with_related::<BuffOf>(BuffTalonW::new(0.4, 2.0));
}
