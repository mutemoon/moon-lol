use core::f32;
use std::collections::HashSet;
use std::time::Instant;

use bevy::prelude::*;
use lol_config::ConfigNavigationGrid;
use serde::{Deserialize, Serialize};

use crate::{
    calculate_occupied_grid_cells, get_nav_path, world_pos_to_grid_xy, ArbitrationPipelinePlugin,
    Bounding, CommandRotate, FinalDecision, LastDecision, NavigationStats, PipelineStages,
    RequestBuffer,
};

#[derive(Default)]
pub struct PluginMovement;

impl Plugin for PluginMovement {
    fn build(&self, app: &mut App) {
        app.register_type::<Movement>();

        app.add_observer(on_event_movement_end);

        app.add_plugins(ArbitrationPipelinePlugin::<CommandMovement, MovementPipeline>::default());

        app.add_systems(
            PreUpdate,
            calculate_global_occupied_cells.in_set(MovementPipeline::Calculate),
        );

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
    pub pathfind: Option<(Vec2, f32)>,
    pub source: String,
}

#[derive(Component, Default)]
pub struct MovementBlock;

#[derive(EntityEvent, Debug, Clone, PartialEq)]
pub struct CommandMovement {
    pub entity: Entity,
    pub priority: i32,
    pub action: MovementAction,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MovementAction {
    Start {
        way: MovementWay,
        speed: Option<f32>,
        source: String,
    },
    Stop,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MovementWay {
    Pathfind(Vec2),
    Path(Vec<Vec2>),
}

#[derive(EntityEvent, Debug)]
pub struct EventMovementStart {
    entity: Entity,
}

#[derive(EntityEvent, Debug)]
pub struct EventMovementEnd {
    pub entity: Entity,
    pub source: String,
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum MovementPipeline {
    Calculate,
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
    pub fn reset_path(&mut self, path: &Vec<Vec2>, source: &str) {
        self.path = path.clone();
        self.speed = None;
        self.current_target_index = 0;
        self.completed = false;
        self.velocity = Vec2::ZERO;
        self.direction = Vec2::ZERO;
        self.source = source.to_string();
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

            commands.trigger(EventMovementEnd {
                entity,
                source: movement_state.source.clone(),
            });
        } else {
            movement_state.direction = last_direction;
            movement_state.velocity = last_direction * speed;
        }

        if last_direction.length_squared() > 0.0 {
            commands.trigger(CommandRotate {
                entity,
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

fn calculate_global_occupied_cells(
    mut grid: ResMut<ConfigNavigationGrid>,
    entities_with_bounding: Query<(Entity, &GlobalTransform, &Bounding)>,
    mut stats: ResMut<NavigationStats>,
) {
    let start = Instant::now();
    // 计算所有实体的 occupied_cells（不排除任何实体）
    let occupied_cells = calculate_occupied_grid_cells(&grid, &entities_with_bounding, &[]);
    grid.occupied_cells = occupied_cells;
    stats.calculate_occupied_grid_cells_time += start.elapsed();
    stats.calculate_occupied_grid_cells_count += 1;
    stats.occupied_grid_cells_num = grid.occupied_cells.len() as u32;
}

fn apply_final_movement_decision(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &Transform,
        &FinalDecision<CommandMovement>,
        &mut MovementState,
        Option<&Bounding>,
    )>,
    mut grid: ResMut<ConfigNavigationGrid>,
    mut stats: ResMut<NavigationStats>,
    time: Res<Time>,
) {
    for (entity, transform, decision, mut movement_state, bounding) in query.iter_mut() {
        match &decision.0.action {
            MovementAction::Start { way, speed, source } => {
                match way {
                    MovementWay::Pathfind(target) => {
                        if let Some((last_target, last_time)) = movement_state.pathfind {
                            if (target - last_target).length() < f32::EPSILON
                                && time.elapsed_secs() - last_time < 1.0
                            {
                                continue;
                            }
                        }

                        movement_state.pathfind = Some((*target, time.elapsed_secs()));

                        // 从全局 occupied_cells 中临时移除当前实体占据的格子（仅当实体有 Bounding 时）
                        let removed_cells = if let Some(bounding) = bounding {
                            let entity_pos = transform.translation.xz();
                            let entity_grid_pos = world_pos_to_grid_xy(&grid, entity_pos);
                            let mut exclude_cells = HashSet::new();

                            // 计算当前实体占据的格子（根据 Bounding 组件的半径）
                            let radius_in_cells = (bounding.radius / grid.cell_size).ceil() as i32;
                            for dx in -radius_in_cells..=radius_in_cells {
                                for dy in -radius_in_cells..=radius_in_cells {
                                    let new_x = entity_grid_pos.0 as i32 + dx;
                                    let new_y = entity_grid_pos.1 as i32 + dy;

                                    if new_x >= 0 && new_y >= 0 {
                                        let new_pos = (new_x as usize, new_y as usize);
                                        if new_pos.0 < grid.x_len && new_pos.1 < grid.y_len {
                                            exclude_cells.insert(new_pos);
                                        }
                                    }
                                }
                            }

                            // 临时从全局 occupied_cells 中移除当前实体占据的格子
                            let removed: HashSet<_> = grid
                                .occupied_cells
                                .iter()
                                .filter(|cell| exclude_cells.contains(cell))
                                .copied()
                                .collect();
                            grid.occupied_cells
                                .retain(|cell| !exclude_cells.contains(cell));

                            removed
                        } else {
                            HashSet::new()
                        };

                        if let Some(path) =
                            get_nav_path(&transform.translation.xz(), target, &grid, &mut stats)
                        {
                            movement_state.reset_path(&path, source);
                        }

                        // 恢复全局 occupied_cells
                        grid.occupied_cells.extend(removed_cells);
                    }
                    MovementWay::Path(path) => {
                        movement_state.reset_path(path, source);
                    }
                }

                if let Some(speed) = speed {
                    movement_state.with_speed(*speed);
                }

                commands.trigger(EventMovementStart { entity });
            }
            MovementAction::Stop => {
                movement_state.clear_path();
            }
        }
    }
}

fn on_event_movement_end(trigger: On<EventMovementEnd>, mut commands: Commands) {
    commands
        .entity(trigger.event_target())
        .remove::<LastDecision<CommandMovement>>();
}
