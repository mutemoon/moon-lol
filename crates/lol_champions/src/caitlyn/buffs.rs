use bevy::prelude::*;
use lol_core::base::buff::Buff;

/// 女警被动 - 爆头
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "CaitlynPassive" })]
pub struct BuffCaitlynPassive {
    pub stacks: u8,
}

impl BuffCaitlynPassive {
    pub fn new() -> Self {
        Self { stacks: 1 }
    }
}
