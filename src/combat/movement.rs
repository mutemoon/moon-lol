use crate::{
    combat::{navigation::Obstacle, Bounding},
    system_debug,
};
use bevy::prelude::*;
use rvo2::RVOSimulatorWrapper;
use std::{collections::HashMap, time::Instant};
use vleue_navigator::prelude::*;

pub struct PluginMovement;

impl Plugin for PluginMovement {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(FixedUpdate, (update, update_path_movement));

        app.add_event::<CommandMovementMoveTo>();
        app.add_observer(command_movement_move_to);

        app.add_event::<CommandMovementFollowPath>();
        app.add_observer(command_movement_follow_path);

        app.add_event::<EventMovementMoveEnd>();
    }
}

#[derive(Component)]
#[require(MovementVelocity)]
pub struct Movement {
    pub speed: f32,
}

#[derive(Component, Default)]
pub struct MovementVelocity(pub Vec2);

#[derive(Component)]
pub struct MovementDestination(pub Vec2);

#[derive(Component)]
pub struct MovementPath(pub Vec<Vec2>);

#[derive(Component)]
pub struct MovementPathState {
    pub current_target_index: usize,
    pub completed: bool,
}

#[derive(Event, Debug)]
pub struct EventMovementMoveEnd;

#[derive(Event, Debug)]
pub struct CommandMovementMoveTo(pub Vec2);

#[derive(Event, Debug)]
pub struct CommandMovementFollowPath(pub Vec<Vec2>);

#[derive(Resource)]
pub struct ObstacleVerticesArray(pub Vec<Vec<[f32; 2]>>);

fn setup(
    mut commands: Commands,
    cachable_obstacles: Query<
        (&GlobalTransform, &PrimitiveObstacle),
        (With<Obstacle>, Without<Movement>),
    >,
) {
    let start = Instant::now();

    let mut obstacle_vertices_array = ObstacleVerticesArray(Vec::new());
    for (global_transform, &primitive_obstacle) in cachable_obstacles.iter() {
        let vertices = primitive_obstacle.get_polygons(
            global_transform,
            &Transform::default(),
            (global_transform.forward(), 0.0),
        );

        let mut vertices_array: Vec<[f32; 2]> = vertices
            .iter()
            .flat_map(|v| v.iter().map(|v| [v.x, v.y]))
            .collect();

        vertices_array.reverse();
        vertices_array.pop();

        obstacle_vertices_array.0.push(vertices_array);
    }
    commands.insert_resource(obstacle_vertices_array);

    debug!("init_obstacle: {:?}", start.elapsed());
}

