use bevy::prelude::*;

use crate::life::Death;
use crate::log::{CommandLog, EnumLogCategory};
use crate::movement::{
    CommandMovement, EventMovementEnd, MovementAction, MovementSource, MovementWay,
};

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

#[derive(EntityEvent)]
pub struct EventRunStart {
    entity: Entity,
}

#[derive(EntityEvent)]
pub struct EventRunEnd {
    entity: Entity,
}

#[derive(EntityEvent)]
pub struct CommandRunStart {
    pub entity: Entity,
    pub target: RunTarget,
}

#[derive(EntityEvent)]
pub struct CommandRunStop {
    pub entity: Entity,
}

#[derive(Clone)]
pub enum RunTarget {
    Position(Vec2),
    Target(Entity),
}

fn on_command_run_start(trigger: On<CommandRunStart>, mut commands: Commands) {
    let entity = trigger.event_target();

    commands.entity(entity).insert(Run {
        target: trigger.target.clone(),
    });

    commands.trigger(EventRunStart { entity });
}

fn on_command_run_stop(trigger: On<CommandRunStop>, mut commands: Commands) {
    let entity = trigger.event_target();

    commands.entity(entity).remove::<Run>();

    commands.trigger(CommandMovement {
        entity,
        priority: 0,
        action: MovementAction::Stop,
    });
}

fn fixed_update(
    mut commands: Commands,
    q: Query<(Entity, &Run), Without<Death>>,
    q_transform: Query<&Transform>,
) {
    for (entity, run) in q.iter() {

        match run.target {
            RunTarget::Position(position) => {
                let Ok(transform) = q_transform.get(entity) else {
                    continue;
                };

                commands.trigger(CommandLog {
                    entity,
                    info: format!("寻路到 {:?}", position),
                    category: EnumLogCategory::Run,
                });
                commands.trigger(CommandMovement {
                    entity,
                    priority: 0,
                    action: MovementAction::Start {
                        way: MovementWay::Pathfind(Vec3::new(
                            position.x,
                            transform.translation.y,
                            position.y,
                        )),
                        speed: None,
                        source: MovementSource::Run,
                    },
                });
            }
            RunTarget::Target(target) => {
                let Ok(transform) = q_transform.get(target) else {
                    continue;
                };

                commands.trigger(CommandLog {
                    entity,
                    info: format!("寻路到实体 {:?} Vec3({})", target, transform.translation),
                    category: EnumLogCategory::Run,
                });
                commands.trigger(CommandMovement {
                    entity,
                    priority: 0,
                    action: MovementAction::Start {
                        way: MovementWay::Pathfind(transform.translation),
                        speed: None,
                        source: MovementSource::Run,
                    },
                });
            }
        }
    }
}

fn on_event_movement_end(trigger: On<EventMovementEnd>, mut commands: Commands) {
    if trigger.source != MovementSource::Run {
        return;
    }

    let entity = trigger.event_target();

    commands.entity(entity).remove::<Run>();

    commands.trigger(EventRunEnd { entity });
}
