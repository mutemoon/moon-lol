pub mod buffs;

use bevy::prelude::*;
use lol_base::animation_names::{ANIM_SPELL1, ANIM_SPELL2, ANIM_SPELL3, ANIM_SPELL4};
use lol_base::render_cmd::CommandAnimationPlay;
use lol_core::action::damage::{
    ActionDamage, ActionDamageEffect, DamageShape, TargetDamage, TargetFilter,
};
use lol_core::action::dash::{ActionDash, DashMoveType};
use lol_core::attack::CommandAttackReset;
use lol_core::base::ability_resource::AbilityResource;
use lol_core::base::buff::BuffOf;
use lol_core::buffs::common_buffs::BuffSelfHeal;
use lol_core::damage::DamageType;
use lol_core::entities::champion::Champion;
use lol_core::skill::{CoolDown, EventSkillCast, Skill, SkillRecastWindow, SkillSlot};

use crate::renekton::buffs::BuffRenektonR;

const RENECKTON_E_RECAST_WINDOW: f32 = 4.0;

#[derive(Default)]
pub struct PluginRenekton;

impl Plugin for PluginRenekton {
    fn build(&self, app: &mut App) {
        app.add_observer(on_renekton_q);
        app.add_observer(on_renekton_w);
        app.add_observer(on_renekton_e);
        app.add_observer(on_renekton_r);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Renekton"))]
#[reflect(Component)]
pub struct Renekton;

fn on_renekton_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_renekton: Query<(), With<Renekton>>,
    mut q_ability_resource: Query<&mut AbilityResource>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_renekton.get(entity).is_err() {
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
    // Q is a cleave that deals damage in a circle
    let rage = q_ability_resource
        .get(entity)
        .map(|r| r.value)
        .unwrap_or(0.0);
    if rage >= 50.0 {
        // 消耗 50 怒气，强化版伤害和治疗
        if let Ok(mut resource) = q_ability_resource.get_mut(entity) {
            resource.value -= 50.0;
        }
        commands.trigger(ActionDamage {
            entity,
            skill: skill_spell,
            effects: vec![ActionDamageEffect {
                shape: DamageShape::Circle { radius: 300.0 },
                damage_list: vec![TargetDamage {
                    filter: TargetFilter::All,
                    amount: "total_damage".to_string(),
                    damage_type: DamageType::Physical,
                    ..Default::default()
                }],
                ..Default::default()
            }],
        });
        commands
            .entity(entity)
            .with_related::<BuffOf>(BuffSelfHeal::new(80.0)); // 翻倍治疗
    } else {
        commands.trigger(ActionDamage {
            entity,
            skill: skill_spell,
            effects: vec![ActionDamageEffect {
                shape: DamageShape::Circle { radius: 250.0 },
                damage_list: vec![TargetDamage {
                    filter: TargetFilter::All,
                    amount: "total_damage".to_string(),
                    damage_type: DamageType::Physical,
                    ..Default::default()
                }],
                ..Default::default()
            }],
        });
        commands
            .entity(entity)
            .with_related::<BuffOf>(BuffSelfHeal::new(40.0));
    };
}

fn on_renekton_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_renekton: Query<(), With<Renekton>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_renekton.get(entity).is_err() {
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
    // W is an empowered auto attack that stuns
    commands.trigger(CommandAttackReset { entity });
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Nearest {
                max_distance: 150.0,
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

fn on_renekton_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_renekton: Query<(), With<Renekton>>,
    _q_transform: Query<&Transform>,
    q_skill: Query<(&Skill, &CoolDown, Option<&SkillRecastWindow>)>,
) {
    let entity = trigger.event_target();
    if q_renekton.get(entity).is_err() {
        return;
    }

    let Ok((skill, cooldown, recast)) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::E) {
        return;
    }

    let skill_entity = trigger.skill_entity;
    let point = trigger.point;
    let _skill_spell = skill.spell.clone();
    let stage = recast.map(|w| w.stage).unwrap_or(1);

    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL3.to_string(),
        repeat: false,
        duration: None,
    });

    if stage == 1 {
        // First cast: Dash forward
        commands.trigger(ActionDash {
            entity,
            point: point,
            move_type: DashMoveType::Pointer { max: 200.0 },
            speed: 700.0,
        });
        commands.entity(skill_entity).insert(SkillRecastWindow::new(
            2,
            2,
            RENECKTON_E_RECAST_WINDOW,
        ));
    } else {
        // Second cast: Dash again
        commands.trigger(ActionDash {
            entity,
            point: point,
            move_type: DashMoveType::Pointer { max: 200.0 },
            speed: 700.0,
        });
        commands.entity(skill_entity).remove::<SkillRecastWindow>();
        commands.entity(skill_entity).insert((CoolDown {
            duration: cooldown.duration,
            timer: Some(Timer::from_seconds(cooldown.duration, TimerMode::Once)),
        },));
    };
}

fn on_renekton_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_renekton: Query<(), With<Renekton>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_renekton.get(entity).is_err() {
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
    // R is a transformation that deals damage around and generates rage
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 300.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Physical,
                ..Default::default()
            }],
            ..Default::default()
        }],
    });
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffRenektonR::new(0.0, 5.0, 15.0));
}
