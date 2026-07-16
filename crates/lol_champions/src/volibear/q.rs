//! Volibear Q - 雷霆巨爪 (Thundering Roar)
//!
//! 强化下次普攻：额外物理伤害 + 眩晕 + 移速。

use bevy::prelude::*;
use lol_base::animation_names::ANIM_SPELL1;
use lol_base::render_cmd::CommandAnimationPlay;
use lol_base::spell::Spell;
use lol_core::attack::CommandAttackReset;
use lol_core::base::buff::BuffOf;
use lol_core::buffs::common_buffs::BuffMoveSpeed;
use lol_core::buffs::on_hit::{BuffOnHitBonusDamage, BuffOnHitCounter, BuffOnHitStun};
use lol_core::damage::Damage;
use lol_core::skill::{EventSkillCast, Skill, SkillSlot, get_skill_data_value, get_skill_value};

use crate::volibear::Volibear;

pub fn on_volibear_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_volibear: Query<(), With<Volibear>>,
    q_skill: Query<&Skill>,
    q_damage: Query<&Damage>,
    res_spells: Res<Assets<Spell>>,
) {
    let entity = trigger.event_target();
    if q_volibear.get(entity).is_err() {
        return;
    }
    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::Q) {
        return;
    }
    let Some(spell_obj) = res_spells.get(&skill.spell) else {
        return;
    };
    let ad = q_damage.get(entity).map(|d| d.0).unwrap_or(0.0);

    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL1.to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandAttackReset { entity });

    let bonus = get_skill_value(spell_obj, "calculated_damage", skill.level, |stat| {
        if stat == 2 { ad } else { 0.0 }
    })
    .unwrap_or(0.0);
    let ms_bonus = get_skill_data_value(spell_obj, "MaxSpeed", skill.level).unwrap_or(0.17);
    let duration = get_skill_data_value(spell_obj, "Duration", skill.level).unwrap_or(4.0);
    let stun_duration = get_skill_data_value(spell_obj, "StunDuration", skill.level).unwrap_or(1.0);

    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffOnHitCounter::new(1, duration))
        .with_related::<BuffOf>(BuffOnHitBonusDamage {
            flat: bonus,
            ratio: 0.0,
        })
        .with_related::<BuffOf>(BuffOnHitStun {
            duration: stun_duration,
        })
        .with_related::<BuffOf>(BuffMoveSpeed::new(ms_bonus, duration));
}