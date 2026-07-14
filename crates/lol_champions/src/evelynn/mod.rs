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
pub struct PluginEvelynn;

impl Plugin for PluginEvelynn {
    fn build(&self, app: &mut App) {
        app.add_observer(on_evelynn_q);
        app.add_observer(on_evelynn_w);
        app.add_observer(on_evelynn_e);
        app.add_observer(on_evelynn_r);
        app.add_observer(on_evelynn_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Evelynn"))]
#[reflect(Component)]
pub struct Evelynn;

fn on_evelynn_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_evelynn: Query<(), With<Evelynn>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_evelynn.get(entity).is_err() {
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
                radius: 800.0,
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

fn on_evelynn_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_evelynn: Query<(), With<Evelynn>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_evelynn.get(entity).is_err() {
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
    // W is a charm/slow - handled by damage observer;
}

fn on_evelynn_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_evelynn: Query<(), With<Evelynn>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_evelynn.get(entity).is_err() {
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
    // E is targeted damage
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Nearest {
                max_distance: 210.0,
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

fn on_evelynn_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_evelynn: Query<(), With<Evelynn>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_evelynn.get(entity).is_err() {
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
    // R is AoE damage with execute
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 500.0 },
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

fn on_evelynn_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_evelynn: Query<(), With<Evelynn>>,
) {
    let source = trigger.source;
    if q_evelynn.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // W slows then charms
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffSlow::new(0.45, 2.5));
}
