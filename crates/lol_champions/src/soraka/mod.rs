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

use crate::soraka::buffs::BuffSorakaE;

#[derive(Default)]
pub struct PluginSoraka;

impl Plugin for PluginSoraka {
    fn build(&self, app: &mut App) {
        app.add_observer(on_soraka_q);
        app.add_observer(on_soraka_w);
        app.add_observer(on_soraka_e);
        app.add_observer(on_soraka_r);
        app.add_observer(on_soraka_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Soraka"))]
#[reflect(Component)]
pub struct Soraka;

fn on_soraka_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_soraka: Query<(), With<Soraka>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_soraka.get(entity).is_err() {
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
    // Q is starlon fallback - damage
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 575.0 },
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

fn on_soraka_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_soraka: Query<(), With<Soraka>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_soraka.get(entity).is_err() {
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
    // W is infuse magic - heal;
}

fn on_soraka_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_soraka: Query<(), With<Soraka>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_soraka.get(entity).is_err() {
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
    // E is barrier of mind - silence
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 300.0 },
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

fn on_soraka_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_soraka: Query<(), With<Soraka>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_soraka.get(entity).is_err() {
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
    // R is wishes - global heal;
}

fn on_soraka_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_soraka: Query<(), With<Soraka>>,
) {
    let source = trigger.source;
    if q_soraka.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // E silences
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffSorakaE::new(0.5, 1.0));
}
