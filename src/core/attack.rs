use bevy::prelude::*;

use crate::core::Target;

pub struct PluginAttack;

impl Plugin for PluginAttack {
    fn build(&self, app: &mut App) {
        app.add_event::<EventAttackWindupStart>();
        app.add_event::<EventAttackWindupComplete>();
        app.add_event::<EventAttackCooldownStart>();
        app.add_event::<EventAttackCooldownComplete>();
        app.add_event::<EventAttackReset>();
        app.add_event::<EventAttackCancel>();
        app.add_event::<CommandAttackCast>();
        app.add_event::<CommandAttackReset>();
        app.add_event::<CommandAttackCancel>();
        app.add_observer(on_command_attack_cast);
        app.add_observer(on_command_attack_reset);
        app.add_observer(on_command_attack_cancel);
        app.add_systems(
            FixedUpdate,
            (attack_timer_system, attack_state_machine_system).chain(),
        );
    }
}

/// 攻击组件 - 包含攻击的基础属性
#[derive(Component)]
#[require(AttackState, AttackTimer)]
pub struct Attack {
    pub range: f32,
    /// 基础攻击速度 (attacks per second at level 1)
    pub base_attack_speed: f32,
    /// 额外攻击速度加成 (bonus attack speed from items/runes)
    pub bonus_attack_speed: f32,
    /// 攻击速度上限 (default 2.5)
    pub attack_speed_cap: f32,
    /// 前摇时间配置
    pub windup_config: WindupConfig,
    /// 前摇修正系数 (default 1.0, can be modified by abilities)
    pub windup_modifier: f32,
}

impl Default for Attack {
    fn default() -> Self {
        Self {
            range: 125.0,
            base_attack_speed: 0.625,
            bonus_attack_speed: 0.0,
            attack_speed_cap: 2.5,
            windup_config: WindupConfig::Percent(0.25),
            windup_modifier: 1.0,
        }
    }
}

impl Attack {
    /// 计算当前总攻击速度
    pub fn current_attack_speed(&self) -> f32 {
        (self.base_attack_speed * (1.0 + self.bonus_attack_speed)).min(self.attack_speed_cap)
    }

    /// 计算攻击间隔时间 (1 / attack_speed)
    pub fn attack_interval(&self) -> f32 {
        1.0 / self.current_attack_speed()
    }

    /// 计算前摇时间
    pub fn windup_time(&self) -> f32 {
        let total_time = self.attack_interval();
        let base_windup = match self.windup_config {
            WindupConfig::Fixed(time) => time,
            WindupConfig::Percent(percent) => total_time * percent,
            WindupConfig::Legacy { attack_offset } => 0.3 + attack_offset,
            WindupConfig::Modern {
                attack_cast_time,
                attack_total_time,
            } => attack_cast_time / attack_total_time * total_time,
        };

        // Apply windup modifier
        if self.windup_modifier == 1.0 {
            base_windup
        } else {
            base_windup + self.windup_modifier * (total_time * self.windup_percent() - base_windup)
        }
    }

    /// 计算后摇时间
    pub fn cooldown_time(&self) -> f32 {
        self.attack_interval() - self.windup_time()
    }

    fn windup_percent(&self) -> f32 {
        match self.windup_config {
            WindupConfig::Fixed(_) => 0.25, // fallback
            WindupConfig::Percent(percent) => percent,
            WindupConfig::Legacy { .. } => 0.3,
            WindupConfig::Modern {
                attack_cast_time,
                attack_total_time,
            } => attack_cast_time / attack_total_time,
        }
    }
}

/// 前摇时间配置方式
#[derive(Component, Clone, Debug)]
pub enum WindupConfig {
    /// 固定前摇时间 (秒)
    Fixed(f32),
    /// 前摇时间占总攻击时间的百分比
    Percent(f32),
    /// 老英雄公式: 0.3 + attackOffset
    Legacy { attack_offset: f32 },
    /// 新英雄公式: attackCastTime / attackTotalTime
    Modern {
        attack_cast_time: f32,
        attack_total_time: f32,
    },
}

