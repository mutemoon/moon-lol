//! Darius E - 无情立场 (Apprehend)
//!
//! 主动：朝施法方向锥形拉回范围内的敌人到 Darius 脚边，击飞 0.75 秒，
//! 并施加 40% 减速 1 秒。Darius 自身不位移。
//!
//! 使用 [`ActionDisplace`] 统一位移体系：Cone{535, 90°} + PullToward + Knockup(0.75) + Slow。

use bevy::prelude::*;
use lol_base::animation_names::ANIM_SPELL3;
use lol_base::render_cmd::CommandAnimationPlay;
use lol_core::action::displace::{
    ActionDisplace, DisplaceEffect, DisplaceMotion, DisplaceTargetSelection,
};
use lol_core::skill::{EventSkillCast, Skill, SkillSlot};

use crate::darius::Darius;

/// E 锥形范围（拉回距离，wiki: 535）
pub const DARIUS_E_RANGE: f32 = 535.0;
/// E 锥形张角（度）
pub const DARIUS_E_CONE_ANGLE: f32 = 90.0;
/// E 击飞持续时间（秒，wiki: 固定 0.75）
pub const DARIUS_E_KNOCKUP_DURATION: f32 = 0.75;
/// E 拉回速度
pub const DARIUS_E_PULL_SPEED: f32 = 1200.0;
/// E 减速强度（40%）
pub const DARIUS_E_SLOW_PERCENT: f32 = 0.4;
/// E 减速持续时间（秒）
pub const DARIUS_E_SLOW_DURATION: f32 = 1.0;

pub fn on_darius_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_darius: Query<(), With<Darius>>,
    q_skill: Query<&Skill>,
    q_transform: Query<&Transform>,
) {
    let entity = trigger.event_target();
    if q_darius.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::E) {
        return;
    }

    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL3.to_string(),
        repeat: false,
        duration: None,
    });

    let Ok(transform) = q_transform.get(entity) else {
        return;
    };

    let pos = transform.translation.xz();
    let forward = (trigger.point - pos).normalize_or_zero();
    let forward = if forward == Vec2::ZERO {
        transform.forward().xz()
    } else {
        forward
    };

    // 使用统一位移体系：锥形拉回 + 击飞 + 减速
    commands.trigger(ActionDisplace {
        entity,
        targets: DisplaceTargetSelection::Cone {
            range: DARIUS_E_RANGE,
            angle: DARIUS_E_CONE_ANGLE,
            direction: forward,
        },
        motion: DisplaceMotion::PullToward {
            distance: DARIUS_E_RANGE,
            speed: DARIUS_E_PULL_SPEED,
        },
        effects: vec![
            DisplaceEffect::Knockup {
                duration: DARIUS_E_KNOCKUP_DURATION,
            },
            DisplaceEffect::Slow {
                percent: DARIUS_E_SLOW_PERCENT,
                duration: DARIUS_E_SLOW_DURATION,
            },
        ],
        cone_hit_policy: None,
    });

    debug!("Darius E: 无情立场，使用 ActionDisplace 锥形拉回");
}