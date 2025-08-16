use std::time::Instant;

use core::f32;

use crate::{
    core::{navigation, ConfigMap},
    system_debug,
};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub struct PluginMovement;

impl Plugin for PluginMovement {
    fn build(&self, app: &mut App) {
        app.register_type::<Movement>();

        app.add_systems(FixedUpdate, update_path_movement);

        app.add_event::<CommandMovementMoveTo>();
        app.add_observer(command_movement_move_to);

        app.add_event::<CommandMovementFollowPath>();
        app.add_observer(command_movement_follow_path);

        app.add_event::<EventMovementEnd>();
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
    pub direction: Vec2,
    pub velocity: Vec2,
    pub current_target_index: usize,
    pub completed: bool,
}

impl MovementState {
    pub fn set_path(&mut self, path: Vec<Vec2>) {
        self.path = path;
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
}

#[derive(Event, Debug)]
pub struct EventMovementStart;

#[derive(Event, Debug)]
pub struct EventMovementEnd;

#[derive(Event, Debug)]
pub struct CommandMovementMoveTo(pub Vec2);

#[derive(Event, Debug)]
pub struct CommandMovementFollowPath(pub Vec<Vec2>);

fn update_path_movement(
    mut commands: Commands,
    mut query: Query<(Entity, &Movement, &mut MovementState)>,
    mut q_transform: Query<&mut Transform>,
    timer: Res<Time<Fixed>>,
) {
    let dt = timer.delta_secs();

    for (entity, movement, mut movement_state) in query.iter_mut() {
        if movement_state.completed || movement_state.path.is_empty() {
            continue;
        }

        let mut transform = q_transform.get_mut(entity).unwrap();
        let current_pos = transform.translation.xz();

        // 找到当前应该前往的目标点
        let target = find_next_target_point(&mut movement_state, current_pos);

        if let Some(target) = target {
            // 计算移动方向和速度
            let direction = target - current_pos;
            let distance = direction.length();

            if distance > movement.speed * dt {
                // 还没到达目标点，继续移动
                let normalized_direction = direction / distance;
                movement_state.direction = normalized_direction;
                movement_state.velocity = normalized_direction * movement.speed;

                // 更新位置
                let new_pos = current_pos + movement_state.velocity * dt;
                transform.translation.x = new_pos.x;
                transform.translation.z = new_pos.y;

                // 更新旋转朝向移动方向
                if movement_state.velocity.length() > 0.1 {
                    transform.rotation = Quat::from_rotation_y(
                        -(movement_state.velocity.y.atan2(movement_state.velocity.x)
                            + f32::consts::PI / 2.0),
                    );
                }
            } else {
                // 到达当前目标点
                transform.translation.x = target.x;
                transform.translation.z = target.y;
                movement_state.velocity = Vec2::ZERO;
                movement_state.direction = Vec2::ZERO;

                let new_index = movement_state.current_target_index + 1;

                if new_index >= movement_state.path.len() {
                    // 完成路径移动
                    movement_state.completed = true;
                    movement_state.clear_path();
                    commands.trigger_targets(EventMovementEnd, entity);
                } else {
                    // 更新路径状态到下一个点
                    movement_state.current_target_index = new_index;
                }
            }
        } else {
            // 没有有效的目标点，结束移动
            movement_state.completed = true;
            movement_state.velocity = Vec2::ZERO;
            movement_state.direction = Vec2::ZERO;
            movement_state.clear_path();
            commands.trigger_targets(EventMovementEnd, entity);
        }
    }
}

fn find_next_target_point(
    movement_state: &mut MovementState,
    current_position: Vec2,
) -> Option<Vec2> {
    let path = &movement_state.path;

    if path.is_empty() || movement_state.current_target_index >= path.len() {
        return None;
    }

    // 如果还没有开始移动，找到最近的前进方向的点
    if movement_state.current_target_index == 0 {
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

        movement_state.current_target_index = closest_index;
    }

    Some(path[movement_state.current_target_index])
}

fn command_movement_move_to(
    trigger: Trigger<CommandMovementMoveTo>,
    mut commands: Commands,
    configs: Res<ConfigMap>,
    mut q_transform: Query<(&Transform, &mut MovementState)>,
) {
    let entity = trigger.target();
    let destination = trigger.event().0;

    // 获取当前位置
    if let Ok((transform, mut movement_state)) = q_transform.get_mut(entity) {
        let start_pos = transform.translation;
        let end_pos = Vec3::new(destination.x, start_pos.y, destination.y);

        let start = Instant::now();
        // 使用A*算法规划路径，对于单点移动，创建长度为1的路径
        if let Some(path) = navigation::find_path(&configs, start_pos, end_pos) {
            let duration = start.elapsed();
            system_debug!(
                "command_movement_move_to",
                "Path found in {:.6}ms",
                duration.as_millis()
            );
            // 设置新的路径
            movement_state.set_path(path);
            commands.trigger_targets(EventMovementStart, entity);
        }
    }
}

fn command_movement_follow_path(
    trigger: Trigger<CommandMovementFollowPath>,
    mut commands: Commands,
    mut q_transform: Query<(&Transform, &mut MovementState)>,
) {
    let entity = trigger.target();
    let path = trigger.event().0.clone();

    if !path.is_empty() {
        // 设置新路径
        if let Ok((_, mut movement_state)) = q_transform.get_mut(entity) {
            movement_state.set_path(path);
            commands.trigger_targets(EventMovementStart, entity);
        }
    }
}
