pub mod buffs;

use bevy::prelude::*;
use lol_base::animation_names::{ANIM_SPELL1, ANIM_SPELL2, ANIM_SPELL3, ANIM_SPELL4};
use lol_base::render_cmd::CommandAnimationPlay;
use lol_core::action::damage::{
    ActionDamage, ActionDamageEffect, DamageShape, TargetDamage, TargetFilter,
};
use lol_core::action::dash::{ActionDash, DashMoveType};
use lol_core::base::buff::BuffOf;
use lol_core::buffs::cc_debuffs::DebuffSlow;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{CoolDown, EventSkillCast, Skill, SkillRecastWindow, SkillSlot};

use crate::akali::buffs::{BuffAkaliPassive, BuffAkaliStealth, BuffAkaliW};

#[derive(Default)]
pub struct PluginAkali;

impl Plugin for PluginAkali {
    fn build(&self, app: &mut App) {
        app.add_observer(on_akali_q);
        app.add_observer(on_akali_w);
        app.add_observer(on_akali_e);
        app.add_observer(on_akali_r);
        app.add_observer(on_akali_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Akali"))]
#[reflect(Component)]
pub struct Akali;

fn on_akali_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_akali: Query<(), With<Akali>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_akali.get(entity).is_err() {
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
    // Q is a cone damage that slows distant enemies
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 500.0,
                angle: 45.0,
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

    // Mark for passive ring
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffAkaliPassive::new());
}

fn on_akali_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_akali: Query<(), With<Akali>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_akali.get(entity).is_err() {
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
    // W drops a smoke bomb and grants stealth and move speed
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffAkaliW::new());
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffAkaliStealth::new());
}

fn on_akali_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_akali: Query<(), With<Akali>>,
    _q_transform: Query<&Transform>,
    q_skill: Query<(&Skill, &CoolDown, Option<&SkillRecastWindow>)>,
) {
    let entity = trigger.event_target();
    if q_akali.get(entity).is_err() {
        return;
    }

    let Ok((skill, cooldown, recast)) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::E) {
        return;
    }

    let skill_spell = skill.spell.clone();
    let skill_entity = trigger.skill_entity;
    let point = trigger.point;
    let stage = recast.map(|w| w.stage).unwrap_or(1);

    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL3.to_string(),
        repeat: false,
        duration: None,
    });

    if stage == 1 {
        // First cast: throw shuriken and mark first enemy
        commands.trigger(ActionDamage {
            entity,
            skill: skill_spell,
            effects: vec![ActionDamageEffect {
                shape: DamageShape::Sector {
                    radius: 825.0,
                    angle: 45.0,
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
        // Mark for recast
        commands
            .entity(skill_entity)
            .insert(SkillRecastWindow::new(2, 2, 16.0));
    } else {
        // Second cast: dash to marked target
        commands.trigger(ActionDash {
            entity,
            point: point,
            move_type: DashMoveType::Pointer { max: 825.0 },
            speed: 1200.0,
        });
        commands.entity(skill_entity).remove::<SkillRecastWindow>();
        commands.entity(skill_entity).insert((CoolDown {
            duration: cooldown.duration,
            timer: Some(Timer::from_seconds(cooldown.duration, TimerMode::Once)),
        },));
    };
}

fn on_akali_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_akali: Query<(), With<Akali>>,
    _q_transform: Query<&Transform>,
    q_skill: Query<(&Skill, &CoolDown, Option<&SkillRecastWindow>)>,
) {
    let entity = trigger.event_target();
    if q_akali.get(entity).is_err() {
        return;
    }

    let Ok((skill, cooldown, recast)) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::R) {
        return;
    }

    let skill_spell = skill.spell.clone();
    let skill_entity = trigger.skill_entity;
    let point = trigger.point;
    let stage = recast.map(|w| w.stage).unwrap_or(1);

    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL4.to_string(),
        repeat: false,
        duration: None,
    });

    if stage == 1 {
        // First cast: dash to target
    } else {
        // Second cast: execute damage based on missing health
        commands.trigger(ActionDamage {
            entity,
            skill: skill_spell.clone(),
            effects: vec![ActionDamageEffect {
                shape: DamageShape::Circle { radius: 300.0 },
                damage_list: vec![TargetDamage {
                    filter: TargetFilter::Champion,
                    amount: "total_damage".to_string(),
                    damage_type: DamageType::Magic,
                    ..Default::default()
                }],
                ..Default::default()
            }],
        });
    }

    // R is a dash that can be recast
    commands.trigger(ActionDash {
        entity,
        point: point,
        move_type: DashMoveType::Pointer { max: 675.0 },
        speed: 900.0,
    });

    if stage >= 2 {
        commands.entity(skill_entity).remove::<SkillRecastWindow>();
        commands.entity(skill_entity).insert((CoolDown {
            duration: cooldown.duration,
            timer: Some(Timer::from_seconds(cooldown.duration, TimerMode::Once)),
        },));
    } else {
        commands
            .entity(skill_entity)
            .insert(SkillRecastWindow::new(2, 2, 10.0));
    };
}

/// Listen for Akali damage hits
fn on_akali_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_akali: Query<(), With<Akali>>,
) {
    let source = trigger.source;
    if q_akali.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Q slows distant enemies by 50%
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffSlow::new(0.5, 1.0));
}
