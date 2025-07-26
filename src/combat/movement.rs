use std::time::Instant;

use bevy::prelude::*;
use rvo2::RVOSimulatorWrapper;
use vleue_navigator::prelude::*;

use crate::{
    combat::{navigation::Obstacle, *},
    game::GameState,
    system_debug,
};

#[derive(Event, Debug)]
pub struct CommandMove {
    pub target: Vec3,
}

#[derive(Event, Debug)]
pub struct MoveEnd;

#[derive(Event, Debug)]
pub struct ActionSetMoveTarget(pub Vec2);

/// 移动目标位置组件
#[derive(Component)]
pub struct MoveDestination(pub Vec2);

/// 移动速度组件
#[derive(Component, Default)]
pub struct MoveSpeed(pub f32);

/// 移动速度向量组件
#[derive(Component, Default)]
pub struct MoveVelocity(pub Option<Vec2>);

pub struct PluginMove;

#[derive(Resource)]
pub struct ObstacleVerticesArray(pub Vec<Vec<[f32; 2]>>);

impl Plugin for PluginMove {
    fn build(&self, app: &mut App) {
        app.add_event::<CommandMove>();
        app.add_event::<MoveEnd>();
        app.add_systems(OnEnter(GameState::Playing), setup);
        app.add_systems(FixedUpdate, update_move_rvo);
        app.add_observer(action_set_move_target);
        app.add_observer(on_command_move);
        app.add_observer(on_move_end);
    }
}

fn setup(
    mut commands: Commands,
    cachable_obstacles: Query<
        (&GlobalTransform, &PrimitiveObstacle),
        (With<Obstacle>, Without<MoveDestination>),
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

fn update_move_rvo(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut MoveDestination,
        &MoveSpeed,
        &mut MoveVelocity,
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

    for (entity, move_destination, move_speed, move_velocity, bounding) in query.iter_mut() {
        let transform = match q_transform.get(entity) {
            Ok(transform) => transform,
            Err(_) => {
                warn!("Entity {:?} missing Transform component", entity);
                continue;
            }
        };
        let position = transform.translation.xy();

        let (old_velocity, pref_velocity) = {
            let target = move_destination.0;
            let direction = target - position;
            let velocity = if direction.length() > 0.0 {
                direction.normalize() * move_speed.0
            } else {
                Vec2::ZERO
            };

            let old_velocity = move_velocity.0.unwrap_or(velocity);
            (old_velocity, velocity)
        };

        let neighbor_dist = bounding.radius * 2.0;
        let max_neighbors = 25;
        let time_horizon = 10.0;
        let time_horizon_obst = 3.0;
        let radius = 35.0;
        let max_speed = move_speed.0;
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

        simulator.set_agent_pref_velocity(index, &[pref_velocity.x, pref_velocity.y]);
    }

    simulator.do_step();

    for (index, (entity, move_destination, _move_speed, mut move_velocity, _bounding)) in
        query.iter_mut().enumerate()
    {
        let target = move_destination.0;

        // 第二阶段：处理可变查询
        let mut transform = match q_transform.get_mut(entity) {
            Ok(transform) => transform,
            Err(_) => {
                warn!("Entity {:?} missing Transform during update", entity);
                continue;
            }
        };
        let current_pos = simulator.get_agent_position(index);
        let current_velocity = simulator.get_agent_velocity(index);

        // 更新位置
        transform.translation = Vec3::new(current_pos[0], current_pos[1], transform.translation.z);

        // 判断是否接近目标点
        if target.distance(Vec2::new(current_pos[0], current_pos[1])) < 10.0 {
            transform.translation = Vec3::new(target.x, target.y, transform.translation.z);
            commands.trigger_targets(MoveEnd, entity);
        }

        // 更新旋转和速度
        transform.rotation = Quat::from_rotation_z(-current_velocity[0].atan2(current_velocity[1]));
        move_velocity.0 = Some(Vec2::new(current_velocity[0], current_velocity[1]));
    }
}

fn action_set_move_target(trigger: Trigger<ActionSetMoveTarget>, mut commands: Commands) {
    let entity = trigger.target();
    let destination = trigger.event().0;

    system_debug!(
        "action_set_move_target",
        "Entity {:?} received move command to {:?}",
        entity,
        destination,
    );

    commands.entity(entity).insert(MoveDestination(destination));
}

fn on_command_move(trigger: Trigger<CommandMove>, mut commands: Commands) {
    system_debug!(
        "on_command_move",
        "Entity {:?} received move command to {:?}",
        trigger.target(),
        trigger.target.xy(),
    );
    commands.trigger_targets(ActionSetMoveTarget(trigger.target.xy()), trigger.target());
}

fn on_move_end(trigger: Trigger<MoveEnd>, mut commands: Commands) {
    system_debug!(
        "on_move_end",
        "Entity {:?} reached move destination",
        trigger.target()
    );
    commands
        .entity(trigger.target())
        .remove::<MoveDestination>();
}
