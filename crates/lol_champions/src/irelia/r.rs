//! R - 先锋之刃 (Vanguard's Edge)
//!
//! 发射一片飞刃，对命中敌人造成魔法伤害并标记"不稳"+ 减速。
//! 伤害携带 `IRELIA_R_DAMAGE_TAG`，由 `on_irelia_damage_hit` 识别后施加
//! 不稳标记（DebuffIreliaUnsteady）与减速（DebuffSlow）。
//!
//! 单段施法，冷却由核心 AfterCast 自动管理。

use bevy::prelude::*;
use lol_base::animation_names::ANIM_SPELL4;
use lol_base::render_cmd::CommandAnimationPlay;
use lol_base::spell::Spell;
use lol_core::damage::{CommandDamageCreate, DamageType};
use lol_core::entities::champion::Champion;
use lol_core::skill::get_skill_value;
use lol_core::team::Team;

use crate::irelia::IRELIA_R_DAMAGE_TAG;

/// R 命中判定半径（飞刃覆盖范围）
pub const IRELIA_R_RADIUS: f32 = 350.0;

pub fn cast_irelia_r(
    commands: &mut Commands,
    entity: Entity,
    point: Vec2,
    skill_level: usize,
    spell: &Spell,
    q_enemies: &Query<(Entity, &Transform, &Team), With<Champion>>,
    team: Team,
    ad: f32,
) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL4.to_string(),
        repeat: false,
        duration: None,
    });

    let amount = get_skill_value(spell, "missile_damage", skill_level, |stat| {
        if stat == 2 { ad } else { 0.0 }
    })
    .unwrap_or(0.0);

    for (target, tf, t) in q_enemies.iter() {
        if *t == team {
            continue;
        }
        if tf.translation.xz().distance(point) > IRELIA_R_RADIUS {
            continue;
        }
        commands.entity(target).trigger(|e| CommandDamageCreate {
            entity: e,
            source: entity,
            damage_type: DamageType::Magic,
            amount,
            tag: Some(IRELIA_R_DAMAGE_TAG),
        });
    }
}
