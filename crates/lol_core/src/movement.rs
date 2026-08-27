use core::f32;
use std::collections::HashSet;

use bevy::prelude::*;
use lol_base::grid::ConfigNavigationGrid;
use serde::{Deserialize, Serialize};

use crate::base::bounding::Bounding;
use crate::base::pipeline::{
    ArbitrationPipelinePlugin, FinalDecision, LastDecision, PipelineStages, RequestBuffer,
};
use crate::life::Death;
use crate::log::{CommandLog, EnumLogCategory};
use crate::navigation::grid::ResourceGrid;
use crate::navigation::navigation::{
    AStarCache, NavigationDebugState, NavigationStats, get_nav_path_with_debug, is_path_blocked,
    update_occupied_cells_flat, world_pos_to_grid_xy,
};
use crate::rotate::CommandRotate;

#[derive(Default)]
pub struct PluginMovement;

impl Plugin for PluginMovement {
    fn build(&self, app: &mut App) {
        app.add_observer(on_event_movement_end);
        app.add_observer(on_reset_movement);

        app.add_plugins(ArbitrationPipelinePlugin::<CommandMovement, MovementPipeline>::default());

        app.add_systems(
            FixedPostUpdate,
            (
                reduce_movement_by_priority.in_set(MovementPipeline::Reduce),
                (
                    apply_final_movement_decision.run_if(resource_exists::<ResourceGrid>),
                    update_path_movement,
                )
                    .chain()
                    .in_set(MovementPipeline::Apply),
            ),
        );
    }
}

pub fn on_reset_movement(
    _trigger: On<crate::action::EventReset>,
    mut commands: Commands,
    mut q_movement: Query<&mut MovementState>,
    q_run: Query<Entity, With<crate::run::Run>>,
    q_movement_block: Query<Entity, With<MovementBlock>>,
    q_cast_block: Query<Entity, With<CastBlock>>,
    q_slow: Query<Entity, With<MovementSlow>>,
) {
    for mut state in q_movement.iter_mut() {
        state.path.clear();
        state.speed = None;
        state.direction = Vec2::ZERO;
        state.velocity = Vec2::ZERO;
        state.current_target_index = 0;
        state.completed = true;
        state.pathfind = None;
        state.source = MovementSource::Run;
    }
    for entity in q_run.iter() {
        commands.entity(entity).remove::<crate::run::Run>();
    }
    for entity in q_movement_block.iter() {
        commands.entity(entity).remove::<MovementBlock>();
    }
    for entity in q_cast_block.iter() {
        commands.entity(entity).remove::<CastBlock>();
    }
    for entity in q_slow.iter() {
        commands.entity(entity).remove::<MovementSlow>();
    }
}

