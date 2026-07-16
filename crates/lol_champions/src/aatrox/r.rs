//! Aatrox R - 世界终结者 (World Ender)
//!
//! 10s 内 +AD + 移速；到期移除。

use bevy::prelude::*;
use lol_base::animation_names::ANIM_SPELL4;
use lol_base::render_cmd::CommandAnimationPlay;
use lol_base::spell::Spell;
use lol_core::base::buff::BuffOf;
use lol_core::buffs::common_buffs::BuffMoveSpeed;
use lol_core::damage::Damage;
use lol_core::skill::{EventSkillCast, Skill, SkillSlot, get_skill_data_value};

use crate::aatrox::Aatrox;
use crate::aatrox::buffs::AatroxRState;

/// R 伤害标签
pub const AATROX_R_TAG: u32 = 13;

pub fn on_aatrox_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_aatrox: Query<(), With<Aatrox>>,
    q_skill: Query<&Skill>,
    q_damage: Query<&Damage>,
    res_spells: Res<Assets<Spell>>,
) {
    let entity = trigger.event_target();
    if q_aatrox.get(entity).is_err() {
        return;
    }
    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::R) {
        return;
    }
    let Some(spell_obj) = res_spells.get(&skill.spell) else {
        return;
    };
    let ad = q_damage.get(entity).map(|d| d.0).unwrap_or(0.0);

    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL4.to_string(),
        repeat: false,
        duration: None,
    });
    let duration = get_skill_data_value(spell_obj, "RDuration", skill.level).unwrap_or(10.0);
    let ms_bonus =
        get_skill_data_value(spell_obj, "RMovementSpeedBonus", skill.level).unwrap_or(0.4);
    let ad_amp = get_skill_data_value(spell_obj, "RTotalADAmp", skill.level).unwrap_or(0.1);
    let bonus_ad = ad * ad_amp;

    commands
        .entity(entity)
        .insert(AatroxRState::new(duration, bonus_ad));
    commands.entity(entity).queue(move |mut e: EntityWorldMut| {
        if let Some(mut damage) = e.get_mut::<Damage>() {
            damage.0 += bonus_ad;
        }
    });
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffMoveSpeed::new(ms_bonus, duration));
}

/// R 持续期：到时移除额外 AD 与状态。
pub fn update_aatrox_r(
    time: Res<Time>,
    mut commands: Commands,
    mut q_r: Query<(Entity, &mut AatroxRState), With<Aatrox>>,
) {
    for (entity, mut state) in q_r.iter_mut() {
        state.timer.tick(time.delta());
        if state.timer.just_finished() {
            let bonus_ad = state.bonus_ad;
            commands.entity(entity).queue(move |mut e: EntityWorldMut| {
                if let Some(mut damage) = e.get_mut::<Damage>() {
                    damage.0 = (damage.0 - bonus_ad).max(0.0);
                }
            });
            commands.entity(entity).remove::<AatroxRState>();
        }
    }
}