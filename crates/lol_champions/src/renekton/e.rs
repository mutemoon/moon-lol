//! Renekton E - 横冲直撞 (Slice and Dice)
//!
//! 两段充能突进：E1 冲锋（200 码），E2 再次冲锋（200 码），4 秒内可重施。

use bevy::prelude::*;
use lol_base::animation_names::ANIM_SPELL3;
use lol_base::render_cmd::CommandAnimationPlay;
use lol_core::action::dash::{ActionDash, DashMoveType};
use lol_core::skill::{CoolDown, EventSkillCast, Skill, SkillRecastWindow, SkillSlot};

use crate::renekton::Renekton;

/// E 重施窗口时长（秒）
pub const RENECKTON_E_RECAST_WINDOW: f32 = 4.0;

pub fn on_renekton_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_renekton: Query<(), With<Renekton>>,
    _q_transform: Query<&Transform>,
    q_skill: Query<(&Skill, &CoolDown, Option<&SkillRecastWindow>)>,
) {
    let entity = trigger.event_target();
    if q_renekton.get(entity).is_err() {
        return;
    }

    let Ok((skill, cooldown, recast)) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::E) {
        return;
    }

    let skill_entity = trigger.skill_entity;
    let point = trigger.point;
    let stage = recast.map(|w| w.stage).unwrap_or(1);

    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL3.to_string(),
        repeat: false,
        duration: None,
    });

    if stage == 1 {
        commands.trigger(ActionDash {
            entity,
            point,
            move_type: DashMoveType::Pointer { max: 200.0 },
            speed: 700.0,
        });
        commands.entity(skill_entity).insert(SkillRecastWindow::new(
            2,
            2,
            RENECKTON_E_RECAST_WINDOW,
        ));
    } else {
        commands.trigger(ActionDash {
            entity,
            point,
            move_type: DashMoveType::Pointer { max: 200.0 },
            speed: 700.0,
        });
        commands.entity(skill_entity).remove::<SkillRecastWindow>();
        commands.entity(skill_entity).insert((CoolDown {
            duration: cooldown.duration,
            timer: Some(Timer::from_seconds(cooldown.duration, TimerMode::Once)),
        },));
    };
}