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

use crate::karma::buffs::{BuffKarmaE, BuffKarmaGatheringFire, BuffKarmaQ};

#[derive(Default)]
pub struct PluginKarma;

impl Plugin for PluginKarma {
    fn build(&self, app: &mut App) {
        app.add_observer(on_karma_q);
        app.add_observer(on_karma_w);
        app.add_observer(on_karma_e);
        app.add_observer(on_karma_r);
        app.add_observer(on_karma_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Karma"))]
#[reflect(Component)]
pub struct Karma;

fn on_karma_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_karma: Query<(), With<Karma>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_karma.get(entity).is_err() {
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

    // Q is a skillshot that damages and slows
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 950.0,
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

fn on_karma_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_karma: Query<(), With<Karma>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_karma.get(entity).is_err() {
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

    // W roots after delay
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 675.0 },
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

fn on_karma_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_karma: Query<(), With<Karma>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_karma.get(entity).is_err() {
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

    // E provides shield and movement speed
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffKarmaE::new(80.0, 0.4, 2.0));
}

fn on_karma_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_karma: Query<(), With<Karma>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_karma.get(entity).is_err() {
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

    // R empowers next skill (handled by gathering fire passive);
}

fn on_karma_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_karma: Query<(), With<Karma>>,
) {
    let source = trigger.source;
    if q_karma.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Q applies slow
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffKarmaQ::new(0.4, 1.5));

    // Passive reduces R cooldown on hit
    commands
        .entity(source)
        .with_related::<BuffOf>(BuffKarmaGatheringFire::new(2.0));
}
