pub mod buffs;

use bevy::prelude::*;
use lol_base::animation_names::{ANIM_SPELL1, ANIM_SPELL2, ANIM_SPELL3, ANIM_SPELL4};
use lol_base::render_cmd::CommandAnimationPlay;
use lol_core::action::damage::{
    ActionDamage, ActionDamageEffect, DamageShape, TargetDamage, TargetFilter,
};
use lol_core::action::dash::{ActionDash, DashMoveType};
use lol_core::action::knockback::{CommandKnockback, DisplaceDirection};
use lol_core::base::buff::BuffOf;
use lol_core::buffs::cc_debuffs::{DebuffSlow, update_debuff_knockup};
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{CoolDown, EventSkillCast, Skill, SkillRecastWindow, SkillSlot};

use crate::leesin::buffs::BuffLeeSinIronWill;

const LEESIN_RECAST_WINDOW: f32 = 3.0;

#[derive(Default)]
pub struct PluginLeeSin;

impl Plugin for PluginLeeSin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_leesin_q);
        app.add_observer(on_leesin_w);
        app.add_observer(on_leesin_e);
        app.add_observer(on_leesin_r);
        app.add_observer(on_leesin_damage_hit);
        app.add_systems(FixedUpdate, update_debuff_knockup);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("LeeSin"))]
#[reflect(Component)]
pub struct LeeSin;

/// 标记李青当前释放的技能，用于伤害命中 observer 判断是哪个技能命中
/// stage: 1=E1, 2=E2, 3=R
#[derive(Component, Debug, Clone)]
pub struct LeeSinActiveAbility {
    pub stage: u8,
}

fn on_leesin_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_leesin: Query<(), With<LeeSin>>,
    q_transform: Query<&Transform>,
    q_skill: Query<(&Skill, &CoolDown, Option<&SkillRecastWindow>)>,
) {
    let entity = trigger.event_target();
    if q_leesin.get(entity).is_err() {
        return;
    }

    let Ok((skill, cooldown, recast)) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::Q) {
        return;
    }

    let _q_transform = &q_transform;
    let skill_entity = trigger.skill_entity;
    let point = trigger.point;
    let skill_spell = skill.spell.clone();
    let stage = recast.map(|w| w.stage).unwrap_or(1);

    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL1.to_string(),
        repeat: false,
        duration: None,
    });

    if stage == 1 {
        // First cast: Sonic Wave - skillshot that marks enemy
        commands.trigger(ActionDamage {
            entity,
            skill: skill_spell,
            effects: vec![ActionDamageEffect {
                shape: DamageShape::Sector {
                    radius: 400.0,
                    angle: 30.0,
                },
                damage_list: vec![TargetDamage {
                    filter: TargetFilter::All,
                    amount: "total_damage".to_string(),
                    damage_type: DamageType::Physical,
                }],
            }],
        });
        // Insert recast window for second cast (Resonating Strike)
        commands
            .entity(skill_entity)
            .insert(SkillRecastWindow::new(2, 2, LEESIN_RECAST_WINDOW));
    } else {
        // Second cast: Resonating Strike - dash to marked enemy
        commands.trigger(ActionDash {
            entity,
            point: point,
            move_type: DashMoveType::Pointer { max: 500.0 },
            speed: 800.0,
        });
        commands.entity(skill_entity).remove::<SkillRecastWindow>();
        commands.entity(skill_entity).insert((CoolDown {
            duration: cooldown.duration,
            timer: Some(Timer::from_seconds(cooldown.duration, TimerMode::Once)),
        },));
        debug!(
            "{:?} 释放了 {} 技能，当前阶段 {}，开始冷却",
            entity, "Lee Sin Q", stage
        );
    };
}

