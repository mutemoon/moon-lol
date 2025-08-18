mod astar;
mod smoother;

pub use astar::*;
pub use smoother::*;

use bevy::prelude::*;
use std::time::Instant;

use crate::core::{CommandMovementFollowPath, ConfigNavigationGrid, Movement};
use crate::system_debug;

#[derive(Event, Debug)]
pub struct CommandNavigationTo(pub Vec2);

#[derive(Default)]
pub struct PluginNavigaton;

impl Plugin for PluginNavigaton {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update);
        app.add_observer(command_movement_move_to);
    }
}

fn update(grid: Res<ConfigNavigationGrid>, mut q_movement: Query<&mut Transform, With<Movement>>) {
    for mut transform in q_movement.iter_mut() {
        transform.translation = grid.get_world_position_by_position(&transform.translation.xz());
    }
}

fn command_movement_move_to(
    trigger: Trigger<CommandNavigationTo>,
    mut commands: Commands,
    grid: Res<ConfigNavigationGrid>,
    mut q_transform: Query<&Transform>,
) {
    let entity = trigger.target();
    let destination = trigger.event().0;

    // 获取当前位置
    if let Ok(transform) = q_transform.get_mut(entity) {
        let start_pos = transform.translation;
        let end_pos = Vec3::new(destination.x, start_pos.y, destination.y);

        let start = Instant::now();
        // 使用A*算法规划路径，对于单点移动，创建长度为1的路径
        if let Some(result) = find_path(&grid, start_pos, end_pos) {
            system_debug!(
                "command_movement_move_to",
                "Path found in {:.6}ms",
                start.elapsed().as_millis()
            );

            commands.trigger_targets(CommandMovementFollowPath(result), entity);
        }
    }
}

/// 主要的寻路函数，结合A*和漏斗算法
pub fn find_path(grid: &ConfigNavigationGrid, start: Vec3, end: Vec3) -> Option<Vec<Vec2>> {
    // 首先使用A*找到网格路径
    let grid_path = find_grid_path(grid, start, end)?;

    return Some(post_process_path(
        grid,
        &simplify_path(&grid_path),
        &start,
        &end,
    ));
}

pub fn post_process_path(
    grid: &ConfigNavigationGrid,
    path: &Vec<(f32, f32)>,
    start: &Vec3,
    end: &Vec3,
) -> Vec<Vec2> {
    let mut simplified_path = path
        .iter()
        .map(|(x, y)| grid.get_position_by_float_xy(&Vec2::new(*x, *y)))
        .collect::<Vec<_>>();

    simplified_path.remove(0);
    simplified_path.insert(0, start.xz());

    simplified_path.pop();
    simplified_path.push(end.xz());

    // 路径优化：从后往前，检测两点之间的格子，根据 grid.get_cell_by_position().is_walkable() 判断该格子是否可走，来优化路径
    // optimize_path_line_of_sight(grid, &mut simplified_path);

    return simplified_path;
}

/// 检测两个点之间是否有直接的视线（所有格子都可行走）
fn has_line_of_sight(grid: &ConfigNavigationGrid, start: Vec2, end: Vec2) -> bool {
    // 使用Bresenham算法或类似的方法来遍历两点之间的所有格子
    let dx = (end.x - start.x).abs();
    let dy = (end.y - start.y).abs();

    // 确定步进的数量，使用较大的坐标差值
    let steps = (dx.max(dy) / (grid.cell_size * 0.5)).ceil() as i32;

    if steps == 0 {
        return true;
    }

    let step_x = (end.x - start.x) / steps as f32;
    let step_y = (end.y - start.y) / steps as f32;

    // 检查路径上的每个点
    for i in 0..=steps {
        let check_pos = Vec2::new(start.x + step_x * i as f32, start.y + step_y * i as f32);

        // 如果任何一个格子不可行走，则返回false
        if !grid.get_cell_by_position(&check_pos).is_walkable() {
            return false;
        }
    }

    true
}

/// 使用视线检测优化路径，移除不必要的中间点
fn optimize_path_line_of_sight(grid: &ConfigNavigationGrid, path: &mut Vec<Vec2>) {
    if path.len() <= 2 {
        return;
    }

    let mut i = 0;
    while i + 2 < path.len() {
        // 检查当前点是否可以直接到达后面的点（跳过中间点）
        let mut j = i + 2;
        let mut can_skip_to = i + 1; // 默认只能到达下一个点

        // 从最远的点开始检查，找到能够直接到达的最远点
        while j < path.len() {
            if has_line_of_sight(grid, path[i], path[j]) {
                can_skip_to = j;
                j += 1;
            } else {
                break;
            }
        }

        // 如果可以跳过中间点，则移除它们
        if can_skip_to > i + 1 {
            let remove_count = can_skip_to - i - 1;
            for _ in 0..remove_count {
                path.remove(i + 1);
            }
        }

        i += 1;
    }
}
