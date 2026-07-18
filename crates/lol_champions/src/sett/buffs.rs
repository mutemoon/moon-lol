//! Sett 专属状态组件。
//!
//! - [`SettPunchState`]：被动（沙场战心）左右拳交替，右拳附带 0.55×AD 额外物理伤害。
//! - [`SettGrit`]：W（强腕重击）被动储存的"灰心"——受到伤害时累积，上限 50% 最大生命，
//!   脱战 4 秒后衰减；W 主动施放时全部转化为护盾。

use bevy::prelude::*;

/// W 灰心上限占最大生命比例
pub const SETT_GRIT_CAP_RATIO: f32 = 0.50;
/// W 灰心脱战衰减时长（秒）
pub const SETT_GRIT_COMBAT_DURATION: f32 = 4.0;
/// W 护盾持续时长（秒）
pub const SETT_W_SHIELD_DURATION: f32 = 0.75;

/// 被动左右拳计数。偶数次（右拳）附带额外伤害。
#[derive(Component, Debug, Default)]
pub struct SettPunchState {
    pub count: u32,
}

/// W 灰心储存。`stored` 为当前灰心值，`combat_timer` 用于脱战衰减判定。
#[derive(Component, Debug)]
pub struct SettGrit {
    pub stored: f32,
    pub combat_timer: Timer,
}

impl SettGrit {
    pub fn new() -> Self {
        Self {
            stored: 0.0,
            combat_timer: Timer::from_seconds(SETT_GRIT_COMBAT_DURATION, TimerMode::Once),
        }
    }
}

impl Default for SettGrit {
    fn default() -> Self {
        Self::new()
    }
}

/// W 护盾计时器：护盾 0.75 秒后强制消失。
#[derive(Component, Debug)]
pub struct SettWShield {
    pub timer: Timer,
}

/// 被动右拳 AD 加成比例
pub const SETT_PASSIVE_RIGHT_PUNCH_RATIO: f32 = 0.55;
/// E 伤害标签（on_sett_damage_hit 据此施加眩晕）
pub const SETT_E_TAG: u32 = 1;
/// E 眩晕时长
pub const SETT_E_STUN_DURATION: f32 = 0.5;
/// R 伤害标签（on_sett_damage_hit 据此施加减速）
pub const SETT_R_TAG: u32 = 2;
/// R 减速百分比
pub const SETT_R_SLOW_PERCENT: f32 = 0.40;
/// R 减速持续时长
pub const SETT_R_SLOW_DURATION: f32 = 1.5;

/// R 突进中，落地后以落点为圆心触发 AoE 伤害 + 减速。
#[derive(Component, Debug, Default)]
pub struct SettRLandingPending {
    pub damage: f32,
    pub slow_percent: f32,
    pub slow_duration: f32,
    /// 被 Sett R 抱起的敌方英雄，落地后投掷
    pub grabbed_target: Option<Entity>,
}

impl SettWShield {
    pub fn new() -> Self {
        Self {
            timer: Timer::from_seconds(SETT_W_SHIELD_DURATION, TimerMode::Once),
        }
    }
}
