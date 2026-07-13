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

use crate::kayle::buffs::{BuffKaylePassive, BuffKayleR, BuffKayleW};

#[derive(Default)]
pub struct PluginKayle;

impl Plugin for PluginKayle {
    fn build(&self, app: &mut App) {
        app.add_observer(on_kayle_q);
        app.add_observer(on_kayle_w);
        app.add_observer(on_kayle_e);
        app.add_observer(on_kayle_r);
        app.add_observer(on_kayle_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Kayle"))]
#[reflect(Component)]
pub struct Kayle;

fn on_kayle_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_kayle: Query<(), With<Kayle>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_kayle.get(entity).is_err() {
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

    // Q is a skillshot that slows
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
            }],
        }],
    });
}

fn on_kayle_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_kayle: Query<(), With<Kayle>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_kayle.get(entity).is_err() {
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

    // W heals and grants movespeed
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffKayleW::new(80.0, 0.35, 2.5));
}

fn on_kayle_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_kayle: Query<(), With<Kayle>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_kayle.get(entity).is_err() {
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

    // E enhances next attack;
}

fn on_kayle_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_kayle: Query<(), With<Kayle>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_kayle.get(entity).is_err() {
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

    // R makes Kayle invulnerable and deals damage after delay
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffKayleR::new(2.5));

    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 900.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
        }],
    });
}

fn on_kayle_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_kayle: Query<(), With<Kayle>>,
) {
    let source = trigger.source;
    if q_kayle.get(source).is_err() {
        return;
    }

    let _target = trigger.event_target();

    // Passive grants attackspeed
    commands
        .entity(source)
        .with_related::<BuffOf>(BuffKaylePassive::new(0.15, 3.0));
}
