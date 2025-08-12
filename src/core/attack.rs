use bevy::prelude::*;

pub struct PluginAttack;

impl Plugin for PluginAttack {
    fn build(&self, app: &mut App) {
        app.add_event::<EventAttackLock>();
        app.add_event::<EventAttackAttack>();
        app.add_event::<EventAttackRecover>();
        app.add_event::<EventAttackTargetInRange>();
        app.add_event::<CommandAttackLock>();
        app.add_observer(handle_attack_lock_command);
    }
}

#[derive(Component)]
#[require(AttackState)]
pub struct Attack {
    pub range: f32,
    pub speed: f32,
}

#[derive(Component, Default)]
pub struct AttackState {
    pub machine_state: AttackMachineState,
    pub target: Option<Entity>,
}

#[derive(Debug, PartialEq, Default)]
pub enum AttackMachineState {
    #[default]
    Idle,
    Locking,
    Attacking,
    Recovering,
}

#[derive(Event, Debug)]
pub struct CommandAttackLock {
    pub target: Entity,
}

#[derive(Event, Debug)]
pub struct EventAttackLock;

#[derive(Event, Debug)]
pub struct EventAttackAttack {
    pub target: Entity,
}

#[derive(Event, Debug)]
pub struct EventAttackRecover;

#[derive(Event, Debug)]
pub struct EventAttackTargetInRange {
    pub target: Entity,
}

/// 处理攻击锁定命令的观察者函数
fn handle_attack_lock_command(
    trigger: Trigger<CommandAttackLock>,
    mut commands: Commands,
    mut query: Query<&mut AttackState>,
) {
    let entity = trigger.target();
    let command = trigger.event();

    if let Ok(mut attack_state) = query.get_mut(entity) {
        // 只有在空闲状态下才能开始锁定
        if attack_state.machine_state == AttackMachineState::Idle {
            attack_state.machine_state = AttackMachineState::Locking;
            attack_state.target = Some(command.target);

            // 触发攻击锁定事件
            commands.trigger_targets(EventAttackLock, entity);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_attack_lock_to_locking_success() {
        // 创建测试应用
        let mut app = App::new();
        app.add_plugins(PluginAttack);

        // 创建攻击者实体
        let attacker = app
            .world_mut()
            .spawn((
                Attack {
                    range: 100.0,
                    speed: 1.0,
                },
                AttackState::default(),
            ))
            .id();

        // 创建目标实体
        let target = app.world_mut().spawn_empty().id();

        // 验证初始状态为Idle
        {
            let attack_state = app.world().get::<AttackState>(attacker).unwrap();
            assert_eq!(attack_state.machine_state, AttackMachineState::Idle);
            assert_eq!(attack_state.target, None);
        }

        // 触发攻击锁定命令
        app.world_mut()
            .trigger_targets(CommandAttackLock { target }, attacker);

        // 更新应用以处理事件
        app.update();

        // 验证状态转换成功
        {
            let attack_state = app.world().get::<AttackState>(attacker).unwrap();
            assert_eq!(attack_state.machine_state, AttackMachineState::Locking);
            assert_eq!(attack_state.target, Some(target));
        }

        // 验证EventAttackLock事件被触发
        let mut attack_lock_events = app.world_mut().resource_mut::<Events<EventAttackLock>>();
        let events: Vec<_> = attack_lock_events.drain().collect();
        assert_eq!(events.len(), 1);
    }
}
