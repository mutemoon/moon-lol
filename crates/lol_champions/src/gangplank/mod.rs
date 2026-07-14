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

use crate::gangplank::buffs::BuffGangplankPassive;

#[derive(Default)]
pub struct PluginGangplank;

impl Plugin for PluginGangplank {
    fn build(&self, app: &mut App) {
        app.add_observer(on_gangplank_q);
        app.add_observer(on_gangplank_w);
        app.add_observer(on_gangplank_e);
        app.add_observer(on_gangplank_r);
        app.add_observer(on_gangplank_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Gangplank"))]
#[reflect(Component)]
pub struct Gangplank;

fn on_gangplank_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_gangplank: Query<(), With<Gangplank>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_gangplank.get(entity).is_err() {
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
    // Q is targeted damage
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Nearest {
                max_distance: 625.0,
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

fn on_gangplank_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_gangplank: Query<(), With<Gangplank>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_gangplank.get(entity).is_err() {
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
    // W removes CC and heals;
}

fn on_gangplank_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_gangplank: Query<(), With<Gangplank>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_gangplank.get(entity).is_err() {
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
    // E places barrel;
}

fn on_gangplank_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_gangplank: Query<(), With<Gangplank>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_gangplank.get(entity).is_err() {
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
    // R is global AoE
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 20000.0 },
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

fn on_gangplank_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_gangplank: Query<(), With<Gangplank>>,
) {
    let source = trigger.source;
    if q_gangplank.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Apply passive
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffGangplankPassive::new());
}
