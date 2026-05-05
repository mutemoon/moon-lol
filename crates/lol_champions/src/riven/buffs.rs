use bevy::prelude::*;
use lol_core::base::buff::Buff;

/// W 眩晕 buff，添加在受影响的敌人身上
#[derive(Component, Debug, Clone)]
pub struct BuffStun {
    pub timer: Timer,
}

/// R 被动 buff，作为子实体添加
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "RivenR" })]
pub struct BuffRivenR {
    pub timer: Timer,
}

/// 标记 Q3 位移中，落地后触发击退
#[derive(Component, Debug, Default)]
pub struct RivenQ3Pending;
