use bevy::prelude::*;

/// 被动 - 黑暗起兮 (Darkness Rise)
///
/// 普攻 / Q 命中 / E 命中各获得 1 层 Darkness，满 3 层激活 DoT 光环（半径 375），
/// 持续对附近敌人造成魔法伤害，并提供 3% 移速与普攻 40% AP 附伤。
/// 脱战 4 秒后失效。
///
/// TODO: 层数叠加、满层激活、DoT 光环、移速/普攻附伤、脱战计时。

/// 莫德凯撒被动 Darkness 层数组件。
#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct MordekaiserDarkness {
    /// 当前层数，0..=3
    pub stacks: u8,
}

impl MordekaiserDarkness {
    pub const MAX: u8 = 3;
}
