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

use crate::ornn::buffs::BuffOrnnQ;

#[derive(Default)]
pub struct PluginOrnn;

impl Plugin for PluginOrnn {
    fn build(&self, app: &mut App) {
        app.add_observer(on_ornn_q);
        app.add_observer(on_ornn_w);
        app.add_observer(on_ornn_e);
        app.add_observer(on_ornn_r);
        app.add_observer(on_ornn_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Ornn"))]
#[reflect(Component)]
pub struct Ornn;

fn on_ornn_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_ornn: Query<(), With<Ornn>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_ornn.get(entity).is_err() {
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
    // Q is volcanic rupture - damage and slow
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 750.0,
                angle: 30.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Physical,
            }],
        }],
    });

    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffOrnnQ::new(0.4, 2.0));
}

fn on_ornn_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_ornn: Query<(), With<Ornn>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_ornn.get(entity).is_err() {
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
    // W is bellows breath - continuous damage and brittle
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 500.0,
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

fn on_ornn_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_ornn: Query<(), With<Ornn>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_ornn.get(entity).is_err() {
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
    // E is searing charge - dash that creates shockwave on terrain hit
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 350.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Physical,
            }],
        }],
    });
}

fn on_ornn_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_ornn: Query<(), With<Ornn>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_ornn.get(entity).is_err() {
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
    // R is call of the forge god - large AoE damage and knockup
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 3000.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
        }],
    });
}

fn on_ornn_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_ornn: Query<(), With<Ornn>>,
) {
    let source = trigger.source;
    if q_ornn.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Q slows
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffOrnnQ::new(0.4, 2.0));
}
