use bevy::prelude::*;
use bevy::time::{Timer, TimerMode};
use lol_base::render_cmd::{CommandSkinParticleDespawn, CommandSkinParticleSpawn};
use lol_core::attack::EventAttackEnd;
use lol_core::base::buff::{Buff, BuffOf, Buffs};
use lol_core::base::level::Level;
use lol_core::damage::{CommandDamageCreate, Damage, DamageType};
use lol_core::skill::EventSkillCast;

use crate::riven::Riven;

/// 被动最多叠加层数
const RIVEN_PASSIVE_MAX_CHARGES: u8 = 3;
/// 被动持续时间（秒）：每次施法刷新
const RIVEN_PASSIVE_DURATION: f32 = 6.0;
/// 被动额外伤害倍率：1级 30%
const RIVEN_PASSIVE_RATIO_MIN: f32 = 0.30;
/// 被动额外伤害倍率：18级 46.76%
const RIVEN_PASSIVE_RATIO_MAX: f32 = 0.4676;

/// 锐雯被动：施放技能叠加层数（最多3层，6秒），普攻命中消耗1层造成额外伤害。
#[derive(Component, Clone, Debug)]
#[require(Buff = Buff { name: "RivenPassive" })]
pub struct BuffRivenPassive {
    pub charges: u8,
    pub timer: Timer,
}

impl Default for BuffRivenPassive {
    fn default() -> Self {
        Self {
            charges: 0,
            timer: Timer::from_seconds(RIVEN_PASSIVE_DURATION, TimerMode::Once),
        }
    }
}

fn fresh_timer() -> Timer {
    Timer::from_seconds(RIVEN_PASSIVE_DURATION, TimerMode::Once)
}

/// 被动充能光效：身体 + 武器两处粒子成对挂撤
const RIVEN_PASSIVE_PARTICLES: [&str; 2] = ["Riven_P_Buff", "Riven_P_Buff_Wpn"];

fn spawn_passive_particles(commands: &mut Commands, entity: Entity) {
    for key in RIVEN_PASSIVE_PARTICLES {
        commands.trigger(CommandSkinParticleSpawn {
            entity,
            hash: key.to_string(),
            rotation: None,
            resolver_entity: None,
        });
    }
}

fn despawn_passive_particles(commands: &mut Commands, entity: Entity) {
    for key in RIVEN_PASSIVE_PARTICLES {
        commands.trigger(CommandSkinParticleDespawn {
            entity,
            hash: key.to_string(),
            resolver_entity: None,
        });
    }
}

/// 按等级计算被动额外伤害倍率（1级 30% -> 18级 46.76%，线性插值）
pub(crate) fn passive_ratio_for_level(level: u32) -> f32 {
    let t = level.saturating_sub(1) as f32 / 17.0;
    RIVEN_PASSIVE_RATIO_MIN + t * (RIVEN_PASSIVE_RATIO_MAX - RIVEN_PASSIVE_RATIO_MIN)
}

/// 锐雯施放任意技能时叠加一层被动（最多3层）并刷新持续时间。
/// 统一由 EventSkillCast 驱动，无需在每个技能 observer 内单独授予。
pub fn on_riven_skill_cast_charge_passive(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_riven: Query<&Buffs, With<Riven>>,
    mut q_passive: Query<&mut BuffRivenPassive>,
) {
    let caster = trigger.event().entity;

    // 先尝试在已有 buff 中找被动并叠加（首次施法时角色可能尚无 Buffs 组件）
    if let Ok(buffs) = q_riven.get(caster) {
        for buff_entity in buffs.iter() {
            if let Ok(mut passive) = q_passive.get_mut(buff_entity) {
                // 层数从 0 回升：重新点亮充能光效
                if passive.charges == 0 {
                    spawn_passive_particles(&mut commands, caster);
                }
                passive.charges = (passive.charges + 1).min(RIVEN_PASSIVE_MAX_CHARGES);
                passive.timer = fresh_timer();
                return;
            }
        }
    }

    // 无被动：新建1层（with_related 会自动为角色建立 Buffs 关系目标）
    spawn_passive_particles(&mut commands, caster);
    commands
        .entity(caster)
        .with_related::<BuffOf>(BuffRivenPassive {
            charges: 1,
            timer: fresh_timer(),
        });
}

/// 普攻命中时（EventAttackEnd）若有被动层数，消耗1层并造成额外伤害。
/// 直接触发 EventAttackEnd 仅触发 on-hit，不含基础普攻伤害。
pub fn on_damage_create_trigger_bonus(
    trigger: On<EventAttackEnd>,
    mut commands: Commands,
    q_riven: Query<(&Damage, &Level), With<Riven>>,
    q_buffs: Query<&Buffs>,
    mut q_passive: Query<&mut BuffRivenPassive>,
) {
    let source = trigger.entity;

    let Ok((damage, level)) = q_riven.get(source) else {
        return;
    };
    let Ok(buffs) = q_buffs.get(source) else {
        return;
    };

    for buff_entity in buffs.iter() {
        let Ok(mut passive) = q_passive.get_mut(buff_entity) else {
            continue;
        };
        if passive.charges == 0 {
            continue;
        }

        let ratio = passive_ratio_for_level(level.value);
        let bonus_damage = damage.0 * ratio;

        commands.trigger(CommandDamageCreate {
            entity: trigger.target,
            source,
            damage_type: DamageType::Physical,
            amount: bonus_damage,
            tag: None,
        });

        passive.charges -= 1;
        // 层数耗尽：撤下充能光效
        if passive.charges == 0 {
            despawn_passive_particles(&mut commands, source);
        }
        info!(
            "{:?} 锐雯被动触发，额外伤害: {:.1}（剩余 {} 层）",
            source, bonus_damage, passive.charges
        );
        return;
    }
}

/// FixedUpdate：tick 被动计时器，到期后移除整个 buff（层数清空）。
pub fn update_riven_passive_timer(
    mut commands: Commands,
    time: Res<Time>,
    mut q_passive: Query<(Entity, &BuffOf, &mut BuffRivenPassive)>,
) {
    let mut expired = Vec::new();
    for (entity, buff_of, mut passive) in q_passive.iter_mut() {
        passive.timer.tick(time.delta());
        if passive.timer.is_finished() {
            // 到期时若仍有层数，充能光效还亮着，一并撤下
            if passive.charges > 0 {
                despawn_passive_particles(&mut commands, buff_of.0);
            }
            expired.push(entity);
        }
    }
    for entity in expired {
        commands.entity(entity).despawn();
    }
}
