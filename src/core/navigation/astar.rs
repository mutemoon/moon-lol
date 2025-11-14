use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

use bevy::prelude::*;

use lol_config::ConfigNavigationGrid;

#[derive(Debug, Clone)]
struct AStarNode {
    pos: (usize, usize),
    g_cost: f32,
    h_cost: f32,
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

/// A* 搜索结果
#[derive(Debug, Clone)]
pub struct AStarResult {
    pub path: Vec<(usize, usize)>,
    pub visited_cells: Vec<(usize, usize)>,
}

/// 使用A*算法找到网格路径
pub fn find_grid_path(
    grid: &ConfigNavigationGrid,
    start: &Vec2,
    end: &Vec2,
) -> Option<Vec<(usize, usize)>> {
    find_grid_path_with_result(grid, start, end).map(|result| result.path)
}

/// 使用A*算法找到网格路径，返回详细结果
pub fn find_grid_path_with_result(
    grid: &ConfigNavigationGrid,
    start: &Vec2,
    end: &Vec2,
) -> Option<AStarResult> {
    let start_pos = grid.get_cell_xy_by_position(start);
    let end_pos = grid.get_cell_xy_by_position(end);

    debug!(
        "A* pathfinding: start=({}, {}) end=({}, {})",
        start_pos.0, start_pos.1, end_pos.0, end_pos.1
    );

    if !grid.is_walkable_by_xy(start_pos) {
        warn!("A* pathfinding: Invalid start position");
        return None;
    }

    let mut open_set = BinaryHeap::new();
    let mut closed_set = HashMap::new();
    let mut came_from = HashMap::new();
    let mut g_scores = HashMap::new();
    let mut visited_cells = Vec::new(); // 跟踪访问过的单元格

    let start_node = AStarNode {
        pos: start_pos,
        g_cost: 0.0,
        h_cost: heuristic_cost(grid.cell_size, start_pos, end_pos),
    };

    open_set.push(start_node);
    g_scores.insert(start_pos, 0.0);

    let mut iterations = 0;

    while let Some(current) = open_set.pop() {
        iterations += 1;

        // 记录访问过的单元格
        visited_cells.push(current.pos);

        if current.pos == end_pos {
            debug!("A* pathfinding: Found path in {} iterations", iterations);
            // 重建网格路径
            let mut path = Vec::new();
            let mut current_pos = end_pos;

            while let Some(&parent_pos) = came_from.get(&current_pos) {
                path.push(current_pos);
                current_pos = parent_pos;
            }
            path.push(start_pos);
            path.reverse();

            return Some(AStarResult {
                path,
                visited_cells,
            });
        }

        if let Some(&existing_g_cost) = closed_set.get(&current.pos) {
            if current.g_cost > existing_g_cost {
                continue;
            }
        }

        closed_set.insert(current.pos, current.g_cost);

        for neighbor_pos in get_neighbors(grid, current.pos) {
            if closed_set.contains_key(&neighbor_pos) {
                continue;
            }

            let tentative_g_cost =
                current.g_cost + distance_cost(grid.cell_size, current.pos, neighbor_pos);

            if let Some(&existing_g_cost) = g_scores.get(&neighbor_pos) {
                if tentative_g_cost >= existing_g_cost {
                    continue;
                }
            }

            let neighbor_node = AStarNode {
                pos: neighbor_pos,
                g_cost: tentative_g_cost,
                h_cost: heuristic_cost(grid.cell_size, neighbor_pos, end_pos),
            };

            came_from.insert(neighbor_pos, current.pos);
            g_scores.insert(neighbor_pos, tentative_g_cost);
            open_set.push(neighbor_node);
        }
    }

    warn!(
        "A* pathfinding: No path found after {} iterations",
        iterations
    );
    // 即使没有找到路径，也返回访问过的单元格信息
    Some(AStarResult {
        path: Vec::new(),
        visited_cells,
    })
}

fn get_neighbors(grid: &ConfigNavigationGrid, pos: (usize, usize)) -> Vec<(usize, usize)> {
    let mut neighbors = Vec::new();

    let directions = [
        (-1, 0),
        (0, -1),
        (0, 1),
        (1, 0),
        (-1, -1),
        (-1, 1),
        (1, -1),
        (1, 1),
    ];

    for (dx, dy) in directions {
        let new_x = pos.0 as i32 + dx;
        let new_y = pos.1 as i32 + dy;

        if new_x < 0 || new_y < 0 {
            continue;
        }

        let new_pos = (new_x as usize, new_y as usize);

        if !grid.is_walkable_by_xy(new_pos) {
            continue;
        }

        neighbors.push(new_pos);
    }

    neighbors
}

fn distance_cost(cell_size: f32, from: (usize, usize), to: (usize, usize)) -> f32 {
    let dx = (to.0 as i32 - from.0 as i32).abs() as f32;
    let dy = (to.1 as i32 - from.1 as i32).abs() as f32;

    // 对角线移动成本更高
    if dx == 1.0 && dy == 1.0 {
        1.414 * cell_size
    } else {
        cell_size
    }
}

fn heuristic_cost(cell_size: f32, from: (usize, usize), to: (usize, usize)) -> f32 {
    let dx = (to.0 as i32 - from.0 as i32).abs() as f32;
    let dy = (to.1 as i32 - from.1 as i32).abs() as f32;
    let euclidean_distance = (dx * dx + dy * dy).sqrt() * cell_size;

    // 引入一个非常小的权重来打破平局，p 应该很小
    // 例如 1.0 / (地图最大距离)
    const P: f32 = 1.0 / (300.0 * 300.0);
    return euclidean_distance * (1.0 + P);
}
