use bevy::prelude::*;

/// 伤害数字插件
///
/// 实际逻辑已移至 `PluginUIFloatingNumber`（`floating_number.rs`），
/// 此处仅保留空插件以保持模块结构，未来可移除。
#[derive(Default)]
pub struct PluginUIDamage;

impl Plugin for PluginUIDamage {
    fn build(&self, _app: &mut App) {}
}
