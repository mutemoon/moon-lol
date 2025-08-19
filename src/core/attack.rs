use bevy::prelude::*;

use crate::core::Target;

#[derive(Default)]
pub struct PluginAttack;

impl Plugin for PluginAttack {
    fn build(&self, app: &mut App) {
        app.add_event::<EventAttackCast>();
        app.add_event::<EventAttackDone>();
        app.add_event::<EventAttackCooldown>();
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
            (attack_state_machine_system, check_target_validity_system),
        );
    }
}

/// 攻击组件 - 包含攻击的基础属性
#[derive(Component, Clone)]
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
            // 修复：直接对前摇时间应用修正系数
            base_windup * self.windup_modifier
        }
    }

    /// 计算后摇时间
    pub fn cooldown_time(&self) -> f32 {
        self.attack_interval() - self.windup_time()
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
#[derive(Component)]
pub struct AttackState {
    pub status: AttackStatus,
    /// 当前阶段开始的时间
    pub cast_time: f32,
    /// 是否继续攻击
    pub continue_attack: bool,
}

impl Default for AttackState {
    fn default() -> Self {
        Self {
            status: AttackStatus::Idle,
            cast_time: 0.0,
            continue_attack: true,
        }
    }
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
pub struct EventAttackCast {
    pub target: Entity,
}

#[derive(Event, Debug)]
pub struct EventAttackDone {
    pub target: Entity,
}

#[derive(Event, Debug)]
pub struct EventAttackCooldown;

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
                let elapsed = current_time - self.cast_time;
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
    mut query: Query<(&mut AttackState, &Target, &Attack)>,
    time: Res<Time<Fixed>>,
) {
    let entity = trigger.target();

    if let Ok((mut attack_state, target, attack)) = query.get_mut(entity) {
        info!("on_command_attack_cast: {:?} -> {:?}", entity, target.0);

        let current_time = time.elapsed_secs();

        // 检查当前状态
        match &attack_state.status {
            AttackStatus::Idle => {
                // 空闲状态：直接开始攻击
                attack_state.status = AttackStatus::Windup { target: target.0 };
                attack_state.cast_time = current_time;
                commands.trigger_targets(EventAttackCast { target: target.0 }, entity);
            }
            AttackStatus::Windup {
                target: current_target,
            } => {
                // 前摇状态：检查目标是否相同
                if *current_target == target.0 {
                    // 攻击同一个目标，不做任何改变
                    return;
                }

                // 不同目标：检查是否可以取消
                let windup_time = attack.windup_time();
                if attack_state.can_cancel(current_time, windup_time) {
                    // 可以取消：立即切换到新目标
                    attack_state.status = AttackStatus::Windup { target: target.0 };
                    attack_state.cast_time = current_time;
                    commands.trigger_targets(EventAttackCancel, entity);
                    commands.trigger_targets(EventAttackCast { target: target.0 }, entity);
                }
                // 如果不可取消，则忽略新命令
            }
            AttackStatus::Cooldown { .. } => {
                // 后摇状态：忽略新命令，等待当前攻击完成
                // 但可以更新目标，为下一次攻击做准备
                // 这里不处理，让系统自然完成当前攻击
            }
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
                attack_state.cast_time = time.elapsed_secs();
                commands.trigger_targets(EventAttackCancel, entity);
            }
            // 在后摇阶段重置 = 跳过后摇，立刻开始下一次攻击 (好事)
            AttackStatus::Cooldown { target } => {
                info!("Attack reset during cooldown - skipping to next attack");
                attack_state.status = AttackStatus::Windup { target: *target };
                attack_state.cast_time = time.elapsed_secs();
                commands.trigger_targets(EventAttackReset, entity);
            }
            _ => {
                // 在其他状态下重置攻击计时
                attack_state.status = AttackStatus::Idle;
                attack_state.cast_time = time.elapsed_secs();
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
        attack_state.continue_attack = false;

        // 检查是否可以取消
        if attack_state.can_cancel(current_time, windup_time) {
            attack_state.status = AttackStatus::Idle;
            attack_state.cast_time = current_time;
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
                let elapsed = current_time - attack_state.cast_time;
                let windup_time = attack.windup_time();

                // 检查前摇是否完成
                if elapsed >= windup_time {
                    attack_state.status = AttackStatus::Cooldown { target: *target };
                    attack_state.cast_time = current_time;

                    commands.trigger_targets(EventAttackDone { target: *target }, entity);
                }
            }

            AttackStatus::Cooldown { target: _ } => {
                let elapsed = current_time - attack_state.cast_time;
                let cooldown_time = attack.cooldown_time();

                // 检查后摇是否完成
                if elapsed >= cooldown_time {
                    attack_state.status = AttackStatus::Idle;
                    attack_state.cast_time = current_time;

                    commands.trigger_targets(EventAttackCooldown, entity);

                    if attack_state.continue_attack {
                        commands.trigger_targets(CommandAttackCast, entity);
                    }
                }
            }

            AttackStatus::Idle => {}
        }
    }
}

/// 检查目标有效性的系统
/// 如果攻击者正在攻击的目标不存在，则取消攻击
fn check_target_validity_system(
    mut commands: Commands,
    query: Query<(Entity, &AttackState)>,
    entities: Query<Entity>,
) {
    for (attacker, attack_state) in query.iter() {
        if let Some(target) = attack_state.current_target() {
            // 检查目标实体是否仍然存在
            if entities.get(target).is_err() {
                // 目标不存在，取消攻击
                commands.trigger_targets(CommandAttackCancel, attacker);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{PluginCommand, PluginTarget};

    // ===== 测试常量定义 =====
    const TEST_FPS: f32 = 30.0;
    const EPSILON: f32 = 1e-6;

    // ===== 测试辅助工具 =====

    /// 测试装置 - 封装通用的测试设置
    struct TestHarness {
        app: App,
        attacker: Entity,
        target: Entity,
    }

    impl TestHarness {
        /// 创建新的测试装置
        fn new() -> Self {
            let mut app = App::new();
            app.add_plugins(MinimalPlugins);
            app.add_plugins(PluginTarget);
            app.add_plugins(PluginAttack);
            app.add_plugins(PluginCommand);
            app.insert_resource(Time::<Fixed>::from_hz(TEST_FPS as f64));

            let target = app.world_mut().spawn_empty().id();
            let attacker = app.world_mut().spawn_empty().id();

            TestHarness {
                app,
                attacker,
                target,
            }
        }

        /// 使用构建者模式配置攻击者
        fn with_attacker(mut self, attack_component: Attack) -> Self {
            self.app.world_mut().entity_mut(self.attacker).insert((
                attack_component,
                AttackState::default(),
                crate::core::Target(self.target),
            ));
            self
        }

        /// 创建额外的目标实体
        fn spawn_target(&mut self) -> Entity {
            self.app.world_mut().spawn_empty().id()
        }

        /// 切换攻击者的目标
        fn switch_target(&mut self, new_target: Entity) {
            self.app
                .world_mut()
                .entity_mut(self.attacker)
                .insert(crate::core::Target(new_target));
        }

        /// 推进时间
        fn advance_time(&mut self, seconds: f32) {
            let ticks = (seconds * TEST_FPS).ceil() as u32;
            for _ in 0..ticks {
                let mut time = self.app.world_mut().resource_mut::<Time<Fixed>>();
                time.advance_by(std::time::Duration::from_secs_f64(1.0 / TEST_FPS as f64));
                drop(time);
                self.app.world_mut().run_schedule(FixedUpdate);
            }
        }

        /// 发送攻击命令
        fn attack(&mut self) {
            self.app
                .world_mut()
                .trigger_targets(CommandAttackCast, self.attacker);
            self.app.update();
        }

        /// 发送取消命令
        fn cancel(&mut self) {
            self.app
                .world_mut()
                .trigger_targets(CommandAttackCancel, self.attacker);
            self.app.update();
        }

        /// 发送重置命令
        fn reset(&mut self) {
            self.app
                .world_mut()
                .trigger_targets(CommandAttackReset, self.attacker);
            self.app.update();
        }

        /// 获取攻击状态
        fn attack_state(&self) -> &AttackState {
            self.app.world().get::<AttackState>(self.attacker).unwrap()
        }

        /// 获取攻击组件
        fn attack_component(&self) -> &Attack {
            self.app.world().get::<Attack>(self.attacker).unwrap()
        }

        /// 获取当前时间
        fn current_time(&self) -> f32 {
            self.app.world().resource::<Time<Fixed>>().elapsed_secs()
        }

        /// 检查攻击是否可以取消
        fn can_cancel(&self) -> bool {
            let attack_state = self.attack_state();
            let attack = self.attack_component();
            let current_time = self.current_time();
            let windup_time = attack.windup_time();
            attack_state.can_cancel(current_time, windup_time)
        }

        /// 移除目标实体（模拟死亡）
        fn kill_target(&mut self, target: Entity) {
            self.app.world_mut().entity_mut(target).despawn();
        }
    }

    // ===== 自定义断言函数 =====

    /// 断言攻击状态为空闲
    fn assert_attack_state_is_idle(harness: &TestHarness, message: &str) {
        let state = harness.attack_state();
        assert!(
            state.is_idle(),
            "{} (expected Idle, found {:?})",
            message,
            state.status
        );
    }

    /// 断言攻击状态为前摇
    fn assert_attack_state_is_windup(harness: &TestHarness, message: &str) {
        let state = harness.attack_state();
        assert!(
            state.is_windup(),
            "{} (expected Windup, found {:?})",
            message,
            state.status
        );
    }

    /// 断言攻击状态为后摇
    fn assert_attack_state_is_cooldown(harness: &TestHarness, message: &str) {
        let state = harness.attack_state();
        assert!(
            state.is_cooldown(),
            "{} (expected Cooldown, found {:?})",
            message,
            state.status
        );
    }

    /// 断言攻击目标
    fn assert_attack_target(harness: &TestHarness, expected_target: Entity, message: &str) {
        let state = harness.attack_state();
        assert_eq!(state.current_target(), Some(expected_target), "{}", message);
    }

    /// 断言浮点数相等（使用容差）
    macro_rules! assert_float_eq {
        ($left:expr, $right:expr, $tol:expr) => {
            assert!(
                ($left - $right).abs() < $tol,
                "assertion failed: `(left ≈ right)` (left: `{:?}`, right: `{:?}`, tolerance: `{:?}`)",
                $left,
                $right,
                $tol
            );
        };
        ($left:expr, $right:expr) => {
            assert_float_eq!($left, $right, EPSILON);
        };
    }

    // ===== 一、核心状态机与流程 (Core State Machine & Flow) =====

    /// 目标 1：完整的攻击循环
    #[test]
    fn test_complete_attack_cycle() {
        let mut harness = TestHarness::new().with_attacker(Attack {
            base_attack_speed: 1.0,                                     // 1秒攻击间隔
            windup_config: WindupConfig::Legacy { attack_offset: 0.0 }, // 0.3秒前摇
            ..Default::default()
        });

        // 记录初始时间
        let initial_time = harness.current_time();

        // 开始攻击
        harness.attack();
        assert_attack_state_is_windup(&harness, "攻击命令应该触发前摇状态");
        assert_attack_target(&harness, harness.target, "攻击目标应该正确");
        assert!(
            harness.attack_state().cast_time >= initial_time,
            "cast_time应该被更新"
        );

        // 推进到前摇结束
        harness.advance_time(0.3);
        assert_attack_state_is_cooldown(&harness, "前摇结束后应该进入后摇状态");
        assert_attack_target(&harness, harness.target, "后摇期间目标应该保持不变");

        // 推进到后摇结束
        harness.advance_time(0.7);
        assert_attack_state_is_windup(&harness, "后摇结束后应该自动开始下一次攻击");
        assert_attack_target(&harness, harness.target, "下一次攻击的目标应该相同");
    }

    /// 目标 2：连续攻击同一目标
    #[test]
    fn test_consecutive_attacks_same_target() {
        let mut harness = TestHarness::new().with_attacker(Attack {
            base_attack_speed: 1.0,                                     // 1秒攻击间隔
            windup_config: WindupConfig::Legacy { attack_offset: 0.0 }, // 0.3秒前摇
            ..Default::default()
        });

        // 第一次攻击
        harness.attack();
        assert_attack_state_is_windup(&harness, "第一次攻击应该触发前摇状态");

        // 完成第一次攻击周期
        harness.advance_time(1.0);
        assert_attack_state_is_windup(&harness, "后摇结束后应该自动开始下一次攻击");
        assert_attack_target(&harness, harness.target, "下一次攻击的目标应该相同");

        // 手动发送第二次攻击命令（测试同目标不重新开始）
        let cast_time_before = harness.attack_state().cast_time;
        harness.attack();
        assert_attack_state_is_windup(&harness, "第二次攻击应该保持前摇状态");
        assert_attack_target(&harness, harness.target, "攻击目标应该保持不变");
        assert_eq!(
            harness.attack_state().cast_time,
            cast_time_before,
            "攻击同一目标时不应该重新开始"
        );

        // 验证攻击时间配置正确
        let attack = harness.attack_component();
        assert_float_eq!(attack.windup_time(), 0.3);
        assert_float_eq!(attack.cooldown_time(), 0.7);
    }

    /// 目标 3：攻击中切换目标
    #[test]
    fn test_switch_target_during_attack() {
        let mut harness = TestHarness::new().with_attacker(Attack {
            base_attack_speed: 1.0,
            windup_config: WindupConfig::Legacy { attack_offset: 0.0 },
            ..Default::default()
        });

        let target2 = harness.spawn_target();

        // 开始攻击第一个目标
        harness.attack();
        assert_attack_state_is_windup(&harness, "攻击命令应该触发前摇状态");
        assert_attack_target(&harness, harness.target, "初始攻击目标应该正确");

        // 完成前摇，进入后摇
        harness.advance_time(0.3);
        assert_attack_state_is_cooldown(&harness, "前摇结束后应该进入后摇状态");

        // 在后摇期间切换目标
        harness.switch_target(target2);

        // 等待后摇结束
        harness.advance_time(0.7);
        assert_attack_state_is_windup(&harness, "后摇结束后应该自动开始下一次攻击");
        assert_attack_target(&harness, target2, "下一次攻击的目标应该是新目标");

        // 手动发送攻击新目标的命令（测试同目标不重新开始）
        let cast_time_before = harness.attack_state().cast_time;
        harness.attack();
        assert_attack_state_is_windup(&harness, "攻击新目标应该保持前摇状态");
        assert_attack_target(&harness, target2, "攻击目标应该保持为新目标");
        assert_eq!(
            harness.attack_state().cast_time,
            cast_time_before,
            "攻击同一目标时不应该重新开始"
        );
    }

    // ===== 二、攻击取消机制 (Attack Cancellation Mechanics) =====

    /// 目标 4：在"可取消"阶段取消前摇
    #[test]
    fn test_cancel_attack_during_cancellable_windup() {
        let mut harness = TestHarness::new().with_attacker(Attack {
            base_attack_speed: 1.0,
            windup_config: WindupConfig::Legacy { attack_offset: 0.0 }, // 0.3秒前摇
            ..Default::default()
        });

        // 开始攻击
        harness.attack();
        assert_attack_state_is_windup(&harness, "攻击命令应该触发前摇状态");

        // 等待到可取消期 (0.1秒后，距离攻击生效还有0.2秒，超过2帧宽限期)
        harness.advance_time(0.1);
        assert!(harness.can_cancel(), "攻击在不可取消期后应该可以取消");

        // 发送取消命令
        harness.cancel();
        assert_attack_state_is_idle(&harness, "可取消期内的攻击应该被取消");

        // 验证可以立即响应新指令
        harness.attack();
        assert_attack_state_is_windup(&harness, "应该能立即开始新的攻击");
    }

    /// 目标 5：在"不可取消"的宽限期内尝试取消前摇
    #[test]
    fn test_cancel_attack_during_uncancellable_grace_period() {
        let mut harness = TestHarness::new().with_attacker(Attack {
            base_attack_speed: 1.0,
            windup_config: WindupConfig::Legacy { attack_offset: 0.0 }, // 0.3秒前摇
            ..Default::default()
        });

        // 开始攻击
        harness.attack();
        assert_attack_state_is_windup(&harness, "攻击命令应该触发前摇状态");

        // 推进到攻击生效前的不可取消期 (0.3 - 0.066 = 0.234秒后)
        harness.advance_time(0.234);
        assert!(!harness.can_cancel(), "攻击生效前2帧应该不可取消");

        // 发送取消命令 - 应该被忽略
        harness.cancel();
        assert_attack_state_is_windup(&harness, "不可取消期内的攻击不应该被取消");

        // 继续推进时间，攻击应该正常完成
        harness.advance_time(0.066);
        assert_attack_state_is_cooldown(&harness, "攻击应该正常进入后摇状态");
    }

    // ===== 三、攻击重置 (走A) 机制 (Attack Reset / Kiting) =====

    /// 目标 6：在后摇 (Cooldown) 期间重置攻击
    #[test]
    fn test_attack_reset_during_cooldown() {
        let mut harness = TestHarness::new().with_attacker(Attack {
            base_attack_speed: 1.0, // 1秒攻击间隔，0.3秒前摇，0.7秒后摇
            windup_config: WindupConfig::Legacy { attack_offset: 0.0 },
            ..Default::default()
        });

        // 完成前摇，进入后摇阶段
        harness.attack();
        harness.advance_time(0.3); // 完成前摇
        assert_attack_state_is_cooldown(&harness, "应该进入后摇状态");

        // 在后摇期间重置攻击 - 应该跳过后摇，立即开始下一次攻击
        harness.reset();
        assert_attack_state_is_windup(&harness, "重置后应该立即进入新的前摇状态");
        assert_attack_target(&harness, harness.target, "重置后目标应该保持不变");
    }

    /// 测试攻击重置事件触发
    #[test]
    fn test_attack_reset_event_triggering() {
        let mut harness = TestHarness::new().with_attacker(Attack {
            base_attack_speed: 1.0,
            windup_config: WindupConfig::Legacy { attack_offset: 0.0 },
            ..Default::default()
        });

        // 完成前摇，进入后摇
        harness.attack();
        harness.advance_time(0.3);
        assert_attack_state_is_cooldown(&harness, "应该进入后摇状态");

        // 在后摇期间重置攻击
        harness.reset();
        assert_attack_state_is_windup(&harness, "重置后应该进入前摇状态");
    }

    // ===== 四、攻击速度影响 (Impact of Attack Speed) =====

    /// 目标 7：攻速变化对攻击时间的影响
    #[test]
    fn test_attack_speed_impact_on_timing() {
        let mut harness = TestHarness::new().with_attacker(Attack {
            base_attack_speed: 0.625, // 基础攻速
            bonus_attack_speed: 0.0,
            windup_config: WindupConfig::Modern {
                attack_cast_time: 0.25,
                attack_total_time: 1.0,
            }, // 使用Modern模式，前摇时间会随攻速变化
            ..Default::default()
        });

        // 记录初始攻击时间
        let initial_attack = harness.attack_component();
        let initial_interval = initial_attack.attack_interval();
        let initial_windup = initial_attack.windup_time();
        let initial_cooldown = initial_attack.cooldown_time();

        // 提升攻击速度
        {
            let mut attack = harness
                .app
                .world_mut()
                .get_mut::<Attack>(harness.attacker)
                .unwrap();
            attack.bonus_attack_speed = 1.0; // 100%额外攻击速度
        }

        // 验证攻击时间缩短
        let new_attack = harness.attack_component();
        let new_interval = new_attack.attack_interval();
        let new_windup = new_attack.windup_time();
        let new_cooldown = new_attack.cooldown_time();

        assert!(new_interval < initial_interval, "攻击间隔应该缩短");
        assert!(new_windup < initial_windup, "前摇时间应该缩短");
        assert!(new_cooldown < initial_cooldown, "后摇时间应该缩短");

        // 验证总时间缩短
        let total_initial = initial_windup + initial_cooldown;
        let total_new = new_windup + new_cooldown;
        assert!(total_new < total_initial, "总攻击时间应该缩短");
    }

    /// 目标 8：攻击速度达到上限
    #[test]
    fn test_attack_speed_cap() {
        let attack = Attack {
            base_attack_speed: 0.625,
            bonus_attack_speed: 10.0, // 1000%额外攻击速度（远超上限）
            attack_speed_cap: 2.5,
            ..Default::default()
        };

        // 应该被限制在2.5
        assert_float_eq!(attack.current_attack_speed(), 2.5);

        // 攻击间隔应该是最小值
        let min_interval = 1.0 / 2.5;
        assert_float_eq!(attack.attack_interval(), min_interval);

        // 进一步增加bonus_attack_speed不应该改变结果
        let attack_higher = Attack {
            bonus_attack_speed: 20.0,
            ..attack
        };
        assert_float_eq!(attack_higher.current_attack_speed(), 2.5);
        assert_float_eq!(attack_higher.attack_interval(), min_interval);
    }

    /// 目标 9：极高攻速下前摇完全不可取消
    #[test]
    fn test_extremely_high_attack_speed_uncancellable() {
        let attack = Attack {
            base_attack_speed: 10.0, // 极高基础攻速
            bonus_attack_speed: 5.0, // 极高额外攻速
            windup_config: WindupConfig::Modern {
                attack_cast_time: 0.05,
                attack_total_time: 1.0,
            },
            ..Default::default()
        };

        // 计算前摇时间
        let windup_time = attack.windup_time();
        println!(
            "Windup time: {}, Grace period: {}",
            windup_time, UNCANCELLABLE_GRACE_PERIOD
        );

        // 如果前摇时间小于等于宽限期，则整个前摇都不可取消
        if windup_time <= UNCANCELLABLE_GRACE_PERIOD {
            // 在这种情况下，任何取消尝试都应该被忽略
            let attack_state = AttackState {
                status: AttackStatus::Windup {
                    target: Entity::PLACEHOLDER,
                },
                cast_time: 0.0,
                continue_attack: true,
            };

            let current_time = 0.1; // 前摇进行中
            let can_cancel = attack_state.can_cancel(current_time, windup_time);
            assert!(!can_cancel, "极高攻速下前摇应该完全不可取消");
        }
    }

    // ===== 五、前摇配置与修正 (Windup Configuration & Modifiers) =====

    /// 验证Legacy前摇公式
    #[test]
    fn test_legacy_windup_formula() {
        let test_cases = [
            (0.1, 0.4),  // attack_offset: 0.1, expected: 0.3 + 0.1 = 0.4
            (-0.1, 0.2), // attack_offset: -0.1, expected: 0.3 - 0.1 = 0.2
            (0.0, 0.3),  // attack_offset: 0.0, expected: 0.3
        ];

        for (attack_offset, expected_windup) in test_cases {
            let attack = Attack {
                base_attack_speed: 1.0,
                windup_config: WindupConfig::Legacy { attack_offset },
                ..Default::default()
            };

            assert_float_eq!(attack.windup_time(), expected_windup);
        }
    }

    /// 验证Modern前摇公式
    #[test]
    fn test_modern_windup_formula() {
        let test_cases = [
            // (attack_cast_time, attack_total_time, base_speed, expected_windup)
            (0.25, 1.0, 1.0, 0.25),      // (0.25/1.0) * 1.0 = 0.25
            (0.25, 1.0, 2.0, 0.125),     // (0.25/1.0) * 0.5 = 0.125
            (0.3, 1.2, 1.5, 0.16666667), // (0.3/1.2) * (1/1.5) = 0.25 * 0.6667 = 0.1667
        ];

        for (attack_cast_time, attack_total_time, base_speed, expected_windup) in test_cases {
            let attack = Attack {
                base_attack_speed: base_speed,
                windup_config: WindupConfig::Modern {
                    attack_cast_time,
                    attack_total_time,
                },
                ..Default::default()
            };

            assert_float_eq!(attack.windup_time(), expected_windup);
        }
    }

    /// 验证windup_modifier的效果
    #[test]
    fn test_windup_modifier_effect() {
        let test_cases = [
            (1.0, 0.3),  // 无修正
            (0.5, 0.15), // 缩短50%
            (1.5, 0.45), // 延长50%
            (0.1, 0.03), // 极快前摇
        ];

        for (modifier, expected_windup) in test_cases {
            let attack = Attack {
                base_attack_speed: 1.0,
                windup_config: WindupConfig::Modern {
                    attack_cast_time: 0.3,
                    attack_total_time: 1.0,
                },
                windup_modifier: modifier,
                ..Default::default()
            };

            assert_float_eq!(attack.windup_time(), expected_windup);
        }

        // 测试Legacy模式下的修正系数
        let legacy_attack = Attack {
            base_attack_speed: 1.0,
            windup_config: WindupConfig::Legacy { attack_offset: 0.1 },
            windup_modifier: 0.8,
            ..Default::default()
        };
        let expected_legacy = (0.3 + 0.1) * 0.8; // 0.32
        assert_float_eq!(legacy_attack.windup_time(), expected_legacy);
    }

    // ===== 六、目标与距离验证 (Targeting & Range) =====

    /// 目标在攻击前摇期间死亡或失效
    #[test]
    fn test_target_death_during_windup() {
        let mut harness = TestHarness::new().with_attacker(Attack {
            base_attack_speed: 1.0,
            windup_config: WindupConfig::Legacy { attack_offset: 0.0 },
            ..Default::default()
        });

        // 开始攻击
        harness.attack();
        assert_attack_state_is_windup(&harness, "攻击命令应该触发前摇状态");

        // 移除目标（模拟死亡）
        harness.kill_target(harness.target);

        // 手动运行FixedUpdate调度来触发目标有效性检查
        harness.app.world_mut().run_schedule(FixedUpdate);

        // 验证攻击被取消（现在有了check_target_validity_system）
        assert_attack_state_is_idle(&harness, "目标失效后攻击应该被取消");
    }

    /// 目标在攻击前摇期间移出攻击范围
    #[test]
    fn test_target_out_of_range_during_windup() {
        // 这个测试需要移动系统和距离检测系统
        // 暂时跳过，等待相关系统实现
    }

    // ===== 七、辅助测试函数和浮点数精度 =====

    /// 攻击速度计算验证
    #[test]
    fn test_attack_speed_calculations() {
        let attack = Attack {
            base_attack_speed: 0.625,
            bonus_attack_speed: 1.0, // 100%额外攻击速度
            attack_speed_cap: 2.5,
            windup_config: WindupConfig::Legacy { attack_offset: 0.0 },
            ..Default::default()
        };

        // 0.625 * (1 + 1.0) = 1.25每秒攻击次数
        assert_float_eq!(attack.current_attack_speed(), 1.25);
        // 1 / 1.25 = 0.8秒每次攻击
        assert_float_eq!(attack.attack_interval(), 0.8);
        // 0.3 + 0.0 = 0.3秒前摇
        assert_float_eq!(attack.windup_time(), 0.3);
        // 0.8 - 0.3 = 0.5秒后摇
        assert_float_eq!(attack.cooldown_time(), 0.5);
    }

    /// 浮点数精度测试
    #[test]
    fn test_floating_point_precision() {
        let attack = Attack {
            base_attack_speed: 0.625,
            bonus_attack_speed: 0.6, // 60%额外攻击速度
            windup_config: WindupConfig::Modern {
                attack_cast_time: 0.25,
                attack_total_time: 1.0,
            },
            ..Default::default()
        };

        // 计算期望值
        let expected_speed = 0.625 * (1.0 + 0.6); // 1.0
        let expected_interval = 1.0 / expected_speed; // 1.0
        let expected_windup = 0.25 / 1.0 * expected_interval; // 0.25

        // 使用容差比较
        assert_float_eq!(attack.current_attack_speed(), expected_speed);
        assert_float_eq!(attack.attack_interval(), expected_interval);
        assert_float_eq!(attack.windup_time(), expected_windup);
    }

    /// 攻击状态查询函数测试
    #[test]
    fn test_attack_state_queries() {
        // 测试默认空闲状态
        let attack_state = AttackState::default();
        assert!(attack_state.is_idle());
        assert!(!attack_state.is_windup());
        assert!(!attack_state.is_cooldown());
        assert!(!attack_state.is_attacking());
        assert_eq!(attack_state.current_target(), None);

        // 测试前摇状态
        let windup_state = AttackState {
            status: AttackStatus::Windup {
                target: Entity::PLACEHOLDER,
            },
            cast_time: 0.0,
            continue_attack: true,
        };
        assert!(!windup_state.is_idle());
        assert!(windup_state.is_windup());
        assert!(!windup_state.is_cooldown());
        assert!(windup_state.is_attacking());
        assert_eq!(windup_state.current_target(), Some(Entity::PLACEHOLDER));

        // 测试后摇状态
        let cooldown_state = AttackState {
            status: AttackStatus::Cooldown {
                target: Entity::PLACEHOLDER,
            },
            cast_time: 0.0,
            continue_attack: true,
        };
        assert!(!cooldown_state.is_idle());
        assert!(!cooldown_state.is_windup());
        assert!(cooldown_state.is_cooldown());
        assert!(cooldown_state.is_attacking());
        assert_eq!(cooldown_state.current_target(), Some(Entity::PLACEHOLDER));
    }

    /// 在前摇期间重置攻击
    #[test]
    fn test_attack_reset_during_windup() {
        let mut harness = TestHarness::new().with_attacker(Attack {
            base_attack_speed: 1.0,
            windup_config: WindupConfig::Legacy { attack_offset: 0.0 },
            ..Default::default()
        });

        // 开始攻击
        harness.attack();
        assert_attack_state_is_windup(&harness, "攻击命令应该触发前摇状态");

        // 在前摇期间重置攻击 - 应该取消当前攻击
        harness.reset();
        assert_attack_state_is_idle(&harness, "前摇期间重置应该回到空闲状态");
    }

    /// Modern模式下的攻速缩放测试
    #[test]
    fn test_modern_windup_with_attack_speed_scaling() {
        let mut harness = TestHarness::new().with_attacker(Attack {
            base_attack_speed: 1.0,  // 1秒基础攻击间隔
            bonus_attack_speed: 1.0, // 100%额外攻击速度，总共2.0攻击速度，0.5秒攻击间隔
            windup_config: WindupConfig::Modern {
                attack_cast_time: 0.25,
                attack_total_time: 1.0,
            }, // 25%前摇比例
            ..Default::default()
        });

        let attack = harness.attack_component();
        // 前摇时间应该是: (0.25/1.0) * 0.5 = 0.125秒
        assert_float_eq!(attack.windup_time(), 0.125);
        // 后摇时间应该是: 0.5 - 0.125 = 0.375秒
        assert_float_eq!(attack.cooldown_time(), 0.375);

        // 测试完整攻击周期
        harness.attack();
        assert_attack_state_is_windup(&harness, "攻击命令应该触发前摇状态");

        // 推进到前摇结束
        harness.advance_time(0.125);
        assert_attack_state_is_cooldown(&harness, "前摇结束后应该进入后摇状态");

        // 推进到后摇结束
        harness.advance_time(0.375);
        assert_attack_state_is_windup(&harness, "后摇结束后应该自动开始下一次攻击");
        assert_attack_target(&harness, harness.target, "下一次攻击的目标应该相同");
    }

    /// 不可取消宽限期测试
    #[test]
    fn test_uncancellable_grace_period() {
        let attack = Attack {
            windup_config: WindupConfig::Modern {
                attack_cast_time: 0.05,
                attack_total_time: 0.95,
            }, // 0.05秒前摇（少于宽限期）
            base_attack_speed: 1.0,
            ..Default::default()
        };

        // 前摇时间少于宽限期，所以整个前摇都不可取消
        assert!(attack.windup_time() < UNCANCELLABLE_GRACE_PERIOD);
    }

    // ===== 八、目标切换与攻击取消的交互 (Target Switching & Attack Cancellation Interaction) =====

    /// 测试场景1：攻击目标A，在不可取消期间切换到目标B
    /// 期望：当前攻击仍攻击目标A，但下一次自动攻击应该攻击目标B
    #[test]
    fn test_new_target_command_during_uncancellable_period() {
        let mut harness = TestHarness::new().with_attacker(Attack {
            base_attack_speed: 1.0,                                     // 1秒攻击间隔
            windup_config: WindupConfig::Legacy { attack_offset: 0.0 }, // 0.3秒前摇
            ..Default::default()
        });

        let target_b = harness.spawn_target();

        // 开始攻击目标A
        harness.attack();
        assert_attack_state_is_windup(&harness, "攻击命令应该触发前摇状态");
        assert_attack_target(&harness, harness.target, "攻击目标应该是A");

        // 推进到攻击生效前的不可取消期 (0.3 - 0.066 = 0.234秒后)
        harness.advance_time(0.234);
        assert!(!harness.can_cancel(), "攻击生效前2帧应该不可取消");

        // 在不可取消期间，切换目标到B
        harness.switch_target(target_b);

        // 继续推进时间，攻击应该正常完成，目标仍为A
        harness.advance_time(0.066);
        assert_attack_state_is_cooldown(&harness, "攻击应该正常进入后摇状态");
        assert_attack_target(&harness, harness.target, "当前攻击的目标应该仍然是A");

        // 等待后摇结束，系统应该自动开始下一次攻击
        harness.advance_time(0.7);
        assert_attack_state_is_windup(&harness, "应该自动开始下一次攻击");
        assert_attack_target(&harness, target_b, "下一次攻击的目标应该是B");
    }

    /// 测试场景2：攻击目标A，在可取消期间攻击目标B
    /// 期望：会立即取消当前攻击，重新开始攻击目标B且重新计时
    #[test]
    fn test_new_target_command_during_cancellable_period() {
        let mut harness = TestHarness::new().with_attacker(Attack {
            base_attack_speed: 1.0,                                     // 1秒攻击间隔
            windup_config: WindupConfig::Legacy { attack_offset: 0.0 }, // 0.3秒前摇
            ..Default::default()
        });

        let target_b = harness.spawn_target();

        // 开始攻击目标A
        harness.attack();
        assert_attack_state_is_windup(&harness, "攻击命令应该触发前摇状态");
        assert_attack_target(&harness, harness.target, "攻击目标应该是A");

        // 等待到可取消期 (0.1秒后，距离攻击生效还有0.2秒，超过2帧宽限期)
        harness.advance_time(0.1);
        assert!(harness.can_cancel(), "攻击在不可取消期后应该可以取消");

        // 记录当前时间，用于验证重新计时
        let time_before_switch = harness.current_time();

        // 在可取消期间，切换目标到B并发送攻击命令
        harness.switch_target(target_b);
        harness.attack();

        // 验证立即切换到目标B的前摇状态
        assert_attack_state_is_windup(&harness, "应该立即开始攻击目标B");
        assert_attack_target(&harness, target_b, "当前攻击的目标应该是B");

        // 验证攻击时间重新计时
        assert!(
            harness.attack_state().cast_time >= time_before_switch,
            "攻击时间应该重新计时"
        );

        // 验证攻击B的完整流程
        harness.advance_time(0.3); // 完成前摇
        assert_attack_state_is_cooldown(&harness, "前摇结束后应该进入后摇状态");
        assert_attack_target(&harness, target_b, "后摇期间目标应该是B");

        // 完成后摇
        harness.advance_time(0.7);
        assert_attack_state_is_windup(&harness, "后摇结束后应该自动开始下一次攻击");
        assert_attack_target(&harness, target_b, "下一次攻击的目标应该是B");
    }

    // ===== 九、边缘和异常情况测试 (Edge Cases & Exception Handling) =====

    /// 测试后摇期间发送取消命令
    #[test]
    fn test_cancel_attack_during_cooldown() {
        let mut harness = TestHarness::new().with_attacker(Attack {
            base_attack_speed: 1.0,
            windup_config: WindupConfig::Legacy { attack_offset: 0.0 },
            ..Default::default()
        });

        // 开始攻击并完成前摇
        harness.attack();
        harness.advance_time(0.3); // 完成前摇
        assert_attack_state_is_cooldown(&harness, "前摇结束后应该进入后摇状态");

        // 在后摇期间发送取消命令
        harness.cancel();
        assert_attack_state_is_idle(&harness, "后摇期间应该可以取消攻击");

        // 验证可以立即开始新攻击
        harness.attack();
        assert_attack_state_is_windup(&harness, "取消后应该能立即开始新攻击");
    }

    /// 测试空闲状态下发送取消或重置命令
    #[test]
    fn test_cancel_and_reset_in_idle_state() {
        let mut harness = TestHarness::new().with_attacker(Attack::default());

        // 验证初始状态为空闲
        assert_attack_state_is_idle(&harness, "初始状态应该是空闲");

        // 在空闲状态下发送取消命令
        harness.cancel();
        assert_attack_state_is_idle(&harness, "空闲状态下取消命令不应改变状态");

        // 在空闲状态下发送重置命令
        harness.reset();
        assert_attack_state_is_idle(&harness, "空闲状态下重置命令不应改变状态");
    }

    /// 演示：continue_attack控制测试
    #[test]
    fn test_continue_attack_control() {
        let mut harness = TestHarness::new().with_attacker(Attack {
            base_attack_speed: 1.0,
            windup_config: WindupConfig::Legacy { attack_offset: 0.0 },
            ..Default::default()
        });

        // 正常攻击循环
        harness.attack();
        assert!(harness.attack_state().continue_attack, "默认应该继续攻击");

        // 完成整个攻击周期
        harness.advance_time(1.0);
        assert_attack_state_is_windup(&harness, "没有取消命令时应该自动继续攻击");

        // 测试取消命令停止自动攻击
        harness.cancel();
        assert!(
            !harness.attack_state().continue_attack,
            "取消后不应该继续自动攻击"
        );
        assert_attack_state_is_idle(&harness, "取消后应该回到空闲状态");
    }
}
