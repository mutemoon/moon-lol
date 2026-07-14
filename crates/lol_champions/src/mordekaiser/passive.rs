use bevy::prelude::*;
use bevy::time::{Timer, TimerMode};

/// 被动 - 黑暗起兮 (Darkness Rise)
///
/// 普攻 / Q 命中 / E 命中各获得 1 层 Darkness，满 3 层激活 DoT 光环（半径 375），
/// 持续对附近敌人造成魔法伤害（每层 30% AP），并提供 3% 移速与普攻 40% AP 附伤。
/// 脱战 4 秒后失效。

/// 被动最大层数（ron `MaximumStacks`）。
pub const MORDE_PASSIVE_MAX_STACKS: u8 = 3;
/// 脱战失效时长（秒，ron `CombatTrackingDuration`）。
pub const MORDE_PASSIVE_COMBAT_DURATION: f32 = 4.0;
/// DoT 光环半径（ron `DoTRadius`）。
pub const MORDE_PASSIVE_DOT_RADIUS: f32 = 375.0;
/// DoT 周期（秒）。
pub const MORDE_PASSIVE_DOT_TICK: f32 = 0.5;
/// 每层每周期 AP 加成（ron `aura_damage_per_stack` = 0.3 * AP，stat 0）。
pub const MORDE_PASSIVE_DOT_AP_RATIO: f32 = 0.3;
/// 激活期间移速加成（ron `MovementSpeed` = 0.03）。
pub const MORDE_PASSIVE_MS_BONUS: f32 = 0.03;
/// 激活期间普攻附带 AP（ron `PercentAPAddedToAutos` = 0.4）。
pub const MORDE_PASSIVE_AUTO_AP_RATIO: f32 = 0.4;

/// 被动 DoT 伤害标签：区分光环 DoT，避免 DoT 再次叠层 / 触发普攻附伤。
pub const MORDE_PASSIVE_DOT_TAG: u32 = 1;
/// 被动普攻附伤标签：区分附伤额外实例，避免附伤再次叠层 / 触发自身。
pub const MORDE_PASSIVE_AUTO_TAG: u32 = 2;

/// 莫德凯撒被动 Darkness 状态（挂在自身）。
///
/// `combat_timer` 每次命中重置，到期（脱战 4 秒）则清空层数并失效；
/// `dot_timer` 仅在 `active` 时推进，每周期对半径内敌人结算 DoT；
/// `ms_bonus` 记录已叠加到 [`lol_core::movement::Movement`] 上的移速数值，失效时据此还原。
#[derive(Component, Debug, Clone)]
pub struct MordekaiserDarkness {
    /// 当前层数，0..=3
    pub stacks: u8,
    /// 是否已满层激活
    pub active: bool,
    /// 脱战计时
    pub combat_timer: Timer,
    /// DoT 周期计时
    pub dot_timer: Timer,
    /// 已施加的移速加成（用于失效时还原）
    pub ms_bonus: f32,
}

impl MordekaiserDarkness {
    pub const MAX: u8 = MORDE_PASSIVE_MAX_STACKS;

    /// 首次命中：1 层，未激活。
    pub fn new() -> Self {
        Self {
            stacks: 1,
            active: false,
            combat_timer: Timer::from_seconds(MORDE_PASSIVE_COMBAT_DURATION, TimerMode::Once),
            dot_timer: Timer::from_seconds(MORDE_PASSIVE_DOT_TICK, TimerMode::Repeating),
            ms_bonus: 0.0,
        }
    }
}

impl Default for MordekaiserDarkness {
    fn default() -> Self {
        Self {
            stacks: 0,
            active: false,
            combat_timer: Timer::from_seconds(MORDE_PASSIVE_COMBAT_DURATION, TimerMode::Once),
            dot_timer: Timer::from_seconds(MORDE_PASSIVE_DOT_TICK, TimerMode::Repeating),
            ms_bonus: 0.0,
        }
    }
}
