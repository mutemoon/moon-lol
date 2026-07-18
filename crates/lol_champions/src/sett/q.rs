//! Sett Q - 屈人之威 (Knuckle Down)
//!
//! 强化下 2 次普攻：附带目标最大生命百分比物理伤害 + 30% 移速 4s。

use bevy::prelude::*;
use lol_base::animation_names::ANIM_SPELL1;
use lol_base::render_cmd::CommandAnimationPlay;
use lol_core::attack::CommandAttackReset;
use lol_core::base::buff::BuffOf;
use lol_core::buffs::common_buffs::BuffMoveSpeed;
use lol_core::buffs::on_hit::{BuffOnHitCounter, BuffOnHitTargetMaxHp};
use lol_core::skill::{EventSkillCast, Skill, SkillSlot};

use crate::sett::Sett;

/// Q 每级目标最大生命百分比
pub const SETT_Q_MAX_HP_RATIO: [f32; 5] = [0.03, 0.035, 0.04, 0.045, 0.05];
pub const SETT_Q_ATTACKS: u8 = 2;
pub const SETT_Q_DURATION: f32 = 4.0;
pub const SETT_Q_MS_BONUS: f32 = 0.30;

pub fn on_sett_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_sett: Query<(), With<Sett>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_sett.get(entity).is_err() {
        return;
    }
    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::Q) {
        return;
    }

    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL1.to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandAttackReset { entity });

    let lvl_idx = skill
        .level
        .saturating_sub(1)
        .min(SETT_Q_MAX_HP_RATIO.len() - 1);
    let ratio = SETT_Q_MAX_HP_RATIO[lvl_idx];

    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffOnHitCounter::new(SETT_Q_ATTACKS, SETT_Q_DURATION))
        .with_related::<BuffOf>(BuffOnHitTargetMaxHp { ratio })
        .with_related::<BuffOf>(BuffMoveSpeed::new(SETT_Q_MS_BONUS, SETT_Q_DURATION));
}