fn on_leesin_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_leesin: Query<(), With<LeeSin>>,
    q_transform: Query<&Transform>,
    q_skill: Query<(&Skill, &CoolDown, Option<&SkillRecastWindow>)>,
) {
    let entity = trigger.event_target();
    if q_leesin.get(entity).is_err() {
        return;
    }

    let Ok((skill, cooldown, recast)) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::W) {
        return;
    }

    let _q_transform = &q_transform;
    let skill_entity = trigger.skill_entity;
    let point = trigger.point;
    let _skill_spell = skill.spell.clone();
    let stage = recast.map(|w| w.stage).unwrap_or(1);

    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL2.to_string(),
        repeat: false,
        duration: None,
    });

    if stage == 1 {
        // First cast: Safeguard - dash to ally/windwall
        commands.trigger(ActionDash {
            entity,
            point: point,
            move_type: DashMoveType::Pointer { max: 300.0 },
            speed: 700.0,
        });
        // Insert recast window for second cast (Iron Will)
        commands
            .entity(skill_entity)
            .insert(SkillRecastWindow::new(2, 2, LEESIN_RECAST_WINDOW));
    } else {
        // Second cast: Iron Will - lifesteal and attack speed buff
        commands
            .entity(entity)
            .with_related::<BuffOf>(BuffLeeSinIronWill::new(0.1, 0.1, 4.0));
        commands.entity(skill_entity).remove::<SkillRecastWindow>();
        commands.entity(skill_entity).insert((CoolDown {
            duration: cooldown.duration,
            timer: Some(Timer::from_seconds(cooldown.duration, TimerMode::Once)),
        },));
        debug!(
            "{:?} 释放了 {} 技能，当前阶段 {}，开始冷却",
            entity, "Lee Sin W", stage
        );
    };
}

fn on_leesin_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_leesin: Query<(), With<LeeSin>>,
    q_skill: Query<(&Skill, &CoolDown, Option<&SkillRecastWindow>)>,
) {
    let entity = trigger.event_target();
    if q_leesin.get(entity).is_err() {
        return;
    }

    let Ok((skill, cooldown, recast)) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::E) {
        return;
    }

    let skill_entity = trigger.skill_entity;
    let skill_spell = skill.spell.clone();
    let stage = recast.map(|w| w.stage).unwrap_or(1);

    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL3.to_string(),
        repeat: false,
        duration: None,
    });

    if stage == 1 {
        // First cast: Tempest - AoE damage (no slow)
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
        // Insert recast window for second cast (Cripple)
        commands
            .entity(skill_entity)
            .insert(SkillRecastWindow::new(2, 2, LEESIN_RECAST_WINDOW));
    } else {
        // Second cast: Cripple - slow enemies already affected by Tempest
        // Mark E2 so observer applies slow on damage hit
        commands
            .entity(entity)
            .insert(LeeSinActiveAbility { stage: 2 });
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
        commands.entity(skill_entity).remove::<SkillRecastWindow>();
        commands.entity(skill_entity).insert((CoolDown {
            duration: cooldown.duration,
            timer: Some(Timer::from_seconds(cooldown.duration, TimerMode::Once)),
        },));
        debug!(
            "{:?} 释放了 {} 技能，当前阶段 {}，开始冷却",
            entity, "Lee Sin E", stage
        );
    };
}

fn on_leesin_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_leesin: Query<(), With<LeeSin>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_leesin.get(entity).is_err() {
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
    // Mark R so observer applies knockback + stun on damage hit
    commands
        .entity(entity)
        .insert(LeeSinActiveAbility { stage: 3 });

    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Nearest {
                max_distance: 150.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::Champion,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Physical,
            }],
        }],
    });
}

/// 监听李青造成的伤害，应用E2减速和R击退眩晕
fn on_leesin_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_leesin: Query<(Entity, &LeeSinActiveAbility)>,
) {
    let target_entity = trigger.event_target();
    let Ok((leesin_entity, active_ability)) = q_leesin.get(trigger.source) else {
        return;
    };

    match active_ability.stage {
        2 => {
            // E2命中给目标减速
            commands
                .entity(target_entity)
                .with_related::<BuffOf>(DebuffSlow::new(0.6, 2.0));
        }
        3 => {
            // R命中：击退
            commands
                .entity(target_entity)
                .trigger(|e| CommandKnockback {
                    entity: e,
                    source: leesin_entity,
                    distance: 200.0,
                    speed: 1200.0,
                    duration: Some(1.0),
                    direction: DisplaceDirection::Away,
                });

            // R用完后移除标记
            commands
                .entity(leesin_entity)
                .remove::<LeeSinActiveAbility>();
        }
        _ => {}
    }
}
