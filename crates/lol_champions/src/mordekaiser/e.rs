use bevy::prelude::*;
use lol_base::animation_names::ANIM_SPELL3;
use lol_base::render_cmd::CommandAnimationPlay;

/// E - 断魂一拽 (Death's Grasp)
///
/// 释放死亡之爪（飞行 550），命中敌人后将其向莫德凯撒拽回 250，造成魔法伤害。
/// 被动：提供法术穿透。
///
/// TODO: 飞弹发射、命中拽回位移、伤害结算、被动层数叠加、法穿被动。
pub fn cast_mordekaiser_e(commands: &mut Commands, entity: Entity, _point: Vec2) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL3.to_string(),
        repeat: false,
        duration: None,
    });

    debug!("莫德凯撒 E 断魂一拽 施法（框架占位，飞弹/拽回逻辑待实现）");
}
