use bevy::prelude::*;

use crate::base::buff::Buff;

/// 德莱文被动 - 德莱文联盟
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "DravenPassive" })]
pub struct BuffDravenPassive {
    pub stacks: u8,
}

impl BuffDravenPassive {
    pub fn new() -> Self {
        Self { stacks: 1 }
    }
}
