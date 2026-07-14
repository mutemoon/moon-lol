//! Sett 专属状态组件。
//!
//! - [`SettPunchState`]：被动（沙场战心）左右拳交替，右拳附带 0.55×AD 额外物理伤害。
//! - [`SettGrit`]：W（强腕重击）被动储存的"灰心"——受到伤害时累积，上限 50% 最大生命，
//!   脱战 4 秒后衰减；W 主动施放时全部转化为护盾。

use bevy::prelude::*;

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
            combat_timer: Timer::from_seconds(
                crate::sett::SETT_GRIT_COMBAT_DURATION,
                TimerMode::Once,
            ),
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

impl SettWShield {
    pub fn new() -> Self {
        Self {
            timer: Timer::from_seconds(crate::sett::SETT_W_SHIELD_DURATION, TimerMode::Once),
        }
    }
}
