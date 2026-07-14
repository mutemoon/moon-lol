pub mod buffs;

use bevy::prelude::*;
use lol_base::animation_names::{ANIM_SPELL1, ANIM_SPELL2, ANIM_SPELL3, ANIM_SPELL4};
use lol_base::render_cmd::CommandAnimationPlay;
use lol_core::action::damage::{
    ActionDamage, ActionDamageEffect, DamageShape, TargetDamage, TargetFilter,
};
use lol_core::attack::CommandAttackReset;
use lol_core::base::buff::BuffOf;
use lol_core::buffs::cc_debuffs::DebuffSlow;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{EventSkillCast, Skill, SkillSlot};

#[derive(Default)]
pub struct PluginJinx;

impl Plugin for PluginJinx {
    fn build(&self, app: &mut App) {
        app.add_observer(on_jinx_q);
        app.add_observer(on_jinx_w);
        app.add_observer(on_jinx_e);
        app.add_observer(on_jinx_r);
        app.add_observer(on_jinx_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Jinx"))]
#[reflect(Component)]
pub struct Jinx;

fn on_jinx_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_jinx: Query<(), With<Jinx>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_jinx.get(entity).is_err() {
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
    // Q switches between minigun and rocket launcher
    // Minigun gives attackspeed stacks, rocket deals AoE
    commands.trigger(CommandAttackReset { entity });
}

fn on_jinx_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_jinx: Query<(), With<Jinx>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_jinx.get(entity).is_err() {
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
    // W is a skillshot that slows
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 1500.0,
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

fn on_jinx_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_jinx: Query<(), With<Jinx>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_jinx.get(entity).is_err() {
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
    // E places traps that explode and knock up
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 250.0 },
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

fn on_jinx_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_jinx: Query<(), With<Jinx>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_jinx.get(entity).is_err() {
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
    // R is a global rocket with damage based on distance
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 25000.0,
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

fn on_jinx_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_jinx: Query<(), With<Jinx>>,
) {
    let source = trigger.source;
    if q_jinx.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // W applies slow
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffSlow::new(0.4, 2.0));
}
