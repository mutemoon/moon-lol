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

use crate::seraphine::buffs::{BuffSeraphineE, BuffSeraphineW};

#[derive(Default)]
pub struct PluginSeraphine;

impl Plugin for PluginSeraphine {
    fn build(&self, app: &mut App) {
        app.add_observer(on_seraphine_q);
        app.add_observer(on_seraphine_w);
        app.add_observer(on_seraphine_e);
        app.add_observer(on_seraphine_r);
        app.add_observer(on_seraphine_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Seraphine"))]
#[reflect(Component)]
pub struct Seraphine;

fn on_seraphine_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_seraphine: Query<(), With<Seraphine>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_seraphine.get(entity).is_err() {
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
    // Q is high note - damage
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 900.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
        }],
    });
}

fn on_seraphine_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_seraphine: Query<(), With<Seraphine>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_seraphine.get(entity).is_err() {
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
    // W is solo - shield and slow
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffSeraphineW::new(50.0, 2.5));
}

fn on_seraphine_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_seraphine: Query<(), With<Seraphine>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_seraphine.get(entity).is_err() {
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
    // E is beat drop - stun
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 1300.0,
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

fn on_seraphine_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_seraphine: Query<(), With<Seraphine>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_seraphine.get(entity).is_err() {
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
    // R is encore - AoE charm
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 1500.0,
                angle: 50.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
        }],
    });
}

fn on_seraphine_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_seraphine: Query<(), With<Seraphine>>,
) {
    let source = trigger.source;
    if q_seraphine.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // E stuns
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffSeraphineE::new(0.75, 1.0));
}
