//! Darius W - 致残打击 (Crippling Strike)
//!
//! 攻击重置 + 强化普攻（额外伤害 + 减速）

use bevy::prelude::*;
use lol_base::animation_names::ANIM_SPELL2;
use lol_base::render_cmd::CommandAnimationPlay;
use lol_base::spell::Spell;
use lol_core::attack::CommandAttackReset;
use lol_core::base::buff::BuffOf;
use lol_core::buffs::on_hit::{BuffOnHitBonusDamage, BuffOnHitCounter, BuffOnHitSlow};
use lol_core::skill::{EventSkillCast, Skill, SkillSlot, get_skill_value};

use crate::darius::Darius;

pub fn on_darius_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_darius: Query<(), With<Darius>>,
    q_skill: Query<&Skill>,
    res_spells: Res<Assets<Spell>>,
) {
    let entity = trigger.event_target();
    if q_darius.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::W) {
        return;
    }
    let Some(spell_obj) = res_spells.get(&skill.spell) else {
        return;
    };

    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL2.to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandAttackReset { entity });

    // 从 RON 读取总伤害倍率（empowered_attack_damage = total_AD * total_multiplier），
    // 减去 1.0（基础普攻）得到额外伤害比例
    let total_mult = get_skill_value(spell_obj, "empowered_attack_damage", skill.level, |stat| {
        if stat == 2 { 1.0 } else { 0.0 }
    })
    .unwrap_or(1.5);
    let ratio = total_mult - 1.0;

    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffOnHitCounter::new(1, 1.0))
        .with_related::<BuffOf>(BuffOnHitBonusDamage { flat: 0.0, ratio })
        .with_related::<BuffOf>(BuffOnHitSlow {
            percent: 0.5,
            duration: 1.0,
        });
}