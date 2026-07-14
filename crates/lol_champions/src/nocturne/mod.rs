pub mod buffs;

use bevy::prelude::*;
use lol_base::animation_names::{ANIM_SPELL1, ANIM_SPELL2, ANIM_SPELL3, ANIM_SPELL4};
use lol_base::render_cmd::CommandAnimationPlay;
use lol_core::action::damage::{
    ActionDamage, ActionDamageEffect, DamageShape, TargetDamage, TargetFilter,
};
use lol_core::base::buff::BuffOf;
use lol_core::buffs::cc_debuffs::DebuffFear;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{EventSkillCast, Skill, SkillSlot};

use crate::nocturne::buffs::BuffNocturneW;

#[derive(Default)]
pub struct PluginNocturne;

impl Plugin for PluginNocturne {
    fn build(&self, app: &mut App) {
        app.add_observer(on_nocturne_q);
        app.add_observer(on_nocturne_w);
        app.add_observer(on_nocturne_e);
        app.add_observer(on_nocturne_r);
        app.add_observer(on_nocturne_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Nocturne"))]
#[reflect(Component)]
pub struct Nocturne;

fn on_nocturne_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_nocturne: Query<(), With<Nocturne>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_nocturne.get(entity).is_err() {
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
    // Q is a throwing blade that leaves a trail
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 1200.0,
                angle: 15.0,
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

fn on_nocturne_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_nocturne: Query<(), With<Nocturne>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_nocturne.get(entity).is_err() {
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
    // W grants attackspeed
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffNocturneW::new(0.5, 5.0));
}

fn on_nocturne_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_nocturne: Query<(), With<Nocturne>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_nocturne.get(entity).is_err() {
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
    // E is a fear after delay
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 425.0 },
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

fn on_nocturne_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_nocturne: Query<(), With<Nocturne>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_nocturne.get(entity).is_err() {
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
    // R is a global fear
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 2500.0 },
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

fn on_nocturne_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_nocturne: Query<(), With<Nocturne>>,
) {
    let source = trigger.source;
    if q_nocturne.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // E fears
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffFear::new(2.0));
}
