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

use crate::twitch::buffs::{BuffTwitchPassive, BuffTwitchW};

#[derive(Default)]
pub struct PluginTwitch;

impl Plugin for PluginTwitch {
    fn build(&self, app: &mut App) {
        app.add_observer(on_twitch_q);
        app.add_observer(on_twitch_w);
        app.add_observer(on_twitch_e);
        app.add_observer(on_twitch_r);
        app.add_observer(on_twitch_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Twitch"))]
#[reflect(Component)]
pub struct Twitch;

fn on_twitch_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_twitch: Query<(), With<Twitch>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_twitch.get(entity).is_err() {
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
}

fn on_twitch_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_twitch: Query<(), With<Twitch>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_twitch.get(entity).is_err() {
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
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 955.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
        }],
    });
}

fn on_twitch_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_twitch: Query<(), With<Twitch>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_twitch.get(entity).is_err() {
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
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 1200.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Physical,
            }],
        }],
    });
}

fn on_twitch_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_twitch: Query<(), With<Twitch>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_twitch.get(entity).is_err() {
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
}

fn on_twitch_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_twitch: Query<(), With<Twitch>>,
) {
    let source = trigger.source;
    if q_twitch.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    commands
        .entity(target)
        .with_related::<BuffOf>(BuffTwitchPassive::new(1, 2.0, 6.0));
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffTwitchW::new(0.35, 3.0));
}
