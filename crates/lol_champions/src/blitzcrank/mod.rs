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

use crate::blitzcrank::buffs::BuffBlitzcrankW;

#[derive(Default)]
pub struct PluginBlitzcrank;

impl Plugin for PluginBlitzcrank {
    fn build(&self, app: &mut App) {
        app.add_observer(on_blitzcrank_q);
        app.add_observer(on_blitzcrank_w);
        app.add_observer(on_blitzcrank_e);
        app.add_observer(on_blitzcrank_r);
        app.add_observer(on_blitzcrank_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Blitzcrank"))]
#[reflect(Component)]
pub struct Blitzcrank;

fn on_blitzcrank_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_blitzcrank: Query<(), With<Blitzcrank>>,
    _q_transform: Query<&Transform>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_blitzcrank.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::Q) {
        return;
    }

    let _skill_spell = skill.spell.clone();
    let point = trigger.point;
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL1.to_string(),
        repeat: false,
        duration: None,
    });
    // Q is a hook that pulls enemy
    commands.trigger(ActionDash {
        entity,
        point: point,
        move_type: DashMoveType::Pointer { max: 1115.0 },
        speed: 900.0,
    });
}

fn on_blitzcrank_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_blitzcrank: Query<(), With<Blitzcrank>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_blitzcrank.get(entity).is_err() {
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
    // W grants movement and attack speed
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffBlitzcrankW::new());
}

fn on_blitzcrank_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_blitzcrank: Query<(), With<Blitzcrank>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_blitzcrank.get(entity).is_err() {
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
    // E is an empowered attack that knocks up
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Nearest {
                max_distance: 100.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
        }],
    });
}

fn on_blitzcrank_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_blitzcrank: Query<(), With<Blitzcrank>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_blitzcrank.get(entity).is_err() {
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
    // R is an AoE that silences
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

fn on_blitzcrank_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_blitzcrank: Query<(), With<Blitzcrank>>,
) {
    let source = trigger.source;
    if q_blitzcrank.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Q stuns on hit
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffStun::new(0.65));
}
