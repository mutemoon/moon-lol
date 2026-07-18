use bevy::prelude::*;
use bevy::time::{Timer, TimerMode};
use lol_core::damage::{AbilityPower, CommandDamageCreate, DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::life::Health;
use lol_core::movement::Movement;
use lol_core::team::Team;

use crate::mordekaiser::Mordekaiser;
use crate::mordekaiser::buffs::{
    MORDE_W_DAMAGE_TAKEN_CONVERSION, MORDE_W_MAX_HEALTH_CAP, MordekaiserWStorage,
};

/// 被动 - 黑暗起兮 (Darkness Rise)
///
/// 普攻 / Q 命中 / E 命中各获得 1 层 Darkness，满 3 层激活 DoT 光环（半径 375），
/// 持续对附近敌人造成魔法伤害（每层 30% AP），并提供 3% 移速与普攻 40% AP 附伤。
/// 脱战 4 秒后失效。

/// 被动最大层数（ron `MaximumStacks`）。
pub const MORDE_PASSIVE_MAX_STACKS: u8 = 3;
/// 脱战失效时长（秒，ron `CombatTrackingDuration`）。
pub const MORDE_PASSIVE_COMBAT_DURATION: f32 = 4.0;
/// DoT 光环半径（ron `DoTRadius`）。
pub const MORDE_PASSIVE_DOT_RADIUS: f32 = 375.0;
/// DoT 周期（秒）。
pub const MORDE_PASSIVE_DOT_TICK: f32 = 0.5;
/// 每层每周期 AP 加成（ron `aura_damage_per_stack` = 0.3 * AP，stat 0）。
pub const MORDE_PASSIVE_DOT_AP_RATIO: f32 = 0.3;
/// 激活期间移速加成（ron `MovementSpeed` = 0.03）。
pub const MORDE_PASSIVE_MS_BONUS: f32 = 0.03;
/// 激活期间普攻附带 AP（ron `PercentAPAddedToAutos` = 0.4）。
pub const MORDE_PASSIVE_AUTO_AP_RATIO: f32 = 0.4;

/// 被动 DoT 伤害标签：区分光环 DoT，避免 DoT 再次叠层 / 触发普攻附伤。
pub const MORDE_PASSIVE_DOT_TAG: u32 = 1;
/// 被动普攻附伤标签：区分附伤额外实例，避免附伤再次叠层 / 触发自身。
pub const MORDE_PASSIVE_AUTO_TAG: u32 = 2;

/// 莫德凯撒被动 Darkness 状态（挂在自身）。
///
/// `combat_timer` 每次命中重置，到期（脱战 4 秒）则清空层数并失效；
/// `dot_timer` 仅在 `active` 时推进，每周期对半径内敌人结算 DoT；
/// `ms_bonus` 记录已叠加到 [`lol_core::movement::Movement`] 上的移速数值，失效时据此还原。
#[derive(Component, Debug, Clone)]
pub struct MordekaiserDarkness {
    /// 当前层数，0..=3
    pub stacks: u8,
    /// 是否已满层激活
    pub active: bool,
    /// 脱战计时
    pub combat_timer: Timer,
    /// DoT 周期计时
    pub dot_timer: Timer,
    /// 已施加的移速加成（用于失效时还原）
    pub ms_bonus: f32,
}

impl MordekaiserDarkness {
    pub const MAX: u8 = MORDE_PASSIVE_MAX_STACKS;

    /// 首次命中：1 层，未激活。
    pub fn new() -> Self {
        Self {
            stacks: 1,
            active: false,
            combat_timer: Timer::from_seconds(MORDE_PASSIVE_COMBAT_DURATION, TimerMode::Once),
            dot_timer: Timer::from_seconds(MORDE_PASSIVE_DOT_TICK, TimerMode::Repeating),
            ms_bonus: 0.0,
        }
    }
}

impl Default for MordekaiserDarkness {
    fn default() -> Self {
        Self {
            stacks: 0,
            active: false,
            combat_timer: Timer::from_seconds(MORDE_PASSIVE_COMBAT_DURATION, TimerMode::Once),
            dot_timer: Timer::from_seconds(MORDE_PASSIVE_DOT_TICK, TimerMode::Repeating),
            ms_bonus: 0.0,
        }
    }
}

/// 监听莫德凯撒造成的伤害：叠加被动 Darkness，满层激活后普攻附带 40% AP 魔法伤害。
pub fn on_mordekaiser_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_morde: Query<(), With<Mordekaiser>>,
    mut q_darkness: Query<&mut MordekaiserDarkness>,
    q_ap: Query<&AbilityPower>,
    mut q_movement: Query<&mut Movement>,
) {
    let morde = trigger.source;
    if q_morde.get(morde).is_err() {
        return;
    }
    // DoT 与普攻附伤不再叠层，避免递归
    let tag = trigger.event().tag;
    if tag == Some(MORDE_PASSIVE_DOT_TAG) || tag == Some(MORDE_PASSIVE_AUTO_TAG) {
        return;
    }

    let now_active = match q_darkness.get_mut(morde) {
        Ok(mut d) => {
            d.stacks = (d.stacks + 1).min(MORDE_PASSIVE_MAX_STACKS);
            d.combat_timer.reset();
            if d.stacks >= MORDE_PASSIVE_MAX_STACKS && !d.active {
                d.active = true;
                d.dot_timer.reset();
                // 激活：+3% 移速（基于当前移速）
                if let Ok(mut mv) = q_movement.get_mut(morde) {
                    let bonus = mv.speed * MORDE_PASSIVE_MS_BONUS;
                    d.ms_bonus = bonus;
                    mv.speed += bonus;
                }
            }
            d.active
        }
        Err(_) => {
            // 首次命中：插入 1 层
            commands.entity(morde).insert(MordekaiserDarkness::new());
            false
        }
    };

    // 激活期间普攻（物理）附带 40% AP 魔法伤害
    if now_active && trigger.event().damage_type == DamageType::Physical {
        let ap = q_ap.get(morde).map(|a| a.0).unwrap_or(0.0);
        let bonus = MORDE_PASSIVE_AUTO_AP_RATIO * ap;
        if bonus > 0.0 {
            let target = trigger.event_target();
            commands.entity(target).trigger(|e| CommandDamageCreate {
                entity: e,
                source: morde,
                damage_type: DamageType::Magic,
                amount: bonus,
                tag: Some(MORDE_PASSIVE_AUTO_TAG),
            });
        }
    }
}

