//! W - 防御之舞 (Defiant Dance)
//!
//! 两段施法：
//! - W1：进入防御姿态，减免 50% 伤害，开始蓄力（最多 1.5 秒）。
//! - W2：释放双刃，依蓄力时长在最小/最大伤害间线性插值，造成物理伤害。
//!
//! 减伤由通用 `BuffDamageReduction` 承载（无自带计时器），蓄力与回收由
//! `BuffIreliaW` 计时器统一管理：W2 主动释放或蓄力到期都会清除减伤。

use bevy::prelude::*;
use bevy::time::{Timer, TimerMode};
use lol_base::animation_names::ANIM_SPELL2;
use lol_base::render_cmd::CommandAnimationPlay;
use lol_base::spell::Spell;
use lol_core::base::buff::{Buff, BuffOf, Buffs};
use lol_core::buffs::damage_reduction::BuffDamageReduction;
use lol_core::damage::{CommandDamageCreate, DamageType};
use lol_core::entities::champion::Champion;
use lol_core::skill::{CoolDown, SkillRecastWindow, get_skill_value};
use lol_core::team::Team;

/// 减伤比例（50%，物理/魔法通用）
pub const IRELIA_W_DR: f32 = 0.5;
/// 蓄力最大时长（秒）= 重施窗口时长
pub const IRELIA_W_MAX_DURATION: f32 = 1.5;
/// 达到最大伤害所需蓄力时长（秒）
pub const IRELIA_W_CHARGE_FOR_MAX: f32 = 0.75;
/// 释放伤害半径（ron `castRadius`）
pub const IRELIA_W_RADIUS: f32 = 300.0;

/// W 蓄力计时 buff（挂在 Irelia 自身）：`elapsed` 即蓄力时长，到期回收减伤。
#[derive(Component, Clone, Debug)]
#[require(Buff = Buff { name: "IreliaW" })]
pub struct BuffIreliaW {
    pub timer: Timer,
}

impl BuffIreliaW {
    pub fn new() -> Self {
        Self {
            timer: Timer::from_seconds(IRELIA_W_MAX_DURATION, TimerMode::Once),
        }
    }
}

pub fn cast_irelia_w(
    commands: &mut Commands,
    entity: Entity,
    skill_entity: Entity,
    stage: u8,
    point: Vec2,
    spell: &Spell,
    skill_level: usize,
    cooldown: &CoolDown,
    q_enemies: &Query<(Entity, &Transform, &Team), With<Champion>>,
    team: Team,
    q_buffs: &Query<&Buffs>,
    q_w: &Query<&BuffIreliaW>,
    q_dr: &Query<&BuffDamageReduction>,
    ad: f32,
) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL2.to_string(),
        repeat: false,
        duration: None,
    });

    if stage == 1 {
        // W1：减伤 + 蓄力计时 + 重施窗口
        commands
            .entity(entity)
            .with_related::<BuffOf>(BuffDamageReduction::new(IRELIA_W_DR, None));
        commands
            .entity(entity)
            .with_related::<BuffOf>(BuffIreliaW::new());
        commands
            .entity(skill_entity)
            .insert(SkillRecastWindow::new(2, 2, IRELIA_W_MAX_DURATION));
        return;
    }

    // W2：依蓄力时长结算伤害
    let charge = q_buffs
        .get(entity)
        .ok()
        .and_then(|buffs| buffs.iter().find_map(|b| q_w.get(b).ok()))
        .map(|w| w.timer.elapsed().as_secs_f32())
        .unwrap_or(0.0);
    let frac = (charge / IRELIA_W_CHARGE_FOR_MAX).clamp(0.0, 1.0);

    let stat_getter = |stat: u8| if stat == 2 { ad } else { 0.0 };
    let min = get_skill_value(spell, "min_damage_calc", skill_level, stat_getter).unwrap_or(0.0);
    let max = get_skill_value(spell, "max_damage_calc", skill_level, stat_getter).unwrap_or(0.0);
    let amount = min + frac * (max - min);

    // 对释放点周围敌方英雄造成物理伤害
    for (target, tf, t) in q_enemies.iter() {
        if *t == team {
            continue;
        }
        if tf.translation.xz().distance(point) > IRELIA_W_RADIUS {
            continue;
        }
        commands.entity(target).trigger(|e| CommandDamageCreate {
            entity: e,
            source: entity,
            damage_type: DamageType::Physical,
            amount,
            tag: None,
        });
    }

    // 清除减伤 + 蓄力 buff，关闭重施窗口，进入冷却
    clear_irelia_w_buffs(commands, entity, q_buffs, q_w, q_dr);
    commands.entity(skill_entity).remove::<SkillRecastWindow>();
    commands.entity(skill_entity).insert(CoolDown {
        duration: cooldown.duration,
        timer: Some(Timer::from_seconds(cooldown.duration, TimerMode::Once)),
    });
}

/// 移除 Irelia 身上的 W 减伤与蓄力 buff（W2 释放与蓄力到期共用）。
fn clear_irelia_w_buffs(
    commands: &mut Commands,
    entity: Entity,
    q_buffs: &Query<&Buffs>,
    q_w: &Query<&BuffIreliaW>,
    q_dr: &Query<&BuffDamageReduction>,
) {
    let Some(buffs) = q_buffs.get(entity).ok() else {
        return;
    };
    let mut to_despawn = Vec::new();
    for buff in buffs.iter() {
        if q_w.get(buff).is_ok() || q_dr.get(buff).is_ok() {
            to_despawn.push(buff);
        }
    }
    for buff in to_despawn {
        commands.entity(buff).despawn();
    }
}

/// FixedUpdate：tick 蓄力计时，到期回收减伤并销毁蓄力 buff。
pub fn update_irelia_w(
    mut commands: Commands,
    time: Res<Time<Fixed>>,
    mut q_w: Query<(Entity, &mut BuffIreliaW, &BuffOf)>,
    q_buffs: Query<&Buffs>,
    q_dr: Query<&BuffDamageReduction>,
) {
    let mut expired: Vec<(Entity, Entity)> = Vec::new();
    for (entity, mut w, bo) in q_w.iter_mut() {
        w.timer.tick(time.delta());
        if w.timer.is_finished() {
            expired.push((entity, bo.0));
        }
    }
    for (w_entity, parent) in expired {
        // 销毁蓄力 buff
        commands.entity(w_entity).despawn();
        // 同步销毁父实体上的 W 减伤（减伤无自带计时器）
        if let Ok(buffs) = q_buffs.get(parent) {
            for b in buffs.iter() {
                if q_dr.get(b).is_ok() {
                    commands.entity(b).despawn();
                }
            }
        }
    }
}
