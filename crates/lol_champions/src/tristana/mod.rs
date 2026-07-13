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

use crate::tristana::buffs::BuffTristanaW;

#[derive(Default)]
pub struct PluginTristana;

impl Plugin for PluginTristana {
    fn build(&self, app: &mut App) {
        app.add_observer(on_tristana_q);
        app.add_observer(on_tristana_w);
        app.add_observer(on_tristana_e);
        app.add_observer(on_tristana_r);
        app.add_observer(on_tristana_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Tristana"))]
#[reflect(Component)]
pub struct Tristana;

fn on_tristana_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_tristana: Query<(), With<Tristana>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_tristana.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::Q) {
        return;
    }

    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL1.to_string(),
        repeat: false,
        duration: None,
    });
}

fn on_tristana_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_tristana: Query<(), With<Tristana>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_tristana.get(entity).is_err() {
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
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 900.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
        }],
    });
}

fn on_tristana_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_tristana: Query<(), With<Tristana>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_tristana.get(entity).is_err() {
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
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Nearest {
                max_distance: 700.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Physical,
            }],
        }],
    });
}

fn on_tristana_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_tristana: Query<(), With<Tristana>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_tristana.get(entity).is_err() {
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
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Nearest {
                max_distance: 700.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Physical,
            }],
        }],
    });
}

fn on_tristana_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_tristana: Query<(), With<Tristana>>,
) {
    let source = trigger.source;
    if q_tristana.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    commands
        .entity(target)
        .with_related::<BuffOf>(BuffTristanaW::new(0.5, 2.0));
}
