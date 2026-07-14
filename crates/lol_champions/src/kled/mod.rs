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

use crate::kled::buffs::{BuffKledE, BuffKledR, BuffKledW};

#[derive(Default)]
pub struct PluginKled;

impl Plugin for PluginKled {
    fn build(&self, app: &mut App) {
        app.add_observer(on_kled_q);
        app.add_observer(on_kled_w);
        app.add_observer(on_kled_e);
        app.add_observer(on_kled_r);
        app.add_observer(on_kled_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Kled"))]
#[reflect(Component)]
pub struct Kled;

fn on_kled_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_kled: Query<(), With<Kled>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_kled.get(entity).is_err() {
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

    // Q throws bear trap
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 800.0,
                angle: 15.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Physical,
                ..Default::default()
            }],
            ..Default::default()
        }],
    });
}

fn on_kled_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_kled: Query<(), With<Kled>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_kled.get(entity).is_err() {
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

    // W grants attackspeed
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffKledW::new(0.7, 4.0));
}

fn on_kled_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_kled: Query<(), With<Kled>>,
    _q_transform: Query<&Transform>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_kled.get(entity).is_err() {
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

    // E is a dash
    commands.trigger(ActionDash {
        entity,
        point: point,
        move_type: DashMoveType::Pointer { max: 550.0 },
        speed: 900.0,
    });

    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffKledE::new(0.5, 2.0));
}

fn on_kled_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_kled: Query<(), With<Kled>>,
    _q_transform: Query<&Transform>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_kled.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::R) {
        return;
    }

    let point = trigger.point;
    let _skill_spell = skill.spell.clone();
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL4.to_string(),
        repeat: false,
        duration: None,
    });

    // R is a charge that provides shield
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffKledR::new(0.5, 100.0, 4.0));

    commands.trigger(ActionDash {
        entity,
        point: point,
        move_type: DashMoveType::Pointer { max: 3500.0 },
        speed: 1500.0,
    });
}

fn on_kled_damage_hit(
    trigger: On<EventDamageCreate>,
    _commands: Commands,
    q_kled: Query<(), With<Kled>>,
) {
    let source = trigger.source;
    if q_kled.get(source).is_err() {
        return;
    }

    let _target = trigger.event_target();

    // Passive: Kled gains courage on hit
}
