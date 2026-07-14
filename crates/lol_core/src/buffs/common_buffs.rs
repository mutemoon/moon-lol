use bevy::prelude::*;

use crate::base::buff::Buff;

/// 施法期间阻塞 buff（通用）
/// 阻止移动和技能施放。
///
/// 标记（`MovementBlock`/`CastBlock`）由 `PluginCc` 的 `On<Add/Remove, BuffCastBlock>`
/// 观察者桥接到角色，本组件只携带倒计时逻辑（Buff 自己管自己）。
/// 非 `ControlTag`：自施法锁不可被净化。
#[derive(Component, Debug)]
#[require(Buff = Buff { name: "CastBlock" })]
pub struct BuffCastBlock {
    pub timer: Timer,
}

impl BuffCastBlock {
    pub fn new(duration: f32) -> Self {
        Self {
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 移动速度加成 buff（通用）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "MoveSpeed" })]
pub struct BuffMoveSpeed {
    pub bonus_percent: f32,
    pub timer: Timer,
}

impl BuffMoveSpeed {
    pub fn new(bonus_percent: f32, duration: f32) -> Self {
        Self {
            bonus_percent,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 双抗加成 buff（通用）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "Resist" })]
pub struct BuffResist {
    pub armor: f32,
    pub magic_resist: f32,
    pub timer: Timer,
}

impl BuffResist {
    pub fn new(armor: f32, magic_resist: f32, duration: f32) -> Self {
        Self {
            armor,
            magic_resist,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 自我治疗 buff（通用）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "SelfHeal" })]
pub struct BuffSelfHeal {
    pub amount: f32,
}

impl BuffSelfHeal {
    pub fn new(amount: f32) -> Self {
        Self { amount }
    }
}