#[derive(Component, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
#[require(MovementState)]
pub struct Movement {
    pub speed: f32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MovementSource {
    Run,
    Dash,
    Knockback,
    Missile,
    Player,
    AI,
    Skill(String),
    Pathfind,
}

impl Default for MovementSource {
    fn default() -> Self {
        Self::Run
    }
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
    pub source: MovementSource,
}

#[derive(Component, Default)]
pub struct MovementBlock;

/// 施法阻塞组件，如果实体拥有此组件，则无法施放新技能
#[derive(Component, Default)]
pub struct CastBlock;

/// 减速标记：由 CC 系统按最强活跃减速写入角色（percent 0.0-1.0）。
/// 移动系统据此按比例降低本帧位移速度。轻量标记，逻辑在 DebuffSlow buff 实体上。
#[derive(Component, Debug, Clone, Default)]
pub struct MovementSlow {
    pub percent: f32,
}

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
        source: MovementSource,
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
    pub source: MovementSource,
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
    pub fn reset_path(&mut self, path: &Vec<Vec3>, source: MovementSource) -> &mut Self {
        *self = MovementState {
            path: path.clone(),
            source,
            ..default()
        };
        self
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

    pub fn with_pathfind(&mut self, pathfind: (Vec3, f32)) -> &mut Self {
        self.pathfind = Some(pathfind);
        self
    }
}

fn calculate_and_set_exclude_cells(grid: &mut ConfigNavigationGrid, entity_pos: Vec2, radius: f32) {
    let entity_grid_pos = world_pos_to_grid_xy(grid, entity_pos);
    let mut exclude_cells = HashSet::new();

    let radius_in_cells = (radius / grid.cell_size).floor() as i32;
    for dx in -radius_in_cells..=radius_in_cells {
        for dy in -radius_in_cells..=radius_in_cells {
            let new_x = entity_grid_pos.0 as i32 + dx;
            let new_y = entity_grid_pos.1 as i32 + dy;

            if new_x < 0 || new_y < 0 {
                continue;
            }

            let new_pos = (new_x as usize, new_y as usize);
            if new_pos.0 >= grid.x_len || new_pos.1 >= grid.y_len {
                continue;
            }

            exclude_cells.insert(new_pos);
        }
    }

    grid.exclude_cells = exclude_cells;
}

fn update_path_movement(
    mut commands: Commands,
    mut query: Query<
        (Entity, &Movement, &mut MovementState, Option<&MovementSlow>),
        (Without<MovementBlock>, Without<Death>),
    >,
    mut q_transform: Query<&mut Transform>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    for (entity, movement, mut movement_state, slow) in query.iter_mut() {
        if movement_state.completed || movement_state.path.is_empty() {
            continue;
        }

        let mut transform = q_transform.get_mut(entity).unwrap();

        let speed = movement_state.speed.unwrap_or(movement.speed);
        // 减速：按最强活跃减速比例降低本帧速度（系统只认 MovementSlow 标记）
        let speed = speed * slow.map_or(1.0, |s| 1.0 - s.percent);

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

                transform.translation.x = new_pos_xz.x;
                transform.translation.z = new_pos_xz.y;
                transform.translation.y = new_y;

                remaining_distance_this_frame = 0.0;
            } else {
                commands.trigger(CommandLog {
                    entity,
                    info: format!("移动最后一小步到达转折点 {:?}", target),
                    category: EnumLogCategory::Movement,
                });
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

            commands.trigger(EventMovementEnd {
                entity,
                source: movement_state.source.clone(),
            });
            movement_state.clear_path();
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
    mut query: Query<(
        Entity,
        &mut RequestBuffer<CommandMovement>,
        Option<&LastDecision<CommandMovement>>,
    )>,
) {
    for (entity, mut buffer, last_decision) in query.iter_mut() {
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

        buffer.0.clear();
    }
}

const REPLAN_COOLDOWN_SECS: f32 = 0.25;

fn apply_final_movement_decision(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &Transform,
        &FinalDecision<CommandMovement>,
        &mut MovementState,
        Option<&Bounding>,
    )>,
    entities_with_bounding: Query<(Entity, &GlobalTransform, &Bounding)>,
    res_grid: Res<ResourceGrid>,
    mut assets_grid: ResMut<Assets<ConfigNavigationGrid>>,
    mut stats: Option<ResMut<NavigationStats>>,
    mut nav_debug: Option<ResMut<NavigationDebugState>>,
    mut astar_cache: Option<ResMut<AStarCache>>,
    time: Res<Time>,
) {
    let Some(mut grid) = assets_grid.get_mut(&res_grid.0) else {
        return;
    };
    let mut default_stats = NavigationStats::default();
    let stats_ref = match stats.as_mut() {
        Some(s) => &mut **s,
        None => &mut default_stats,
    };
    let mut cache_ref = astar_cache.as_deref_mut();
    let mut occupied_grid_updated = false;
    for (entity, transform, decision, mut movement_state, bounding) in query.iter_mut() {
        if matches!(&decision.0.action, MovementAction::Stop) {
            movement_state.clear_path();
            continue;
        }

        let MovementAction::Start { way, speed, source } = &decision.0.action else {
            unreachable!()
        };

        match way {
            MovementWay::Pathfind(target) => {
                if !occupied_grid_updated {
                    update_occupied_cells_flat(&mut grid, &entities_with_bounding, stats_ref);
                    occupied_grid_updated = true;
                }

                if let Some(bounding) = bounding {
                    calculate_and_set_exclude_cells(
                        &mut grid,
                        transform.translation.xz(),
                        bounding.radius,
                    );
                };

                stats_ref.exclude_count += 1;
                let now = time.elapsed_secs();

                // 检查是否需要重新规划路径
                let need_replan =
                    if let Some((last_target, last_replan_time)) = movement_state.pathfind {
                        let target_teleported = (target - last_target).xz().length() > 150.0;
                        let cooldown_elapsed = (now - last_replan_time) >= REPLAN_COOLDOWN_SECS;

                        if target_teleported {
                            commands.trigger(CommandLog {
                                entity,
                                info: format!("目标位置发生突变: {}", target_teleported),
                                category: EnumLogCategory::Movement,
                            });
                            true
                        } else if cooldown_elapsed {
                            let target_moved = (target - last_target).xz().length() > 20.0;
                            let path_blocked = is_path_blocked(
                                &grid,
                                &movement_state.path,
                                movement_state.current_target_index,
                                Some(transform.translation.xz()),
                            );
                            if target_moved || path_blocked {
                                commands.trigger(CommandLog {
                                    entity,
                                    info: format!(
                                        "目标移动或路径受阻: moved={}, blocked={}",
                                        target_moved, path_blocked
                                    ),
                                    category: EnumLogCategory::Movement,
                                });
                                true
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    } else {
                        // 第一次规划
                        commands.trigger(CommandLog {
                            entity,
                            info: "第一次规划".to_string(),
                            category: EnumLogCategory::Movement,
                        });
                        true
                    };

                if !need_replan {
                    continue;
                }

                commands.trigger(CommandLog {
                    entity,
                    info: format!("寻路到 {:?}", target),
                    category: EnumLogCategory::Movement,
                });

                let debug_ref = nav_debug.as_mut().map(|d| &mut **d);

                if let Some(path) = get_nav_path_with_debug(
                    &transform.translation.xz(),
                    &target.xz(),
                    &grid,
                    stats_ref,
                    debug_ref,
                    cache_ref.as_deref_mut(),
                ) {
                    if !path.is_empty() {
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
                        movement_state
                            .reset_path(&path_3d, source.clone())
                            .with_pathfind((*target, now));
                    } else {
                        movement_state.pathfind = Some((*target, now));
                        movement_state.clear_path();
                        movement_state.completed = true;
                        commands.trigger(EventMovementEnd {
                            entity,
                            source: source.clone(),
                        });
                    }
                } else {
                    commands.trigger(CommandLog {
                        entity,
                        info: "寻路失败".to_string(),
                        category: EnumLogCategory::Movement,
                    });
                    movement_state.pathfind = Some((*target, now));
                    movement_state.clear_path();
                    movement_state.completed = true;
                    commands.trigger(EventMovementEnd {
                        entity,
                        source: source.clone(),
                    });
                }
            }
            MovementWay::Path(path) => {
                commands.trigger(CommandLog {
                    entity,
                    info: format!("设置路径 {:?}", path),
                    category: EnumLogCategory::Movement,
                });
                movement_state.reset_path(path, source.clone());
            }
        }

        if let Some(speed) = speed {
            commands.trigger(CommandLog {
                entity,
                info: format!("设置速度 {:?}", speed),
                category: EnumLogCategory::Movement,
            });
            movement_state.with_speed(*speed);
        }

        commands.trigger(EventMovementStart { entity });
    }
}

fn on_event_movement_end(trigger: On<EventMovementEnd>, mut commands: Commands) {
    commands
        .entity(trigger.event_target())
        .try_remove::<LastDecision<CommandMovement>>();
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::ecs::system::RunSystemOnce;
    use lol_base::grid::{ConfigNavigationGridCell, GridFlagsVisionPathing};

    use super::*;

    fn make_test_grid() -> ConfigNavigationGrid {
        let cell = ConfigNavigationGridCell {
            heuristic: 1.0,
            vision_pathing_flags: GridFlagsVisionPathing::Walkable,
            ..default()
        };

        ConfigNavigationGrid {
            min_position: Vec2::ZERO,
            cell_size: 50.0,
            x_len: 100,
            y_len: 100,
            cells: vec![vec![cell; 100]; 100],
            height_x_len: 100,
            height_y_len: 100,
            height_samples: vec![vec![0.0; 100]; 100],
            occupied_cells: Default::default(),
            exclude_cells: Default::default(),
        }
    }

    #[test]
    fn test_replan_cooldown_throttles_high_frequency_pathfind() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let mut assets_grid = Assets::<ConfigNavigationGrid>::default();
        let grid_handle = assets_grid.add(make_test_grid());
        app.insert_resource(assets_grid);
        app.insert_resource(ResourceGrid(grid_handle));
        app.insert_resource(NavigationStats::default());

        let entity = app
            .world_mut()
            .spawn((
                Transform::from_xyz(100.0, 0.0, 100.0),
                Movement { speed: 300.0 },
                MovementState::default(),
                FinalDecision(CommandMovement {
                    entity: Entity::PLACEHOLDER,
                    priority: 0,
                    action: MovementAction::Start {
                        way: MovementWay::Pathfind(Vec3::new(4000.0, 0.0, 4000.0)),
                        speed: None,
                        source: MovementSource::Run,
                    },
                }),
            ))
            .id();

        // 第一次执行，进行首次寻路
        let _ = app
            .world_mut()
            .run_system_once(apply_final_movement_decision);
        let stats_1 = app.world().resource::<NavigationStats>().get_nav_path_count;
        assert_eq!(stats_1, 1, "首次规划应执行 1 次寻路");

        // 立即在同一帧/短时间内（16ms 后）再次执行相同目标的决策
        {
            let mut time = app.world_mut().resource_mut::<Time>();
            time.advance_by(Duration::from_millis(16));
        }

        // 重新插入决策（模拟 run.rs 每帧发送）
        app.world_mut()
            .entity_mut(entity)
            .insert(FinalDecision(CommandMovement {
                entity,
                priority: 0,
                action: MovementAction::Start {
                    way: MovementWay::Pathfind(Vec3::new(4000.0, 0.0, 4000.0)),
                    speed: None,
                    source: MovementSource::Run,
                },
            }));

        let _ = app
            .world_mut()
            .run_system_once(apply_final_movement_decision);
        let stats_2 = app.world().resource::<NavigationStats>().get_nav_path_count;
        assert_eq!(stats_2, 1, "冷却时间内相同目标不应重复寻路");

        // 玩家改变目标位置
        app.world_mut()
            .entity_mut(entity)
            .insert(FinalDecision(CommandMovement {
                entity,
                priority: 0,
                action: MovementAction::Start {
                    way: MovementWay::Pathfind(Vec3::new(2000.0, 0.0, 2000.0)),
                    speed: None,
                    source: MovementSource::Run,
                },
            }));

        let _ = app
            .world_mut()
            .run_system_once(apply_final_movement_decision);
        let stats_3 = app.world().resource::<NavigationStats>().get_nav_path_count;
        assert_eq!(stats_3, 2, "目标改变应立即触发新寻路");
    }
}
