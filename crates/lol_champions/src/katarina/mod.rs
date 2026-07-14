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

use crate::katarina::buffs::{BuffKatarinaVoracity, BuffKatarinaW};

#[derive(Default)]
pub struct PluginKatarina;

impl Plugin for PluginKatarina {
    fn build(&self, app: &mut App) {
        app.add_observer(on_katarina_q);
        app.add_observer(on_katarina_w);
        app.add_observer(on_katarina_e);
        app.add_observer(on_katarina_r);
        app.add_observer(on_katarina_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Katarina"))]
#[reflect(Component)]
pub struct Katarina;

fn on_katarina_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_katarina: Query<(), With<Katarina>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_katarina.get(entity).is_err() {
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

    // Q bounces between enemies
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 625.0 },
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

fn on_katarina_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_katarina: Query<(), With<Katarina>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_katarina.get(entity).is_err() {
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

    // W throws dagger up and grants movespeed
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffKatarinaW::new(0.8, 2.0));
}

fn on_katarina_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_katarina: Query<(), With<Katarina>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_katarina.get(entity).is_err() {
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

    // E is a dash to target
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 100.0 },
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

fn on_katarina_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_katarina: Query<(), With<Katarina>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_katarina.get(entity).is_err() {
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

    // R throws daggers in area
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 550.0 },
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

fn on_katarina_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_katarina: Query<(), With<Katarina>>,
) {
    let source = trigger.source;
    if q_katarina.get(source).is_err() {
        return;
    }

    let _target = trigger.event_target();

    // Passive: kill reduces cooldowns
    commands
        .entity(source)
        .with_related::<BuffOf>(BuffKatarinaVoracity::new(15.0, 3.0));
}
