pub mod buffs;

use bevy::prelude::*;
use lol_base::animation_names::{ANIM_SPELL1, ANIM_SPELL2, ANIM_SPELL3, ANIM_SPELL4};
use lol_base::render_cmd::CommandAnimationPlay;
use lol_core::action::damage::{
    ActionDamage, ActionDamageEffect, DamageShape, TargetDamage, TargetFilter,
};
use lol_core::base::buff::BuffOf;
use lol_core::buffs::cc_debuffs::DebuffStun;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{EventSkillCast, Skill, SkillSlot};

use crate::annie::buffs::{BuffAnniePassive, BuffAnnieShield};

#[derive(Default)]
pub struct PluginAnnie;

impl Plugin for PluginAnnie {
    fn build(&self, app: &mut App) {
        app.add_observer(on_annie_q);
        app.add_observer(on_annie_w);
        app.add_observer(on_annie_e);
        app.add_observer(on_annie_r);
        app.add_observer(on_annie_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Annie"))]
#[reflect(Component)]
pub struct Annie;

fn on_annie_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_annie: Query<(), With<Annie>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_annie.get(entity).is_err() {
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
    // Q is targeted damage
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 625.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
        }],
    });

    // Increment passive stacks
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffAnniePassive::increment());
}

fn on_annie_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_annie: Query<(), With<Annie>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_annie.get(entity).is_err() {
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
    // W is a cone
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 600.0,
                angle: 50.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
        }],
    });

    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffAnniePassive::increment());
}

fn on_annie_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_annie: Query<(), With<Annie>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_annie.get(entity).is_err() {
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
    // E grants shield
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffAnnieShield::new());
}

fn on_annie_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_annie: Query<(), With<Annie>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_annie.get(entity).is_err() {
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
    // R summons Tibbers - area damage
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 600.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
        }],
    });

    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffAnniePassive::increment());
}

fn on_annie_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_annie: Query<(), With<Annie>>,
) {
    let source = trigger.source;
    if q_annie.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Check if Annie has 4 passive stacks for stun
    // For now, just stun
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffStun::new(1.5));
}