/// 监听莫德凯撒受到的伤害：按 7.5% 储存为 W 护盾原料（上限 30% 最大生命）。
pub fn on_mordekaiser_damage_taken(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_morde: Query<(), With<Mordekaiser>>,
    q_storage: Query<&MordekaiserWStorage>,
    q_health: Query<&Health>,
) {
    let morde = trigger.event_target();
    if q_morde.get(morde).is_err() {
        return;
    }
    let final_dmg = trigger.event().damage_result.final_damage;
    if final_dmg <= 0.0 {
        return;
    }
    let max_hp = q_health.get(morde).map(|h| h.max).unwrap_or(0.0);
    let cap = MORDE_W_MAX_HEALTH_CAP * max_hp;
    let gained = final_dmg * MORDE_W_DAMAGE_TAKEN_CONVERSION;
    let new_stored = match q_storage.get(morde) {
        Ok(s) => (s.stored + gained).min(cap),
        Err(_) => gained.min(cap),
    };
    commands
        .entity(morde)
        .insert(MordekaiserWStorage { stored: new_stored });
}

/// 被动 Darkness：脱战 4 秒失效；激活期间每 0.5 秒对半径内敌人造成 30% AP×层数 魔法伤害。
pub fn update_mordekaiser_passive(
    mut commands: Commands,
    time: Res<Time>,
    mut q_morde: Query<(Entity, &mut MordekaiserDarkness, &Transform, &Team), With<Mordekaiser>>,
    q_ap: Query<&AbilityPower>,
    mut q_movement: Query<&mut Movement>,
    q_enemies: Query<(Entity, &Transform, &Team), With<Champion>>,
) {
    for (morde, mut darkness, transform, team) in q_morde.iter_mut() {
        darkness.combat_timer.tick(time.delta());
        if darkness.combat_timer.just_finished() {
            // 脱战：失效并还原移速
            if darkness.active {
                if let Ok(mut mv) = q_movement.get_mut(morde) {
                    mv.speed -= darkness.ms_bonus;
                }
            }
            darkness.stacks = 0;
            darkness.active = false;
            darkness.ms_bonus = 0.0;
            continue;
        }

        if darkness.active {
            darkness.dot_timer.tick(time.delta());
            if darkness.dot_timer.just_finished() {
                let ap = q_ap.get(morde).map(|a| a.0).unwrap_or(0.0);
                let amount = MORDE_PASSIVE_DOT_AP_RATIO * ap * darkness.stacks as f32;
                if amount > 0.0 {
                    let pos = transform.translation;
                    for (enemy, enemy_tf, enemy_team) in q_enemies.iter() {
                        if enemy_team == team {
                            continue;
                        }
                        if enemy_tf.translation.distance(pos) > MORDE_PASSIVE_DOT_RADIUS {
                            continue;
                        }
                        commands.entity(enemy).trigger(|e| CommandDamageCreate {
                            entity: e,
                            source: morde,
                            damage_type: DamageType::Magic,
                            amount,
                            tag: Some(MORDE_PASSIVE_DOT_TAG),
                        });
                    }
                }
            }
        }
    }
}
