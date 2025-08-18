use core::f32;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Default)]
pub struct PluginMovement;

impl Plugin for PluginMovement {
    fn build(&self, app: &mut App) {
        app.register_type::<Movement>();

        app.add_systems(FixedUpdate, update_path_movement);

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

        // 本帧可移动的总距离
        let mut remaining_distance_this_frame = movement.speed * dt;
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
            movement_state.velocity = last_direction * movement.speed;
        }

        // 更新旋转
        if last_direction.length_squared() > 0.0 {
            transform.rotation = Quat::from_rotation_y(
                -(last_direction.y.atan2(last_direction.x) + f32::consts::PI / 2.0),
            );
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
