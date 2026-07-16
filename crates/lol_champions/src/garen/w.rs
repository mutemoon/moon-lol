//! Garen W - 勇气 (Courage)
//!
//! 获得 30% 韧性、30% 伤害减免和护盾，持续 2s。

use bevy::prelude::*;
use lol_base::animation_names::ANIM_SPELL2;
use lol_base::render_cmd::CommandAnimationPlay;
use lol_core::base::buff::Buff;
use lol_core::base::buff::BuffOf;
use lol_core::damage::DamageType;
use lol_core::skill::{EventSkillCast, Skill, SkillSlot};

use crate::garen::Garen;

/// W 韧性加成
pub const GAREN_W_TENACITY: f32 = 0.3;
/// W 伤害减免
pub const GAREN_W_DAMAGE_REDUCTION: f32 = 0.3;
/// W 护盾值
pub const GAREN_W_SHIELD: f32 = 100.0;
/// W 持续时间
pub const GAREN_W_DURATION: f32 = 2.0;

/// 盖伦W技能buff - 韧性和伤害减免
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "GarenW" })]
pub struct BuffGarenW {
    /// 韧性加成百分比 (e.g., 0.3 = 30%)
    pub tenacity: f32,
    /// 伤害减免百分比 (e.g., 0.2 = 20%)
    pub damage_reduction: f32,
    /// 护盾值
    pub shield: f32,
    /// 持续时间
    pub duration: f32,
    /// 计时器
    pub timer: Timer,
}

impl BuffGarenW {
    pub fn new(tenacity: f32, damage_reduction: f32, shield: f32, duration: f32) -> Self {
        Self {
            tenacity,
            damage_reduction,
            shield,
            duration,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }

    pub fn tick(&mut self, delta: f32) -> bool {
        self.timer.tick(std::time::Duration::from_secs_f32(delta));
        self.timer.is_finished()
    }

    /// 检查buff是否对指定伤害类型有效
    pub fn applies_to(&self, _damage_type: DamageType) -> bool {
        true // 对所有伤害类型有效
    }
}

pub fn on_garen_w(
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
    if !matches!(skill.slot, SkillSlot::W) {
        return;
    }

    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL2.to_string(),
        repeat: false,
        duration: None,
    });
    // W provides tenacity, damage reduction, and a shield
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffGarenW::new(
            GAREN_W_TENACITY,
            GAREN_W_DAMAGE_REDUCTION,
            GAREN_W_SHIELD,
            GAREN_W_DURATION,
        ));

    debug!(
        "{:?} 释放了 {} 技能，获得 {}% 韧性、{}% 伤害减免和 {} 护盾",
        entity,
        "Garen W",
        (GAREN_W_TENACITY * 100.0) as i32,
        (GAREN_W_DAMAGE_REDUCTION * 100.0) as i32,
        GAREN_W_SHIELD as i32
    );
}