fn update(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &Movement,
        &MovementDestination,
        &mut MovementVelocity,
        &Bounding,
    )>,
    mut q_transform: Query<&mut Transform>,
    timer: Res<Time<Fixed>>,
    obstacle_vertices_array: Res<ObstacleVerticesArray>,
) {
    let mut simulator = RVOSimulatorWrapper::new();
    simulator.set_time_step(timer.timestep().as_secs_f32());

    for vertices_array in obstacle_vertices_array.0.iter() {
        simulator.add_obstacle(&vertices_array);
    }

    simulator.process_obstacles();

    let mut entity_to_index: HashMap<Entity, usize> = HashMap::new();

    for (entity, movement, movement_destination, movement_velocity, bounding) in query.iter_mut() {
        let destination = movement_destination.0;

        let transform = q_transform.get(entity).unwrap();

        let position = transform.translation.xz();

        let (old_velocity, pref_velocity) = {
            let target = destination;
            let direction = target - position;
            let velocity = if direction.length() > 0.0 {
                direction.normalize() * movement.speed
            } else {
                Vec2::ZERO
            };

            (movement_velocity.0, velocity)
        };

        let neighbor_dist = bounding.radius * 2.0;
        let max_neighbors = 25;
        let time_horizon = 10.0;
        let time_horizon_obst = 3.0;
        let radius = bounding.radius;
        let max_speed = movement.speed;
        let index = simulator.add_agent(
            &[position.x, position.y],
            neighbor_dist,
            max_neighbors,
            time_horizon,
            time_horizon_obst,
            radius,
            max_speed,
            &[old_velocity.x, old_velocity.y],
        );

        entity_to_index.insert(entity, index);

        simulator.set_agent_pref_velocity(index, &[pref_velocity.x, pref_velocity.y]);
    }

    simulator.do_step();

    for (entity, _, movement_destination, mut movement_velocity, _) in query.iter_mut() {
        let target = movement_destination.0;

        let mut transform = q_transform.get_mut(entity).unwrap();

        let index = *entity_to_index.get(&entity).unwrap(); // 使用正确的索引

        let current_pos = simulator.get_agent_position(index);
        let current_velocity = simulator.get_agent_velocity(index);

        transform.translation = Vec3::new(current_pos[0], transform.translation.y, current_pos[1]);

        if target.distance(Vec2::new(current_pos[0], current_pos[1])) < 10.0 {
            transform.translation = Vec3::new(target.x, transform.translation.y, target.y);
            commands.entity(entity).remove::<MovementDestination>();
            commands.trigger_targets(EventMovementMoveEnd, entity);
        }

        transform.rotation = Quat::from_rotation_y(current_velocity[0].atan2(current_velocity[1]));

        movement_velocity.0 = Vec2::new(current_velocity[0], current_velocity[1]);
    }
}

fn update_path_movement(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &Movement,
        &MovementPath,
        &mut MovementPathState,
        &mut MovementVelocity,
        &Bounding,
    )>,
    mut q_transform: Query<&mut Transform>,
    timer: Res<Time<Fixed>>,
    obstacle_vertices_array: Res<ObstacleVerticesArray>,
) {
    let mut simulator = RVOSimulatorWrapper::new();
    simulator.set_time_step(timer.timestep().as_secs_f32());

    for vertices_array in obstacle_vertices_array.0.iter() {
        simulator.add_obstacle(&vertices_array);
    }

    simulator.process_obstacles();

    let mut entity_to_index: HashMap<Entity, usize> = HashMap::new();

    for (entity, movement, movement_path, mut path_state, movement_velocity, bounding) in
        query.iter_mut()
    {
        if path_state.completed || movement_path.0.is_empty() {
            continue;
        }

        let transform = q_transform.get(entity).unwrap();
        let position = transform.translation.xz();

        // 找到当前应该前往的目标点
        let target = find_next_target_point(&movement_path.0, &mut path_state, position);

        if target.is_none() {
            path_state.completed = true;
            commands.entity(entity).remove::<MovementPath>();
            commands.entity(entity).remove::<MovementPathState>();
            commands.trigger_targets(EventMovementMoveEnd, entity);
            continue;
        }

        let target = target.unwrap();

        let (old_velocity, pref_velocity) = {
            let direction = target - position;
            let velocity = if direction.length() > 0.0 {
                direction.normalize() * movement.speed
            } else {
                Vec2::ZERO
            };

            (movement_velocity.0, velocity)
        };

        let neighbor_dist = bounding.radius * 2.0;
        let max_neighbors = 25;
        let time_horizon = 10.0;
        let time_horizon_obst = 3.0;
        let radius = bounding.radius;
        let max_speed = movement.speed;
        let index = simulator.add_agent(
            &[position.x, position.y],
            neighbor_dist,
            max_neighbors,
            time_horizon,
            time_horizon_obst,
            radius,
            max_speed,
            &[old_velocity.x, old_velocity.y],
        );

        entity_to_index.insert(entity, index);
        simulator.set_agent_pref_velocity(index, &[pref_velocity.x, pref_velocity.y]);
    }

    simulator.do_step();

    for (entity, _, movement_path, mut path_state, mut movement_velocity, _) in query.iter_mut() {
        if path_state.completed || movement_path.0.is_empty() {
            continue;
        }

        let mut transform = q_transform.get_mut(entity).unwrap();
        let index = *entity_to_index.get(&entity).unwrap();

        let current_pos = simulator.get_agent_position(index);
        let current_velocity = simulator.get_agent_velocity(index);

        transform.translation = Vec3::new(current_pos[0], transform.translation.y, current_pos[1]);

        // 检查是否到达当前目标点
        if path_state.current_target_index < movement_path.0.len() {
            let target = movement_path.0[path_state.current_target_index];
            if target.distance(Vec2::new(current_pos[0], current_pos[1])) < 10.0 {
                path_state.current_target_index += 1;

                // 如果到达最后一个点，完成路径移动
                if path_state.current_target_index >= movement_path.0.len() {
                    path_state.completed = true;
                    commands.entity(entity).remove::<MovementPath>();
                    commands.entity(entity).remove::<MovementPathState>();
                    commands.trigger_targets(EventMovementMoveEnd, entity);
                }
            }
        }

        transform.rotation = Quat::from_rotation_y(current_velocity[0].atan2(current_velocity[1]));
        movement_velocity.0 = Vec2::new(current_velocity[0], current_velocity[1]);
    }
}

