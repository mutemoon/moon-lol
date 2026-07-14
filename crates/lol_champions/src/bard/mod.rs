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
pub struct PluginBard;

impl Plugin for PluginBard {
    fn build(&self, app: &mut App) {
        app.add_observer(on_bard_q);
        app.add_observer(on_bard_w);
        app.add_observer(on_bard_e);
        app.add_observer(on_bard_r);
        app.add_observer(on_bard_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Bard"))]
#[reflect(Component)]
pub struct Bard;

fn on_bard_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_bard: Query<(), With<Bard>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_bard.get(entity).is_err() {
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
    // Q is a binding missile
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 850.0,
                angle: 30.0,
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

fn on_bard_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_bard: Query<(), With<Bard>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_bard.get(entity).is_err() {
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
    // W is a heal shrine - no direct damage;
}

fn on_bard_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_bard: Query<(), With<Bard>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_bard.get(entity).is_err() {
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
    // E is a tunnel - no direct damage;
}

fn on_bard_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_bard: Query<(), With<Bard>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_bard.get(entity).is_err() {
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
    // R is a global AoE stun
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 3400.0 },
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

fn on_bard_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_bard: Query<(), With<Bard>>,
) {
    let source = trigger.source;
    if q_bard.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Q slows
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffSlow::new(0.6, 1.5));
}
