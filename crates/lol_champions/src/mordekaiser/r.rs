use bevy::prelude::*;
use lol_base::animation_names::ANIM_SPELL4;
use lol_base::render_cmd::CommandAnimationPlay;

/// R - 死亡领域 (Realm of Death)
///
/// 诅咒敌方英雄，将其与自身一同放逐至 1v1 死亡领域 7 秒（领域半径 1200）。
/// 目标在领域内死亡则窃取其 10% 属性并生成鬼魂。
///
/// TODO: 飞弹发射、领域隔离、属性窃取、鬼魂生成、冷却。
pub fn cast_mordekaiser_r(commands: &mut Commands, entity: Entity, _point: Vec2) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL4.to_string(),
        repeat: false,
        duration: None,
    });

    debug!("莫德凯撒 R 死亡领域 施法（框架占位，领域/窃取逻辑待实现）");
}
