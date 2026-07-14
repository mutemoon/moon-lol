pub mod buffs;

use bevy::prelude::*;
use lol_base::animation_names::{ANIM_SPELL1, ANIM_SPELL2, ANIM_SPELL3, ANIM_SPELL4};
use lol_base::render_cmd::CommandAnimationPlay;
use lol_core::action::damage::{
    ActionDamage, ActionDamageEffect, DamageShape, TargetDamage, TargetFilter,
};
use lol_core::action::dash::{ActionDash, DashMoveType};
use lol_core::base::buff::BuffOf;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{EventSkillCast, Skill, SkillSlot};

use crate::galio::buffs::{BuffGalioPassive, BuffGalioW};

#[derive(Default)]
pub struct PluginGalio;

impl Plugin for PluginGalio {
    fn build(&self, app: &mut App) {
        app.add_observer(on_galio_q);
        app.add_observer(on_galio_w);
        app.add_observer(on_galio_e);
        app.add_observer(on_galio_r);
        app.add_observer(on_galio_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Galio"))]
#[reflect(Component)]
pub struct Galio;

fn on_galio_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_galio: Query<(), With<Galio>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_galio.get(entity).is_err() {
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
    // Q is a tornado
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 825.0,
                angle: 60.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
                ..Default::default()
            }],
            ..Default::default()
        }],
    });
}

fn on_galio_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_galio: Query<(), With<Galio>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_galio.get(entity).is_err() {
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
    // W provides shield and reduces damage
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffGalioW::new());
}

fn on_galio_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_galio: Query<(), With<Galio>>,
    _q_transform: Query<&Transform>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_galio.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::E) {
        return;
    }

    let point = trigger.point;
    let _skill_spell = skill.spell.clone();
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL3.to_string(),
        repeat: false,
        duration: None,
    });
    // E is a dash that knocks up
    commands.trigger(ActionDash {
        entity,
        point: point,
        move_type: DashMoveType::Pointer { max: 650.0 },
        speed: 900.0,
    });
}

fn on_galio_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_galio: Query<(), With<Galio>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_galio.get(entity).is_err() {
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
    // R is a large AoE knockback
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 400.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
                ..Default::default()
            }],
            ..Default::default()
        }],
    });
}

fn on_galio_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_galio: Query<(), With<Galio>>,
) {
    let source = trigger.source;
    if q_galio.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Apply passive
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffGalioPassive::new());
}
