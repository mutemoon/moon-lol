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

use crate::swain::buffs::BuffSwainW;

#[derive(Default)]
pub struct PluginSwain;

impl Plugin for PluginSwain {
    fn build(&self, app: &mut App) {
        app.add_observer(on_swain_q);
        app.add_observer(on_swain_w);
        app.add_observer(on_swain_e);
        app.add_observer(on_swain_r);
        app.add_observer(on_swain_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Swain"))]
#[reflect(Component)]
pub struct Swain;

fn on_swain_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_swain: Query<(), With<Swain>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_swain.get(entity).is_err() {
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
    // Q is death flare - damage
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 700.0,
                angle: 30.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
        }],
    });
}

fn on_swain_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_swain: Query<(), With<Swain>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_swain.get(entity).is_err() {
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
    // W is vision of empire - damage
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 350.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
        }],
    });
}

fn on_swain_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_swain: Query<(), With<Swain>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_swain.get(entity).is_err() {
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
    // E is nevermove - root
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 650.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
        }],
    });
}

fn on_swain_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_swain: Query<(), With<Swain>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_swain.get(entity).is_err() {
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
    // R is demonic ascension - transformation;
}

fn on_swain_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_swain: Query<(), With<Swain>>,
) {
    let source = trigger.source;
    if q_swain.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // W stuns
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffSwainW::new(0.75, 1.0));
}
