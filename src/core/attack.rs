use bevy::ecs::error::ignore;
use bevy::ecs::system::command::trigger;
use bevy::prelude::*;
use league_core::SpellObject;
use lol_config::{HashKey, LeagueProperties};
use serde::{Deserialize, Serialize};

use crate::{
    Buffs, CommandDamageCreate, CommandMissileCreate, CommandRotate, Damage, DamageType, EventDead,
};

#[derive(Default)]
pub struct PluginAttack;

impl Plugin for PluginAttack {
    fn build(&self, app: &mut App) {
        app.add_observer(on_command_attack_start);
        app.add_observer(on_command_attack_reset);
        app.add_observer(on_command_attack_stop);
        app.add_observer(on_event_dead);

        app.add_systems(FixedUpdate, fixed_update);
    }
}

/// 攻击组件 - 包含攻击的基础属性
#[derive(Debug, Component, Clone)]
pub struct Attack {
    pub range: f32,
    /// 基础攻击速度 (1级时的每秒攻击次数)
    pub base_attack_speed: f32,
    /// 额外攻击速度加成 (来自装备/符文的攻击速度)
    pub bonus_attack_speed: f32,
    /// 攻击速度上限 (默认 2.5)
    pub attack_speed_cap: f32,
    /// 前摇时间配置
    pub windup_config: WindupConfig,
    /// 前摇修正系数 (默认 1.0，可以被技能修改)
    pub windup_modifier: f32,
    /// 攻击导弹
    pub spell_key: Option<HashKey<SpellObject>>,
}

/// 前摇时间配置方式
#[derive(Component, Clone, Debug)]
pub enum WindupConfig {
    /// 老英雄公式: 0.3 + attackOffset
    Legacy { attack_offset: f32 },
    /// 新英雄公式: attackCastTime / attackTotalTime
    Modern {
        attack_cast_time: f32,
        attack_total_time: f32,
    },
}

/// 攻击状态机
#[derive(Component, Clone, Serialize, Deserialize)]
pub struct AttackState {
    pub status: AttackStatus,
    /// 攻击目标
    pub target: Option<Entity>,
}

#[derive(Component)]
pub struct AttackBuff {
    pub bonus_attack_speed: f32,
}

/// 攻击状态 - 更详细的状态表示
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AttackStatus {
    /// 前摇阶段 - 举起武器准备攻击
    Windup { target: Entity, end_time: f32 },
    /// 后摇阶段 - 武器收回，等待下一次攻击
    Cooldown { end_time: f32 },
}

#[derive(EntityEvent, Debug)]
pub struct CommandAttackStart {
    pub entity: Entity,
    pub target: Entity,
}

#[derive(EntityEvent, Debug)]
pub struct CommandAttackReset {
    pub entity: Entity,
}

#[derive(EntityEvent, Debug)]
pub struct CommandAttackStop {
    pub entity: Entity,
}

#[derive(EntityEvent, Debug)]
pub struct EventAttackStart {
    pub entity: Entity,
    pub target: Entity,
    pub duration: f32,
}

#[derive(EntityEvent, Debug)]
pub struct EventAttackEnd {
    pub entity: Entity,
    pub target: Entity,
}

#[derive(EntityEvent, Debug)]
pub struct EventAttackReady {
    pub entity: Entity,
}

impl Attack {
    pub fn new(range: f32, windup_duration_secs: f32, total_duration_secs: f32) -> Self {
        Self {
            range,
            base_attack_speed: 1.0 / total_duration_secs,
            bonus_attack_speed: 0.0,
            attack_speed_cap: 2.5,
            windup_config: WindupConfig::Modern {
                attack_cast_time: windup_duration_secs,
                attack_total_time: total_duration_secs,
            },
            windup_modifier: 1.0,
            spell_key: None,
        }
    }

    pub fn from_legacy(range: f32, base_attack_speed: f32, windup_offset: f32) -> Self {
        Self {
            range,
            base_attack_speed,
            bonus_attack_speed: 0.0,
            attack_speed_cap: 2.5,
            windup_config: WindupConfig::Legacy {
                attack_offset: windup_offset,
            },
            windup_modifier: 1.0,
            spell_key: None,
        }
    }

    pub fn with_missile(mut self, missile: Option<HashKey<SpellObject>>) -> Self {
        self.spell_key = missile;
        self
    }

    pub fn with_bonus_attack_speed(mut self, bonus_attack_speed: f32) -> Self {
        self.bonus_attack_speed = bonus_attack_speed;
        self
    }

