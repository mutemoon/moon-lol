use bevy::prelude::*;

use crate::{CommandMovement, EventMovementEnd, MovementAction, MovementWay};

#[derive(Default)]
pub struct PluginRun;

impl Plugin for PluginRun {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, fixed_update);

        app.add_observer(on_event_movement_end);
        app.add_observer(on_command_run_start);
        app.add_observer(on_command_run_stop);
    }
}

#[derive(Component)]
pub struct Run {
    pub target: RunTarget,
}

#[derive(Event)]
pub struct EventRunStart;

#[derive(Event)]
pub struct EventRunEnd;

#[derive(Event)]
pub struct CommandRunStart {
    pub target: RunTarget,
}

#[derive(Event)]
pub struct CommandRunStop;

#[derive(Clone)]
pub enum RunTarget {
    Position(Vec2),
    Target(Entity),
}

fn on_command_run_start(trigger: Trigger<CommandRunStart>, mut commands: Commands) {
    let entity = trigger.target();
    commands
        .entity(entity)
        .insert(Run {
            target: trigger.target.clone(),
        })
        .trigger(EventRunStart);
}

fn on_command_run_stop(trigger: Trigger<CommandRunStop>, mut commands: Commands) {
    let entity = trigger.target();
    commands
        .entity(entity)
        .remove::<Run>()
        .trigger(CommandMovement {
            priority: 0,
            action: MovementAction::Stop,
        });
}

fn fixed_update(mut commands: Commands, q: Query<(Entity, &Run)>, q_transform: Query<&Transform>) {
    for (entity, run) in q.iter() {
        match run.target {
            RunTarget::Position(position) => {
                commands.entity(entity).trigger(CommandMovement {
                    priority: 0,
                    action: MovementAction::Start {
                        way: MovementWay::Pathfind(position),
                        speed: None,
                        source: "Run".to_string(),
                    },
                });
            }
            RunTarget::Target(target) => {
                let Ok(transform) = q_transform.get(target) else {
                    return;
                };
                commands.entity(entity).trigger(CommandMovement {
                    priority: 0,
                    action: MovementAction::Start {
                        way: MovementWay::Pathfind(transform.translation.xz()),
                        speed: None,
                        source: "Run".to_string(),
                    },
                });
            }
        }
    }
}

fn on_event_movement_end(trigger: Trigger<EventMovementEnd>, mut commands: Commands) {
    let entity = trigger.target();
    if trigger.event().source != "Run" {
        return;
    }
    commands.entity(entity).remove::<Run>().trigger(EventRunEnd);
}
