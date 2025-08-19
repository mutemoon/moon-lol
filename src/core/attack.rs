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
pub const GAME_TICK_DURATION: f32 = 0.033; // 30 FPS 游戏帧
pub const UNCANCELLABLE_GRACE_PERIOD: f32 = 2.0 * GAME_TICK_DURATION; // 0.066 秒

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
