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
pub struct PluginFizz;

impl Plugin for PluginFizz {
    fn build(&self, app: &mut App) {
        app.add_observer(on_fizz_q);
        app.add_observer(on_fizz_w);
        app.add_observer(on_fizz_e);
        app.add_observer(on_fizz_r);
        app.add_observer(on_fizz_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Fizz"))]
#[reflect(Component)]
pub struct Fizz;

fn on_fizz_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_fizz: Query<(), With<Fizz>>,
    _q_transform: Query<&Transform>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_fizz.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::Q) {
        return;
    }

    let point = trigger.point;
    let _skill_spell = skill.spell.clone();
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL1.to_string(),
        repeat: false,
        duration: None,
    });
    // Q is a dash to target
    commands.trigger(ActionDash {
        entity,
        point: point,
        move_type: DashMoveType::Pointer { max: 550.0 },
        speed: 1000.0,
    });
}

fn on_fizz_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_fizz: Query<(), With<Fizz>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_fizz.get(entity).is_err() {
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
    // W is a damage buff on next attack;
}

fn on_fizz_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_fizz: Query<(), With<Fizz>>,
    _q_transform: Query<&Transform>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_fizz.get(entity).is_err() {
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
    // E is a dash that can't be targeted
    commands.trigger(ActionDash {
        entity,
        point: point,
        move_type: DashMoveType::Pointer { max: 400.0 },
        speed: 800.0,
    });
}

fn on_fizz_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_fizz: Query<(), With<Fizz>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_fizz.get(entity).is_err() {
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
    // R is a shark nado
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 1300.0 },
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

fn on_fizz_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_fizz: Query<(), With<Fizz>>,
) {
    let source = trigger.source;
    if q_fizz.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // E slows
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffSlow::new(0.5, 2.0));
}
