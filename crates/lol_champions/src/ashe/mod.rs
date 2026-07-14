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

use crate::ashe::buffs::BuffAsheQ;

#[derive(Default)]
pub struct PluginAshe;

impl Plugin for PluginAshe {
    fn build(&self, app: &mut App) {
        app.add_observer(on_ashe_q);
        app.add_observer(on_ashe_w);
        app.add_observer(on_ashe_e);
        app.add_observer(on_ashe_r);
        app.add_observer(on_ashe_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Ashe"))]
#[reflect(Component)]
pub struct Ashe;

fn on_ashe_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_ashe: Query<(), With<Ashe>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_ashe.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::Q) {
        return;
    }

    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL1.to_string(),
        repeat: false,
        duration: None,
    });
    // Q grants attack speed buff and fires multiple arrows
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffAsheQ::new());
}

fn on_ashe_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_ashe: Query<(), With<Ashe>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_ashe.get(entity).is_err() {
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
    // W is a cone volley
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 1200.0,
                angle: 40.0,
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

fn on_ashe_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_ashe: Query<(), With<Ashe>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_ashe.get(entity).is_err() {
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
    // E is global vision - no damage;
}

fn on_ashe_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_ashe: Query<(), With<Ashe>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_ashe.get(entity).is_err() {
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
    // R is a global arrow that stuns - use large sector to simulate global range
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 20000.0,
                angle: 10.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::Champion,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
                ..Default::default()
            }],
            ..Default::default()
        }],
    });
}

fn on_ashe_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_ashe: Query<(), With<Ashe>>,
) {
    let source = trigger.source;
    if q_ashe.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Apply frost slow on all damage
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffSlow::new(0.3, 2.0));
}
