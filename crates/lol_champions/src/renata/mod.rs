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

use crate::renata::buffs::{BuffRenataQ, BuffRenataR, BuffRenataW};

#[derive(Default)]
pub struct PluginRenata;

impl Plugin for PluginRenata {
    fn build(&self, app: &mut App) {
        app.add_observer(on_renata_q);
        app.add_observer(on_renata_w);
        app.add_observer(on_renata_e);
        app.add_observer(on_renata_r);
        app.add_observer(on_renata_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Renata"))]
#[reflect(Component)]
pub struct Renata;

fn on_renata_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_renata: Query<(), With<Renata>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_renata.get(entity).is_err() {
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
    // Q is header lash - damage and slow
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

fn on_renata_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_renata: Query<(), With<Renata>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_renata.get(entity).is_err() {
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
    // W is loyalty program - attackspeed buff to ally
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffRenataW::new(0.5, 5.0));
}

fn on_renata_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_renata: Query<(), With<Renata>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_renata.get(entity).is_err() {
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
    // E is trusim - damage and shield
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 800.0,
                angle: 20.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
        }],
    });
}

fn on_renata_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_renata: Query<(), With<Renata>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_renata.get(entity).is_err() {
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
    // R is hostile takeovers - AoE stun
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 1500.0,
                angle: 60.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
        }],
    });
}

fn on_renata_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_renata: Query<(), With<Renata>>,
) {
    let source = trigger.source;
    if q_renata.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Q slows
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffRenataQ::new(0.5, 1.5));
    // R stuns
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffRenataR::new(0.75, 1.0));
}
