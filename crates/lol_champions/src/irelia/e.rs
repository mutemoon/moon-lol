//! E - 完美合奏 (Flawless Duet)
//!
//! 两段施法：
//! - E1：投掷第一把匕首，开启重施窗口。
//! - E2：投掷第二把匕首，对交汇区域敌人造成魔法伤害并眩晕 + 标记"不稳"。
//!
//! E2 伤害携带 `IRELIA_E2_DAMAGE_TAG`，由 `on_irelia_damage_hit` 识别后施加
//! 眩晕（DebuffStun）与不稳标记（DebuffIreliaUnsteady）。

use bevy::prelude::*;
use bevy::time::{Timer, TimerMode};
use lol_base::animation_names::ANIM_SPELL3;
use lol_base::render_cmd::CommandAnimationPlay;
use lol_base::spell::Spell;
use lol_core::damage::{CommandDamageCreate, DamageType};
use lol_core::entities::champion::Champion;
use lol_core::skill::{CoolDown, SkillRecastWindow, get_skill_value};
use lol_core::team::Team;

use crate::irelia::IRELIA_E2_DAMAGE_TAG;

/// E 重施窗口时长（秒）
pub const IRELIA_E_RECAST_WINDOW: f32 = 4.0;
/// E2 交汇区域半径
pub const IRELIA_E_RADIUS: f32 = 200.0;

pub fn cast_irelia_e(
    commands: &mut Commands,
    entity: Entity,
    skill_entity: Entity,
    stage: u8,
    point: Vec2,
    spell: &Spell,
    skill_level: usize,
    cooldown: &CoolDown,
    q_enemies: &Query<(Entity, &Transform, &Team), With<Champion>>,
    team: Team,
    ad: f32,
) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL3.to_string(),
        repeat: false,
        duration: None,
    });

    if stage == 1 {
        // E1：开启重施窗口
        commands
            .entity(skill_entity)
            .insert(SkillRecastWindow::new(2, 2, IRELIA_E_RECAST_WINDOW));
        return;
    }

    // E2：交汇区域魔法伤害
    let amount = get_skill_value(spell, "total_damage", skill_level, |stat| {
        if stat == 2 { ad } else { 0.0 }
    })
    .unwrap_or(0.0);

    for (target, tf, t) in q_enemies.iter() {
        if *t == team {
            continue;
        }
        if tf.translation.xz().distance(point) > IRELIA_E_RADIUS {
            continue;
        }
        commands.entity(target).trigger(|e| CommandDamageCreate {
            entity: e,
            source: entity,
            damage_type: DamageType::Magic,
            amount,
            tag: Some(IRELIA_E2_DAMAGE_TAG),
        });
    }

    // 关闭重施窗口，进入冷却
    commands.entity(skill_entity).remove::<SkillRecastWindow>();
    commands.entity(skill_entity).insert(CoolDown {
        duration: cooldown.duration,
        timer: Some(Timer::from_seconds(cooldown.duration, TimerMode::Once)),
    });
}
