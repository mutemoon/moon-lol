pub mod buffs;

use bevy::prelude::*;
use lol_base::animation_names::{ANIM_SPELL1, ANIM_SPELL2, ANIM_SPELL3, ANIM_SPELL4};
use lol_base::render_cmd::CommandAnimationPlay;
use lol_core::action::damage::{
    ActionDamage, ActionDamageEffect, DamageShape, TargetDamage, TargetFilter,
};
use lol_core::base::buff::BuffOf;
use lol_core::buffs::common_buffs::BuffMoveSpeed;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{EventSkillCast, Skill, SkillSlot};

use crate::masteryi::buffs::{BuffMasterYiE, BuffMasterYiR};

#[derive(Default)]
pub struct PluginMasterYi;

impl Plugin for PluginMasterYi {
    fn build(&self, app: &mut App) {
        app.add_observer(on_masteryi_q);
        app.add_observer(on_masteryi_w);
        app.add_observer(on_masteryi_e);
        app.add_observer(on_masteryi_r);
        app.add_observer(on_masteryi_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("MasterYi"))]
#[reflect(Component)]
pub struct MasterYi;

fn on_masteryi_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_masteryi: Query<(), With<MasterYi>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_masteryi.get(entity).is_err() {
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
    // Q is a dash that damages multiple targets
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 400.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Physical,
            }],
        }],
    });
}

fn on_masteryi_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_masteryi: Query<(), With<MasterYi>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_masteryi.get(entity).is_err() {
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
    // W is meditate (heal and damage reduction);
}

fn on_masteryi_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_masteryi: Query<(), With<MasterYi>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_masteryi.get(entity).is_err() {
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
    // E grants bonus true damage on attacks
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffMasterYiE::new(0.3, 5.0));
}

fn on_masteryi_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_masteryi: Query<(), With<MasterYi>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_masteryi.get(entity).is_err() {
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
    // R grants attackspeed and movespeed
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffMasterYiR::new(0.8, 0.45, 10.0));
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffMoveSpeed::new(0.45, 10.0));
}

fn on_masteryi_damage_hit(
    trigger: On<EventDamageCreate>,
    _commands: Commands,
    q_masteryi: Query<(), With<MasterYi>>,
) {
    let source = trigger.source;
    if q_masteryi.get(source).is_err() {
        return;
    }

    // Passive: Double Strike on 4th attack
}
