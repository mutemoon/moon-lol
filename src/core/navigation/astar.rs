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
        other
            .f_cost()
            .partial_cmp(&self.f_cost())
            .unwrap_or(Ordering::Equal)
    }
}

#[derive(Debug, Clone)]
pub struct AStarResult {
    pub path: Vec<(usize, usize)>,
    pub visited_cells: Vec<(usize, usize)>,
}

pub fn find_grid_path(
    grid: &ConfigNavigationGrid,
    start: &Vec2,
    end: &Vec2,
) -> Option<Vec<(usize, usize)>> {
    find_grid_path_with_result(grid, start, end).map(|result| result.path)
}

pub fn find_grid_path_with_result(
    grid: &ConfigNavigationGrid,
    start: &Vec2,
    end: &Vec2,
) -> Option<AStarResult> {
    let start_pos = grid.get_cell_xy_by_position(start);
    let end_pos = grid.get_cell_xy_by_position(end);

    if !grid.is_walkable_by_xy(start_pos) || !grid.is_walkable_by_xy(end_pos) {
        warn!("双向 A* 起点或终点位置无效");
        return None;
    }

    if start_pos == end_pos {
        return Some(AStarResult {
            path: vec![start_pos],
            visited_cells: vec![start_pos],
        });
    }

    let mut open_fwd = BinaryHeap::new();
    let mut open_bwd = BinaryHeap::new();

    let mut g_fwd = HashMap::new();
    let mut g_bwd = HashMap::new();

    let mut came_from_fwd = HashMap::new();
    let mut came_from_bwd = HashMap::new();

    let mut visited_cells = Vec::new();

    // 初始化正向搜索
    g_fwd.insert(start_pos, 0.0);
    open_fwd.push(AStarNode {
        pos: start_pos,
        g_cost: 0.0,
        h_cost: heuristic_cost(grid.cell_size, start_pos, end_pos),
    });

    // 初始化反向搜索
    g_bwd.insert(end_pos, 0.0);
    open_bwd.push(AStarNode {
        pos: end_pos,
        g_cost: 0.0,
        h_cost: heuristic_cost(grid.cell_size, end_pos, start_pos),
    });

    let mut best_path_cost = f32::MAX;
    let mut best_connection = None;
    let mut iterations = 0;

    while !open_fwd.is_empty() && !open_bwd.is_empty() {
        iterations += 1;
        if iterations > 10000 {
            warn!("双向 A* 超过迭代次数限制");
            return None;
        }

        // 优化验证：如果两端最小的 f_cost 之和已经超过了已知的最佳路径，则不可能找到更优解
        if let (Some(f), Some(b)) = (open_fwd.peek(), open_bwd.peek()) {
            if f.f_cost() + b.f_cost() >= best_path_cost && best_connection.is_some() {
                break;
            }
        }

        // 平衡扩展：选择节点较少的一端进行扩展
        let expand_forward = open_fwd.len() <= open_bwd.len();

        let current_node = if expand_forward {
            open_fwd.pop().unwrap()
        } else {
            open_bwd.pop().unwrap()
        };

        visited_cells.push(current_node.pos);

        let (current_g_map, other_g_map, current_came_from, current_open, target_pos) =
            if expand_forward {
                (
                    &mut g_fwd,
                    &g_bwd,
                    &mut came_from_fwd,
                    &mut open_fwd,
                    end_pos,
                )
            } else {
                (
                    &mut g_bwd,
                    &g_fwd,
                    &mut came_from_bwd,
                    &mut open_bwd,
                    start_pos,
                )
            };

        // 惰性删除检查：如果在该方向已经有更优路径到达此点，跳过
        if let Some(&g) = current_g_map.get(&current_node.pos) {
            if current_node.g_cost > g {
                continue;
            }
        }

        // 检查是否在当前节点与另一端相遇
        if let Some(&other_g) = other_g_map.get(&current_node.pos) {
            let total_cost = current_node.g_cost + other_g;
            if total_cost < best_path_cost {
                best_path_cost = total_cost;
                best_connection = Some(current_node.pos);
            }
        }

        // 处理邻居的逻辑提取为闭包以减少缩进
        let mut process_neighbor = |neighbor_pos: (usize, usize)| {
            let tentative_g =
                current_node.g_cost + movement_cost(grid, current_node.pos, neighbor_pos);

            if let Some(&existing_g) = current_g_map.get(&neighbor_pos) {
                if tentative_g >= existing_g {
                    return;
                }
            }

            current_came_from.insert(neighbor_pos, current_node.pos);
            current_g_map.insert(neighbor_pos, tentative_g);

            current_open.push(AStarNode {
                pos: neighbor_pos,
                g_cost: tentative_g,
                h_cost: heuristic_cost(grid.cell_size, neighbor_pos, target_pos),
            });

            // 扩展时立即检查连接，加速收敛
            if let Some(&other_g) = other_g_map.get(&neighbor_pos) {
                let total_cost = tentative_g + other_g;
                if total_cost < best_path_cost {
                    best_path_cost = total_cost;
                    best_connection = Some(neighbor_pos);
                }
            }
        };

        for neighbor_pos in get_neighbors(grid, current_node.pos) {
            process_neighbor(neighbor_pos);
        }
    }

    if let Some(meet_node) = best_connection {
        debug!("双向 A* 找到路径 迭代次数 {}", iterations);
        let path = reconstruct_bidirectional_path(meet_node, &came_from_fwd, &came_from_bwd);
        return Some(AStarResult {
            path,
            visited_cells,
        });
    }

    Some(AStarResult {
        path: Vec::new(),
        visited_cells,
    })
}

