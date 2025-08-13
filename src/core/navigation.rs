use bevy::prelude::*;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

use crate::core::{Configs, Movement};

pub struct PluginNavigaton;

impl Plugin for PluginNavigaton {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedPreUpdate, update);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct GridPos {
    x: usize,
    y: usize,
}

#[derive(Debug, Clone)]
struct AStarNode {
    pos: GridPos,
    g_cost: f32,
    h_cost: f32,
    parent: Option<GridPos>,
}

impl AStarNode {
    fn f_cost(&self) -> f32 {
        self.g_cost + self.h_cost
    }
}

impl PartialEq for AStarNode {
    fn eq(&self, other: &Self) -> bool {
        self.f_cost().eq(&other.f_cost())
    }
}

impl Eq for AStarNode {}

impl PartialOrd for AStarNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for AStarNode {
    fn cmp(&self, other: &Self) -> Ordering {
        // 反转比较，因为BinaryHeap是最大堆，我们需要最小堆
        other
            .f_cost()
            .partial_cmp(&self.f_cost())
            .unwrap_or(Ordering::Equal)
    }
}

pub fn find_path(configs: &Configs, start: Vec3, end: Vec3) -> Option<Vec<Vec2>> {
    let grid = &configs.navigation_grid;

    let start_pos = world_to_grid(grid, start);
    let end_pos = world_to_grid(grid, end);

    if !is_valid_pos(grid, start_pos) || !is_valid_pos(grid, end_pos) {
        return None;
    }

    let mut open_set = BinaryHeap::new();
    let mut closed_set = HashMap::new();
    let mut came_from = HashMap::new();

    let start_node = AStarNode {
        pos: start_pos,
        g_cost: 0.0,
        h_cost: heuristic_cost(grid, start_pos, end_pos),
        parent: None,
    };

    open_set.push(start_node);

    while let Some(current) = open_set.pop() {
        if current.pos == end_pos {
            // 重建路径
            let mut path = Vec::new();
            let mut current_pos = end_pos;

            while let Some(&parent_pos) = came_from.get(&current_pos) {
                let world_pos = grid_to_world(grid, current_pos);
                path.push(Vec2::new(world_pos.x, world_pos.z));
                current_pos = parent_pos;
            }

            // 添加起始点
            let start_world = grid_to_world(grid, start_pos);
            path.push(Vec2::new(start_world.x, start_world.z));

            path.reverse();
            return Some(path);
        }

        closed_set.insert(current.pos, current.g_cost);

        // 检查邻居
        for neighbor_pos in get_neighbors(grid, current.pos) {
            if closed_set.contains_key(&neighbor_pos) {
                continue;
            }

            let tentative_g_cost = current.g_cost + distance_cost(grid, current.pos, neighbor_pos);

            let neighbor_node = AStarNode {
                pos: neighbor_pos,
                g_cost: tentative_g_cost,
                h_cost: heuristic_cost(grid, neighbor_pos, end_pos),
                parent: Some(current.pos),
            };

            // 检查是否已经在open_set中有更好的路径
            let mut should_add = true;
            if let Some(existing_g_cost) = closed_set.get(&neighbor_pos) {
                if tentative_g_cost >= *existing_g_cost {
                    should_add = false;
                }
            }

            if should_add {
                came_from.insert(neighbor_pos, current.pos);
                open_set.push(neighbor_node);
            }
        }
    }

    None
}

fn world_to_grid(grid: &crate::core::ConfigNavigationGrid, world_pos: Vec3) -> GridPos {
    let x = ((world_pos.x - grid.min_grid_pos.x) / grid.cell_size).round() as usize;
    let y = ((-world_pos.z - grid.min_grid_pos.z) / grid.cell_size).round() as usize;

    GridPos {
        x: x.clamp(0, grid.x_len - 1),
        y: y.clamp(0, grid.y_len - 1),
    }
}

fn grid_to_world(grid: &crate::core::ConfigNavigationGrid, grid_pos: GridPos) -> Vec3 {
    grid.get_cell_pos(grid_pos.x, grid_pos.y)
}

fn is_valid_pos(grid: &crate::core::ConfigNavigationGrid, pos: GridPos) -> bool {
    pos.x < grid.x_len
        && pos.y < grid.y_len
        && !grid.cells[pos.x].is_empty()
        && pos.y < grid.cells[pos.x].len()
}

fn get_neighbors(grid: &crate::core::ConfigNavigationGrid, pos: GridPos) -> Vec<GridPos> {
    let mut neighbors = Vec::new();

    let directions = [
        (-1, -1),
        (-1, 0),
        (-1, 1),
        (0, -1),
        (0, 1),
        (1, -1),
        (1, 0),
        (1, 1),
    ];

    for (dx, dy) in directions {
        let new_x = pos.x as i32 + dx;
        let new_y = pos.y as i32 + dy;

        if new_x >= 0 && new_y >= 0 {
            let neighbor_pos = GridPos {
                x: new_x as usize,
                y: new_y as usize,
            };

            if is_valid_pos(grid, neighbor_pos) {
                neighbors.push(neighbor_pos);
            }
        }
    }

    neighbors
}

fn distance_cost(grid: &crate::core::ConfigNavigationGrid, from: GridPos, to: GridPos) -> f32 {
    let dx = (to.x as i32 - from.x as i32).abs() as f32;
    let dy = (to.y as i32 - from.y as i32).abs() as f32;

    // 对角线移动成本更高
    if dx == 1.0 && dy == 1.0 {
        1.414 * grid.cell_size // sqrt(2)
    } else {
        grid.cell_size
    }
}

fn heuristic_cost(grid: &crate::core::ConfigNavigationGrid, from: GridPos, to: GridPos) -> f32 {
    // 使用预制的启发式值加上欧几里得距离
    let cell_heuristic = grid.cells[from.x][from.y].heuristic;

    let dx = (to.x as i32 - from.x as i32).abs() as f32;
    let dy = (to.y as i32 - from.y as i32).abs() as f32;
    let euclidean_distance = (dx * dx + dy * dy).sqrt() * grid.cell_size;

    cell_heuristic + euclidean_distance
}

fn update(configs: Res<Configs>, mut q_movement: Query<&mut Transform, With<Movement>>) {
    for mut transform in q_movement.iter_mut() {
        let cell = configs
            .navigation_grid
            .get_cell_by_pos(transform.translation);
        transform.translation.y = cell.y;

        if transform.translation.y < 0.0 {
            transform.translation.y = 0.0;
        }
    }
}
