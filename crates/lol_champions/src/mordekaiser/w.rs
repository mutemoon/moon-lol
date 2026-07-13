use bevy::prelude::*;
use lol_base::animation_names::ANIM_SPELL2;
use lol_base::render_cmd::CommandAnimationPlay;

/// W - 不坏之身 (Indestructible)
///
/// 被动：受到伤害的 7.5% 储存为护盾（上限 30% 最大生命）。
/// 主动：开盾（5% 最大生命基础 + 储存值），持续 5 秒；重施消耗剩余护盾治疗自身。
/// 护盾 1 秒后开始衰减，每秒 0.5% 最大生命。
///
/// TODO: 伤害储存、护盾组件、重施窗口、治疗结算、衰减。
pub fn cast_mordekaiser_w(commands: &mut Commands, entity: Entity) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL2.to_string(),
        repeat: false,
        duration: None,
    });

    debug!("莫德凯撒 W 不坏之身 施法（框架占位，护盾/治疗逻辑待实现）");
}
