use bevy::prelude::*;

use crate::core::Target;

pub struct PluginAttack;

impl Plugin for PluginAttack {
    fn build(&self, app: &mut App) {
        app.add_event::<EventAttackLock>();
        app.add_event::<EventAttackAttack>();
        app.add_event::<EventAttackRecover>();
        app.add_event::<EventAttackTargetInRange>();
        app.add_event::<CommandAttackLock>();
        app.add_observer(on_command_attack_lock);
    }
}

#[derive(Component)]
#[require(AttackState)]
pub struct Attack {
    pub range: f32,
    pub speed: f32,
    pub cast_time: AttackCastTime,
}

#[derive(Component)]
pub enum AttackCastTime {
    CastTime(f32),
    CastTimePercent(f32),
}

#[derive(Component, Default)]
pub struct AttackState {
    pub status: AttackStatus,
}

/// 攻击状态机 - 语义化的状态表示
#[derive(Component, Default, Debug, Clone, PartialEq)]
pub enum AttackStatus {
    #[default]
    Idle,
    Locked {
        target: Entity,
        lock_time: f32,
    },
    Attacking {
        target: Entity,
        attack_start_time: f32,
    },
    Cooldown {
        target: Entity,
        cooldown_end_time: f32,
    },
}

impl AttackState {
    pub fn is_idle(&self) -> bool {
        matches!(self.status, AttackStatus::Idle)
    }

    pub fn is_locked(&self) -> bool {
        matches!(self.status, AttackStatus::Locked { .. })
    }

    pub fn is_attacking(&self) -> bool {
        matches!(self.status, AttackStatus::Attacking { .. })
    }

    pub fn is_cooldown(&self) -> bool {
        matches!(self.status, AttackStatus::Cooldown { .. })
    }
}

#[derive(Event, Debug)]
pub struct CommandAttackLock;

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

fn on_command_attack_lock(
    trigger: Trigger<CommandAttackLock>,
    mut commands: Commands,
    mut query: Query<(&mut AttackState, &Target)>,
    time: Res<Time<Fixed>>,
) {
    let entity = trigger.target();

    if let Ok((mut attack_state, target)) = query.get_mut(entity) {
        // 只有在空闲状态时才能锁定新目标
        if attack_state.is_idle() {
            attack_state.status = AttackStatus::Locked {
                target: target.0,
                lock_time: time.elapsed_secs(),
            };
            commands.trigger_targets(EventAttackLock, entity);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::{CommandCommandAttack, PluginCommand, PluginTarget};

    use super::*;

    #[test]
    fn test_command_attack_lock_to_locking_success() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(PluginTarget);
        app.add_plugins(PluginAttack);
        app.add_plugins(PluginCommand);

        let attacker = app
            .world_mut()
            .spawn((
                Attack {
                    range: 100.0,
                    speed: 1.25,
                    cast_time: AttackCastTime::CastTime(0.393),
                },
                AttackState::default(),
            ))
            .id();

        let target = app.world_mut().spawn_empty().id();

        {
            let attack_state = app.world().get::<AttackState>(attacker).unwrap();
            assert!(attack_state.is_idle());
        }

        app.world_mut()
            .trigger_targets(CommandCommandAttack { target }, attacker);

        app.update();

        {
            let attack_state = app.world().get::<AttackState>(attacker).unwrap();
            assert!(attack_state.is_locked());
        }
    }
}
