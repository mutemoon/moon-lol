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

use crate::nami::buffs::{BuffNamiE, BuffNamiQ};

#[derive(Default)]
pub struct PluginNami;

impl Plugin for PluginNami {
    fn build(&self, app: &mut App) {
        app.add_observer(on_nami_q);
        app.add_observer(on_nami_w);
        app.add_observer(on_nami_e);
        app.add_observer(on_nami_r);
        app.add_observer(on_nami_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Nami"))]
#[reflect(Component)]
pub struct Nami;

fn on_nami_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_nami: Query<(), With<Nami>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_nami.get(entity).is_err() {
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
    // Q is a bubble that roots
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 850.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
        }],
    });
}

fn on_nami_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_nami: Query<(), With<Nami>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_nami.get(entity).is_err() {
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
    // W bounces between allies and enemies
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 725.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
        }],
    });
}

fn on_nami_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_nami: Query<(), With<Nami>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_nami.get(entity).is_err() {
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
    // E buffs allied attacks with bonus damage and slow
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffNamiE::new(30.0, 0.3, 6.0));
}

fn on_nami_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_nami: Query<(), With<Nami>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_nami.get(entity).is_err() {
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
    // R is a giant wave that knocks up
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 2750.0,
                angle: 45.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
        }],
    });
}

fn on_nami_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_nami: Query<(), With<Nami>>,
) {
    let source = trigger.source;
    if q_nami.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Q roots
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffNamiQ::new(1.5, 1.5));
}
