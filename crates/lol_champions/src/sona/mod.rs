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

use crate::sona::buffs::{BuffSonaE, BuffSonaW};

#[derive(Default)]
pub struct PluginSona;

impl Plugin for PluginSona {
    fn build(&self, app: &mut App) {
        app.add_observer(on_sona_q);
        app.add_observer(on_sona_w);
        app.add_observer(on_sona_e);
        app.add_observer(on_sona_r);
        app.add_observer(on_sona_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Sona"))]
#[reflect(Component)]
pub struct Sona;

fn on_sona_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_sona: Query<(), With<Sona>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_sona.get(entity).is_err() {
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
    // Q is hymn of valor - damage
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 700.0 },
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

fn on_sona_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_sona: Query<(), With<Sona>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_sona.get(entity).is_err() {
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
    // W is aria of perseverance - shield
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffSonaW::new(40.0, 1.5));
}

fn on_sona_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_sona: Query<(), With<Sona>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_sona.get(entity).is_err() {
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
    // E is song of celerity - movespeed buff
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffSonaE::new(0.2, 2.5));
}

fn on_sona_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_sona: Query<(), With<Sona>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_sona.get(entity).is_err() {
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
    // R is cure - AoE stun
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 900.0,
                angle: 40.0,
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

fn on_sona_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_sona: Query<(), With<Sona>>,
) {
    let source = trigger.source;
    if q_sona.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // R stuns
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffSonaE::new(0.2, 2.5));
}
