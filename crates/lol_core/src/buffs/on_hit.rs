use bevy::prelude::*;

use crate::attack::{BuffAttack, EventAttackEnd};
use crate::base::buff::{Buff, BuffOf, Buffs};
use crate::buffs::cc_debuffs::{DebuffSlow, DebuffStun};
use crate::damage::{CommandDamageCreate, Damage, DamageType};
use crate::life::Health;

/// 强化普攻计数器 — 控制"下次攻击强化"的次数和过期时间
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "OnHitCounter" })]
pub struct BuffOnHitCounter {
    pub remaining: u8,
    pub timer: Timer,
}

impl BuffOnHitCounter {
    pub fn new(hits: u8, duration: f32) -> Self {
        Self {
            remaining: hits,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// 强化普攻额外伤害
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "OnHitBonusDamage" })]
pub struct BuffOnHitBonusDamage {
    /// 固定额外伤害
    pub flat: f32,
    /// 基于攻击力的比例（如 0.5 = 50% AD）
    pub ratio: f32,
}

/// 强化普攻基于目标最大生命值的额外伤害（如 Sett Q）
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "OnHitTargetMaxHp" })]
pub struct BuffOnHitTargetMaxHp {
    /// 目标最大生命百分比（如 0.03 = 3%）
    pub ratio: f32,
}

/// 强化普攻减速
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "OnHitSlow" })]
pub struct BuffOnHitSlow {
    pub percent: f32,
    pub duration: f32,
}

/// 强化普攻眩晕
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "OnHitStun" })]
pub struct BuffOnHitStun {
    pub duration: f32,
}

/// 统一消费 EventAttackEnd 的所有 on-hit 组件
///
/// 技能只需挂上 `BuffOnHitCounter` + 所需的 on-hit 效果组件，
/// 这里自动在每次普攻命中时消费它们。
pub fn on_event_attack_end_consume_on_hit(
    trigger: On<EventAttackEnd>,
    mut commands: Commands,
    q_buffs: Query<&Buffs>,
    mut q_counter: Query<&mut BuffOnHitCounter>,
    q_bonus_damage: Query<&BuffOnHitBonusDamage>,
    q_target_max_hp: Query<&BuffOnHitTargetMaxHp>,
    q_slow: Query<&BuffOnHitSlow>,
    q_stun: Query<&BuffOnHitStun>,
    q_damage: Query<&Damage>,
    q_target_health: Query<&Health>,
) {
    let attacker = trigger.event_target();
    let target = trigger.target;

    let Ok(buffs) = q_buffs.get(attacker) else {
        return;
    };

    // 找到计数器，没有计数器说明没有强化普攻
    let Some(counter_entity) = buffs.iter().find(|b| q_counter.get(*b).is_ok()) else {
        return;
    };
    let Ok(mut counter) = q_counter.get_mut(counter_entity) else {
        return;
    };

    // 额外伤害
    if let Some(bonus) = buffs.iter().find_map(|b| q_bonus_damage.get(b).ok()) {
        let base_dmg = q_damage.get(attacker).map(|d| d.0).unwrap_or(0.0);
        let extra = bonus.flat + base_dmg * bonus.ratio;
        if extra > 0.0 {
            commands.entity(target).trigger(|e| CommandDamageCreate {
                entity: e,
                source: attacker,
                damage_type: DamageType::Physical,
                amount: extra,
                tag: None,
            });
        }
    }

    // 基于目标最大生命的额外伤害
    if let Some(maxhp) = buffs.iter().find_map(|b| q_target_max_hp.get(b).ok()) {
        let target_max = q_target_health.get(target).map(|h| h.max).unwrap_or(0.0);
        let extra = target_max * maxhp.ratio;
        if extra > 0.0 {
            commands.entity(target).trigger(|e| CommandDamageCreate {
                entity: e,
                source: attacker,
                damage_type: DamageType::Physical,
                amount: extra,
                tag: None,
            });
        }
    }

    // 减速
    if let Some(slow) = buffs.iter().find_map(|b| q_slow.get(b).ok()) {
        commands
            .entity(target)
            .with_related::<BuffOf>(DebuffSlow::new(slow.percent, slow.duration));
    }

    // 眩晕
    if let Some(stun) = buffs.iter().find_map(|b| q_stun.get(b).ok()) {
        commands
            .entity(target)
            .with_related::<BuffOf>(DebuffStun::new(stun.duration));
    }

    // 消耗次数
    counter.remaining = counter.remaining.saturating_sub(1);
    debug!("on-hit: 消耗强化普攻次数，剩余 {}", counter.remaining);
}

/// FixedUpdate 中计时，次数耗尽或超时后清理所有 on-hit 组件
pub fn update_on_hit_buff_timers(
    mut commands: Commands,
    q_buffs: Query<&Buffs>,
    mut q_counters: Query<(Entity, &BuffOf, &mut BuffOnHitCounter)>,
    q_bonus_damage: Query<Entity, With<BuffOnHitBonusDamage>>,
    q_target_max_hp: Query<Entity, With<BuffOnHitTargetMaxHp>>,
    q_slow: Query<Entity, With<BuffOnHitSlow>>,
    q_stun: Query<Entity, With<BuffOnHitStun>>,
    time: Res<Time<Fixed>>,
) {
    for (counter_entity, buff_of, mut counter) in q_counters.iter_mut() {
        counter.timer.tick(time.delta());
        if counter.remaining > 0 && !counter.timer.is_finished() {
            continue;
        }

        // 计数器过期或次数耗尽，移除所有 on-hit 组件
        let parent = buff_of.0;
        let Ok(buffs) = q_buffs.get(parent) else {
            commands.entity(counter_entity).despawn();
            continue;
        };

        for buff_entity in buffs.iter() {
            if q_bonus_damage.get(buff_entity).is_ok()
                || q_target_max_hp.get(buff_entity).is_ok()
                || q_slow.get(buff_entity).is_ok()
                || q_stun.get(buff_entity).is_ok()
                || buff_entity == counter_entity
            {
                commands.entity(buff_entity).despawn();
            }
        }

        // 移除攻速加成（如果有）
        commands.entity(parent).remove::<BuffAttack>();
        debug!("on-hit: 强化普攻已过期，清理所有 on-hit 组件");
    }
}

#[derive(Default)]
pub struct PluginOnHit;

impl Plugin for PluginOnHit {
    fn build(&self, app: &mut App) {
        app.add_observer(on_event_attack_end_consume_on_hit);
        app.add_systems(FixedUpdate, update_on_hit_buff_timers);
    }
}