fn find_next_target_point(
    path: &[Vec2],
    path_state: &mut MovementPathState,
    current_position: Vec2,
) -> Option<Vec2> {
    if path.is_empty() || path_state.current_target_index >= path.len() {
        return None;
    }

    // 如果还没有开始移动，找到最近的前进方向的点
    if path_state.current_target_index == 0 {
        let mut closest_index = 0;
        let mut min_distance = f32::INFINITY;

        for (i, &point) in path.iter().enumerate() {
            let distance = current_position.distance(point);
            if distance < min_distance {
                min_distance = distance;
                closest_index = i;
            }
        }

        // 确保不往回走，如果找到的最近点不是第一个点，检查是否应该选择下一个点
        if closest_index > 0 {
            let prev_point = path[closest_index - 1];
            let curr_point = path[closest_index];

            // 计算从前一个点到当前点的方向向量
            let path_direction = (curr_point - prev_point).normalize();
            // 计算从前一个点到当前位置的向量
            let position_direction = (current_position - prev_point).normalize();

            // 如果当前位置在路径方向的前方，选择当前点；否则选择下一个点（如果存在）
            if path_direction.dot(position_direction) > 0.0 && closest_index + 1 < path.len() {
                closest_index += 1;
            }
        }

        path_state.current_target_index = closest_index;
    }

    Some(path[path_state.current_target_index])
}

fn command_movement_move_to(trigger: Trigger<CommandMovementMoveTo>, mut commands: Commands) {
    let entity = trigger.target();
    let destination = trigger.event().0;

    system_debug!(
        "action_set_move_target",
        "Entity {:?} received move command to {:?}",
        entity,
        destination,
    );

    // 清除路径移动状态，切换到目标点移动
    commands.entity(entity).remove::<MovementPath>();
    commands.entity(entity).remove::<MovementPathState>();
    commands
        .entity(entity)
        .insert(MovementDestination(destination));
}

fn command_movement_follow_path(
    trigger: Trigger<CommandMovementFollowPath>,
    mut commands: Commands,
) {
    let entity = trigger.target();
    let path = trigger.event().0.clone();

    system_debug!(
        "action_set_move_path",
        "Entity {:?} received path movement command with {} points",
        entity,
        path.len(),
    );

    if !path.is_empty() {
        // 清除目标点移动状态，切换到路径移动
        commands.entity(entity).remove::<MovementDestination>();
        commands.entity(entity).insert(MovementPath(path));
        commands.entity(entity).insert(MovementPathState {
            current_target_index: 0,
            completed: false,
        });
    }
}