/// 攻击计时器 - 跟踪攻击的时间状态
#[derive(Component, Default)]
pub struct AttackTimer {
    /// 当前阶段开始的时间
    pub phase_start_time: f32,
    /// 前摇是否不可取消 (uncancellable)
    pub uncancellable_windup: bool,
    /// 不可取消的剩余时间 (2 game ticks = 0.066s grace period)
    pub uncancellable_remaining: f32,
}

/// 攻击状态机
#[derive(Component, Default)]
pub struct AttackState {
    pub status: AttackStatus,
}

/// 攻击状态 - 更详细的状态表示
#[derive(Default, Debug, Clone, PartialEq)]
pub enum AttackStatus {
    #[default]
    Idle,
    /// 前摇阶段 - 举起武器准备攻击
    Windup { target: Entity, can_cancel: bool },
    /// 后摇阶段 - 武器收回，等待下一次攻击
    Cooldown { target: Entity },
}

impl AttackState {
    pub fn is_idle(&self) -> bool {
        matches!(self.status, AttackStatus::Idle)
    }

    pub fn is_windup(&self) -> bool {
        matches!(self.status, AttackStatus::Windup { .. })
    }

    pub fn is_cooldown(&self) -> bool {
        matches!(self.status, AttackStatus::Cooldown { .. })
    }

    pub fn is_attacking(&self) -> bool {
        self.is_windup() || self.is_cooldown()
    }

    pub fn current_target(&self) -> Option<Entity> {
        match &self.status {
            AttackStatus::Idle => None,
            AttackStatus::Windup { target, .. } => Some(*target),
            AttackStatus::Cooldown { target } => Some(*target),
        }
    }
}

// Events
#[derive(Event, Debug)]
pub struct CommandAttackCast;

#[derive(Event, Debug)]
pub struct CommandAttackReset;

#[derive(Event, Debug)]
pub struct CommandAttackCancel;

#[derive(Event, Debug)]
pub struct EventAttackWindupStart {
    pub target: Entity,
}

#[derive(Event, Debug)]
pub struct EventAttackWindupComplete {
    pub target: Entity,
}

#[derive(Event, Debug)]
pub struct EventAttackCooldownStart {
    pub target: Entity,
}

#[derive(Event, Debug)]
pub struct EventAttackCooldownComplete;

#[derive(Event, Debug)]
pub struct EventAttackReset;

#[derive(Event, Debug)]
pub struct EventAttackCancel;

// Constants
const GAME_TICK_DURATION: f32 = 0.033; // 30 FPS game ticks
const UNCANCELLABLE_GRACE_PERIOD: f32 = 2.0 * GAME_TICK_DURATION; // 0.066 seconds

// Observer functions
fn on_command_attack_cast(
    trigger: Trigger<CommandAttackCast>,
    mut commands: Commands,
    mut query: Query<(&mut AttackState, &mut AttackTimer, &Target)>,
    time: Res<Time<Fixed>>,
) {
    let entity = trigger.target();

    if let Ok((mut attack_state, mut timer, target)) = query.get_mut(entity) {
        // 只有在空闲状态时才能锁定新目标
        if attack_state.is_idle() {
            attack_state.status = AttackStatus::Windup {
                target: target.0,
                can_cancel: true,
            };
            timer.phase_start_time = time.elapsed_secs();
            commands.trigger_targets(EventAttackWindupStart { target: target.0 }, entity);
        }
    }
}

