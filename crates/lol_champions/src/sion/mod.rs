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

use crate::sion::buffs::{BuffSionE, BuffSionQ};

#[derive(Default)]
pub struct PluginSion;

impl Plugin for PluginSion {
    fn build(&self, app: &mut App) {
        app.add_observer(on_sion_q);
        app.add_observer(on_sion_w);
        app.add_observer(on_sion_e);
        app.add_observer(on_sion_r);
        app.add_observer(on_sion_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Sion"))]
#[reflect(Component)]
pub struct Sion;

fn on_sion_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_sion: Query<(), With<Sion>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_sion.get(entity).is_err() {
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
    // Q is brutal strike - damage and knockup
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 600.0,
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

fn on_sion_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_sion: Query<(), With<Sion>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_sion.get(entity).is_err() {
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
    // W is roar of kor - damage
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 300.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
        }],
    });
}

fn on_sion_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_sion: Query<(), With<Sion>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_sion.get(entity).is_err() {
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
    // E is unwind - damage and slow
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 750.0,
                angle: 15.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Physical,
            }],
        }],
    });
}

fn on_sion_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_sion: Query<(), With<Sion>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_sion.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::R) {
        return;
    }

    let _skill_spell = skill.spell.clone();
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL4.to_string(),
        repeat: false,
        duration: None,
    });
    // R is unleash the nightmare - charge;
}

fn on_sion_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_sion: Query<(), With<Sion>>,
) {
    let source = trigger.source;
    if q_sion.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Q knocks up
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffSionQ::new(0.5, 1.0));
    // E slows
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffSionE::new(0.4, 2.0));
}
