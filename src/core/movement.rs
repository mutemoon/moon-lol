use core::f32;
use std::{
    collections::{HashMap, HashSet},
    time::Instant,
};

use bevy::prelude::*;
use lol_config::ConfigNavigationGrid;
use serde::{Deserialize, Serialize};

use crate::{
    get_nav_path_with_debug, is_path_blocked, world_pos_to_grid_xy, ArbitrationPipelinePlugin,
    Bounding, CommandRotate, FinalDecision, LastDecision, NavigationDebug, NavigationStats,
    PipelineStages, RequestBuffer,
};

#[derive(Default)]
pub struct PluginMovement;

impl Plugin for PluginMovement {
    fn build(&self, app: &mut App) {
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

#[derive(Component, Default, Debug)]
pub struct MovementState {
    pub path: Vec<Vec3>,
    pub speed: Option<f32>,
    pub direction: Vec2,
    pub velocity: Vec2,
    pub current_target_index: usize,
    pub completed: bool,
    pub pathfind: Option<(Vec3, f32)>,
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
    Pathfind(Vec3),
    Path(Vec<Vec3>),
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
    pub fn reset_path(&mut self, path: &Vec<Vec3>, source: &str) {
        self.completed = false;
        self.current_target_index = 0;
        self.direction = Vec2::ZERO;
        self.path = path.clone();
        // self.pathfind = None;
        self.source = source.to_string();
        self.speed = None;
        self.velocity = Vec2::ZERO;
    }

    pub fn clear_path(&mut self) {
        *self = MovementState::default();
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

            let current_pos_xz = transform.translation.xz();
            let target_xz = target.xz();
            let vector_to_target_xz = target_xz - current_pos_xz;
            let distance_to_target_xz = vector_to_target_xz.length();

            if distance_to_target_xz.abs() < f32::EPSILON {
                let new_index = movement_state.current_target_index + 1;
                if new_index >= movement_state.path.len() {
                    movement_state.completed = true;
                    break;
                } else {
                    movement_state.current_target_index = new_index;
                    continue;
                }
            }

            last_direction = vector_to_target_xz.normalize();

            if remaining_distance_this_frame < distance_to_target_xz {
                let move_fraction = remaining_distance_this_frame / distance_to_target_xz;
                let new_pos_xz = current_pos_xz + last_direction * remaining_distance_this_frame;
                let new_y = transform.translation.y.lerp(target.y, move_fraction);

                debug!("{} 移动一小步 {}", entity, remaining_distance_this_frame);
                transform.translation.x = new_pos_xz.x;
                transform.translation.z = new_pos_xz.y;
                transform.translation.y = new_y;

                remaining_distance_this_frame = 0.0;
            } else {
                debug!("{} 移动最后一小步到达转折点 {}", entity, target);
                transform.translation.x = target.x;
                transform.translation.z = target.z;
                transform.translation.y = target.y;

                remaining_distance_this_frame -= distance_to_target_xz;

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
    mut nav_debug: ResMut<NavigationDebug>,
    time: Res<Time>,
) {
    for (entity, transform, decision, mut movement_state, bounding) in query.iter_mut() {
        match &decision.0.action {
            MovementAction::Start { way, speed, source } => {
                match way {
                    MovementWay::Pathfind(target) => {
                        let start = Instant::now();

                        if let Some(bounding) = bounding {
                            let entity_pos = transform.translation.xz();
                            let entity_grid_pos = world_pos_to_grid_xy(&grid, entity_pos);
                            let mut exclude_cells = HashSet::new();

                            // 计算当前实体占据的格子（根据 Bounding 组件的半径）
                            let radius_in_cells = (bounding.radius / grid.cell_size).floor() as i32;
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

                            grid.exclude_cells = exclude_cells;
                        };

                        stats.exclude_time += start.elapsed();
                        stats.exclude_count += 1;

                        // 检查是否需要重新规划路径
                        let need_replan = if let Some((last_target, _)) = movement_state.pathfind {
                            // 目标位置发生变化
                            let target_changed =
                                (target - last_target).xz().length() > f32::EPSILON;
                            // 路径上有障碍物阻挡
                            let path_blocked = is_path_blocked(
                                &grid,
                                &movement_state.path,
                                movement_state.current_target_index,
                            );
                            if target_changed {
                                debug!("{} 的目标位置发生变化: {}", entity, target_changed);
                            }
                            if path_blocked {
                                debug!("{} 的路径上有障碍物阻挡: {}", entity, path_blocked);
                            }
                            target_changed || path_blocked
                        } else {
                            // 第一次规划
                            debug!("{} 第一次规划", entity);
                            true
                        };

                        if !need_replan {
                            debug!("{} 不需要重新规划，{:#?}", entity, movement_state);
                            continue;
                        }

                        movement_state.pathfind = Some((*target, time.elapsed_secs()));

                        debug!("{} 寻路到 {:?}", entity, target);

                        let debug_ref = if nav_debug.enabled {
                            Some(&mut *nav_debug)
                        } else {
                            None
                        };

                        if let Some(path) = get_nav_path_with_debug(
                            &transform.translation.xz(),
                            &target.xz(),
                            &grid,
                            &mut stats,
                            debug_ref,
                        ) {
                            let start_y = transform.translation.y;
                            let total = path.len() as f32;
                            let path_3d = path
                                .into_iter()
                                .enumerate()
                                .map(|(i, p)| {
                                    let t = (i as f32 + 1.0) / total;
                                    let y = start_y + (target.y - start_y) * t;
                                    Vec3::new(p.x, y, p.y)
                                })
                                .collect();
                            movement_state.reset_path(&path_3d, source);
                        } else {
                            debug!("{} 寻路失败", entity);
                        }
                    }
                    MovementWay::Path(path) => {
                        debug!("{} 设置路径 {:?}", entity, path);
                        movement_state.reset_path(path, source);
                    }
                }

                if let Some(speed) = speed {
                    debug!("{} 设置速度 {:?}", entity, speed);
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
        .try_remove::<LastDecision<CommandMovement>>();
}
