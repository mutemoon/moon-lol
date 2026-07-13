use bevy::prelude::*;
use lol_base::animation_names::ANIM_SPELL1;
use lol_base::render_cmd::CommandAnimationPlay;

/// Q - 破灭之锤 (Obliterate)
///
/// 向前挥砸权杖，对矩形区域（起点 400 / 长度 625 / 宽度 160）内敌人造成魔法伤害，
/// 孤立目标额外增伤。
///
/// TODO: 矩形范围伤害场、孤立判定、伤害结算、被动层数叠加。
pub fn cast_mordekaiser_q(commands: &mut Commands, entity: Entity, _point: Vec2) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL1.to_string(),
        repeat: false,
        duration: None,
    });

    debug!("莫德凯撒 Q 破灭之锤 施法（框架占位，伤害逻辑待实现）");
}