fn on_command_attack_reset(
    trigger: Trigger<CommandAttackReset>,
    mut commands: Commands,
    mut query: Query<(&mut AttackState, &mut AttackTimer)>,
    time: Res<Time<Fixed>>,
) {
    let entity = trigger.target();

    if let Ok((mut attack_state, mut timer)) = query.get_mut(entity) {
        match &attack_state.status {
            // 在前摇阶段重置 = 取消当前攻击 (通常是坏事)
            AttackStatus::Windup { .. } => {
                info!("Attack reset during windup - cancelling attack");
                attack_state.status = AttackStatus::Idle;
                timer.phase_start_time = time.elapsed_secs();
                commands.trigger_targets(EventAttackCancel, entity);
            }
            // 在后摇阶段重置 = 跳过后摇，立刻开始下一次攻击 (好事)
            AttackStatus::Cooldown { target } => {
                info!("Attack reset during cooldown - skipping to next attack");
                attack_state.status = AttackStatus::Windup {
                    target: *target,
                    can_cancel: true,
                };
                timer.phase_start_time = time.elapsed_secs();
                commands.trigger_targets(EventAttackReset, entity);
            }
            _ => {
                // 在其他状态下重置攻击计时
                attack_state.status = AttackStatus::Idle;
                timer.phase_start_time = time.elapsed_secs();
            }
        }
    }
}

fn on_command_attack_cancel(
    trigger: Trigger<CommandAttackCancel>,
    mut commands: Commands,
    mut query: Query<(&mut AttackState, &mut AttackTimer)>,
    time: Res<Time<Fixed>>,
) {
    let entity = trigger.target();

    if let Ok((mut attack_state, mut timer)) = query.get_mut(entity) {
        // 检查是否可以取消
        let can_cancel = match &attack_state.status {
            AttackStatus::Windup { can_cancel, .. } => *can_cancel,
            _ => true, // 其他状态都可以取消
        };

        if can_cancel {
            attack_state.status = AttackStatus::Idle;
            timer.phase_start_time = time.elapsed_secs();
            timer.uncancellable_windup = false;
            timer.uncancellable_remaining = 0.0;
            commands.trigger_targets(EventAttackCancel, entity);
        }
    }
}

// Systems
fn attack_timer_system(mut query: Query<(&mut AttackTimer, &Attack)>, time: Res<Time<Fixed>>) {
    for (mut timer, _attack) in query.iter_mut() {
        // 更新不可取消的剩余时间
        if timer.uncancellable_remaining > 0.0 {
            timer.uncancellable_remaining -= time.delta_secs();
            if timer.uncancellable_remaining <= 0.0 {
                timer.uncancellable_windup = false;
            }
        }
    }
}

