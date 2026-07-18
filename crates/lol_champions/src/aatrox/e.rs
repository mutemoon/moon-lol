//! Aatrox E - 暗影冲锋 (Umbral Dash)
//!
//! 向指针方向突进（最大 300）。

use bevy::prelude::*;
use lol_base::animation_names::ANIM_SPELL3;
use lol_base::render_cmd::CommandAnimationPlay;
use lol_base::spell::Spell;
use lol_core::action::dash::{ActionDash, DashMoveType};
use lol_core::skill::{EventSkillCast, Skill, SkillSlot, get_skill_data_value};

use crate::aatrox::Aatrox;

/// E 最大突进距离
pub const AATROX_E_MAX_RANGE: f32 = 300.0;
/// E 突进速度
pub const AATROX_E_DASH_SPEED: f32 = 800.0;

pub fn on_aatrox_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_aatrox: Query<(), With<Aatrox>>,
    q_skill: Query<&Skill>,
    res_spells: Res<Assets<Spell>>,
) {
    let entity = trigger.event_target();
    if q_aatrox.get(entity).is_err() {
        return;
    }
    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::E) {
        return;
    }
    let Some(spell_obj) = res_spells.get(&skill.spell) else {
        return;
    };

    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL3.to_string(),
        repeat: false,
        duration: None,
    });
    let max_range =
        get_skill_data_value(spell_obj, "EMaxRange", skill.level).unwrap_or(AATROX_E_MAX_RANGE);
    let speed =
        get_skill_data_value(spell_obj, "EDashSpeed", skill.level).unwrap_or(AATROX_E_DASH_SPEED);
    commands.trigger(ActionDash {
        entity,
        point: trigger.point,
        move_type: DashMoveType::Pointer { max: max_range },
        speed,
    });
}
