//! 被动 - 艾欧尼亚热诚 (Ionian Fervor)
//!
//! 施放技能叠加层数（最多 4 层，6 秒），每层提供 8% 攻速；
//! 满层时普攻命中附带额外魔法伤害（20% AD）。
//!
//! 层数由 `EventSkillCast` 驱动（与锐雯被动一致），刷新持续时间。
//! 攻速由通用 `BuffAttack` 承载，满层额外伤害由 `EventAttackEnd` 观察者直接结算。

use bevy::prelude::*;
use bevy::time::{Timer, TimerMode};
use lol_core::attack::{BuffAttack, EventAttackEnd};
use lol_core::base::buff::{Buff, BuffOf, Buffs};
use lol_core::damage::{CommandDamageCreate, Damage, DamageType};
use lol_core::skill::EventSkillCast;

use crate::irelia::Irelia;

/// 被动持续时间（秒）：每次施法刷新
pub const IRELIA_FERVOR_DURATION: f32 = 6.0;
/// 被动最大层数
pub const IRELIA_FERVOR_MAX_STACKS: u8 = 4;
/// 每层攻速加成（8%）
pub const IRELIA_FERVOR_AS_PER_STACK: f32 = 0.08;
/// 满层普攻额外伤害倍率（20% AD），对应 ron `on_hit_bonus` 计算
pub const IRELIA_FERVOR_ON_HIT_RATIO: f32 = 0.2;

fn fresh_timer() -> Timer {
    Timer::from_seconds(IRELIA_FERVOR_DURATION, TimerMode::Once)
}

/// 艾欧尼亚热诚层数 buff（挂在 Irelia 自身）
#[derive(Component, Clone, Debug)]
#[require(Buff = Buff { name: "IreliaFervor" })]
pub struct BuffIreliaFervor {
    pub charges: u8,
    pub timer: Timer,
}

impl Default for BuffIreliaFervor {
    fn default() -> Self {
        Self {
            charges: 0,
            timer: fresh_timer(),
        }
    }
}

/// 施放任意技能时叠加一层被动（最多 4 层）并刷新持续时间，同步更新攻速。
pub fn on_irelia_skill_cast_stack_passive(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_irelia: Query<&Buffs, With<Irelia>>,
    mut q_fervor: Query<&mut BuffIreliaFervor>,
) {
    let caster = trigger.event().entity;

    let mut new_charges: Option<u8> = None;

    // 先尝试在已有 buff 中找被动并叠加（首次施法时角色可能尚无 Buffs 组件）
    if let Ok(buffs) = q_irelia.get(caster) {
        for buff_entity in buffs.iter() {
            if let Ok(mut fervor) = q_fervor.get_mut(buff_entity) {
                fervor.charges = (fervor.charges + 1).min(IRELIA_FERVOR_MAX_STACKS);
                fervor.timer = fresh_timer();
                new_charges = Some(fervor.charges);
                break;
            }
        }
    }

    // 无被动 buff：新建 1 层
    let charges = match new_charges {
        Some(c) => c,
        None => {
            commands
                .entity(caster)
                .with_related::<BuffOf>(BuffIreliaFervor {
                    charges: 1,
                    timer: fresh_timer(),
                });
            1
        }
    };

    // 同步攻速：每层 8%
    commands.entity(caster).insert(BuffAttack {
        bonus_attack_speed: charges as f32 * IRELIA_FERVOR_AS_PER_STACK,
    });
}

/// 满层（4 层）时，普攻命中附带 20% AD 额外魔法伤害。
/// 直接触发 EventAttackEnd 仅触发 on-hit，不含基础普攻伤害。
pub fn on_irelia_passive_attack_end(
    trigger: On<EventAttackEnd>,
    mut commands: Commands,
    q_irelia: Query<&Damage, With<Irelia>>,
    q_buffs: Query<&Buffs>,
    q_fervor: Query<&BuffIreliaFervor>,
) {
    let source = trigger.entity;
    let Ok(ad) = q_irelia.get(source) else {
        return;
    };
    let Ok(buffs) = q_buffs.get(source) else {
        return;
    };

    let maxed = buffs
        .iter()
        .find_map(|b| q_fervor.get(b).ok())
        .map(|f| f.charges >= IRELIA_FERVOR_MAX_STACKS)
        .unwrap_or(false);
    if !maxed {
        return;
    }

    let bonus = ad.0 * IRELIA_FERVOR_ON_HIT_RATIO;
    commands.trigger(CommandDamageCreate {
        entity: trigger.target,
        source,
        damage_type: DamageType::Magic,
        amount: bonus,
        tag: None,
    });
}

/// FixedUpdate：tick 被动计时器，到期后移除攻速 buff 与层数 buff。
pub fn update_irelia_fervor(
    mut commands: Commands,
    time: Res<Time<Fixed>>,
    mut q_fervor: Query<(Entity, &mut BuffIreliaFervor, &BuffOf)>,
) {
    let mut expired = Vec::new();
    for (entity, mut fervor, _) in q_fervor.iter_mut() {
        fervor.timer.tick(time.delta());
        if fervor.timer.is_finished() {
            expired.push(entity);
        }
    }
    for entity in expired {
        // 到期：回收攻速并销毁层数 buff
        if let Ok((_, _, bo)) = q_fervor.get(entity) {
            commands.entity(bo.0).remove::<BuffAttack>();
        }
        commands.entity(entity).despawn();
    }
}
