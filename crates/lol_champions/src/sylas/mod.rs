use bevy::prelude::*;
use lol_base::animation_names::{ANIM_SPELL1, ANIM_SPELL2, ANIM_SPELL3, ANIM_SPELL4};
use lol_base::render_cmd::CommandAnimationPlay;
use lol_core::action::damage::{
    ActionDamage, ActionDamageEffect, DamageShape, TargetDamage, TargetFilter,
};
use lol_core::action::dash::{ActionDash, DashMoveType};
use lol_core::base::buff::BuffOf;
use lol_core::buffs::cc_debuffs::DebuffSlow;
use lol_core::buffs::common_buffs::BuffSelfHeal;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{CoolDown, EventSkillCast, Skill, SkillRecastWindow, SkillSlot};

const SYLAS_E_RECAST_WINDOW: f32 = 4.0;

#[derive(Default)]
pub struct PluginSylas;

impl Plugin for PluginSylas {
    fn build(&self, app: &mut App) {
        app.add_observer(on_sylas_q);
        app.add_observer(on_sylas_w);
        app.add_observer(on_sylas_e);
        app.add_observer(on_sylas_r);
        app.add_observer(on_sylas_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Sylas"))]
#[reflect(Component)]
pub struct Sylas;

fn on_sylas_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_sylas: Query<(), With<Sylas>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_sylas.get(entity).is_err() {
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
    // Q is a lash that slows enemies in the center
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 350.0,
                angle: 60.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
        }],
    });
}

fn on_sylas_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_sylas: Query<(), With<Sylas>>,
    _q_transform: Query<&Transform>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_sylas.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::W) {
        return;
    }

    let _skill_spell = skill.spell.clone();
    let point = trigger.point;
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL2.to_string(),
        repeat: false,
        duration: None,
    });
    // W is a dash to target that deals damage and heals based on missing health
    commands.trigger(ActionDash {
        entity,
        point: point,
        move_type: DashMoveType::Pointer { max: 200.0 },
        speed: 900.0,
    });
    // Heal based on missing health
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffSelfHeal::new(60.0));
}

fn on_sylas_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_sylas: Query<(), With<Sylas>>,
    _q_transform: Query<&Transform>,
    q_skill: Query<(&Skill, &CoolDown, Option<&SkillRecastWindow>)>,
) {
    let entity = trigger.event_target();
    if q_sylas.get(entity).is_err() {
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
        // First cast: Throws chain toward enemy - damage in narrow cone
        commands.trigger(ActionDamage {
            entity,
            skill: skill_spell,
            effects: vec![ActionDamageEffect {
                shape: DamageShape::Sector {
                    radius: 400.0,
                    angle: 20.0,
                },
                damage_list: vec![TargetDamage {
                    filter: TargetFilter::All,
                    amount: "total_damage".to_string(),
                    damage_type: DamageType::Magic,
                }],
            }],
        });
        commands
            .entity(skill_entity)
            .insert(SkillRecastWindow::new(2, 2, SYLAS_E_RECAST_WINDOW));
    } else {
        // Second cast: Dash to enemy and pull
        commands.trigger(ActionDash {
            entity,
            point: point,
            move_type: DashMoveType::Pointer { max: 300.0 },
            speed: 800.0,
        });
        commands.entity(skill_entity).remove::<SkillRecastWindow>();
        commands.entity(skill_entity).insert((CoolDown {
            duration: cooldown.duration,
            timer: Some(Timer::from_seconds(cooldown.duration, TimerMode::Once)),
        },));
    };
}

fn on_sylas_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_sylas: Query<(), With<Sylas>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_sylas.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::R) {
        return;
    }

    let skill_spell = skill.spell.clone();
    let _point = trigger.point;
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL4.to_string(),
        repeat: false,
        duration: None,
    });
    // R 对最近敌方英雄造成魔法伤害
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Nearest {
                max_distance: 400.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::Champion,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
        }],
    });
}

/// 监听 Sylas 造成的伤害，Q 命中减速
fn on_sylas_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_sylas: Query<(), With<Sylas>>,
) {
    let source = trigger.source;
    if q_sylas.get(source).is_err() {
        return;
    }
    let target = trigger.event_target();
    // Q 命中减速
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffSlow::new(0.5, 1.5));
}
