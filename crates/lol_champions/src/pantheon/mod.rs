pub mod buffs;

use bevy::prelude::*;
use lol_base::animation_names::{ANIM_SPELL1, ANIM_SPELL2, ANIM_SPELL3, ANIM_SPELL4};
use lol_base::render_cmd::CommandAnimationPlay;
use lol_core::action::damage::{
    ActionDamage, ActionDamageEffect, DamageShape, TargetDamage, TargetFilter,
};
use lol_core::action::dash::{ActionDash, DashMoveType};
use lol_core::base::buff::BuffOf;
use lol_core::buffs::cc_debuffs::DebuffStun;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{EventSkillCast, Skill, SkillSlot};

use crate::pantheon::buffs::BuffPantheonE;

#[derive(Default)]
pub struct PluginPantheon;

impl Plugin for PluginPantheon {
    fn build(&self, app: &mut App) {
        app.add_observer(on_pantheon_q);
        app.add_observer(on_pantheon_w);
        app.add_observer(on_pantheon_e);
        app.add_observer(on_pantheon_r);
        app.add_observer(on_pantheon_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Pantheon"))]
#[reflect(Component)]
pub struct Pantheon;

fn on_pantheon_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_pantheon: Query<(), With<Pantheon>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_pantheon.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::Q) {
        return;
    }

    let _point = trigger.point;
    let skill_spell = skill.spell.clone();
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL1.to_string(),
        repeat: false,
        duration: None,
    });
    // Q is a spear throw that can be held for more damage
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 400.0,
                angle: 45.0,
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

fn on_pantheon_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_pantheon: Query<(), With<Pantheon>>,
    _q_transform: Query<&Transform>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_pantheon.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::W) {
        return;
    }

    let point = trigger.point;
    let _skill_spell = skill.spell.clone();
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL2.to_string(),
        repeat: false,
        duration: None,
    });
    // W is a dash to target that stuns
    commands.trigger(ActionDash {
        entity,
        point: point,
        move_type: DashMoveType::Pointer { max: 200.0 },
        speed: 1000.0,
    });
}

fn on_pantheon_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_pantheon: Query<(), With<Pantheon>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_pantheon.get(entity).is_err() {
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
    // E is a shield block that deals damage in a cone when released
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 300.0,
                angle: 90.0,
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
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffPantheonE::new(Vec2::ZERO, 1.5));
}

fn on_pantheon_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_pantheon: Query<(), With<Pantheon>>,
    _q_transform: Query<&Transform>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_pantheon.get(entity).is_err() {
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
    // R is a long-range leap that damages enemies in area
    commands.trigger(ActionDash {
        entity,
        point: point,
        move_type: DashMoveType::Pointer { max: 2000.0 },
        speed: 1500.0,
    });
}

/// 监听 Pantheon 造成的伤害，W 命中时眩晕
fn on_pantheon_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_pantheon: Query<(), With<Pantheon>>,
) {
    let source = trigger.source;
    if q_pantheon.get(source).is_err() {
        return;
    }
    let target = trigger.event_target();
    // W 命中时眩晕
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffStun::new(1.0));
}
