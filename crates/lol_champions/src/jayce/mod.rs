pub mod buffs;

use bevy::prelude::*;
use lol_base::animation_names::{ANIM_SPELL1, ANIM_SPELL2, ANIM_SPELL3, ANIM_SPELL4};
use lol_base::render_cmd::CommandAnimationPlay;
use lol_core::action::damage::{
    ActionDamage, ActionDamageEffect, DamageShape, TargetDamage, TargetFilter,
};
use lol_core::action::dash::{ActionDash, DashMoveType};
use lol_core::base::buff::BuffOf;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{EventSkillCast, Skill, SkillSlot};

use crate::jayce::buffs::BuffJaycePassive;

#[derive(Default)]
pub struct PluginJayce;

impl Plugin for PluginJayce {
    fn build(&self, app: &mut App) {
        app.add_observer(on_jayce_q);
        app.add_observer(on_jayce_w);
        app.add_observer(on_jayce_e);
        app.add_observer(on_jayce_r);
        app.add_observer(on_jayce_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Jayce"))]
#[reflect(Component)]
pub struct Jayce;

fn on_jayce_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_jayce: Query<(), With<Jayce>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_jayce.get(entity).is_err() {
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
    // Q is a skillshot
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 1050.0,
                angle: 15.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Physical,
            }],
        }],
    });
}

fn on_jayce_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_jayce: Query<(), With<Jayce>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_jayce.get(entity).is_err() {
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
    // W is an area slow
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 350.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
        }],
    });
}

fn on_jayce_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_jayce: Query<(), With<Jayce>>,
    _q_transform: Query<&Transform>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_jayce.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::E) {
        return;
    }

    let point = trigger.point;
    let _skill_spell = skill.spell.clone();
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL3.to_string(),
        repeat: false,
        duration: None,
    });
    // E is a knockback
    commands.trigger(ActionDash {
        entity,
        point: point,
        move_type: DashMoveType::Pointer { max: 500.0 },
        speed: 1000.0,
    });
}

fn on_jayce_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_jayce: Query<(), With<Jayce>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_jayce.get(entity).is_err() {
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
    // R transforms between hammer and cannon forms;
}

fn on_jayce_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_jayce: Query<(), With<Jayce>>,
) {
    let source = trigger.source;
    if q_jayce.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Apply passive
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffJaycePassive::new());
}