    /// 计算当前总攻击速度
    pub fn current_attack_speed(&self) -> f32 {
        (self.base_attack_speed * (1.0 + self.bonus_attack_speed)).min(self.attack_speed_cap)
    }

    /// 计算攻击间隔时间 (1 / attack_speed)
    pub fn total_duration_secs(&self) -> f32 {
        1.0 / self.current_attack_speed()
    }

    pub fn animation_duration(&self) -> f32 {
        match self.windup_config {
            WindupConfig::Legacy { attack_offset } => (0.3 + attack_offset) * 4.,
            WindupConfig::Modern {
                attack_cast_time, ..
            } => attack_cast_time * 4.,
        }
    }

    /// 计算前摇时间
    pub fn windup_duration_secs(&self) -> f32 {
        let total_time = self.total_duration_secs();
        let base_windup = match self.windup_config {
            WindupConfig::Legacy { attack_offset } => 0.3 + attack_offset,
            WindupConfig::Modern {
                attack_cast_time,
                attack_total_time,
            } => attack_cast_time / attack_total_time * total_time,
        };

        // 应用前摇修正系数
        if self.windup_modifier == 1.0 {
            base_windup
        } else {
            // 修复：直接对前摇时间应用修正系数
            base_windup * self.windup_modifier
        }
    }

    /// 计算后摇时间
    pub fn cooldown_time(&self) -> f32 {
        self.total_duration_secs() - self.windup_duration_secs()
    }
}

impl AttackState {
    pub fn is_windup(&self) -> bool {
        matches!(self.status, AttackStatus::Windup { .. })
    }

    pub fn is_cooldown(&self) -> bool {
        matches!(self.status, AttackStatus::Cooldown { .. })
    }

    pub fn is_attacking(&self) -> bool {
        self.is_windup() || self.is_cooldown()
    }
}

fn update_attack_state(attack_state: &mut Attack, buffs: Vec<&AttackBuff>) {
    attack_state.bonus_attack_speed = buffs
        .iter()
        .map(|v| v.bonus_attack_speed)
        .reduce(|a, b| a + b)
        .unwrap_or(0.0);
}

// 观察者函数
fn on_command_attack_start(
    trigger: On<CommandAttackStart>,
    mut commands: Commands,
    mut q_attack_state: Query<&mut AttackState>,
    mut q_attack: Query<&mut Attack>,
    q_transform: Query<&Transform>,
    q_attack_buff: Query<&AttackBuff>,
    q_buffs: Query<&Buffs>,
    time: Res<Time<Fixed>>,
) {
    let entity = trigger.event_target();
    let target = trigger.target;

    let now = time.elapsed_secs();

    let Ok(mut attack) = q_attack.get_mut(entity) else {
        return;
    };

    let Ok(mut attack_state) = q_attack_state.get_mut(entity) else {
        if let Ok(buffs) = q_buffs.get(entity) {
            let buffs = buffs
                .iter()
                .filter_map(|v| q_attack_buff.get(v).ok())
                .collect::<Vec<_>>();
            update_attack_state(&mut attack, buffs);
        } else {
            update_attack_state(&mut attack, vec![]);
        }

        let Ok(target_position) = q_transform.get(target).map(|v| v.translation.xz()) else {
            return;
        };

        let transform = q_transform.get(entity).unwrap();

        let direction = (target_position - transform.translation.xz()).normalize();

        commands.trigger(CommandRotate {
            entity,
            priority: 1,
            direction,
            angular_velocity: None,
        });

        commands.entity(entity).insert(AttackState {
            status: AttackStatus::Windup {
                target,
                end_time: now + attack.windup_duration_secs(),
            },
            target: Some(target),
        });
        commands.trigger(EventAttackStart {
            entity,
            target,
            duration: attack.total_duration_secs(),
        });
        return;
    };

    match &attack_state.status {
        AttackStatus::Windup {
            target: windup_target,
            ..
        } => {
            if *windup_target == target {
                return;
            }

            debug!("{} 移除攻击状态：攻击目标改变", entity);
            commands.entity(entity).try_remove::<AttackState>();
            commands.trigger(CommandAttackStart { entity, target });
        }
        AttackStatus::Cooldown { .. } => {
            // 冷却阶段也需要设置目标，因为下一次攻击要攻击这个目标
            attack_state.target = Some(target);
        }
    }
}

