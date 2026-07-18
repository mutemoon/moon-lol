//! Volibear 被动 - 风暴之力 (The Stormbringer)
//!
//! 普攻叠加层数（上限 5），每层 +5% 攻速；满层触发连锁闪电；脱战 6s 清零。

use bevy::prelude::*;
use lol_core::attack::{BuffAttack, EventAttackEnd};
use lol_core::base::buff::{BuffOf, Buffs};
use lol_core::buffs::cc_debuffs::DebuffSlow;
use lol_core::damage::{AbilityPower, CommandDamageCreate, Damage, DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::team::Team;

use crate::volibear::Volibear;
use crate::volibear::buffs::VolibearPStacks;

/// 被动每层攻速加成
pub const VOLIBEAR_P_ATTACK_SPEED_PER_STACK: f32 = 0.05;
/// 被动连锁闪电 AP 加成
pub const VOLIBEAR_P_AP_RATIO: f32 = 0.45;
/// 被动连锁闪电 AD 加成
pub const VOLIBEAR_P_AD_RATIO: f32 = 0.2;
/// 被动连锁闪电范围
pub const VOLIBEAR_P_CHAIN_RADIUS: f32 = 300.0;
/// E 减速比例（on_volibear_damage_hit 施加）
pub const VOLIBEAR_E_SLOW_PERCENT: f32 = 0.4;
/// E 减速持续时长（秒）
pub const VOLIBEAR_E_SLOW_DURATION: f32 = 2.0;

/// 监听 Volibear 造成的伤害：仅 None（E 延迟伤害 / Q 强化普攻）施加减速。
pub fn on_volibear_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_volibear: Query<(), With<Volibear>>,
) {
    let source = trigger.source;
    if q_volibear.get(source).is_err() {
        return;
    }
    let target = trigger.event_target();
    if trigger.event().tag.is_none() {
        commands
            .entity(target)
            .with_related::<BuffOf>(DebuffSlow::new(
                VOLIBEAR_E_SLOW_PERCENT,
                VOLIBEAR_E_SLOW_DURATION,
            ));
    }
}

/// 被动：普攻命中叠加层数（上限 5），每层 +5% 攻速；满层触发连锁闪电。
pub fn on_volibear_attack_end(
    trigger: On<EventAttackEnd>,
    mut commands: Commands,
    mut q_volibear: Query<
        (
            &mut VolibearPStacks,
            &Transform,
            &Team,
            &Damage,
            Option<&Buffs>,
        ),
        With<Volibear>,
    >,
    q_buff_attack: Query<&BuffAttack>,
    q_ap: Query<&AbilityPower>,
    q_enemies: Query<(Entity, &Transform, &Team), With<Champion>>,
) {
    let attacker = trigger.event_target();
    let Ok((mut stacks, transform, team, damage, buffs)) = q_volibear.get_mut(attacker) else {
        return;
    };

    let new_count = (stacks.count + 1).min(VolibearPStacks::MAX);
    stacks.count = new_count;
    stacks.timer.reset();

    // 移除旧的 BuffAttack（如果有），上新值
    if let Some(buffs) = buffs {
        for buff in buffs.iter() {
            if q_buff_attack.get(buff).is_ok() {
                commands.entity(buff).despawn();
            }
        }
    }
    if new_count > 0 {
        commands
            .entity(attacker)
            .with_related::<BuffOf>(BuffAttack {
                bonus_attack_speed: new_count as f32 * VOLIBEAR_P_ATTACK_SPEED_PER_STACK,
            });
    }

    // 满层触发连锁闪电（附近敌人受魔法伤害）
    if new_count >= VolibearPStacks::MAX {
        let ad = damage.0;
        let ap = q_ap.get(attacker).map(|a| a.0).unwrap_or(0.0);
        let chain_dmg = VOLIBEAR_P_AP_RATIO * ap + VOLIBEAR_P_AD_RATIO * ad;
        if chain_dmg > 0.0 {
            let pos = transform.translation.xz();
            for (enemy, enemy_transform, enemy_team) in q_enemies.iter() {
                if enemy_team == team {
                    continue;
                }
                let dist = enemy_transform.translation.xz().distance(pos);
                if dist <= VOLIBEAR_P_CHAIN_RADIUS {
                    commands.entity(enemy).trigger(|e| CommandDamageCreate {
                        entity: e,
                        source: attacker,
                        damage_type: DamageType::Magic,
                        amount: chain_dmg,
                        tag: None,
                    });
                }
            }
        }
    }
}

/// 被动脱战：6s 计时器到期后清零层数并移除 BuffAttack。
pub fn update_volibear_p(
    time: Res<Time>,
    mut commands: Commands,
    mut q_volibear: Query<(&mut VolibearPStacks, Option<&Buffs>), With<Volibear>>,
    q_buff_attack: Query<&BuffAttack>,
) {
    for (mut stacks, buffs) in q_volibear.iter_mut() {
        stacks.timer.tick(time.delta());
        if stacks.timer.just_finished() {
            stacks.count = 0;
            if let Some(buffs) = buffs {
                for buff in buffs.iter() {
                    if q_buff_attack.get(buff).is_ok() {
                        commands.entity(buff).despawn();
                    }
                }
            }
        }
    }
}
