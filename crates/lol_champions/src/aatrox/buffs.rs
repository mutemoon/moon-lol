use bevy::prelude::*;
use lol_core::base::buff::Buff;

/// Aatrox W（冥府之链）引爆前的标记。
///
/// 挂在被 W 命中的敌方目标上，1.5s 后由 `update_aatrox_w_marks` 引爆——
/// 造成二次伤害（等于首段）并附加击飞（拉回效果）。
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "AatroxWMark" })]
pub struct DebuffAatroxWMark {
    pub source: Entity,
    pub target: Entity,
    pub damage: f32,
    pub timer: Timer,
}

impl DebuffAatroxWMark {
    pub fn new(source: Entity, target: Entity, damage: f32, duration: f32) -> Self {
        Self {
            source,
            target,
            damage,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// Aatrox 被动（死亡镰刀）状态，直接挂在英雄身上（由 `Aatrox` 的 `#[require]` 注入）。
///
/// `ready` 为真时，下次对敌方普攻附带目标最大生命值 15% 的额外魔法伤害并治疗自身；
/// 触发后进入冷却，倒计时结束再次就绪。
#[derive(Component, Debug)]
pub struct AatroxPassiveState {
    pub ready: bool,
    pub timer: Timer,
}

impl AatroxPassiveState {
    pub const COOLDOWN: f32 = 22.0;
    /// 额外伤害 = 目标最大生命值 × 此比例。
    pub const DAMAGE_RATIO: f32 = 0.15;
    /// 治疗 = 额外伤害 × 此比例（满血时无效）。
    pub const HEAL_RATIO: f32 = 1.0;

    pub fn new() -> Self {
        Self {
            ready: true,
            timer: Timer::from_seconds(Self::COOLDOWN, TimerMode::Once),
        }
    }
}

impl Default for AatroxPassiveState {
    fn default() -> Self {
        Self::new()
    }
}

/// Aatrox R（世界终结者）状态，直接挂在英雄身上，追踪剩余持续时间与额外 AD。
///
/// 移速增益由 `BuffMoveSpeed` 自管（到期自移除）；额外 AD 到期后由 `update_aatrox_r` 扣除。
#[derive(Component, Debug)]
pub struct AatroxRState {
    pub timer: Timer,
    pub bonus_ad: f32,
}

impl AatroxRState {
    pub fn new(duration: f32, bonus_ad: f32) -> Self {
        Self {
            timer: Timer::from_seconds(duration, TimerMode::Once),
            bonus_ad,
        }
    }
}
