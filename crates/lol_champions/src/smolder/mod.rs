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

use crate::smolder::buffs::BuffSmolderW;

#[derive(Default)]
pub struct PluginSmolder;

impl Plugin for PluginSmolder {
    fn build(&self, app: &mut App) {
        app.add_observer(on_smolder_q);
        app.add_observer(on_smolder_w);
        app.add_observer(on_smolder_e);
        app.add_observer(on_smolder_r);
        app.add_observer(on_smolder_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Smolder"))]
#[reflect(Component)]
pub struct Smolder;

fn on_smolder_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_smolder: Query<(), With<Smolder>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_smolder.get(entity).is_err() {
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
    // Q is searing strike - damage
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Nearest {
                max_distance: 550.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
        }],
    });
}

fn on_smolder_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_smolder: Query<(), With<Smolder>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_smolder.get(entity).is_err() {
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
    // W is deep fire brand - damage and slow
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 300.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
        }],
    });
}

fn on_smolder_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_smolder: Query<(), With<Smolder>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_smolder.get(entity).is_err() {
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
    // E is super hot - movespeed;
}

fn on_smolder_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_smolder: Query<(), With<Smolder>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_smolder.get(entity).is_err() {
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
    // R is dragonfire storm - AoE damage
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 1200.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
        }],
    });
}

fn on_smolder_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_smolder: Query<(), With<Smolder>>,
) {
    let source = trigger.source;
    if q_smolder.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // W slows
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffSmolderW::new(0.3, 1.5));
}