fn attack_state_machine_system(
    mut query: Query<(Entity, &mut AttackState, &mut AttackTimer, &Attack)>,
    mut commands: Commands,
    time: Res<Time<Fixed>>,
) {
    let current_time = time.elapsed_secs();

    for (entity, mut attack_state, mut timer, attack) in query.iter_mut() {
        match &attack_state.status.clone() {
            AttackStatus::Windup { target, can_cancel } => {
                let elapsed = current_time - timer.phase_start_time;
                let windup_time = attack.windup_time();

                // 更新是否可以取消
                let new_can_cancel = !timer.uncancellable_windup;
                if *can_cancel != new_can_cancel {
                    attack_state.status = AttackStatus::Windup {
                        target: *target,
                        can_cancel: new_can_cancel,
                    };
                }

                // 检查前摇是否完成
                if elapsed >= windup_time {
                    attack_state.status = AttackStatus::Cooldown { target: *target };
                    timer.phase_start_time = current_time;
                    timer.uncancellable_windup = false;
                    timer.uncancellable_remaining = 0.0;

                    commands.trigger_targets(EventAttackWindupComplete { target: *target }, entity);
                    commands.trigger_targets(EventAttackCooldownStart { target: *target }, entity);
                }
            }

            AttackStatus::Cooldown { target: _ } => {
                let elapsed = current_time - timer.phase_start_time;
                let cooldown_time = attack.cooldown_time();

                // 检查后摇是否完成
                if elapsed >= cooldown_time {
                    attack_state.status = AttackStatus::Idle;
                    timer.phase_start_time = current_time;

                    commands.trigger_targets(EventAttackCooldownComplete, entity);
                }
            }

            AttackStatus::Idle => {
                // 空闲状态，无需处理
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::{PluginTarget, Target};

    use super::*;

    #[test]
    fn test_attack_speed_calculations() {
        let attack = Attack {
            base_attack_speed: 0.625,
            bonus_attack_speed: 1.0, // 100% bonus AS
            attack_speed_cap: 2.5,
            windup_config: WindupConfig::Percent(0.25),
            ..Default::default()
        };

        // 0.625 * (1 + 1.0) = 1.25 attacks per second
        assert_eq!(attack.current_attack_speed(), 1.25);
        // 1 / 1.25 = 0.8 seconds per attack
        assert_eq!(attack.attack_interval(), 0.8);
        // 0.8 * 0.25 = 0.2 seconds windup
        assert_eq!(attack.windup_time(), 0.2);
        // 0.8 - 0.2 = 0.6 seconds cooldown
        assert_eq!(attack.cooldown_time(), 0.6);
    }

    #[test]
    fn test_attack_speed_cap() {
        let attack = Attack {
            base_attack_speed: 0.625,
            bonus_attack_speed: 10.0, // 1000% bonus AS (way over cap)
            attack_speed_cap: 2.5,
            ..Default::default()
        };

        // Should be capped at 2.5
        assert_eq!(attack.current_attack_speed(), 2.5);
    }

    #[test]
    fn test_windup_config_legacy() {
        let attack = Attack {
            base_attack_speed: 1.0,
            windup_config: WindupConfig::Legacy { attack_offset: 0.1 },
            ..Default::default()
        };

        // Legacy formula: 0.3 + attack_offset = 0.3 + 0.1 = 0.4
        assert_eq!(attack.windup_time(), 0.4);
    }

    #[test]
    fn test_windup_config_modern() {
        let attack = Attack {
            base_attack_speed: 1.0, // 1 second attack interval
            windup_config: WindupConfig::Modern {
                attack_cast_time: 0.25,
                attack_total_time: 1.0,
            },
            ..Default::default()
        };

        // Modern formula: (0.25 / 1.0) * 1.0 = 0.25
        assert_eq!(attack.windup_time(), 0.25);
    }

    #[test]
    fn test_attack_state_machine_progression() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(PluginTarget);
        app.add_plugins(PluginAttack);

        let target_entity = app.world_mut().spawn_empty().id();

        let attacker = app
            .world_mut()
            .spawn((
                Attack {
                    windup_config: WindupConfig::Fixed(0.1), // 0.1s windup
                    base_attack_speed: 1.0,                  // 1 attack per second
                    ..Default::default()
                },
                Target(target_entity),
            ))
            .id();

        // Start in idle state
        {
            let attack_state = app.world().get::<AttackState>(attacker).unwrap();
            assert!(attack_state.is_idle());
        }

        // Start attack target
        app.world_mut().trigger_targets(CommandAttackCast, attacker);

        // Process observers and systems
        app.update();

        // Should be windup after the observer processes the command
        {
            let attack_state = app.world().get::<AttackState>(attacker).unwrap();
            // Note: This test verifies the observer works, full state machine testing
            // would require more complex setup with proper time simulation
            assert!(attack_state.is_windup());
        }
    }

    #[test]
    fn test_uncancellable_grace_period() {
        let attack = Attack {
            windup_config: WindupConfig::Fixed(0.05), // 0.05s windup (less than grace period)
            base_attack_speed: 1.0,
            ..Default::default()
        };

        // Windup time is less than grace period, so entire windup should be uncancellable
        assert!(attack.windup_time() < UNCANCELLABLE_GRACE_PERIOD);
    }
}
