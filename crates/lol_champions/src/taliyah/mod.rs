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

use crate::taliyah::buffs::BuffTaliyahW;

#[derive(Default)]
pub struct PluginTaliyah;

impl Plugin for PluginTaliyah {
    fn build(&self, app: &mut App) {
        app.add_observer(on_taliyah_q);
        app.add_observer(on_taliyah_w);
        app.add_observer(on_taliyah_e);
        app.add_observer(on_taliyah_r);
        app.add_observer(on_taliyah_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Taliyah"))]
#[reflect(Component)]
pub struct Taliyah;

fn on_taliyah_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_taliyah: Query<(), With<Taliyah>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_taliyah.get(entity).is_err() {
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
            shape: DamageShape::Sector {
                radius: 1000.0,
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

fn on_taliyah_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_taliyah: Query<(), With<Taliyah>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_taliyah.get(entity).is_err() {
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
            shape: DamageShape::Circle { radius: 400.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
        }],
    });
}

fn on_taliyah_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_taliyah: Query<(), With<Taliyah>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_taliyah.get(entity).is_err() {
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
            shape: DamageShape::Circle { radius: 800.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
        }],
    });
}

fn on_taliyah_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_taliyah: Query<(), With<Taliyah>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_taliyah.get(entity).is_err() {
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
}

fn on_taliyah_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_taliyah: Query<(), With<Taliyah>>,
) {
    let source = trigger.source;
    if q_taliyah.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    commands
        .entity(target)
        .with_related::<BuffOf>(BuffTaliyahW::new(0.75, 1.0));
}
