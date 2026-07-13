pub mod buffs;

use bevy::prelude::*;
use lol_base::animation_names::{ANIM_SPELL1, ANIM_SPELL2, ANIM_SPELL3, ANIM_SPELL4};
use lol_base::render_cmd::CommandAnimationPlay;
use lol_core::action::damage::{
    ActionDamage, ActionDamageEffect, DamageShape, TargetDamage, TargetFilter,
};
use lol_core::action::dash::{ActionDash, DashMoveType};
use lol_core::base::buff::BuffOf;
use lol_core::buffs::cc_debuffs::DebuffSlow;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{EventSkillCast, Skill, SkillSlot};

#[derive(Default)]
pub struct PluginJarvan;

impl Plugin for PluginJarvan {
    fn build(&self, app: &mut App) {
        app.add_observer(on_jarvan_q);
        app.add_observer(on_jarvan_w);
        app.add_observer(on_jarvan_e);
        app.add_observer(on_jarvan_r);
        app.add_observer(on_jarvan_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("JarvanIV"))]
#[reflect(Component)]
pub struct JarvanIV;

fn on_jarvan_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_jarvan: Query<(), With<JarvanIV>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_jarvan.get(entity).is_err() {
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
    // Q is a line damage and armor shred
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 785.0,
                angle: 20.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Physical,
            }],
        }],
    });
}

fn on_jarvan_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_jarvan: Query<(), With<JarvanIV>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_jarvan.get(entity).is_err() {
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
    // W is an AoE slow
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 600.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
        }],
    });
}

fn on_jarvan_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_jarvan: Query<(), With<JarvanIV>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_jarvan.get(entity).is_err() {
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
    // E grants attack speed aura;
}

fn on_jarvan_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_jarvan: Query<(), With<JarvanIV>>,
    _q_transform: Query<&Transform>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_jarvan.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::R) {
        return;
    }

    let point = trigger.point;
    let _skill_spell = skill.spell.clone();
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL4.to_string(),
        repeat: false,
        duration: None,
    });
    // R is a targeted dash that creates arena
    commands.trigger(ActionDash {
        entity,
        point: point,
        move_type: DashMoveType::Pointer { max: 650.0 },
        speed: 800.0,
    });
}

fn on_jarvan_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_jarvan: Query<(), With<JarvanIV>>,
) {
    let source = trigger.source;
    if q_jarvan.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // W slows
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffSlow::new(0.3, 2.0));
}
