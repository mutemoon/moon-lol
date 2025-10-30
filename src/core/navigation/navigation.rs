use bevy::prelude::*;
use std::time::Instant;

use lol_config::ConfigNavigationGrid;

use crate::core::{find_grid_path, Movement};
use crate::system_debug;

#[derive(Default)]
pub struct PluginNavigaton;

impl Plugin for PluginNavigaton {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update);
    }
}

fn update(grid: Res<ConfigNavigationGrid>, mut q_movement: Query<&mut Transform, With<Movement>>) {
    for mut transform in q_movement.iter_mut() {
        transform.translation = grid.get_world_position_by_position(&transform.translation.xz());
    }
}

pub fn get_nav_path(
    start_pos: &Vec2,
    end_pos: &Vec2,
    grid: &ConfigNavigationGrid,
) -> Option<Vec<Vec2>> {
    let start = Instant::now();

    // 检查起点和终点是否可直达
    let start_grid_pos = (start_pos - grid.min_position) / grid.cell_size;
    let end_grid_pos = (end_pos - grid.min_position) / grid.cell_size;

    let is_walkable_fn = |x, y| grid.get_cell_by_xy((x, y)).is_walkable();

    if has_line_of_sight(start_grid_pos, end_grid_pos, &is_walkable_fn) {
        system_debug!(
            "command_movement_move_to",
            "Direct path found in {:.6}ms",
            start.elapsed().as_millis()
        );
        return Some(vec![start_pos.clone(), end_pos.clone()]);
    }

    // 如果不可直达，则使用A*算法规划路径
    let result = find_path(&grid, start_pos, end_pos);

    system_debug!(
        "command_movement_move_to",
        "A* path found in {:.6}ms",
        start.elapsed().as_millis()
    );

    return result;
}

/// 主要的寻路函数，结合A*和漏斗算法
pub fn find_path(grid: &ConfigNavigationGrid, start: &Vec2, end: &Vec2) -> Option<Vec<Vec2>> {
    // 首先使用A*找到网格路径
    let grid_path = find_grid_path(grid, start, end)?;

    return Some(post_process_path(grid, &grid_path, &start, &end));
}

pub fn post_process_path(
    grid: &ConfigNavigationGrid,
    path: &Vec<(usize, usize)>,
    start: &Vec2,
    end: &Vec2,
) -> Vec<Vec2> {
    if path.is_empty() {
        return Vec::new();
    }

    let mut path = path
        .iter()
        .map(|&(x, y)| vec2(x as f32 + 0.5, y as f32 + 0.5))
        .collect::<Vec<_>>();

    path.remove(0);
    path.insert(0, (start - grid.min_position) / grid.cell_size);

    path.pop();
    path.push((end - grid.min_position) / grid.cell_size);

    let path = optimize_path(&path, &|x, y| grid.get_cell_by_xy((x, y)).is_walkable());

    let path = path
        .into_iter()
        .map(|v| grid.get_position_by_float_xy(&v))
        .collect::<Vec<_>>();

    return path;
}

fn optimize_path(path: &Vec<Vec2>, is_walkable: &impl Fn(usize, usize) -> bool) -> Vec<Vec2> {
    if path.len() <= 2 {
        return path.clone();
    }

    let mut optimized_path = vec![path[0]];
    let mut current_index = 0;
    while current_index < path.len() - 1 {
        // 默认最远能到达的点是下一个点
        let mut furthest_visible_index = current_index + 1;

        // 从路径的末尾向前迭代，寻找第一个可见的点
        for lookahead_index in ((current_index + 2)..path.len()).rev() {
            let start_pos = path[current_index];
            let end_pos = path[lookahead_index];

            // 只要找到一个可见的，那它一定是从后往前看的“最远”的点
            if has_line_of_sight(start_pos, end_pos, is_walkable) {
                furthest_visible_index = lookahead_index;
                break;
            }
        }

        optimized_path.push(path[furthest_visible_index]);
        current_index = furthest_visible_index;
    }

    optimized_path
}

pub fn has_line_of_sight(
    start: Vec2,
    end: Vec2,
    is_walkable: &impl Fn(usize, usize) -> bool,
) -> bool {
    const CORNER_EPSILON: f32 = 1e-6;

    let start_grid_x = start.x.floor() as isize;
    let start_grid_y = start.y.floor() as isize;
    let end_grid_x = end.x.floor() as isize;
    let end_grid_y = end.y.floor() as isize;

    let mut current_grid_x = start_grid_x;
    let mut current_grid_y = start_grid_y;

    if current_grid_x == end_grid_x && current_grid_y == end_grid_y {
        return true;
    }

    let direction = end - start;
    let step_x = direction.x.signum() as isize;
    let step_y = direction.y.signum() as isize;

    let t_delta_x = if direction.x.abs() < CORNER_EPSILON {
        f32::MAX
    } else {
        (1.0 / direction.x).abs()
    };
    let t_delta_y = if direction.y.abs() < CORNER_EPSILON {
        f32::MAX
    } else {
        (1.0 / direction.y).abs()
    };

    let mut t_max_x = if direction.x > 0.0 {
        ((start_grid_x + 1) as f32 - start.x) / direction.x
    } else if direction.x < 0.0 {
        (start.x - start_grid_x as f32) / -direction.x
    } else {
        f32::MAX
    };
    let mut t_max_y = if direction.y > 0.0 {
        ((start_grid_y + 1) as f32 - start.y) / direction.y
    } else if direction.y < 0.0 {
        (start.y - start_grid_y as f32) / -direction.y
    } else {
        f32::MAX
    };

    let steps_to_take = (end_grid_x - start_grid_x).abs() + (end_grid_y - start_grid_y).abs();

    for _ in 0..steps_to_take {
        // --- 核心算法逻辑 ---
        if (t_max_x - t_max_y).abs() < CORNER_EPSILON {
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

        // 检查新位置是否可行走
        if !is_walkable(current_grid_x as usize, current_grid_y as usize) {
            return false;
        }

        // 检查是否到达终点
        if current_grid_x == end_grid_x && current_grid_y == end_grid_y {
            return true;
        }
    }

    true
}
