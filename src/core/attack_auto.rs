use std::time::Duration;

use bevy::prelude::*;

use crate::core::{
    Attack, AttackState, AttackStatus, CommandAttackStart, CommandAttackStop, CommandRunStart,
    CommandRunStop, RunTarget,
};

#[derive(Default)]
pub struct PluginAttackAuto;

impl Plugin for PluginAttackAuto {
    fn build(&self, app: &mut App) {
        app.add_event::<CommandAttackAutoStart>();
        app.add_observer(on_command_attack_auto_start);

        app.add_event::<CommandAttackAutoStop>();
        app.add_observer(on_command_attack_auto_stop);

        app.add_systems(FixedPreUpdate, update_attack_auto);
    }
}

#[derive(Component)]
pub struct AttackAuto {
    pub target: Entity,
    pub timer: Timer,
}

#[derive(Event)]
pub struct CommandAttackAutoStart {
    pub target: Entity,
}

#[derive(Event)]
pub struct CommandAttackAutoStop;

fn on_command_attack_auto_start(
    trigger: Trigger<CommandAttackAutoStart>,
    mut commands: Commands,
    q_transform: Query<&Transform>,
    q_attack: Query<&Attack>,
) {
    let mut timer = Timer::from_seconds(1.0, TimerMode::Repeating);

    timer.tick(Duration::from_secs_f32(1.0));

    let entity = trigger.target();
    let target = trigger.target;

    let mut attack_auto = AttackAuto { target, timer };

    let Ok(transform) = q_transform.get(entity) else {
        return;
    };

    let Ok(target_transform) = q_transform.get(attack_auto.target) else {
        return;
    };

    let Ok(attack) = q_attack.get(entity) else {
        return;
    };

    check_and_action(
        &mut commands,
        trigger.target(),
        attack_auto.target,
        &mut attack_auto,
        transform.translation.xz(),
        target_transform.translation.xz(),
        attack.range,
    );

    commands.entity(trigger.target()).insert(attack_auto);
}

fn on_command_attack_auto_stop(trigger: Trigger<CommandAttackAutoStop>, mut commands: Commands) {
    commands
        .entity(trigger.target())
        .remove::<AttackAuto>()
        .trigger(CommandAttackStop);
}

fn update_attack_auto(
    mut commands: Commands,
    mut query: Query<(Entity, &mut AttackAuto, &Attack)>,
    q_attack_state: Query<&AttackState>,
    q_transform: Query<&Transform>,
    time: Res<Time<Fixed>>,
) {
    for (entity, mut attack_auto, attack) in query.iter_mut() {
        attack_auto.timer.tick(time.delta());

        if let Ok(attack_state) = q_attack_state.get(entity) {
            if matches!(attack_state.status, AttackStatus::Windup { .. }) {
                continue;
            }
        };

        let Ok(transform) = q_transform.get(entity) else {
            continue;
        };

        let Ok(target_transform) = q_transform.get(attack_auto.target) else {
            continue;
        };

        check_and_action(
            &mut commands,
            entity,
            attack_auto.target,
            &mut attack_auto,
            transform.translation.xz(),
            target_transform.translation.xz(),
            attack.range,
        );
    }
}

fn check_and_action(
    commands: &mut Commands,
    entity: Entity,
    target: Entity,
    attack_auto: &mut AttackAuto,
    position: Vec2,
    target_position: Vec2,
    range: f32,
) {
    if position.distance(target_position) > range {
        if attack_auto.timer.just_finished() {
            commands.entity(entity).trigger(CommandRunStart {
                target: RunTarget::Target(target),
            });

            attack_auto.timer.reset();
        }
    } else {
        commands
            .entity(entity)
            .trigger(CommandRunStop)
            .trigger(CommandAttackStart {
                target: attack_auto.target,
            });
    }
}
