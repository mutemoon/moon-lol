use bevy::ecs::entity::Entity;
use bevy::prelude::*;
use bevy::time::{Timer, TimerMode};
use lol_core::base::buff::Buff;

/// 出血持续 5 秒。
pub const DARIUS_BLEED_DURATION: f32 = 5.0;
/// 出血 DoT 周期（秒），每周期结算一次伤害。
pub const DARIUS_BLEED_TICK_INTERVAL: f32 = 1.26;
/// 出血最大层数。
pub const DARIUS_BLEED_MAX_STACKS: u8 = 5;
/// 每层出血每周期造成 0.3*AD 物理伤害（对应 ron `bleed_damage_per_stack`）。
pub const DARIUS_BLEED_AD_RATIO: f32 = 0.3;
/// 诺克萨斯之力提供的总 AD 加成比例（+50% AD）。
pub const DARIUS_NOXIAN_MIGHT_AD_RATIO: f32 = 0.5;
/// 诺克萨斯之力持续时间（秒）。
pub const DARIUS_NOXIAN_MIGHT_DURATION: f32 = 5.0;

/// 诺手被动 - 出血标记，最多 5 层。
///
/// 挂在受击目标身上（作为子 buff）。`source` 记录施加者以便 DoT 读取其 AD。
/// `duration_timer` 在每次叠层时刷新；`tick_timer` 周期性触发 DoT 结算。
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "DariusBleed" })]
pub struct BuffDariusBleed {
    pub stacks: u8,
    pub source: Entity,
    pub duration_timer: Timer,
    pub tick_timer: Timer,
}

impl BuffDariusBleed {
    /// 新建 1 层出血。
    pub fn new(source: Entity) -> Self {
        Self {
            stacks: 1,
            source,
            duration_timer: Timer::from_seconds(DARIUS_BLEED_DURATION, TimerMode::Once),
            tick_timer: Timer::from_seconds(DARIUS_BLEED_TICK_INTERVAL, TimerMode::Repeating),
        }
    }

    /// 叠一层出血（最多 [`DARIUS_BLEED_MAX_STACKS`] 层）并刷新持续时间。
    pub fn add_stack(&mut self) {
        if self.stacks < DARIUS_BLEED_MAX_STACKS {
            self.stacks += 1;
        }
        self.duration_timer = Timer::from_seconds(DARIUS_BLEED_DURATION, TimerMode::Once);
    }
}

/// 诺手被动 - 诺克萨斯之力（叠满 5 层出血触发），提供 +50% AD。
///
/// 挂在 Darius 自身。`ad_bonus` 记录已叠加到 [`lol_core::damage::Damage`] 上的数值，
/// 到期时据此还原。
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "DariusMight" })]
pub struct BuffDariusMight {
    pub ad_bonus: f32,
    pub timer: Timer,
}

impl BuffDariusMight {
    pub fn new(ad_bonus: f32) -> Self {
        Self {
            ad_bonus,
            timer: Timer::from_seconds(DARIUS_NOXIAN_MIGHT_DURATION, TimerMode::Once),
        }
    }
}
