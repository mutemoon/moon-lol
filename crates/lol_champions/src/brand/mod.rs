pub mod buffs;

use bevy::prelude::*;
use lol_base::animation_names::{ANIM_SPELL1, ANIM_SPELL2, ANIM_SPELL3, ANIM_SPELL4};
use lol_base::render_cmd::CommandAnimationPlay;
use lol_core::action::damage::{
    ActionDamage, ActionDamageEffect, DamageShape, TargetDamage, TargetFilter,
};
use lol_core::base::buff::BuffOf;
use lol_core::buffs::cc_debuffs::DebuffSlow;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{EventSkillCast, Skill, SkillSlot};

use crate::brand::buffs::BuffBrandPassive;

#[derive(Default)]
pub struct PluginBrand;

impl Plugin for PluginBrand {
    fn build(&self, app: &mut App) {
        app.add_observer(on_brand_q);
        app.add_observer(on_brand_w);
        app.add_observer(on_brand_e);
        app.add_observer(on_brand_r);
        app.add_observer(on_brand_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Brand"))]
#[reflect(Component)]
pub struct Brand;

fn on_brand_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_brand: Query<(), With<Brand>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_brand.get(entity).is_err() {
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
    // Q is a fireball that stuns ablazed targets
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 1100.0,
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

fn on_brand_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_brand: Query<(), With<Brand>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_brand.get(entity).is_err() {
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
    // W is a ground targeted area
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 250.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
        }],
    });
}

fn on_brand_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_brand: Query<(), With<Brand>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_brand.get(entity).is_err() {
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
    // E spreads to nearby enemies
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 675.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
        }],
    });
}

fn on_brand_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_brand: Query<(), With<Brand>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_brand.get(entity).is_err() {
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
    // R bounces between enemies
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 750.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
        }],
    });
}

fn on_brand_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_brand: Query<(), With<Brand>>,
) {
    let source = trigger.source;
    if q_brand.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Apply blaze passive stacks
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffBrandPassive::new());
    // R slows
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffSlow::new(0.3, 2.0));
}
