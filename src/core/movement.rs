use core::f32;

use bevy::prelude::*;
use lol_config::ConfigNavigationGrid;
use serde::{Deserialize, Serialize};

use crate::core::{
    get_nav_path, rotate_to_direction, ArbitrationPipelinePlugin, FinalDecision, LastDecision,
    PipelineStages, RequestBuffer,
};

#[derive(Default)]
pub struct PluginMovement;

impl Plugin for PluginMovement {
    fn build(&self, app: &mut App) {
        app.register_type::<Movement>();

        app.add_event::<CommandMovementStart>();
        app.add_event::<CommandMovementStop>();

        app.add_event::<EventMovementStart>();
        app.add_event::<EventMovementEnd>();

        app.add_observer(on_command_movement_stop);
        app.add_observer(on_event_movement_end);

        app.add_plugins(ArbitrationPipelinePlugin::<
            CommandMovementStart,
            MovementPipeline,
        >::default());

        app.add_systems(
            FixedPostUpdate,
            (
                // 插入“仲裁”逻辑
                reduce_movement_by_priority.in_set(MovementPipeline::Reduce),
                // 插入“应用”逻辑
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

#[derive(Event, Debug, Clone)]
pub struct CommandMovementStart {
    pub priority: i32,
    pub way: MovementWay,
    pub speed: Option<f32>,
}

#[derive(Debug, Clone)]
pub enum MovementWay {
    Pathfind(Vec2),
    Path(Vec<Vec2>),
}

#[derive(Event, Debug)]
pub struct CommandMovementStop {
    pub priority: i32,
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
    mut query: Query<(Entity, &Movement, &mut MovementState)>,
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

        // 本帧可移动的总距离
        let mut remaining_distance_this_frame = speed * dt;
        // 记录本帧最后的移动方向，用于更新旋转
        let mut last_direction = Vec2::ZERO;

        // 只要本帧还有可移动的距离，就继续处理
        while remaining_distance_this_frame > 0.0 {
            // 首先，检查当前的目标点是否有效
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

        // 在循环结束后，根据最终状态统一更新
        if movement_state.completed {
            movement_state.velocity = Vec2::ZERO;
            movement_state.direction = Vec2::ZERO;
            movement_state.clear_path();
            // 恢复您原来的事件触发命令
            commands.trigger_targets(EventMovementEnd, entity);
        } else {
            movement_state.direction = last_direction;
            movement_state.velocity = last_direction * speed;
        }

        // 更新旋转
        if last_direction.length_squared() > 0.0 {
            rotate_to_direction(&mut transform, last_direction);
        }
    }
}

fn on_command_movement_stop(
    trigger: Trigger<CommandMovementStop>,
    mut commands: Commands,
    mut q_movement: Query<&mut MovementState>,
    mut q_buffer: Query<&mut RequestBuffer<CommandMovementStart>>,
    q_last_decision: Query<&LastDecision<CommandMovementStart>>,
) {
    let entity = trigger.target();

    let Ok(mut movement_state) = q_movement.get_mut(entity) else {
        return;
    };

    if let Ok(last_decision) = q_last_decision.get(entity) {
        if last_decision.0.priority > trigger.priority {
            return;
        }
    }

    if let Ok(mut buffer) = q_buffer.get_mut(entity) {
        buffer.0 = buffer
            .0
            .iter()
            .filter(|req| req.priority > trigger.priority)
            .cloned()
            .collect::<Vec<_>>();
    }

    movement_state.clear_path();

    // commands.trigger_targets(EventMovementEnd, entity);
}

fn reduce_movement_by_priority(
    mut commands: Commands,
    query: Query<(
        Entity,
        &RequestBuffer<CommandMovementStart>,
        Option<&LastDecision<CommandMovementStart>>,
    )>,
) {
    for (entity, buffer, last_decision) in query.iter() {
        if let Some(best_request) = buffer.0.iter().max_by_key(|req| req.priority) {
            if best_request.priority >= last_decision.map(|v| v.0.priority).unwrap_or(0) {
                commands
                    .entity(entity)
                    .insert(FinalDecision(best_request.clone()));
            }
        }
    }
}

fn apply_final_movement_decision(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &Transform,
        &FinalDecision<CommandMovementStart>,
        &mut MovementState,
    )>,
    grid: Res<ConfigNavigationGrid>,
) {
    for (entity, transform, decision, mut movement_state) in query.iter_mut() {
        match &decision.0.way {
            MovementWay::Pathfind(target) => {
                if let Some(path) = get_nav_path(&transform.translation.xz(), target, &grid) {
                    movement_state.reset_path(&path);
                }
            }
            MovementWay::Path(path) => {
                movement_state.reset_path(path);
            }
        }

        if let Some(speed) = decision.0.speed {
            movement_state.with_speed(speed);
        }

        commands.trigger_targets(EventMovementStart, entity);
    }
}

fn on_event_movement_end(trigger: Trigger<EventMovementEnd>, mut commands: Commands) {
    commands
        .entity(trigger.target())
        .remove::<LastDecision<CommandMovementStart>>();
}
