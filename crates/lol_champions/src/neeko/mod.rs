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

use crate::neeko::buffs::BuffNeekoE;

#[derive(Default)]
pub struct PluginNeeko;

impl Plugin for PluginNeeko {
    fn build(&self, app: &mut App) {
        app.add_observer(on_neeko_q);
        app.add_observer(on_neeko_w);
        app.add_observer(on_neeko_e);
        app.add_observer(on_neeko_r);
        app.add_observer(on_neeko_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Neeko"))]
#[reflect(Component)]
pub struct Neeko;

fn on_neeko_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_neeko: Query<(), With<Neeko>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_neeko.get(entity).is_err() {
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
    // Q is a bloom burst
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 800.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
        }],
    });
}

fn on_neeko_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_neeko: Query<(), With<Neeko>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_neeko.get(entity).is_err() {
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
    // W is a shapesplitter dash;
}

fn on_neeko_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_neeko: Query<(), With<Neeko>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_neeko.get(entity).is_err() {
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
    // E is a root
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 1000.0,
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

fn on_neeko_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_neeko: Query<(), With<Neeko>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_neeko.get(entity).is_err() {
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
    // R is an AoE knockup/stun
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 590.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
        }],
    });
}

fn on_neeko_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_neeko: Query<(), With<Neeko>>,
) {
    let source = trigger.source;
    if q_neeko.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // E roots
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffNeekoE::new(1.5, 2.0));
}
