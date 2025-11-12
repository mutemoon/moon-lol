use core::f32;

use bevy::prelude::*;
use lol_config::ConfigNavigationGrid;
use serde::{Deserialize, Serialize};

use crate::{
    get_nav_path, ArbitrationPipelinePlugin, CommandRotate, FinalDecision, LastDecision,
    PipelineStages, RequestBuffer,
};

#[derive(Default)]
pub struct PluginMovement;

impl Plugin for PluginMovement {
    fn build(&self, app: &mut App) {
        app.register_type::<Movement>();

        app.add_observer(on_event_movement_end);

        app.add_plugins(ArbitrationPipelinePlugin::<CommandMovement, MovementPipeline>::default());

        app.add_systems(
            FixedPostUpdate,
            (
                reduce_movement_by_priority.in_set(MovementPipeline::Reduce),
                (apply_final_movement_decision, update_path_movement)
                    .in_set(MovementPipeline::Apply),
            ),
        );
    }
}

#[derive(Component, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
#[require(MovementState)]
pub struct Movement {
    pub speed: f32,
}

#[derive(Component, Default)]
pub struct MovementState {
    pub path: Vec<Vec2>,
    pub speed: Option<f32>,
    pub direction: Vec2,
    pub velocity: Vec2,
    pub current_target_index: usize,
    pub completed: bool,
}

#[derive(Component, Default)]
pub struct MovementBlock;