fn reconstruct_bidirectional_path(
    meet_node: (usize, usize),
    came_from_fwd: &HashMap<(usize, usize), (usize, usize)>,
    came_from_bwd: &HashMap<(usize, usize), (usize, usize)>,
) -> Vec<(usize, usize)> {
    let mut path = Vec::new();

    // 1. 从相遇点回溯到起点
    let mut curr = meet_node;
    path.push(curr);
    while let Some(&parent) = came_from_fwd.get(&curr) {
        path.push(parent);
        curr = parent;
    }
    path.reverse();

    // 2. 从相遇点回溯到终点 (注意：came_from_bwd 记录的是从终点反向搜索的父节点)
    curr = meet_node;
    while let Some(&parent) = came_from_bwd.get(&curr) {
        path.push(parent);
        curr = parent;
    }

    path
}

fn get_neighbors(grid: &ConfigNavigationGrid, pos: (usize, usize)) -> Vec<(usize, usize)> {
    let mut neighbors = Vec::with_capacity(8);
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
        if grid.is_walkable_by_xy(new_pos) {
            neighbors.push(new_pos);
        }
    }
    neighbors
}

fn distance_cost(cell_size: f32, from: (usize, usize), to: (usize, usize)) -> f32 {
    let dx = (to.0 as i32 - from.0 as i32).abs();
    let dy = (to.1 as i32 - from.1 as i32).abs();

    if dx == 1 && dy == 1 {
        1.414 * cell_size
    } else {
        cell_size
    }
}

/// 计算从 from 移动到 to 的实际成本（包含动态障碍物成本）
fn movement_cost(grid: &ConfigNavigationGrid, from: (usize, usize), to: (usize, usize)) -> f32 {
    let base_cost = distance_cost(grid.cell_size, from, to);
    let cell_cost = grid.get_cell_cost(to);
    base_cost + cell_cost
}

fn heuristic_cost(cell_size: f32, from: (usize, usize), to: (usize, usize)) -> f32 {
    let dx = (to.0 as i32 - from.0 as i32).abs() as f32;
    let dy = (to.1 as i32 - from.1 as i32).abs() as f32;
    let euclidean = (dx * dx + dy * dy).sqrt() * cell_size;

    const P: f32 = 1.0 / (300.0 * 300.0);
    euclidean * (1.0 + P)
}
