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

/// 标记 Q 内圈伤害，wiki 规定内圈不叠出血。
pub const DARIUS_Q_INNER_TAG: u32 = 2;
/// 标记 Q 外圈伤害，用于回血统计（外圈命中英雄才回血）。
pub const DARIUS_Q_OUTER_TAG: u32 = 4;

/// Q 外圈命中后的待回血结算。命中每名英雄递增，上限 3 名。
#[derive(Component, Default)]
pub struct DariusQHealPending {
    /// 外圈命中的英雄数（上限 3）
    pub hit_count: u8,
    /// 每名英雄的已损失生命值回血百分比（配置 MissingHealthHeal / 100）
    pub heal_pct_normalized: f32,
}

/// W 击杀待返蓝减CD标记。
///
/// W 施放时插入，`EventAttackEnd` 命中后若击杀目标则返蓝 40 和减 CD 50%。
#[derive(Component)]
pub struct DariusWRefundPending {
    pub skill_entity: Entity,
}

/// W 击杀延迟检测标记。
///
/// `EventAttackEnd` 触发时 W 额外伤害尚在 command 队列中未应用，
/// 此标记由 `on_darius_w_attack_end` 插入，在 `FixedUpdate` 中
/// 等延迟伤害执行完毕后再检查目标是否死亡。
#[derive(Component)]
pub struct DariusWKillPending {
    pub skill_entity: Entity,
    pub target: Entity,
}

/// R 位移追踪标记。
///
/// R 施放时插入，在 `EventMovementEnd` 抵达时根据目标出血层数结算伤害。
#[derive(Component)]
pub struct DariusRLeapPending {
    pub target: Entity,
    pub skill_entity: Entity,
    pub skill_level: u8,
}

/// R 击杀延迟检测标记。
///
/// 抵达目标应用伤害后插入，在 `FixedUpdate` 中检查目标是否死亡，
/// 若击杀则重置冷却/添加重施窗口。
#[derive(Component)]
pub struct DariusRKillPending {
    pub skill_entity: Entity,
    pub target: Entity,
    pub skill_level: u8,
}

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