#[derive(Event, Debug, Clone, PartialEq)]
pub struct CommandMovement {
    pub priority: i32,
    pub action: MovementAction,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MovementAction {
    Start {
        way: MovementWay,
        speed: Option<f32>,
    },
    Stop,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MovementWay {
    Pathfind(Vec2),
    Path(Vec<Vec2>),
}

#[derive(Event, Debug)]
pub struct EventMovementStart;

#[derive(Event, Debug)]
pub struct EventMovementEnd;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum MovementPipeline {
    Modify,
    Reduce,
    Apply,
    Cleanup,
}

impl PipelineStages for MovementPipeline {
    fn modify() -> Self {
        Self::Modify
    }
    fn reduce() -> Self {
        Self::Reduce
    }
    fn apply() -> Self {
        Self::Apply
    }
    fn cleanup() -> Self {
        Self::Cleanup
    }
}

impl MovementState {
    pub fn reset_path(&mut self, path: &Vec<Vec2>) {
        self.path = path.clone();
        self.speed = None;
        self.current_target_index = 0;
        self.completed = false;
        self.velocity = Vec2::ZERO;
        self.direction = Vec2::ZERO;
    }

    pub fn clear_path(&mut self) {
        self.path.clear();
        self.current_target_index = 0;
        self.completed = false;
        self.velocity = Vec2::ZERO;
        self.direction = Vec2::ZERO;
    }

    pub fn is_moving(&self) -> bool {
        self.current_target_index < self.path.len() - 1
    }

    pub fn with_speed(&mut self, speed: f32) -> &mut Self {
        self.speed = Some(speed);
        self
    }
}

fn update_path_movement(
    mut commands: Commands,
    mut query: Query<(Entity, &Movement, &mut MovementState), Without<MovementBlock>>,
    mut q_transform: Query<&mut Transform>,
    time: Res<Time<Fixed>>,
) {
    let dt = time.delta_secs();

    for (entity, movement, mut movement_state) in query.iter_mut() {
        if movement_state.completed || movement_state.path.is_empty() {
            continue;
        }

        let mut transform = q_transform.get_mut(entity).unwrap();

        let speed = movement_state.speed.unwrap_or(movement.speed);

        let mut remaining_distance_this_frame = speed * dt;

        let mut last_direction = Vec2::ZERO;

        while remaining_distance_this_frame > 0.0 {
            let target = match movement_state.path.get(movement_state.current_target_index) {
                Some(p) => *p,
                None => {
                    if !movement_state.completed {
                        movement_state.completed = true;
                    }
                    break;
                }
            };

            let current_pos = transform.translation.xz();
            let vector_to_target = target - current_pos;
            let distance_to_target = vector_to_target.length();

            if distance_to_target.abs() < f32::EPSILON {
                let new_index = movement_state.current_target_index + 1;
                if new_index >= movement_state.path.len() {
                    movement_state.completed = true;
                    break;
                } else {
                    movement_state.current_target_index = new_index;
                    continue;
                }
            }

            last_direction = vector_to_target.normalize();

            if distance_to_target > remaining_distance_this_frame {
                let new_pos = current_pos + last_direction * remaining_distance_this_frame;
                transform.translation.x = new_pos.x;
                transform.translation.z = new_pos.y;
                remaining_distance_this_frame = 0.0;
            } else {
                transform.translation.x = target.x;
                transform.translation.z = target.y;
                remaining_distance_this_frame -= distance_to_target;

                let new_index = movement_state.current_target_index + 1;
                if new_index >= movement_state.path.len() {
                    movement_state.completed = true;
                    break;
                } else {
                    movement_state.current_target_index = new_index;
                }
            }
        }

        if movement_state.completed {
            movement_state.velocity = Vec2::ZERO;
            movement_state.direction = Vec2::ZERO;
            movement_state.clear_path();

            commands.trigger_targets(EventMovementEnd, entity);
        } else {
            movement_state.direction = last_direction;
            movement_state.velocity = last_direction * speed;
        }

        if last_direction.length_squared() > 0.0 {
            commands.entity(entity).trigger(CommandRotate {
                priority: 0,
                direction: last_direction,
                angular_velocity: None,
            });
        }
    }
}

fn reduce_movement_by_priority(
    mut commands: Commands,
    query: Query<(
        Entity,
        &RequestBuffer<CommandMovement>,
        Option<&LastDecision<CommandMovement>>,
    )>,
) {
    for (entity, buffer, last_decision) in query.iter() {
        if buffer.0.is_empty() {
            continue;
        }

        let mut final_decision = last_decision.map(|v| &v.0);
        let mut found = false;

        for command in buffer.0.iter() {
            match (final_decision, &command.action) {
                (None, _) => {
                    final_decision = Some(command);
                    found = true;
                }

                (Some(current), MovementAction::Start { .. }) => match &current.action {
                    MovementAction::Stop => {
                        final_decision = Some(command);
                        found = true;
                    }
                    MovementAction::Start { .. } => {
                        if command.priority >= current.priority {
                            final_decision = Some(command);
                            found = true;
                        }
                    }
                },

                (Some(current), MovementAction::Stop) => {
                    if command.priority >= current.priority {
                        final_decision = Some(command);
                        found = true;
                    }
                }
            }
        }

        if let Some(decision) = final_decision {
            if found {
                commands
                    .entity(entity)
                    .insert(FinalDecision(decision.clone()));
            }
        }
    }
}

fn apply_final_movement_decision(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &Transform,
        &FinalDecision<CommandMovement>,
        &mut MovementState,
    )>,
    grid: Res<ConfigNavigationGrid>,
) {
    for (entity, transform, decision, mut movement_state) in query.iter_mut() {
        match &decision.0.action {
            MovementAction::Start { way, speed } => {
                match way {
                    MovementWay::Pathfind(target) => {
                        if let Some(path) = get_nav_path(&transform.translation.xz(), target, &grid)
                        {
                            movement_state.reset_path(&path);
                        }
                    }
                    MovementWay::Path(path) => {
                        movement_state.reset_path(path);
                    }
                }

                if let Some(speed) = speed {
                    movement_state.with_speed(*speed);
                }

                commands.trigger_targets(EventMovementStart, entity);
            }
            MovementAction::Stop => {
                movement_state.clear_path();
            }
        }
    }
}

fn on_event_movement_end(trigger: Trigger<EventMovementEnd>, mut commands: Commands) {
    commands
        .entity(trigger.target())
        .remove::<LastDecision<CommandMovement>>();
}
