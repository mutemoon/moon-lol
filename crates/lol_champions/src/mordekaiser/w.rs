//! Mordekaiser W - 不坏之身 (Indestructible)
//!
//! 被动：受到伤害的 7.5% 储存为护盾原料（上限 30% 最大生命）。
//! 主动：开盾（5% 最大生命基础 + 储存值，同样受 30% 上限约束），持续 5 秒；
//!       重施消耗剩余护盾，按 HealingPercent 治疗自身。
//! 护盾 1 秒后开始衰减，每秒衰减 0.5% 最大生命。
//!
//! 组合表达：
//! - 实际承伤吸收交给通用 [`BuffShieldWhite`]（被伤害管线自动消费，harness 可读），
//!   本技能仅 spawn 一个"白盾 + W 追踪器"子 buff。
//! - 重施窗口 = [`SkillRecastWindow`]：首段释放后挂在 W 技能上，使二段释放绕过冷却；
//!   二段释放时若仍存在 W 护盾子 buff，则按治疗比例回血并移除护盾与窗口。

use bevy::prelude::*;
use lol_base::animation_names::ANIM_SPELL2;
use lol_base::render_cmd::CommandAnimationPlay;
use lol_base::spell::Spell;
use lol_core::base::buff::{BuffOf, Buffs};
use lol_core::buffs::shield_white::BuffShieldWhite;
use lol_core::life::Health;
use lol_core::skill::{EventSkillCast, Skill, SkillRecastWindow, SkillSlot, get_skill_data_value};

use crate::mordekaiser::Mordekaiser;
use crate::mordekaiser::buffs::{
    BuffMordekaiserWShield, MORDE_W_BASE_SHIELD, MORDE_W_DECAY_PER_SECOND, MORDE_W_DURATION,
    MORDE_W_MAX_HEALTH_CAP, MORDE_W_TIME_BEFORE_DECAY, MordekaiserWStorage,
};

/// 在 morde 的 buff 子实体中查找当前 W 护盾，返回其实体。
fn find_w_shield(
    morde: Entity,
    q_buffs: &Query<&Buffs>,
    q_w_shield: &Query<&BuffMordekaiserWShield>,
) -> Option<Entity> {
    let buffs = q_buffs.get(morde).ok()?;
    for b in buffs.iter() {
        if q_w_shield.get(b).is_ok() {
            return Some(b);
        }
    }
    None
}

pub fn on_mordekaiser_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_morde: Query<(), With<Mordekaiser>>,
    q_skill: Query<&Skill>,
    res_spells: Res<Assets<Spell>>,
    q_health: Query<&Health>,
    q_buffs: Query<&Buffs>,
    q_w_shield: Query<&BuffMordekaiserWShield>,
    q_shield_white: Query<&BuffShieldWhite>,
    q_storage: Query<&MordekaiserWStorage>,
    q_recast: Query<&SkillRecastWindow>,
) {
    let entity = trigger.event_target();
    if q_morde.get(entity).is_err() {
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

    // 已有 W 护盾 -> 重施：消耗护盾治疗自身
    if let Some(shield_entity) = find_w_shield(entity, &q_buffs, &q_w_shield) {
        let remaining = q_shield_white
            .get(shield_entity)
            .map(|s| s.current)
            .unwrap_or(0.0);
        let heal_percent =
            get_skill_data_value(spell_obj, "HealingPercent", skill.level).unwrap_or(0.325);
        let heal = heal_percent * remaining;
        if heal > 0.0 {
            commands.entity(entity).queue(move |mut e: EntityWorldMut| {
                if let Some(mut health) = e.get_mut::<Health>() {
                    health.value = (health.value + heal).min(health.max);
                }
            });
        }
        commands.entity(shield_entity).despawn();
        commands.entity(trigger.skill_entity).remove::<SkillRecastWindow>();
        debug!(
            "莫德凯撒 W 重施：消耗剩余护盾 {:.1}，治疗 {:.1}",
            remaining, heal
        );
        return;
    }

    // 护盾已消失但重施窗口仍在 -> 护盾被打空，无法重施；移除窗口令冷却接管
    let recast_active = q_recast
        .get(trigger.skill_entity)
        .map(|w| !w.timer.is_finished())
        .unwrap_or(false);
    if recast_active {
        commands.entity(trigger.skill_entity).remove::<SkillRecastWindow>();
        debug!("莫德凯撒 W 护盾已耗尽，重施窗口关闭");
        return;
    }

    // 首段：开盾
    let max_hp = q_health.get(entity).map(|h| h.max).unwrap_or(0.0);
    let stored = q_storage.get(entity).map(|s| s.stored).unwrap_or(0.0);
    let cap = MORDE_W_MAX_HEALTH_CAP * max_hp;
    let base = MORDE_W_BASE_SHIELD * max_hp;
    let shield_amount = (base + stored).min(cap).max(0.0);

    let shield_entity = commands
        .spawn((
            BuffShieldWhite::new(shield_amount),
            BuffMordekaiserWShield {
                elapsed: 0.0,
                max_health: max_hp,
            },
        ))
        .id();
    commands
        .entity(entity)
        .add_related::<BuffOf>(&[shield_entity]);

    // 清零储存
    commands
        .entity(entity)
        .insert(MordekaiserWStorage { stored: 0.0 });
    // 挂重施窗口
    commands
        .entity(trigger.skill_entity)
        .insert(SkillRecastWindow::new(1, 2, MORDE_W_DURATION));

    debug!(
        "莫德凯撒 W 不坏之身：开盾 {:.1}（基础 {:.1} + 储存 {:.1}，上限 {:.1}），重施窗口 {}s",
        shield_amount, base, stored, cap, MORDE_W_DURATION
    );
}

/// W 护盾：1 秒后每秒衰减 0.5% 最大生命，5 秒到期移除。
pub fn update_mordekaiser_w_shield(
    mut commands: Commands,
    time: Res<Time>,
    mut q_shield: Query<(Entity, &mut BuffShieldWhite, &mut BuffMordekaiserWShield)>,
) {
    let dt = time.delta().as_secs_f32();
    for (entity, mut shield, mut tracker) in q_shield.iter_mut() {
        tracker.elapsed += dt;
        if tracker.elapsed > MORDE_W_TIME_BEFORE_DECAY {
            let decay = MORDE_W_DECAY_PER_SECOND * tracker.max_health * dt;
            shield.current = (shield.current - decay).max(0.0);
        }
        if tracker.elapsed >= MORDE_W_DURATION {
            commands.entity(entity).despawn();
        }
    }
}