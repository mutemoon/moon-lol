//! Sett E - 迎面痛击 (Facebreaker)
//!
//! 锥形前后双侧检测：前方和后方锥形均命中 → 全部眩晕 + 拉回 + 伤害；
//! 仅单侧命中 → 该侧减速 + 拉回 + 伤害。
//!
//! 使用 [`ActionDisplace`] 统一位移体系 + `ConeHitPolicy`。

use bevy::prelude::*;
use lol_base::animation_names::ANIM_SPELL3;
use lol_base::render_cmd::CommandAnimationPlay;
use lol_base::spell::Spell;
use lol_core::action::displace::{
    ActionDisplace, ConeHitPolicy, DisplaceEffect, DisplaceMotion, DisplaceTargetSelection,
};
use lol_core::damage::{Damage, DamageType};
use lol_core::skill::{EventSkillCast, Skill, SkillSlot, get_skill_value};

use crate::sett::Sett;
use crate::sett::buffs::SETT_E_TAG;

/// E 范围半径
pub const SETT_E_RANGE: f32 = 490.0;
/// E 锥形角度
pub const SETT_E_CONE_ANGLE: f32 = 90.0;
/// E 拉回速度
pub const SETT_E_PULL_SPEED: f32 = 1200.0;
/// 双侧命中眩晕时长
pub const SETT_E_STUN_DURATION: f32 = 1.0;
/// 单侧命中减速比例
pub const SETT_E_SLOW_PERCENT: f32 = 0.5;
/// 单侧命中减速时长
pub const SETT_E_SLOW_DURATION: f32 = 1.0;

pub fn on_sett_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_sett: Query<(), With<Sett>>,
    q_skill: Query<&Skill>,
    q_transform: Query<&Transform>,
    q_damage: Query<&Damage>,
    res_spells: Res<Assets<Spell>>,
) {
    let entity = trigger.event_target();
    if q_sett.get(entity).is_err() {
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

    let Ok(transform) = q_transform.get(entity) else {
        return;
    };

    let pos = transform.translation.xz();
    let forward = {
        let f = (trigger.point - pos).normalize_or_zero();
        if f == Vec2::ZERO {
            transform.forward().xz()
        } else {
            f
        }
    };

    let ad = q_damage.get(entity).map(|d| d.0).unwrap_or(0.0);
    let damage = get_skill_value(spell_obj, "damage_calc", skill.level, |stat| {
        if stat == 2 { ad } else { 0.0 }
    })
    .unwrap_or(0.0);

    // 使用统一位移体系：双锥形检测 + 拉回 + 伤害 + 双侧眩晕/单侧减速
    commands.trigger(ActionDisplace {
        entity,
        targets: DisplaceTargetSelection::Cone {
            range: SETT_E_RANGE,
            angle: SETT_E_CONE_ANGLE,
            direction: forward,
        },
        motion: DisplaceMotion::PullToward {
            distance: SETT_E_RANGE,
            speed: SETT_E_PULL_SPEED,
        },
        effects: vec![DisplaceEffect::Damage {
            amount: damage,
            damage_type: DamageType::Physical,
            tag: Some(SETT_E_TAG),
        }],
        cone_hit_policy: Some(ConeHitPolicy {
            stun_duration: SETT_E_STUN_DURATION,
            slow_percent: SETT_E_SLOW_PERCENT,
            slow_duration: SETT_E_SLOW_DURATION,
        }),
    });

    debug!("Sett E: 迎面痛击，使用 ActionDisplace 双锥形检测");
}
