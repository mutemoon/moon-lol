use bevy::prelude::*;

/// 莫德凯撒 W - 不坏之身护盾组件。
///
/// 记录护盾当前值、上限、衰减计时，以及剩余可重施的治疗窗口。
///
/// TODO: 与通用护盾组件（如 BuffShieldWhite）整合时填充实现。
#[derive(Component, Debug, Clone)]
pub struct BuffMordekaiserWShield {
    /// 当前护盾值
    pub value: f32,
    /// 护盾上限（30% 最大生命）
    pub max: f32,
    /// 护盾持续总时长
    pub duration: f32,
    /// 已持续时间
    pub elapsed: f32,
    /// 是否仍处于重施治疗窗口
    pub can_recast: bool,
}

/// 莫德凯撒 R - 死亡领域，标记被放逐的目标与决斗状态。
#[derive(Component, Debug, Clone)]
pub struct MordekaiserRealm {
    /// 领域持续时长（7 秒）
    pub duration: f32,
    /// 已持续时间
    pub elapsed: f32,
    /// 被放逐的目标实体
    pub target: Entity,
}

/// 莫德凯撒 R 击杀窃取的属性增益组件。
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
