use bevy::prelude::*;

// ===== W - 不坏之身 =====

/// W 被动储存的伤害值（挂在自身）。
///
/// 受到伤害时按 7.5% 折算存入，最高 30% 最大生命；释放 W 时并入护盾并清零。
#[derive(Component, Debug, Clone, Default)]
pub struct MordekaiserWStorage {
    /// 已储存的伤害值
    pub stored: f32,
}

/// W 护盾追踪器（与 [`lol_core::buffs::BuffShieldWhite`] 同挂于护盾子实体）。
///
/// 通用 [`BuffShieldWhite`] 负责实际承伤吸收（被伤害管线自动消费，harness 可读），
/// 本组件仅记录衰减计时与衰减基准，避免与通用护盾职责重叠。
/// 重施判定以"是否存在 W 护盾子 buff"为准，故不再单独维护 can_recast 标记。
#[derive(Component, Debug, Clone)]
pub struct BuffMordekaiserWShield {
    /// 已持续时间
    pub elapsed: f32,
    /// 衰减基准（最大生命，0.5%/秒衰减参照）
    pub max_health: f32,
}

/// W 护盾持续时长（秒，ron `Duration`）。
pub const MORDE_W_DURATION: f32 = 5.0;
/// W 被动储存的受伤折算比例（ron `DamageTakenConversion` = 0.075）。
pub const MORDE_W_DAMAGE_TAKEN_CONVERSION: f32 = 0.075;
/// W 储存 / 护盾相对最大生命的上限（ron `MaxHealthCap` = 0.3）。
pub const MORDE_W_MAX_HEALTH_CAP: f32 = 0.3;
/// W 基础护盾占最大生命比例（ron `BaseShield` = 0.05）。
pub const MORDE_W_BASE_SHIELD: f32 = 0.05;
/// W 护盾开始衰减前的延迟（秒，ron `TimeBeforeDecay` = 1.0）。
pub const MORDE_W_TIME_BEFORE_DECAY: f32 = 1.0;
/// W 护盾每秒衰减占最大生命比例（ron `DecayPerSecond` = 0.005）。
pub const MORDE_W_DECAY_PER_SECOND: f32 = 0.005;

// ===== R - 死亡领域 =====

/// R - 死亡领域，标记自身进入决斗、放逐目标（挂在自身）。
#[derive(Component, Debug, Clone)]
pub struct MordekaiserRealm {
    /// 领域持续时长（秒）
    pub duration: f32,
    /// 已持续时间
    pub elapsed: f32,
    /// 被放逐的目标实体
    pub target: Entity,
    /// 是否已对目标完成属性窃取（避免重复窃取）
    pub stolen: bool,
}

/// R 击杀窃取的属性增益组件（挂在自身）。
#[derive(Component, Debug, Clone, Default)]
pub struct MordekaiserStatSteal {
    /// 窃取的 AD
    pub ad: f32,
    /// 窃取的 AP
    pub ap: f32,
    /// 窃取的生命值
    pub health: f32,
    /// 窃取的护甲
    pub armor: f32,
}

/// R 领域持续时长（秒，ron `SpiritRealmDuration` = 7.0）。
pub const MORDE_R_DURATION: f32 = 7.0;
/// R 窃取属性比例（ron `StatStealPercentScalar` = 0.1）。
pub const MORDE_R_STAT_STEAL_RATIO: f32 = 0.1;
