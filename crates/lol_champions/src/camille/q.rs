//! Camille Q（精准礼仪 / Precision Protocol）。
//!
//! 双段强化普攻：第一段附加 TADRatio * AD 额外伤害，
//! 第二段附加 TADRatio * QEmpoweredAmp * AD 额外伤害。

use bevy::prelude::*;
use lol_base::animation_names::ANIM_SPELL1;
use lol_base::render_cmd::CommandAnimationPlay;
use lol_base::spell::Spell;
use lol_core::attack::CommandAttackReset;
use lol_core::base::buff::BuffOf;
use lol_core::buffs::on_hit::{BuffOnHitBonusDamage, BuffOnHitCounter};
use lol_core::skill::{CoolDown, EventSkillCast, Skill, SkillRecastWindow, SkillSlot, get_skill_data_value};

use crate::camille::Camille;

/// 清除角色上既存的强化普攻计数器与额外伤害 buff（Q1→Q2 切换时刷新用）。
fn clear_camille_on_hit(
    commands: &mut Commands,
    camille: Entity,
    q_buffof: &Query<(Entity, &BuffOf)>,
    q_counter: &Query<&BuffOnHitCounter>,
    q_bonus: &Query<&BuffOnHitBonusDamage>,
) {
    for (e, bo) in q_buffof.iter() {
        if bo.0 != camille {
            continue;
        }
        if q_counter.get(e).is_ok() || q_bonus.get(e).is_ok() {
            commands.entity(e).despawn();
        }
    }
}

pub fn on_camille_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_camille: Query<(), With<Camille>>,
    q_skill: Query<(&Skill, &CoolDown, Option<&SkillRecastWindow>)>,
    q_buffof: Query<(Entity, &BuffOf)>,
    q_onhit_counter: Query<&BuffOnHitCounter>,
    q_onhit_bonus: Query<&BuffOnHitBonusDamage>,
    res_spells: Res<Assets<Spell>>,
) {
    let entity = trigger.event_target();
    if q_camille.get(entity).is_err() {
        return;
    }

    let Ok((skill, cooldown, recast)) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::Q) {
        return;
    }
    let Some(spell_obj) = res_spells.get(&skill.spell) else {
        return;
    };
    let level = skill.level;
    let tad_ratio = get_skill_data_value(spell_obj, "TADRatio", level).unwrap_or(0.15);
    let empowered_amp = get_skill_data_value(spell_obj, "QEmpoweredAmp", level).unwrap_or(2.0);
    let q2_duration = get_skill_data_value(spell_obj, "Q2Duration", level).unwrap_or(2.0);
    let recast_time = get_skill_data_value(spell_obj, "QTotalRecastTime", level).unwrap_or(3.5);

    let stage = recast.map(|w| w.stage).unwrap_or(1);

    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL1.to_string(),
        repeat: false,
        duration: None,
    });

    clear_camille_on_hit(&mut commands, entity, &q_buffof, &q_onhit_counter, &q_onhit_bonus);

    commands.trigger(CommandAttackReset { entity });

    if stage == 1 {
        commands
            .entity(entity)
            .with_related::<BuffOf>(BuffOnHitCounter::new(1, q2_duration))
            .with_related::<BuffOf>(BuffOnHitBonusDamage {
                flat: 0.0,
                ratio: tad_ratio,
            });
        commands
            .entity(trigger.skill_entity)
            .insert(SkillRecastWindow::new(2, 2, recast_time));
    } else {
        commands
            .entity(entity)
            .with_related::<BuffOf>(BuffOnHitCounter::new(1, q2_duration))
            .with_related::<BuffOf>(BuffOnHitBonusDamage {
                flat: 0.0,
                ratio: tad_ratio * empowered_amp,
            });
        commands.entity(trigger.skill_entity).remove::<SkillRecastWindow>();
        commands.entity(trigger.skill_entity).insert((CoolDown {
            duration: cooldown.duration,
            timer: Some(Timer::from_seconds(cooldown.duration, TimerMode::Once)),
        },));
    }
}