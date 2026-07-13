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

use crate::shen::buffs::BuffShenW;

#[derive(Default)]
pub struct PluginShen;

impl Plugin for PluginShen {
    fn build(&self, app: &mut App) {
        app.add_observer(on_shen_q);
        app.add_observer(on_shen_w);
        app.add_observer(on_shen_e);
        app.add_observer(on_shen_r);
        app.add_observer(on_shen_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Shen"))]
#[reflect(Component)]
pub struct Shen;

fn on_shen_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_shen: Query<(), With<Shen>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_shen.get(entity).is_err() {
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
    // Q is shadow dash - damage
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
                damage_type: DamageType::Magic,
            }],
        }],
    });
}

fn on_shen_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_shen: Query<(), With<Shen>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_shen.get(entity).is_err() {
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
    // W is spirits refuge - dodge
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffShenW::new(1.0, 1.5));
}

fn on_shen_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_shen: Query<(), With<Shen>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_shen.get(entity).is_err() {
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
    // E is leap - dash;
}

fn on_shen_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_shen: Query<(), With<Shen>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_shen.get(entity).is_err() {
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
    // R is stand united - global shield;
}

fn on_shen_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_shen: Query<(), With<Shen>>,
) {
    let source = trigger.source;
    if q_shen.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Q marks with spirit
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffShenW::new(1.0, 1.5));
}
