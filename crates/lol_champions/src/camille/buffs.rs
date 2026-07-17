//! Camille 特有 Buff/状态组件。
//!
//! - `BuffCamilleWallCling`：墙壁锚点，E1 飞弹碰墙后挂在子实体上。
//! - `CamilleE2State`：E2 冲刺状态，挂在冠军身上。

use bevy::prelude::*;
use lol_core::base::buff::Buff;

/// 墙壁锚点：E1 粘性飞弹碰墙后标记墙壁位置。
/// 挂在冠军的子实体上，E2 施放时读取并销毁。
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "CamilleWallCling" })]
pub struct BuffCamilleWallCling {
    pub wall_point: Vec3,
}

/// E2 冲刺状态：挂在冠军身上，标记冲刺目标。
/// E2 冲刺结束时读取，对目标施加眩晕与伤害。
#[derive(Component, Debug, Clone)]
pub struct CamilleE2State {
    pub target: Option<Entity>,
    pub stun_duration: f32,
    pub damage: f32,
}

/// R 跃击追踪标记：挂在冠军身上，标记跃击目标与 R 参数。
/// 抵达目标后用于击退其他敌人。
#[derive(Component, Debug, Clone)]
pub struct CamilleRLeapPending {
    pub target: Entity,
    pub percent: f32,
    pub duration: f32,
}