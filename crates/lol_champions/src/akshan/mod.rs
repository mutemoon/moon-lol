pub mod buffs;

use bevy::prelude::*;
use lol_base::animation_names::{ANIM_SPELL1, ANIM_SPELL2, ANIM_SPELL3, ANIM_SPELL4};
use lol_base::render_cmd::CommandAnimationPlay;
use lol_core::action::damage::{
    ActionDamage, ActionDamageEffect, DamageShape, TargetDamage, TargetFilter,
};
use lol_core::base::buff::BuffOf;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{EventSkillCast, Skill, SkillSlot};

use crate::akshan::buffs::BuffAkshanPassive;

#[derive(Default)]
pub struct PluginAkshan;

impl Plugin for PluginAkshan {
    fn build(&self, app: &mut App) {
        app.add_observer(on_akshan_q);
        app.add_observer(on_akshan_w);
        app.add_observer(on_akshan_e);
        app.add_observer(on_akshan_r);
        app.add_observer(on_akshan_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Akshan"))]
#[reflect(Component)]
pub struct Akshan;

fn on_akshan_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_akshan: Query<(), With<Akshan>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_akshan.get(entity).is_err() {
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
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 850.0,
                angle: 20.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Physical,
            }],
        }],
    });
}

fn on_akshan_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_akshan: Query<(), With<Akshan>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_akshan.get(entity).is_err() {
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
}

fn on_akshan_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_akshan: Query<(), With<Akshan>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_akshan.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::E) {
        return;
    }

    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL3.to_string(),
        repeat: false,
        duration: None,
    });
}

fn on_akshan_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_akshan: Query<(), With<Akshan>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_akshan.get(entity).is_err() {
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
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Nearest {
                max_distance: 2500.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Physical,
            }],
        }],
    });
}

fn on_akshan_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_akshan: Query<(), With<Akshan>>,
) {
    let source = trigger.source;
    if q_akshan.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    commands
        .entity(target)
        .with_related::<BuffOf>(BuffAkshanPassive::new(1, 15.0, 3.0));
}
