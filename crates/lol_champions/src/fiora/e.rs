use bevy::prelude::*;
use lol_base::render_cmd::CommandAnimationPlay;
use lol_base::spell::Spell;
use lol_core::attack::{BuffAttack, CommandAttackReset, EventAttackEnd};
use lol_core::base::buff::{Buff, BuffOf, Buffs};
use lol_core::buffs::cc_debuffs::DebuffSlow;
use lol_core::damage::{CommandDamageCreate, Damage, DamageType};
use lol_core::skill::{EventSkillCast, Skill, SkillSlot, get_skill_data_value};

use crate::fiora::Fiora;

/// E 持续时间（ron BuffDuration = 3s）。
const FIORA_E_DURATION: f32 = 3.0;
const FIORA_E_SLOW_PERCENT: f32 = 0.4;
const FIORA_E_SLOW_DURATION: f32 = 1.0;

#[derive(Component, Clone, Debug)]
#[require(Buff = Buff { name: "FioraE" })]
pub struct BuffFioraE {
    pub left: i32,
    pub crit_bonus_ratio: f32,
    pub timer: Timer,
}

impl Default for BuffFioraE {
    fn default() -> Self {
        Self {
            left: 2,
            crit_bonus_ratio: 0.5,
            timer: Timer::from_seconds(FIORA_E_DURATION, TimerMode::Once),
        }
    }
}

pub fn on_fiora_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_fiora: Query<(), With<Fiora>>,
    q_skill: Query<&Skill>,
    res_spells: Res<Assets<Spell>>,
) {
    let entity = trigger.event_target();
    if q_fiora.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::E) {
        return;
    }

    let as_percent = res_spells
        .get(&skill.spell)
        .and_then(|s| get_skill_data_value(s, "ASPercent", skill.level))
        .unwrap_or(0.4);
    let crit_ratio = res_spells
        .get(&skill.spell)
        .and_then(|s| get_skill_data_value(s, "AttackTwoPercentTAD", skill.level))
        .map(|v| (v - 1.0).max(0.0))
        .unwrap_or(0.5);

    commands.entity(entity).insert(BuffAttack {
        bonus_attack_speed: as_percent,
    });
    commands.entity(entity).with_related::<BuffOf>(BuffFioraE {
        left: 2,
        crit_bonus_ratio: crit_ratio,
        timer: Timer::from_seconds(FIORA_E_DURATION, TimerMode::Once),
    });
    commands.trigger(CommandAttackReset { entity });
}

/// 普攻结束：第一击减速目标，第二击造成额外暴击伤害；两击耗尽后移除 buff 与攻速加成。
pub fn on_event_attack_end(
    trigger: On<EventAttackEnd>,
    mut commands: Commands,
    q_buffs: Query<&Buffs>,
    mut q_buff_fiora_e: Query<&mut BuffFioraE>,
    q_damage: Query<&Damage>,
) {
    let entity = trigger.event_target();
    let target = trigger.target;
    let Ok(buffs) = q_buffs.get(entity) else {
        return;
    };
    let ad = q_damage.get(entity).map(|d| d.0).unwrap_or(0.0);

    for buff in buffs.iter() {
        let Ok(mut buff_fiora_e) = q_buff_fiora_e.get_mut(buff) else {
            continue;
        };

        let was = buff_fiora_e.left;
        buff_fiora_e.left -= 1;

        match was {
            2 => {
                commands
                    .entity(target)
                    .with_related::<BuffOf>(DebuffSlow::new(
                        FIORA_E_SLOW_PERCENT,
                        FIORA_E_SLOW_DURATION,
                    ));
            }
            1 => {
                let bonus = buff_fiora_e.crit_bonus_ratio * ad;
                if bonus > 0.0 {
                    commands.entity(target).trigger(|e| CommandDamageCreate {
                        entity: e,
                        source: entity,
                        damage_type: DamageType::Physical,
                        amount: bonus,
                        tag: None,
                    });
                }
            }
            _ => {}
        }

        if buff_fiora_e.left <= 0 {
            commands.entity(buff).despawn();
            commands.entity(entity).remove::<BuffAttack>();
        }
    }
}

/// E 计时：到期后移除 buff 与攻速加成。
pub fn update_fiora_e_buff(
    mut commands: Commands,
    mut q_buff: Query<(Entity, &BuffOf, &mut BuffFioraE)>,
    time: Res<Time<Fixed>>,
) {
    for (buff_entity, buff_of, mut buff) in q_buff.iter_mut() {
        buff.timer.tick(time.delta());
        if buff.timer.is_finished() {
            commands.entity(buff_of.0).remove::<BuffAttack>();
            commands.entity(buff_entity).despawn();
        }
    }
}