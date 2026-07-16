use bevy::prelude::*;
use lol_core::base::buff::Buff;

/// 沃利贝尔W标记 -- 挂在被W1命中的目标上
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "VolibearWMark" })]
pub struct DebuffVolibearWMark {
    pub source: Entity,
    pub timer: Timer,
}

impl DebuffVolibearWMark {
    pub fn new(source: Entity, duration: f32) -> Self {
        Self {
            source,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 被动（风暴之力）层数追踪 -- 直接挂在英雄身上。
///
/// 每次普攻命中叠加一层（上限 5），每层提供额外攻击速度；
/// 脱战 6 秒后清零。由 `Volibear` 的 `#[require]` 在生成时自动插入。
#[derive(Component, Debug)]
pub struct VolibearPStacks {
    pub count: u8,
    pub timer: Timer,
}

impl VolibearPStacks {
    pub const MAX: u8 = 5;
    pub const DURATION: f32 = 6.0;

    pub fn new() -> Self {
        Self {
            count: 0,
            timer: Timer::from_seconds(Self::DURATION, TimerMode::Once),
        }
    }
}

impl Default for VolibearPStacks {
    fn default() -> Self {
        Self::new()
    }
}

/// R 突进中，落地后以落点为圆心触发 AoE 伤害 + 减速 + 增加最大生命。
#[derive(Component, Debug, Default)]
pub struct VolibearRLandingPending {
    pub damage: f32,
    pub slow_percent: f32,
    pub slow_duration: f32,
    pub bonus_hp: f32,
}
