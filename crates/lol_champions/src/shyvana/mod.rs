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

use crate::shyvana::buffs::BuffShyvanaE;

#[derive(Default)]
pub struct PluginShyvana;

impl Plugin for PluginShyvana {
    fn build(&self, app: &mut App) {
        app.add_observer(on_shyvana_q);
        app.add_observer(on_shyvana_w);
        app.add_observer(on_shyvana_e);
        app.add_observer(on_shyvana_r);
        app.add_observer(on_shyvana_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Shyvana"))]
#[reflect(Component)]
pub struct Shyvana;

fn on_shyvana_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_shyvana: Query<(), With<Shyvana>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_shyvana.get(entity).is_err() {
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
    // Q is twin bite - damage
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 250.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Physical,
            }],
        }],
    });
}

fn on_shyvana_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_shyvana: Query<(), With<Shyvana>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_shyvana.get(entity).is_err() {
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
    // W is flame breath - damage over time
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 600.0,
                angle: 25.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
        }],
    });
}

fn on_shyvana_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_shyvana: Query<(), With<Shyvana>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_shyvana.get(entity).is_err() {
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
    // E is dragon descent - knockup
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 450.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
        }],
    });
}

fn on_shyvana_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_shyvana: Query<(), With<Shyvana>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_shyvana.get(entity).is_err() {
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
    // R is shape shift - transformation;
}

fn on_shyvana_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_shyvana: Query<(), With<Shyvana>>,
) {
    let source = trigger.source;
    if q_shyvana.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // E stuns
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffShyvanaE::new(0.5, 1.0));
}
