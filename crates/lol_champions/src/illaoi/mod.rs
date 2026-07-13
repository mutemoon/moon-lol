pub mod buffs;

use bevy::prelude::*;
use lol_base::animation_names::{ANIM_SPELL1, ANIM_SPELL2, ANIM_SPELL3, ANIM_SPELL4};
use lol_base::render_cmd::CommandAnimationPlay;
use lol_core::action::damage::{
    ActionDamage, ActionDamageEffect, DamageShape, TargetDamage, TargetFilter,
};
use lol_core::action::dash::{ActionDash, DashMoveType};
use lol_core::base::buff::BuffOf;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{EventSkillCast, Skill, SkillSlot};

use crate::illaoi::buffs::BuffIllaoiPassive;

#[derive(Default)]
pub struct PluginIllaoi;

impl Plugin for PluginIllaoi {
    fn build(&self, app: &mut App) {
        app.add_observer(on_illaoi_q);
        app.add_observer(on_illaoi_w);
        app.add_observer(on_illaoi_e);
        app.add_observer(on_illaoi_r);
        app.add_observer(on_illaoi_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Illaoi"))]
#[reflect(Component)]
pub struct Illaoi;

fn on_illaoi_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_illaoi: Query<(), With<Illaoi>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_illaoi.get(entity).is_err() {
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
    // Q enhances tentacle damage
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffIllaoiPassive::new());
}

fn on_illaoi_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_illaoi: Query<(), With<Illaoi>>,
    _q_transform: Query<&Transform>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_illaoi.get(entity).is_err() {
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
    // W is a dash to target
    commands.trigger(ActionDash {
        entity,
        point: point,
        move_type: DashMoveType::Pointer { max: 225.0 },
        speed: 1000.0,
    });
}

fn on_illaoi_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_illaoi: Query<(), With<Illaoi>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_illaoi.get(entity).is_err() {
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
    // E pulls soul
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 950.0,
                angle: 30.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Physical,
            }],
        }],
    });
}

fn on_illaoi_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_illaoi: Query<(), With<Illaoi>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_illaoi.get(entity).is_err() {
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
    // R is AoE damage
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 500.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Physical,
            }],
        }],
    });
}

fn on_illaoi_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_illaoi: Query<(), With<Illaoi>>,
) {
    let source = trigger.source;
    if q_illaoi.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Apply passive
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffIllaoiPassive::new());
}
