use bevy::prelude::*;

use crate::core::base::buff::Buff;

/// 琴女被动 - 时光乐章
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "BardPassive" })]
pub struct BuffBardPassive {
    pub meeps: u8,
}

impl BuffBardPassive {
    pub fn new() -> Self {
        Self { meeps: 0 }
    }
}
