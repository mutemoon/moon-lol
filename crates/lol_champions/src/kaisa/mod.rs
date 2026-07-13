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

use crate::kaisa::buffs::{BuffKaisaE, BuffKaisaPlasma, BuffKaisaR};

#[derive(Default)]
pub struct PluginKaisa;

impl Plugin for PluginKaisa {
    fn build(&self, app: &mut App) {
        app.add_observer(on_kaisa_q);
        app.add_observer(on_kaisa_w);
        app.add_observer(on_kaisa_e);
        app.add_observer(on_kaisa_r);
        app.add_observer(on_kaisa_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Kaisa"))]
#[reflect(Component)]
pub struct Kaisa;

fn on_kaisa_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_kaisa: Query<(), With<Kaisa>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_kaisa.get(entity).is_err() {
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

    // Q fires 6 missiles that spread to nearby enemies
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 600.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Physical,
            }],
        }],
    });
}

fn on_kaisa_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_kaisa: Query<(), With<Kaisa>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_kaisa.get(entity).is_err() {
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

    // W is a long-range missile that applies 2 plasma stacks
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 3000.0,
                angle: 10.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
        }],
    });
}

fn on_kaisa_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_kaisa: Query<(), With<Kaisa>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_kaisa.get(entity).is_err() {
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

    // E charges movement speed then grants attackspeed
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffKaisaE::new(0.8, 4.0));
}

fn on_kaisa_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_kaisa: Query<(), With<Kaisa>>,
    _q_transform: Query<&Transform>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_kaisa.get(entity).is_err() {
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

    // R is a dash to a plasma-marked enemy with shield
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffKaisaR::new(100.0, 4.0));

    commands.trigger(ActionDash {
        entity,
        point: point,
        move_type: DashMoveType::Pointer { max: 2000.0 },
        speed: 1500.0,
    });
}

fn on_kaisa_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_kaisa: Query<(), With<Kaisa>>,
) {
    let source = trigger.source;
    if q_kaisa.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Apply plasma stacks (Q applies 1, W applies 2)
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffKaisaPlasma::new(1, 5.0));
}
