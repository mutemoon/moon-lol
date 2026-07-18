//! Garen Q - 致命打击 (Decisive Strike)
//!
//! 获得 30% 移速加成 1.5s，强化下一次普攻造成额外伤害并沉默 1s。

use bevy::prelude::*;
use lol_base::animation_names::ANIM_SPELL1;
use lol_base::render_cmd::CommandAnimationPlay;
use lol_core::base::buff::{Buff, BuffOf};
use lol_core::skill::{EventSkillCast, Skill, SkillSlot};

use crate::garen::Garen;

/// Q 移速加成百分比
pub const GAREN_Q_MOVE_SPEED_BONUS: f32 = 0.3;
/// Q 持续时间
pub const GAREN_Q_DURATION: f32 = 1.5;
/// Q 沉默时长
pub const GAREN_Q_SILENCE_DURATION: f32 = 1.0;

/// 盖伦Q技能buff - 移动速度加成和下次攻击增强
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "GarenQ" })]
pub struct BuffGarenQ {
    /// 移动速度加成百分比 (e.g., 0.3 = 30%)
    pub move_speed_bonus: f32,
    /// 持续时间
    pub duration: f32,
    /// 计时器
    pub timer: Timer,
}

impl BuffGarenQ {
    pub fn new(move_speed_bonus: f32, duration: f32) -> Self {
        Self {
            move_speed_bonus,
            duration,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }

    pub fn tick(&mut self, delta: f32) -> bool {
        self.timer.tick(std::time::Duration::from_secs_f32(delta));
        self.timer.is_finished()
    }
}

/// 盖伦Q的下次攻击增强buff
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "GarenQAttack" })]
pub struct BuffGarenQAttack {
    /// 沉默持续时间
    pub silence_duration: f32,
}

impl BuffGarenQAttack {
    pub fn new(silence_duration: f32) -> Self {
        Self { silence_duration }
    }
}

pub fn on_garen_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_garen: Query<(), With<Garen>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_garen.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::Q) {
        return;
    }

    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL1.to_string(),
        repeat: false,
        duration: None,
    });
    // Q provides movement speed buff and enhanced next attack
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffGarenQ::new(GAREN_Q_MOVE_SPEED_BONUS, GAREN_Q_DURATION));

    // Add the enhanced attack buff (silence on hit)
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffGarenQAttack::new(GAREN_Q_SILENCE_DURATION));

    debug!(
        "{:?} 释放了 {} 技能，获得 {}% 移速加成和沉默效果",
        entity,
        "Garen Q",
        (GAREN_Q_MOVE_SPEED_BONUS * 100.0) as i32
    );
}
