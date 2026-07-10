use bevy::prelude::*;
use lol_core::attack::{BuffAttack, CommandAttackReset, EventAttackEnd};
use lol_core::base::buff::{Buff, BuffOf, Buffs};
use lol_core::buffs::cc_debuffs::DebuffSlow;
use lol_core::damage::{CommandDamageCreate, Damage, DamageType};

/// E 持续时间（ron BuffDuration = 3s）。
const FIORA_E_DURATION: f32 = 3.0;
/// 第一击减速比例（wiki：40%）与持续时间（1s）。
const FIORA_E_SLOW_PERCENT: f32 = 0.4;
const FIORA_E_SLOW_DURATION: f32 = 1.0;

#[derive(Component, Clone, Debug)]
#[require(Buff = Buff { name: "FioraE" })]
pub struct BuffFioraE {
    pub left: i32,
    /// 第二击暴击的额外伤害比例（AttackTwoPercentTAD - 1，1 级 = 0.5）。
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

/// E 施法：按等级赋予攻速，挂上 BuffFioraE（下两次普攻增强），重置普攻。
pub fn cast_fiora_e(
    commands: &mut Commands,
    entity: Entity,
    as_percent: f32,
    crit_bonus_ratio: f32,
) {
    commands.entity(entity).insert(BuffAttack {
        bonus_attack_speed: as_percent,
    });
    commands.entity(entity).with_related::<BuffOf>(BuffFioraE {
        left: 2,
        crit_bonus_ratio,
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
                // 第一击：减速
                commands
                    .entity(target)
                    .with_related::<BuffOf>(DebuffSlow::new(
                        FIORA_E_SLOW_PERCENT,
                        FIORA_E_SLOW_DURATION,
                    ));
            }
            1 => {
                // 第二击：暴击额外伤害 = (AttackTwoPercentTAD - 1) × AD
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
