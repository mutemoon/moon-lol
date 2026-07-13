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

use crate::alistar::buffs::BuffAlistarR;

#[derive(Default)]
pub struct PluginAlistar;

impl Plugin for PluginAlistar {
    fn build(&self, app: &mut App) {
        app.add_observer(on_alistar_q);
        app.add_observer(on_alistar_w);
        app.add_observer(on_alistar_e);
        app.add_observer(on_alistar_r);
        app.add_observer(on_alistar_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Alistar"))]
#[reflect(Component)]
pub struct Alistar;

fn on_alistar_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_alistar: Query<(), With<Alistar>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_alistar.get(entity).is_err() {
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
    // Q is a knockup and stun in area
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 375.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
        }],
    });

    // Stun all enemies in range
    commands
        .entity(entity)
        .with_related::<BuffOf>(DebuffStun::new(1.0));
}

fn on_alistar_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_alistar: Query<(), With<Alistar>>,
    _q_transform: Query<&Transform>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_alistar.get(entity).is_err() {
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
    // W is a dash that knocks back target
    commands.trigger(ActionDash {
        entity,
        point: point,
        move_type: DashMoveType::Pointer { max: 650.0 },
        speed: 800.0,
    });
}

fn on_alistar_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_alistar: Query<(), With<Alistar>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_alistar.get(entity).is_err() {
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
    // E is area damage that stuns on 5th hit
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 350.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
        }],
    });
}

fn on_alistar_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_alistar: Query<(), With<Alistar>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_alistar.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::R) {
        return;
    }

    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL4.to_string(),
        repeat: false,
        duration: None,
    });
    // R grants damage reduction
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffAlistarR::new());
}

fn on_alistar_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_alistar: Query<(), With<Alistar>>,
) {
    let source = trigger.source;
    if q_alistar.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // W stuns and knocks back
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffStun::new(0.75));
}
