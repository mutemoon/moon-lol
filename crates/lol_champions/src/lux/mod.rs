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

use crate::lux::buffs::{BuffLuxIllumination, BuffLuxQ};

#[derive(Default)]
pub struct PluginLux;

impl Plugin for PluginLux {
    fn build(&self, app: &mut App) {
        app.add_observer(on_lux_q);
        app.add_observer(on_lux_w);
        app.add_observer(on_lux_e);
        app.add_observer(on_lux_r);
        app.add_observer(on_lux_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Lux"))]
#[reflect(Component)]
pub struct Lux;

fn on_lux_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_lux: Query<(), With<Lux>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_lux.get(entity).is_err() {
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

    // Q roots enemies
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 1300.0,
                angle: 10.0,
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

fn on_lux_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_lux: Query<(), With<Lux>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_lux.get(entity).is_err() {
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

    // W is a shield;
}

fn on_lux_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_lux: Query<(), With<Lux>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_lux.get(entity).is_err() {
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

    // E slows and deals damage
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

fn on_lux_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_lux: Query<(), With<Lux>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_lux.get(entity).is_err() {
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

    // R is a global beam
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 3400.0,
                angle: 20.0,
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

fn on_lux_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_lux: Query<(), With<Lux>>,
) {
    let source = trigger.source;
    if q_lux.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Q roots
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffLuxQ::new(2.0, 2.0));

    // E slows
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffSlow::new(0.4, 5.0));

    // Passive marks
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffLuxIllumination::new(40.0, 6.0));
}
