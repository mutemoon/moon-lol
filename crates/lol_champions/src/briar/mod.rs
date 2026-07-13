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

use crate::briar::buffs::{BuffBriarPassive, BuffBriarQ, BuffBriarW};

#[derive(Default)]
pub struct PluginBriar;

impl Plugin for PluginBriar {
    fn build(&self, app: &mut App) {
        app.add_observer(on_briar_q);
        app.add_observer(on_briar_w);
        app.add_observer(on_briar_e);
        app.add_observer(on_briar_r);
        app.add_observer(on_briar_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Briar"))]
#[reflect(Component)]
pub struct Briar;

fn on_briar_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_briar: Query<(), With<Briar>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_briar.get(entity).is_err() {
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
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 475.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Physical,
            }],
        }],
    });
}

fn on_briar_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_briar: Query<(), With<Briar>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_briar.get(entity).is_err() {
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
}

fn on_briar_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_briar: Query<(), With<Briar>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_briar.get(entity).is_err() {
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
            shape: DamageShape::Sector {
                radius: 500.0,
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

fn on_briar_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_briar: Query<(), With<Briar>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_briar.get(entity).is_err() {
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
                max_distance: 12000.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
        }],
    });
}

fn on_briar_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_briar: Query<(), With<Briar>>,
) {
    let source = trigger.source;
    if q_briar.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    commands
        .entity(target)
        .with_related::<BuffOf>(BuffBriarPassive::new(1, 10.0, 6.0));
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffBriarQ::new(0.85, 15.0, 2.0));
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffBriarW::new(0.75, 0.4, 4.0));
}