fn on_command_attack_reset(
    trigger: On<CommandAttackReset>,
    mut commands: Commands,
    mut query: Query<&mut AttackState>,
) {
    let entity = trigger.event_target();

    let Ok(attack_state) = query.get_mut(entity) else {
        return;
    };

    debug!("{} 移除攻击状态：攻击重置", entity);
    commands.entity(entity).try_remove::<AttackState>();

    let Some(target) = attack_state.target else {
        return;
    };

    commands.trigger(CommandAttackStart { entity, target });
}

fn on_command_attack_stop(
    trigger: On<CommandAttackStop>,
    mut commands: Commands,
    mut q_attack_state: Query<&mut AttackState>,
) {
    let entity = trigger.event_target();

    let Ok(mut attack_state) = q_attack_state.get_mut(entity) else {
        return;
    };

    match attack_state.status {
        AttackStatus::Windup { .. } => {
            debug!("{} 移除攻击状态：停止攻击", entity);
            commands.entity(entity).try_remove::<AttackState>();
        }
        AttackStatus::Cooldown { .. } => {
            debug!("{} 攻击冷却中，停止下一次攻击", entity);
            attack_state.target = None;
        }
    };
}

fn on_event_dead(
    trigger: On<EventDead>,
    mut commands: Commands,
    q_attack_state: Query<(Entity, &AttackState)>,
) {
    let dead_entity = trigger.event_target();

    for (entity, attack_state) in q_attack_state.iter() {
        if let AttackStatus::Windup { target, .. } = &attack_state.status {
            if *target == dead_entity {
                debug!("{} 移除攻击状态：攻击目标 {} 死亡", dead_entity, entity);
                commands.entity(entity).try_remove::<AttackState>();
            }
        }
    }
}

fn fixed_update(
    mut query: Query<(Entity, &mut AttackState, &Attack, &Damage)>,
    mut commands: Commands,
    res_assets_spell_object: Res<Assets<SpellObject>>,
    res_league_properties: Res<LeagueProperties>,
    time: Res<Time<Fixed>>,
) {
    let now = time.elapsed_secs();

    for (entity, mut attack_state, attack, damage) in query.iter_mut() {
        match &attack_state.status.clone() {
            AttackStatus::Windup { target, end_time } => {
                // 检查前摇是否完成
                if *end_time <= now {
                    attack_state.status = AttackStatus::Cooldown {
                        end_time: now + attack.cooldown_time(),
                    };

                    match &attack.spell_key {
                        Some(spell_key) => {
                            let spell = res_league_properties
                                .get(&res_assets_spell_object, spell_key)
                                .unwrap();

                            if spell.m_spell.as_ref().unwrap().m_cast_type.unwrap_or(0) == 1 {
                                commands.trigger(CommandMissileCreate {
                                    entity,
                                    target: *target,
                                    spell_key: spell_key.clone(),
                                });
                            } else {
                                commands.try_trigger(CommandDamageCreate {
                                    entity: *target,
                                    source: entity,
                                    damage_type: DamageType::Physical,
                                    amount: damage.0,
                                });
                            }
                        }
                        None => {
                            commands.try_trigger(CommandDamageCreate {
                                entity: *target,
                                source: entity,
                                damage_type: DamageType::Physical,
                                amount: damage.0,
                            });
                        }
                    }
                    commands.try_trigger(EventAttackEnd {
                        entity,
                        target: *target,
                    });
                }
            }
            AttackStatus::Cooldown { end_time } => {
                // 检查后摇是否完成
                if *end_time <= now {
                    debug!("{} 移除攻击状态：攻击冷却结束", entity);
                    commands.entity(entity).try_remove::<AttackState>();
                    commands.try_trigger(EventAttackReady { entity });

                    if let Some(target) = attack_state.target {
                        debug!(
                            "{} 攻击冷却结束时依然存在攻击目标，继续攻击 {}",
                            entity, target
                        );
                        commands.try_trigger(CommandAttackStart { entity, target });
                    };
                }
            }
        }
    }
}

pub trait EntityCommandsTrigger {
    fn try_trigger<'a, T: Event<Trigger<'a>: Default>>(&mut self, event: T) -> &mut Self;
}

impl<'w, 'a> EntityCommandsTrigger for Commands<'w, 'a> {
    fn try_trigger<'b, T: Event<Trigger<'b>: Default>>(&mut self, event: T) -> &mut Self {
        self.queue_handled(trigger(event), ignore);
        self
    }
}
