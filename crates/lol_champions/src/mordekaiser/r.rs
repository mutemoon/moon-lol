//! Mordekaiser R - 死亡领域 (Realm of Death)
//!
//! 诅咒最近的敌方英雄（castRange 650），将其与自身一同放逐至 1v1 死亡领域 7 秒。
//! 目标在领域内死亡则窃取其 10% 属性（AD/AP/生命/护甲）并永久持有。
//!
//! 组合表达：
//! - 领域标记 = [`MordekaiserRealm`] 组件挂在自身，由 `update_mordekaiser_realm` 计时到期移除。
//! - 属性窃取 = 目标生命归零时读取其属性，按 10% 加到自身对应组件，并记入
//!   [`MordekaiserStatSteal`] 便于查询。
//!
//! 鬼魂生成（ron `GhostAPRatio` 0.6）与领域空间隔离待后续实现。

use bevy::prelude::*;
use lol_base::animation_names::ANIM_SPELL4;
use lol_base::render_cmd::CommandAnimationPlay;
use lol_base::spell::Spell;
use lol_core::damage::{AbilityPower, Armor, Damage};
use lol_core::entities::champion::Champion;
use lol_core::life::Health;
use lol_core::skill::get_skill_data_value;
use lol_core::team::Team;

use crate::mordekaiser::Mordekaiser;
use crate::mordekaiser::buffs::{
    MORDE_R_DURATION, MORDE_R_STAT_STEAL_RATIO, MordekaiserRealm, MordekaiserStatSteal,
};

/// 施放 Mordekaiser R：诅咒 [point] 附近最近的敌方英雄，开启死亡领域。
pub fn cast_mordekaiser_r(
    commands: &mut Commands,
    entity: Entity,
    _point: Vec2,
    skill_level: usize,
    spell_obj: &Spell,
    q_transform: &Query<&Transform>,
    q_team: &Query<&Team>,
    q_enemies: &Query<(Entity, &Transform), With<Champion>>,
    q_realm: &Query<&MordekaiserRealm>,
) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL4.to_string(),
        repeat: false,
        duration: None,
    });

    // 已在领域内不重复施放
    if q_realm.get(entity).is_ok() {
        return;
    }

    let Ok(transform) = q_transform.get(entity) else {
        return;
    };
    let Ok(team) = q_team.get(entity) else {
        return;
    };
    let pos = transform.translation.xz();
    let cast_range = get_skill_data_value(spell_obj, "castRange", skill_level).unwrap_or(650.0);
    let duration = get_skill_data_value(spell_obj, "SpiritRealmDuration", skill_level)
        .unwrap_or(MORDE_R_DURATION);

    // 范围内最近敌方英雄
    let mut nearest: Option<(Entity, f32)> = None;
    for (enemy, enemy_tf) in q_enemies.iter() {
        if enemy == entity {
            continue;
        }
        let Ok(enemy_team) = q_team.get(enemy) else {
            continue;
        };
        if enemy_team == team {
            continue;
        }
        let d = enemy_tf.translation.xz().distance(pos);
        if d > cast_range {
            continue;
        }
        if nearest.map(|(_, nd)| d < nd).unwrap_or(true) {
            nearest = Some((enemy, d));
        }
    }

    let Some((target, _)) = nearest else {
        debug!("莫德凯撒 R：范围内无敌方英雄，未开启领域");
        return;
    };

    commands.entity(entity).insert(MordekaiserRealm {
        duration,
        elapsed: 0.0,
        target,
        stolen: false,
    });

    debug!(
        "莫德凯撒 R 死亡领域：放逐目标 {:?}，持续 {}s",
        target, duration
    );
}

/// 读取目标属性用于窃取（不含自身 Mordekaiser，避免与自身可变查询冲突）。
type TargetStats<'w> = (
    Option<&'w Damage>,
    Option<&'w AbilityPower>,
    &'w Health,
    Option<&'w Armor>,
);

/// R 领域计时 + 击杀窃取：目标生命归零时窃取 10% 属性，到期移除领域。
pub fn update_mordekaiser_realm(
    mut commands: Commands,
    time: Res<Time>,
    mut q_morde: Query<(Entity, &mut MordekaiserRealm), With<Mordekaiser>>,
    q_target: Query<TargetStats, Without<Mordekaiser>>,
    mut q_morde_dmg: Query<&mut Damage, With<Mordekaiser>>,
    mut q_morde_hp: Query<&mut Health, With<Mordekaiser>>,
    mut q_morde_armor: Query<&mut Armor, With<Mordekaiser>>,
    mut q_morde_ap: Query<&mut AbilityPower, With<Mordekaiser>>,
    mut q_steal: Query<&mut MordekaiserStatSteal>,
) {
    for (morde, mut realm) in q_morde.iter_mut() {
        // 击杀窃取
        if !realm.stolen {
            if let Ok((t_dmg, t_ap, t_hp, t_armor)) = q_target.get(realm.target) {
                if t_hp.value <= 0.0 {
                    let steal_ad = t_dmg.map(|d| d.0).unwrap_or(0.0) * MORDE_R_STAT_STEAL_RATIO;
                    let steal_ap = t_ap.map(|a| a.0).unwrap_or(0.0) * MORDE_R_STAT_STEAL_RATIO;
                    let steal_hp = t_hp.max * MORDE_R_STAT_STEAL_RATIO;
                    let steal_armor =
                        t_armor.map(|a| a.0).unwrap_or(0.0) * MORDE_R_STAT_STEAL_RATIO;

                    if let Ok(mut m_dmg) = q_morde_dmg.get_mut(morde) {
                        m_dmg.0 += steal_ad;
                    }
                    if let Ok(mut m_hp) = q_morde_hp.get_mut(morde) {
                        m_hp.max += steal_hp;
                        m_hp.value += steal_hp;
                    }
                    if let Ok(mut m_armor) = q_morde_armor.get_mut(morde) {
                        m_armor.0 += steal_armor;
                    }
                    match q_morde_ap.get_mut(morde) {
                        Ok(mut ap) => ap.0 += steal_ap,
                        Err(_) => {
                            if steal_ap > 0.0 {
                                commands.entity(morde).insert(AbilityPower(steal_ap));
                            }
                        }
                    }

                    match q_steal.get_mut(morde) {
                        Ok(mut s) => {
                            s.ad += steal_ad;
                            s.ap += steal_ap;
                            s.health += steal_hp;
                            s.armor += steal_armor;
                        }
                        Err(_) => {
                            commands.entity(morde).insert(MordekaiserStatSteal {
                                ad: steal_ad,
                                ap: steal_ap,
                                health: steal_hp,
                                armor: steal_armor,
                            });
                        }
                    }
                    realm.stolen = true;
                    debug!(
                        "莫德凯撒 R 窃取：AD {:.1} AP {:.1} HP {:.1} 护甲 {:.1}",
                        steal_ad, steal_ap, steal_hp, steal_armor
                    );
                }
            }
        }

        // 计时到期
        realm.elapsed += time.delta().as_secs_f32();
        if realm.elapsed >= realm.duration {
            commands.entity(morde).remove::<MordekaiserRealm>();
            debug!("莫德凯撒 R 死亡领域结束");
        }
    }
}
