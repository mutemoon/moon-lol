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
        app.add_systems(FixedUpdate, attack_state_machine_system);
    }
}

/// 攻击组件 - 包含攻击的基础属性
#[derive(Component)]
#[require(AttackState)]
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
}

impl Default for Attack {
    fn default() -> Self {
        Self {
            range: 125.0,
            base_attack_speed: 0.625,
            bonus_attack_speed: 0.0,
            attack_speed_cap: 2.5,
            windup_config: WindupConfig::Legacy { attack_offset: 0.0 },
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
            base_windup + self.windup_modifier * (total_time * self.windup_percent() - base_windup)
        }
    }

    /// 计算后摇时间
    pub fn cooldown_time(&self) -> f32 {
        self.attack_interval() - self.windup_time()
    }

    fn windup_percent(&self) -> f32 {
        match self.windup_config {
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
    /// 老英雄公式: 0.3 + attackOffset
    Legacy { attack_offset: f32 },
    /// 新英雄公式: attackCastTime / attackTotalTime
    Modern {
        attack_cast_time: f32,
        attack_total_time: f32,
    },
}

/// 攻击状态机
#[derive(Component, Default)]
pub struct AttackState {
    pub status: AttackStatus,
    /// 当前阶段开始的时间
    pub phase_start_time: f32,
}

/// 攻击状态 - 更详细的状态表示
#[derive(Default, Debug, Clone, PartialEq)]
pub enum AttackStatus {
    #[default]
    Idle,
    /// 前摇阶段 - 举起武器准备攻击
    Windup { target: Entity },
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

// 事件定义
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

// 常量定义
const GAME_TICK_DURATION: f32 = 0.033; // 30 FPS 游戏帧
const UNCANCELLABLE_GRACE_PERIOD: f32 = 2.0 * GAME_TICK_DURATION; // 0.066 秒

impl AttackState {
    /// 检查攻击是否可以取消
    /// 攻击生效前的两帧不可取消
    pub fn can_cancel(&self, current_time: f32, windup_time: f32) -> bool {
        match &self.status {
            AttackStatus::Windup { .. } => {
                let elapsed = current_time - self.phase_start_time;
                let time_until_hit = windup_time - elapsed;
                // 如果距离攻击生效还有超过2帧的时间，则可以取消
                time_until_hit > UNCANCELLABLE_GRACE_PERIOD
            }
            _ => true, // 其他状态都可以取消
        }
    }
}

// 观察者函数
fn on_command_attack_cast(
    trigger: Trigger<CommandAttackCast>,
    mut commands: Commands,
    mut query: Query<(&mut AttackState, &Target)>,
    time: Res<Time<Fixed>>,
) {
    let entity = trigger.target();

    if let Ok((mut attack_state, target)) = query.get_mut(entity) {
        // 只有在空闲状态时才能锁定新目标
        if attack_state.is_idle() {
            attack_state.status = AttackStatus::Windup { target: target.0 };
            attack_state.phase_start_time = time.elapsed_secs();
            commands.trigger_targets(EventAttackWindupStart { target: target.0 }, entity);
        }
    }
}

fn on_command_attack_reset(
    trigger: Trigger<CommandAttackReset>,
    mut commands: Commands,
    mut query: Query<&mut AttackState>,
    time: Res<Time<Fixed>>,
) {
    let entity = trigger.target();

    if let Ok(mut attack_state) = query.get_mut(entity) {
        match &attack_state.status {
            // 在前摇阶段重置 = 取消当前攻击 (通常是坏事)
            AttackStatus::Windup { .. } => {
                info!("Attack reset during windup - cancelling attack");
                attack_state.status = AttackStatus::Idle;
                attack_state.phase_start_time = time.elapsed_secs();
                commands.trigger_targets(EventAttackCancel, entity);
            }
            // 在后摇阶段重置 = 跳过后摇，立刻开始下一次攻击 (好事)
            AttackStatus::Cooldown { target } => {
                info!("Attack reset during cooldown - skipping to next attack");
                attack_state.status = AttackStatus::Windup { target: *target };
                attack_state.phase_start_time = time.elapsed_secs();
                commands.trigger_targets(EventAttackReset, entity);
            }
            _ => {
                // 在其他状态下重置攻击计时
                attack_state.status = AttackStatus::Idle;
                attack_state.phase_start_time = time.elapsed_secs();
            }
        }
    }
}

fn on_command_attack_cancel(
    trigger: Trigger<CommandAttackCancel>,
    mut commands: Commands,
    mut query: Query<(&mut AttackState, &Attack)>,
    time: Res<Time<Fixed>>,
) {
    let entity = trigger.target();

    if let Ok((mut attack_state, attack)) = query.get_mut(entity) {
        let current_time = time.elapsed_secs();
        let windup_time = attack.windup_time();

        // 检查是否可以取消
        if attack_state.can_cancel(current_time, windup_time) {
            attack_state.status = AttackStatus::Idle;
            attack_state.phase_start_time = current_time;
            commands.trigger_targets(EventAttackCancel, entity);
        }
    }
}

// 系统函数
fn attack_state_machine_system(
    mut query: Query<(Entity, &mut AttackState, &Attack)>,
    mut commands: Commands,
    time: Res<Time<Fixed>>,
) {
    let current_time = time.elapsed_secs();

    for (entity, mut attack_state, attack) in query.iter_mut() {
        match &attack_state.status.clone() {
            AttackStatus::Windup { target } => {
                let elapsed = current_time - attack_state.phase_start_time;
                let windup_time = attack.windup_time();

                // 检查前摇是否完成
                if elapsed >= windup_time {
                    attack_state.status = AttackStatus::Cooldown { target: *target };
                    attack_state.phase_start_time = current_time;

                    commands.trigger_targets(EventAttackWindupComplete { target: *target }, entity);
                    commands.trigger_targets(EventAttackCooldownStart { target: *target }, entity);
                }
            }

            AttackStatus::Cooldown { target: _ } => {
                let elapsed = current_time - attack_state.phase_start_time;
                let cooldown_time = attack.cooldown_time();

                // 检查后摇是否完成
                if elapsed >= cooldown_time {
                    attack_state.status = AttackStatus::Idle;
                    attack_state.phase_start_time = current_time;

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
    use crate::core::{PluginCommand, PluginTarget};

    use super::*;

    fn create_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(PluginTarget);
        app.add_plugins(PluginAttack);
        app.add_plugins(PluginCommand);

        // 设置固定时间步长为30 FPS
        app.insert_resource(Time::<Fixed>::from_hz(30.0));
        app
    }

    fn advance_time(app: &mut App, seconds: f32) {
        let ticks = (seconds * 30.0).ceil() as u32; // 30 FPS
        for _ in 0..ticks {
            // 手动推进固定时间步长
            let mut time = app.world_mut().resource_mut::<Time<Fixed>>();
            time.advance_by(std::time::Duration::from_secs_f32(1.0 / 30.0));
            drop(time);

            // 手动运行FixedUpdate调度
            app.world_mut().run_schedule(FixedUpdate);
        }
    }

    #[test]
    fn test_attack_speed_calculations() {
        let attack = Attack {
            base_attack_speed: 0.625,
            bonus_attack_speed: 1.0, // 100% 额外攻击速度
            attack_speed_cap: 2.5,
            windup_config: WindupConfig::Legacy { attack_offset: 0.0 },
            ..Default::default()
        };

        // 0.625 * (1 + 1.0) = 1.25 每秒攻击次数
        assert_eq!(attack.current_attack_speed(), 1.25);
        // 1 / 1.25 = 0.8 秒每次攻击
        assert_eq!(attack.attack_interval(), 0.8);
        // 0.3 + 0.0 = 0.3 秒前摇
        assert_eq!(attack.windup_time(), 0.3);
        // 0.8 - 0.3 = 0.5 秒后摇
        assert_eq!(attack.cooldown_time(), 0.5);
    }

    #[test]
    fn test_attack_speed_cap() {
        let attack = Attack {
            base_attack_speed: 0.625,
            bonus_attack_speed: 10.0, // 1000% 额外攻击速度 (远超上限)
            attack_speed_cap: 2.5,
            ..Default::default()
        };

        // 应该被限制在 2.5
        assert_eq!(attack.current_attack_speed(), 2.5);
    }

    #[test]
    fn test_windup_config_legacy() {
        let attack = Attack {
            base_attack_speed: 1.0,
            windup_config: WindupConfig::Legacy { attack_offset: 0.1 },
            ..Default::default()
        };

        // 老英雄公式: 0.3 + attack_offset = 0.3 + 0.1 = 0.4
        assert_eq!(attack.windup_time(), 0.4);
    }

    #[test]
    fn test_windup_config_modern() {
        let attack = Attack {
            base_attack_speed: 1.0, // 1 秒攻击间隔
            windup_config: WindupConfig::Modern {
                attack_cast_time: 0.25,
                attack_total_time: 1.0,
            },
            ..Default::default()
        };

        // 新英雄公式: (0.25 / 1.0) * 1.0 = 0.25
        assert_eq!(attack.windup_time(), 0.25);
    }

    #[test]
    fn test_command_attack_same_target() {
        let mut app = create_test_app();

        let target_entity = app.world_mut().spawn_empty().id();
        let attacker = app
            .world_mut()
            .spawn(Attack {
                base_attack_speed: 1.0,                                     // 1秒攻击间隔
                windup_config: WindupConfig::Legacy { attack_offset: 0.0 }, // 0.3秒前摇
                ..Default::default()
            })
            .id();

        // 确保attacker有Target组件
        app.world_mut()
            .entity_mut(attacker)
            .insert(crate::core::Target(target_entity));

        // 第一次攻击命令
        app.world_mut().trigger_targets(CommandAttackCast, attacker);
        app.update();

        // 验证进入前摇状态
        let attack_state = app.world().get::<AttackState>(attacker).unwrap();
        assert!(attack_state.is_windup());
        assert_eq!(attack_state.current_target(), Some(target_entity));

        // 对同一目标发起第二次攻击命令 - 应该被忽略
        app.world_mut().trigger_targets(CommandAttackCast, attacker);
        app.update();

        // 状态应该保持不变
        let attack_state = app.world().get::<AttackState>(attacker).unwrap();
        assert!(attack_state.is_windup());
        assert_eq!(attack_state.current_target(), Some(target_entity));
    }

    #[test]
    fn test_command_attack_different_target() {
        let mut app = create_test_app();

        let target1 = app.world_mut().spawn_empty().id();
        let target2 = app.world_mut().spawn_empty().id();
        let attacker = app
            .world_mut()
            .spawn(Attack {
                base_attack_speed: 1.0,
                windup_config: WindupConfig::Legacy { attack_offset: 0.0 },
                ..Default::default()
            })
            .id();

        // 设置初始目标
        app.world_mut()
            .entity_mut(attacker)
            .insert(crate::core::Target(target1));

        // 攻击第一个目标
        app.world_mut().trigger_targets(CommandAttackCast, attacker);
        app.update();

        let attack_state = app.world().get::<AttackState>(attacker).unwrap();
        assert!(attack_state.is_windup());
        assert_eq!(attack_state.current_target(), Some(target1));

        // 更改目标并尝试攻击不同目标 - 在前摇期间应该被忽略
        app.world_mut()
            .entity_mut(attacker)
            .insert(crate::core::Target(target2));
        app.world_mut().trigger_targets(CommandAttackCast, attacker);
        app.update();

        // 目标应该保持为第一个目标
        let attack_state = app.world().get::<AttackState>(attacker).unwrap();
        assert!(attack_state.is_windup());
        assert_eq!(attack_state.current_target(), Some(target1));
    }

    #[test]
    fn test_attack_cancel_during_uncancellable_period() {
        let mut app = create_test_app();

        let target_entity = app.world_mut().spawn_empty().id();
        let attacker = app
            .world_mut()
            .spawn(Attack {
                base_attack_speed: 1.0,
                windup_config: WindupConfig::Legacy { attack_offset: 0.0 }, // 0.3秒前摇
                ..Default::default()
            })
            .id();

        // 确保attacker有Target组件
        app.world_mut()
            .entity_mut(attacker)
            .insert(crate::core::Target(target_entity));

        // 开始攻击
        app.world_mut().trigger_targets(CommandAttackCast, attacker);
        app.update();

        // 验证进入前摇状态
        let attack_state = app.world().get::<AttackState>(attacker).unwrap();
        assert!(attack_state.is_windup());

        // 推进到攻击生效前的不可取消期 (0.3 - 0.066 = 0.234秒后)
        // 此时距离攻击生效还有0.066秒，应该不可取消
        advance_time(&mut app, 0.234);

        // 验证此时不可取消
        let can_cancel = {
            let attack_state = app.world().get::<AttackState>(attacker).unwrap();
            let attack = app.world().get::<Attack>(attacker).unwrap();
            let current_time = app.world().resource::<Time<Fixed>>().elapsed_secs();
            let windup_time = attack.windup_time();
            attack_state.can_cancel(current_time, windup_time)
        };
        assert!(!can_cancel, "攻击生效前2帧应该不可取消");

        // 发送取消命令 - 应该被忽略
        app.world_mut()
            .trigger_targets(CommandAttackCancel, attacker);
        app.update();

        // 验证攻击没有被取消
        let attack_state = app.world().get::<AttackState>(attacker).unwrap();
        assert!(attack_state.is_windup(), "不可取消期内的攻击不应该被取消");
    }

    #[test]
    fn test_attack_cancel_after_uncancellable_period() {
        let mut app = create_test_app();

        let target_entity = app.world_mut().spawn_empty().id();
        let attacker = app
            .world_mut()
            .spawn(Attack {
                base_attack_speed: 1.0,
                windup_config: WindupConfig::Legacy { attack_offset: 0.0 }, // 0.3秒前摇
                ..Default::default()
            })
            .id();

        // 确保attacker有Target组件
        app.world_mut()
            .entity_mut(attacker)
            .insert(crate::core::Target(target_entity));

        // 开始攻击
        app.world_mut().trigger_targets(CommandAttackCast, attacker);
        app.update();

        // 等待到可取消期 (0.1秒后，距离攻击生效还有0.2秒，超过2帧宽限期)
        advance_time(&mut app, 0.1);

        // 验证现在可以取消
        let can_cancel = {
            let attack_state = app.world().get::<AttackState>(attacker).unwrap();
            let attack = app.world().get::<Attack>(attacker).unwrap();
            let current_time = app.world().resource::<Time<Fixed>>().elapsed_secs();
            let windup_time = attack.windup_time();
            attack_state.can_cancel(current_time, windup_time)
        };
        assert!(can_cancel, "攻击在不可取消期后应该可以取消");

        // 发送取消命令
        app.world_mut()
            .trigger_targets(CommandAttackCancel, attacker);
        app.update();

        // 验证攻击被取消
        let attack_state = app.world().get::<AttackState>(attacker).unwrap();
        assert!(attack_state.is_idle(), "可取消期内的攻击应该被取消");
    }

    #[test]
    fn test_attack_speed_change_during_windup() {
        let mut app = create_test_app();

        let target_entity = app.world_mut().spawn_empty().id();
        let attacker = app
            .world_mut()
            .spawn(Attack {
                base_attack_speed: 1.0, // 1秒攻击间隔，0.3秒前摇
                bonus_attack_speed: 0.0,
                windup_config: WindupConfig::Legacy { attack_offset: 0.0 },
                ..Default::default()
            })
            .id();

        // 确保attacker有Target组件
        app.world_mut()
            .entity_mut(attacker)
            .insert(crate::core::Target(target_entity));

        // 开始攻击
        app.world_mut().trigger_targets(CommandAttackCast, attacker);
        app.update();

        // 记录初始前摇时间
        let initial_windup_time = {
            let attack = app.world().get::<Attack>(attacker).unwrap();
            attack.windup_time()
        };
        assert_eq!(initial_windup_time, 0.3);

        // 在前摇期间增加攻击速度
        advance_time(&mut app, 0.1); // 前摇进行了0.1秒
        {
            let mut attack = app.world_mut().get_mut::<Attack>(attacker).unwrap();
            attack.bonus_attack_speed = 1.0; // 增加100%攻击速度
        }

        // 新的前摇时间应该更短，但已经开始的前摇不会改变生效时间
        let new_windup_time = {
            let attack = app.world().get::<Attack>(attacker).unwrap();
            attack.windup_time()
        };
        assert_eq!(new_windup_time, 0.3); // Legacy模式下前摇时间固定

        // 继续推进时间直到原定的前摇结束
        advance_time(&mut app, 0.2); // 总共0.3秒

        // 验证攻击生效
        let attack_state = app.world().get::<AttackState>(attacker).unwrap();
        assert!(attack_state.is_cooldown(), "攻击应该在原定时间生效");
    }

    #[test]
    fn test_full_attack_cycle() {
        let mut app = create_test_app();

        let target_entity = app.world_mut().spawn_empty().id();
        let attacker = app
            .world_mut()
            .spawn(Attack {
                base_attack_speed: 2.0,                                     // 0.5秒攻击间隔
                windup_config: WindupConfig::Legacy { attack_offset: 0.0 }, // 0.3秒前摇
                ..Default::default()
            })
            .id();

        // 确保attacker有Target组件
        app.world_mut()
            .entity_mut(attacker)
            .insert(crate::core::Target(target_entity));

        // 验证攻击时间计算
        {
            let attack = app.world().get::<Attack>(attacker).unwrap();
            assert!((attack.attack_interval() - 0.5).abs() < 0.001);
            assert!((attack.windup_time() - 0.3).abs() < 0.001);
            assert!((attack.cooldown_time() - 0.2).abs() < 0.001);
        }

        // 开始攻击
        app.world_mut().trigger_targets(CommandAttackCast, attacker);
        app.update();

        // 验证前摇状态
        {
            let attack_state = app.world().get::<AttackState>(attacker).unwrap();
            assert!(attack_state.is_windup(), "应该处于前摇状态");
        }

        // 推进到前摇结束 - 需要稍微多一点时间来确保状态转换
        advance_time(&mut app, 0.35);

        // 验证进入后摇状态
        {
            let attack_state = app.world().get::<AttackState>(attacker).unwrap();
            assert!(attack_state.is_cooldown(), "应该处于后摇状态");
        }

        // 推进到后摇结束
        advance_time(&mut app, 0.25);

        // 验证回到空闲状态
        {
            let attack_state = app.world().get::<AttackState>(attacker).unwrap();
            assert!(attack_state.is_idle(), "应该回到空闲状态");
        }
    }

    #[test]
    fn test_attack_reset_during_cooldown() {
        let mut app = create_test_app();

        let target_entity = app.world_mut().spawn_empty().id();
        let attacker = app
            .world_mut()
            .spawn(Attack {
                base_attack_speed: 1.0, // 1秒攻击间隔，0.3秒前摇，0.7秒后摇
                windup_config: WindupConfig::Legacy { attack_offset: 0.0 },
                ..Default::default()
            })
            .id();

        // 确保attacker有Target组件
        app.world_mut()
            .entity_mut(attacker)
            .insert(crate::core::Target(target_entity));

        // 完成一次攻击到后摇阶段
        app.world_mut().trigger_targets(CommandAttackCast, attacker);
        app.update();
        advance_time(&mut app, 0.3); // 完成前摇

        let attack_state = app.world().get::<AttackState>(attacker).unwrap();
        assert!(attack_state.is_cooldown());

        // 在后摇期间重置攻击 - 应该跳过后摇，立即开始下一次攻击
        app.world_mut()
            .trigger_targets(CommandAttackReset, attacker);
        app.update();

        // 验证立即进入新的前摇状态
        let attack_state = app.world().get::<AttackState>(attacker).unwrap();
        assert!(attack_state.is_windup());
        assert_eq!(attack_state.current_target(), Some(target_entity));
    }

    #[test]
    fn test_attack_reset_during_windup() {
        let mut app = create_test_app();

        let target_entity = app.world_mut().spawn_empty().id();
        let attacker = app
            .world_mut()
            .spawn(Attack {
                base_attack_speed: 1.0,
                windup_config: WindupConfig::Legacy { attack_offset: 0.0 },
                ..Default::default()
            })
            .id();

        // 确保attacker有Target组件
        app.world_mut()
            .entity_mut(attacker)
            .insert(crate::core::Target(target_entity));

        // 开始攻击
        app.world_mut().trigger_targets(CommandAttackCast, attacker);
        app.update();

        let attack_state = app.world().get::<AttackState>(attacker).unwrap();
        assert!(attack_state.is_windup());

        // 在前摇期间重置攻击 - 应该取消当前攻击
        app.world_mut()
            .trigger_targets(CommandAttackReset, attacker);
        app.update();

        // 验证回到空闲状态
        let attack_state = app.world().get::<AttackState>(attacker).unwrap();
        assert!(attack_state.is_idle());
    }

    #[test]
    fn test_modern_windup_with_attack_speed_scaling() {
        let mut app = create_test_app();

        let target_entity = app.world_mut().spawn_empty().id();
        let attacker = app
            .world_mut()
            .spawn(Attack {
                base_attack_speed: 1.0,  // 1秒基础攻击间隔
                bonus_attack_speed: 1.0, // 100%额外攻击速度，总共2.0攻击速度，0.5秒攻击间隔
                windup_config: WindupConfig::Modern {
                    attack_cast_time: 0.25,
                    attack_total_time: 1.0,
                }, // 25%前摇比例
                ..Default::default()
            })
            .id();

        let attack = app.world().get::<Attack>(attacker).unwrap();
        // 前摇时间应该是: (0.25/1.0) * 0.5 = 0.125秒
        assert_eq!(attack.windup_time(), 0.125);
        // 后摇时间应该是: 0.5 - 0.125 = 0.375秒
        assert_eq!(attack.cooldown_time(), 0.375);

        // 确保attacker有Target组件
        app.world_mut()
            .entity_mut(attacker)
            .insert(crate::core::Target(target_entity));

        // 测试完整攻击周期
        app.world_mut().trigger_targets(CommandAttackCast, attacker);
        app.update();

        // 前摇阶段
        let attack_state = app.world().get::<AttackState>(attacker).unwrap();
        assert!(attack_state.is_windup());

        // 推进到前摇结束
        advance_time(&mut app, 0.125);
        let attack_state = app.world().get::<AttackState>(attacker).unwrap();
        assert!(attack_state.is_cooldown());

        // 推进到后摇结束
        advance_time(&mut app, 0.375);
        let attack_state = app.world().get::<AttackState>(attacker).unwrap();
        assert!(attack_state.is_idle());
    }

    #[test]
    fn test_uncancellable_grace_period() {
        let attack = Attack {
            windup_config: WindupConfig::Modern {
                attack_cast_time: 0.05,
                attack_total_time: 0.95,
            }, // 0.05秒前摇 (少于宽限期)
            base_attack_speed: 1.0,
            ..Default::default()
        };

        // 前摇时间少于宽限期，所以整个前摇都不可取消
        assert!(attack.windup_time() < UNCANCELLABLE_GRACE_PERIOD);
    }
}
