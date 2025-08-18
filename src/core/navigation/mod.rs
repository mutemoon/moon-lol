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
    optimize_path_line_of_sight_improved(grid, &mut simplified_path);

    return simplified_path;
}

pub fn has_line_of_sight(
    start: Vec2,
    end: Vec2,
    cell_size: f32,
    is_walkable: impl Fn(usize, usize) -> bool,
) -> bool {
    // --- 定义和初始化部分不变 ---
    let start_grid_x = (start.x / cell_size).floor() as isize;
    let start_grid_y = (start.y / cell_size).floor() as isize;
    let end_grid_x = (end.x / cell_size).floor() as isize;
    let end_grid_y = (end.y / cell_size).floor() as isize;

    let mut current_grid_x = start_grid_x;
    let mut current_grid_y = start_grid_y;

    if !is_walkable(current_grid_x as usize, current_grid_y as usize) {
        return false;
    }

    // 如果起点和终点在同一个格子，直接认为有视线
    if current_grid_x == end_grid_x && current_grid_y == end_grid_y {
        return true;
    }

    let direction = end - start;
    let step_x = if direction.x >= 0.0 { 1 } else { -1 };
    let step_y = if direction.y >= 0.0 { 1 } else { -1 };

    let t_delta_x = if direction.x.abs() < 1e-6 {
        f32::MAX
    } else {
        (cell_size / direction.x).abs()
    };
    let t_delta_y = if direction.y.abs() < 1e-6 {
        f32::MAX
    } else {
        (cell_size / direction.y).abs()
    };

    let mut t_max_x = if direction.x > 0.0 {
        ((start_grid_x + 1) as f32 * cell_size - start.x) / direction.x
    } else if direction.x < 0.0 {
        (start.x - start_grid_x as f32 * cell_size) / -direction.x
    } else {
        f32::MAX
    };

    let mut t_max_y = if direction.y > 0.0 {
        ((start_grid_y + 1) as f32 * cell_size - start.y) / direction.y
    } else if direction.y < 0.0 {
        (start.y - start_grid_y as f32 * cell_size) / -direction.y
    } else {
        f32::MAX
    };

    let steps_to_take = (end_grid_x - start_grid_x).abs() + (end_grid_y - start_grid_y).abs();
    const CORNER_EPSILON: f32 = 1e-6;

    for _ in 0..steps_to_take {
        if (t_max_x - t_max_y).abs() < CORNER_EPSILON {
            // --- 核心修正：豁免对终点交叉点的严格检查 ---
            // let is_final_diagonal_step =
            //     (current_grid_x + step_x == end_grid_x) && (current_grid_y + step_y == end_grid_y);

            // if !is_final_diagonal_step {
            //     // 只有在“中途切角”时，才进行严格的两侧检查
            //     let corner_x_walkable =
            //         is_walkable((current_grid_x + step_x) as usize, current_grid_y as usize);
            //     let corner_y_walkable =
            //         is_walkable(current_grid_x as usize, (current_grid_y + step_y) as usize);

            //     if !corner_x_walkable && !corner_y_walkable {
            //         // 只有当两侧都不可走时，才认为完全阻塞。
            //         // 这是一条更宽松的规则，允许“擦”过单个障碍物的角。
            //         // 如果需要严格规则，请使用 ||
            //         return false;
            //     }
            // }

            // 对角线移动
            current_grid_x += step_x;
            current_grid_y += step_y;
            t_max_x += t_delta_x;
            t_max_y += t_delta_y;
        } else if t_max_x < t_max_y {
            current_grid_x += step_x;
            t_max_x += t_delta_x;
        } else {
            current_grid_y += step_y;
            t_max_y += t_delta_y;
        }

        if !is_walkable(current_grid_x as usize, current_grid_y as usize) {
            return false;
        }

        if current_grid_x == end_grid_x && current_grid_y == end_grid_y {
            return true;
        }
    }

    true
}

/// 使用视线检测优化路径，移除不必要的中间点（优化版）
fn optimize_path_line_of_sight_improved(grid: &ConfigNavigationGrid, path: &mut Vec<Vec2>) {
    if path.len() <= 2 {
        return;
    }

    let mut optimized_path = Vec::new();
    optimized_path.push(path[0]); // 起点总是保留

    let mut current_index = 0;
    while current_index < path.len() - 1 {
        let mut furthest_visible_index = current_index + 1;

        // 从当前点的下一个点再往后查找，看能看到多远
        for lookahead_index in (current_index + 2)..path.len() {
            // 注意：这里也应该使用上面提到的“微小偏移”技巧来提高稳定性
            let start_pos = path[current_index] - grid.min_position;
            let end_pos = path[lookahead_index] - grid.min_position;

            println!(
                "{} -> {} 开始检测 起点: {:?}, 终点: {:?}",
                current_index, lookahead_index, start_pos, end_pos
            );
            if has_line_of_sight(start_pos, end_pos, grid.cell_size, |x, y| {
                let res = grid.get_cell_by_xy((x, y)).is_walkable();
                println!(
                    "检测 x: {}, y: {} 结果{}障碍物",
                    x,
                    y,
                    if res { "没有" } else { "存在" }
                );
                res
            }) {
                // 如果能看到更远的点，就更新索引
                furthest_visible_index = lookahead_index;
            } else {
                // 一旦视线被阻挡，就停止向更远的地方看
                println!("{} -> {} 存在障碍物", current_index, lookahead_index);
                break;
            }
        }

        // 将能看到的最远的点加入新路径
        optimized_path.push(path[furthest_visible_index]);

        // 从这个最远的点开始下一次查找
        current_index = furthest_visible_index;
    }

    // 用优化后的路径替换原始路径
    *path = optimized_path;
}
