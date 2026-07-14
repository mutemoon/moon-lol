pub mod buffs;

use bevy::prelude::*;
use lol_base::animation_names::{ANIM_SPELL1, ANIM_SPELL2, ANIM_SPELL3, ANIM_SPELL4};
use lol_base::render_cmd::CommandAnimationPlay;
use lol_core::action::damage::{
    ActionDamage, ActionDamageEffect, DamageShape, TargetDamage, TargetFilter,
};
use lol_core::action::dash::{ActionDash, DashMoveType};
use lol_core::base::buff::BuffOf;
use lol_core::buffs::cc_debuffs::DebuffStun;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{EventSkillCast, Skill, SkillSlot};

use crate::amumu::buffs::{BuffAmumuPassive, BuffAmumuR};

#[derive(Default)]
pub struct PluginAmumu;

impl Plugin for PluginAmumu {
    fn build(&self, app: &mut App) {
        app.add_observer(on_amumu_q);
        app.add_observer(on_amumu_w);
        app.add_observer(on_amumu_e);
        app.add_observer(on_amumu_r);
        app.add_observer(on_amumu_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Amumu"))]
#[reflect(Component)]
pub struct Amumu;

fn on_amumu_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_amumu: Query<(), With<Amumu>>,
    _q_transform: Query<&Transform>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_amumu.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::Q) {
        return;
    }

    let _skill_spell = skill.spell.clone();
    let point = trigger.point;
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL1.to_string(),
        repeat: false,
        duration: None,
    });
    // Q is a targeted dash that stuns
    commands.trigger(ActionDash {
        entity,
        point: point,
        move_type: DashMoveType::Pointer { max: 1100.0 },
        speed: 1000.0,
    });
}

fn on_amumu_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_amumu: Query<(), With<Amumu>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_amumu.get(entity).is_err() {
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
    // W is toggle damage around self
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 350.0 },
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

fn on_amumu_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_amumu: Query<(), With<Amumu>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_amumu.get(entity).is_err() {
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
    // E is area damage
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 350.0 },
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

fn on_amumu_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_amumu: Query<(), With<Amumu>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_amumu.get(entity).is_err() {
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
    // R is area stun
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 550.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
                ..Default::default()
            }],
            ..Default::default()
        }],
    });

    // Stun all enemies in range
    commands
        .entity(entity)
        .with_related::<BuffOf>(DebuffStun::new(1.5));
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffAmumuR::new());
}

fn on_amumu_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_amumu: Query<(), With<Amumu>>,
) {
    let source = trigger.source;
    if q_amumu.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Q stuns target
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffStun::new(1.0));
    // Apply passive - Cursed Touch
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffAmumuPassive::new());
}
