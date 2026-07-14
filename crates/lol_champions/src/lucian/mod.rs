pub mod buffs;

use bevy::prelude::*;
use lol_base::animation_names::{ANIM_SPELL1, ANIM_SPELL2, ANIM_SPELL3, ANIM_SPELL4};
use lol_base::render_cmd::CommandAnimationPlay;
use lol_core::action::damage::{
    ActionDamage, ActionDamageEffect, DamageShape, TargetDamage, TargetFilter,
};
use lol_core::action::dash::{ActionDash, DashMoveType};
use lol_core::attack::CommandAttackReset;
use lol_core::base::buff::BuffOf;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{EventSkillCast, Skill, SkillSlot};

use crate::lucian::buffs::{BuffLucianPassive, BuffLucianW};

#[derive(Default)]
pub struct PluginLucian;

impl Plugin for PluginLucian {
    fn build(&self, app: &mut App) {
        app.add_observer(on_lucian_q);
        app.add_observer(on_lucian_w);
        app.add_observer(on_lucian_e);
        app.add_observer(on_lucian_r);
        app.add_observer(on_lucian_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Lucian"))]
#[reflect(Component)]
pub struct Lucian;

fn on_lucian_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_lucian: Query<(), With<Lucian>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_lucian.get(entity).is_err() {
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
    // Q is a piercing light beam
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 1000.0,
                angle: 10.0,
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

fn on_lucian_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_lucian: Query<(), With<Lucian>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_lucian.get(entity).is_err() {
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
    // W marks enemies and grants movespeed to Lucian
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffLucianW::new(60.0, 6.0));

    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 900.0,
                angle: 15.0,
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

fn on_lucian_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_lucian: Query<(), With<Lucian>>,
    _q_transform: Query<&Transform>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_lucian.get(entity).is_err() {
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
    // E is a dash
    commands.trigger(CommandAttackReset { entity });

    commands.trigger(ActionDash {
        entity,
        point: point,
        move_type: DashMoveType::Pointer { max: 425.0 },
        speed: 1000.0,
    });
}

fn on_lucian_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_lucian: Query<(), With<Lucian>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_lucian.get(entity).is_err() {
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
    // R is a barrage of shots
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 1200.0,
                angle: 30.0,
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

fn on_lucian_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_lucian: Query<(), With<Lucian>>,
) {
    let source = trigger.source;
    if q_lucian.get(source).is_err() {
        return;
    }

    let _target = trigger.event_target();

    // Passive procs after abilities
    commands
        .entity(source)
        .with_related::<BuffOf>(BuffLucianPassive::new(50.0, 1.0));
}
