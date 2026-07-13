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

use crate::aurora::buffs::{BuffAuroraPassive, BuffAuroraR};

#[derive(Default)]
pub struct PluginAurora;

impl Plugin for PluginAurora {
    fn build(&self, app: &mut App) {
        app.add_observer(on_aurora_q);
        app.add_observer(on_aurora_w);
        app.add_observer(on_aurora_e);
        app.add_observer(on_aurora_r);
        app.add_observer(on_aurora_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Aurora"))]
#[reflect(Component)]
pub struct Aurora;

fn on_aurora_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_aurora: Query<(), With<Aurora>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_aurora.get(entity).is_err() {
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
    // Q is a projectile
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
            }],
        }],
    });

    // Apply slow
    commands
        .entity(entity)
        .with_related::<BuffOf>(DebuffSlow::new(0.4, 2.0));
}

fn on_aurora_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_aurora: Query<(), With<Aurora>>,
    _q_transform: Query<&Transform>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_aurora.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::W) {
        return;
    }

    let _skill_spell = skill.spell.clone();
    let point = trigger.point;
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL2.to_string(),
        repeat: false,
        duration: None,
    });
    // W creates a portal - dash to it
    commands.trigger(ActionDash {
        entity,
        point: point,
        move_type: DashMoveType::Pointer { max: 600.0 },
        speed: 800.0,
    });
}

fn on_aurora_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_aurora: Query<(), With<Aurora>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_aurora.get(entity).is_err() {
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
    // E creates a path
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 700.0,
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

fn on_aurora_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_aurora: Query<(), With<Aurora>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_aurora.get(entity).is_err() {
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
    // R is area damage and freeze
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 500.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
        }],
    });

    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffAuroraR::new());
}

fn on_aurora_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_aurora: Query<(), With<Aurora>>,
) {
    let source = trigger.source;
    if q_aurora.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Passive slow
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffAuroraPassive::new());
}
