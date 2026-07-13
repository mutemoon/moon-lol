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

use crate::diana::buffs::BuffDianaPassive;

#[derive(Default)]
pub struct PluginDiana;

impl Plugin for PluginDiana {
    fn build(&self, app: &mut App) {
        app.add_observer(on_diana_q);
        app.add_observer(on_diana_w);
        app.add_observer(on_diana_e);
        app.add_observer(on_diana_r);
        app.add_observer(on_diana_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Diana"))]
#[reflect(Component)]
pub struct Diana;

fn on_diana_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_diana: Query<(), With<Diana>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_diana.get(entity).is_err() {
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
    // Q is a crescent arc
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 900.0,
                angle: 180.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
        }],
    });
}

fn on_diana_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_diana: Query<(), With<Diana>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_diana.get(entity).is_err() {
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
    // W creates orbs that damage nearby enemies
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 200.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
        }],
    });
}

fn on_diana_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_diana: Query<(), With<Diana>>,
    _q_transform: Query<&Transform>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_diana.get(entity).is_err() {
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
    // E is a dash to target
    commands.trigger(ActionDash {
        entity,
        point: point,
        move_type: DashMoveType::Pointer { max: 825.0 },
        speed: 1000.0,
    });
}

fn on_diana_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_diana: Query<(), With<Diana>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_diana.get(entity).is_err() {
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
    // R pulls and damages nearby enemies
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 475.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
        }],
    });
}

fn on_diana_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_diana: Query<(), With<Diana>>,
) {
    let source = trigger.source;
    if q_diana.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Q applies moonlight
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffDianaPassive::new());
    // R slows
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffSlow::new(0.5, 2.0));
}
