use std::cmp::Ordering;
use std::collections::BinaryHeap;

use bevy::prelude::*;
use lol_base::grid::ConfigNavigationGrid;

#[derive(Debug, Clone)]
pub struct AStarNode {
    idx: usize,
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

#[derive(Resource, Default, Debug)]
pub struct AStarCache {
    pub open_fwd: BinaryHeap<AStarNode>,
    pub open_bwd: BinaryHeap<AStarNode>,
    pub g_fwd: Vec<f32>,
    pub g_bwd: Vec<f32>,
    pub came_from_fwd: Vec<u32>,
    pub came_from_bwd: Vec<u32>,
    pub visited_cells: Vec<(usize, usize)>,
    pub touched_indices: Vec<usize>,
}

impl AStarCache {
    pub fn ensure_capacity(&mut self, total_cells: usize) {
        if self.g_fwd.len() < total_cells {
            self.g_fwd.resize(total_cells, f32::INFINITY);
            self.g_bwd.resize(total_cells, f32::INFINITY);
            self.came_from_fwd.resize(total_cells, u32::MAX);
            self.came_from_bwd.resize(total_cells, u32::MAX);
        }
    }

    pub fn reset(&mut self) {
        self.open_fwd.clear();
        self.open_bwd.clear();
        self.visited_cells.clear();
        for &idx in &self.touched_indices {
            if idx < self.g_fwd.len() {
                self.g_fwd[idx] = f32::INFINITY;
                self.g_bwd[idx] = f32::INFINITY;
                self.came_from_fwd[idx] = u32::MAX;
                self.came_from_bwd[idx] = u32::MAX;
            }
        }
        self.touched_indices.clear();
    }
}

pub fn find_grid_path(
    grid: &ConfigNavigationGrid,
    start: &Vec2,
    end: &Vec2,
) -> Option<Vec<(usize, usize)>> {
    find_grid_path_with_result(grid, start, end).map(|result| result.path)
}

#[inline(always)]
fn pos_to_idx(pos: (usize, usize), x_len: usize) -> usize {
    pos.1 * x_len + pos.0
}

#[inline(always)]
fn idx_to_pos(idx: usize, x_len: usize) -> (usize, usize) {
    (idx % x_len, idx / x_len)
}

pub fn find_grid_path_with_result(
    grid: &ConfigNavigationGrid,
    start: &Vec2,
    end: &Vec2,
) -> Option<AStarResult> {
    let mut cache = AStarCache::default();
    find_grid_path_with_result_cache(grid, start, end, &mut cache)
}

pub fn find_grid_path_with_result_cache(
    grid: &ConfigNavigationGrid,
    start: &Vec2,
    end: &Vec2,
    cache: &mut AStarCache,
) -> Option<AStarResult> {
    let Some(start_pos) = grid.get_cell_xy_by_position(start) else {
        warn!("双向 A* 起点超出地图边界: {:?}", start);
        return None;
    };
    let Some(end_pos) = grid.get_cell_xy_by_position(end) else {
        warn!("双向 A* 终点超出地图边界: {:?}", end);
        return None;
    };

    if !grid.is_walkable_by_xy(start_pos) || !grid.is_walkable_by_xy(end_pos) {
        warn!("双向 A* 起点或终点位置不可行走");
        return None;
    }

    if start_pos == end_pos {
        return Some(AStarResult {
            path: vec![start_pos],
            visited_cells: vec![start_pos],
        });
    }

    let x_len = grid.x_len;
    let y_len = grid.y_len;
    let total_cells = x_len * y_len;
    if total_cells == 0 {
        return None;
    }

    cache.ensure_capacity(total_cells);
    cache.reset();

    let start_idx = pos_to_idx(start_pos, x_len);
    let end_idx = pos_to_idx(end_pos, x_len);

    // 初始化正向搜索
    cache.g_fwd[start_idx] = 0.0;
    cache.touched_indices.push(start_idx);
    cache.open_fwd.push(AStarNode {
        idx: start_idx,
        pos: start_pos,
        g_cost: 0.0,
        h_cost: heuristic_cost(grid.cell_size, start_pos, end_pos),
    });

    // 初始化反向搜索
    cache.g_bwd[end_idx] = 0.0;
    cache.touched_indices.push(end_idx);
    cache.open_bwd.push(AStarNode {
        idx: end_idx,
        pos: end_pos,
        g_cost: 0.0,
        h_cost: heuristic_cost(grid.cell_size, end_pos, start_pos),
    });

    let mut best_path_cost = f32::MAX;
    let mut best_connection = None;
    let mut iterations = 0;

    while !cache.open_fwd.is_empty() && !cache.open_bwd.is_empty() {
        iterations += 1;
        if iterations > 10000 {
            warn!("双向 A* 超过迭代次数限制");
            break;
        }

        // 优化验证：如果两端最小的 f_cost 之和已经超过了已知的最佳路径，则不可能找到更优解
        if let (Some(f), Some(b)) = (cache.open_fwd.peek(), cache.open_bwd.peek()) {
            if f.f_cost() + b.f_cost() >= best_path_cost && best_connection.is_some() {
                break;
            }
        }

        // 平衡扩展：选择节点较少的一端进行扩展
        let expand_forward = cache.open_fwd.len() <= cache.open_bwd.len();

        let current_node = if expand_forward {
            cache.open_fwd.pop().unwrap()
        } else {
            cache.open_bwd.pop().unwrap()
        };

        cache.visited_cells.push(current_node.pos);

        if expand_forward {
            // 惰性删除检查
            if current_node.g_cost > cache.g_fwd[current_node.idx] {
                continue;
            }

            // 检查是否在当前节点与另一端相遇
            let other_g = cache.g_bwd[current_node.idx];
            if other_g < f32::INFINITY {
                let total_cost = current_node.g_cost + other_g;
                if total_cost < best_path_cost {
                    best_path_cost = total_cost;
                    best_connection = Some(current_node.idx);
                }
            }

            for neighbor_pos in get_neighbors(grid, current_node.pos) {
                let neighbor_idx = pos_to_idx(neighbor_pos, x_len);
                let tentative_g =
                    current_node.g_cost + movement_cost(grid, current_node.pos, neighbor_pos);

                if tentative_g >= cache.g_fwd[neighbor_idx] {
                    continue;
                }

                if cache.g_fwd[neighbor_idx] == f32::INFINITY && cache.g_bwd[neighbor_idx] == f32::INFINITY {
                    cache.touched_indices.push(neighbor_idx);
                }

                cache.came_from_fwd[neighbor_idx] = current_node.idx as u32;
                cache.g_fwd[neighbor_idx] = tentative_g;

                cache.open_fwd.push(AStarNode {
                    idx: neighbor_idx,
                    pos: neighbor_pos,
                    g_cost: tentative_g,
                    h_cost: heuristic_cost(grid.cell_size, neighbor_pos, end_pos),
                });

                let other_g = cache.g_bwd[neighbor_idx];
                if other_g < f32::INFINITY {
                    let total_cost = tentative_g + other_g;
                    if total_cost < best_path_cost {
                        best_path_cost = total_cost;
                        best_connection = Some(neighbor_idx);
                    }
                }
            }
        } else {
            // 惰性删除检查
            if current_node.g_cost > cache.g_bwd[current_node.idx] {
                continue;
            }

            // 检查是否在当前节点与另一端相遇
            let other_g = cache.g_fwd[current_node.idx];
            if other_g < f32::INFINITY {
                let total_cost = current_node.g_cost + other_g;
                if total_cost < best_path_cost {
                    best_path_cost = total_cost;
                    best_connection = Some(current_node.idx);
                }
            }

            for neighbor_pos in get_neighbors(grid, current_node.pos) {
                let neighbor_idx = pos_to_idx(neighbor_pos, x_len);
                let tentative_g =
                    current_node.g_cost + movement_cost(grid, current_node.pos, neighbor_pos);

                if tentative_g >= cache.g_bwd[neighbor_idx] {
                    continue;
                }

                if cache.g_fwd[neighbor_idx] == f32::INFINITY && cache.g_bwd[neighbor_idx] == f32::INFINITY {
                    cache.touched_indices.push(neighbor_idx);
                }

                cache.came_from_bwd[neighbor_idx] = current_node.idx as u32;
                cache.g_bwd[neighbor_idx] = tentative_g;

                cache.open_bwd.push(AStarNode {
                    idx: neighbor_idx,
                    pos: neighbor_pos,
                    g_cost: tentative_g,
                    h_cost: heuristic_cost(grid.cell_size, neighbor_pos, start_pos),
                });

                let other_g = cache.g_fwd[neighbor_idx];
                if other_g < f32::INFINITY {
                    let total_cost = tentative_g + other_g;
                    if total_cost < best_path_cost {
                        best_path_cost = total_cost;
                        best_connection = Some(neighbor_idx);
                    }
                }
            }
        }
    }

    if let Some(meet_idx) = best_connection {
        debug!("双向 A* 找到路径 迭代次数 {}", iterations);
        let path = reconstruct_bidirectional_path(meet_idx, &cache.came_from_fwd, &cache.came_from_bwd, x_len);
        if !path.is_empty() {
            return Some(AStarResult {
                path,
                visited_cells: cache.visited_cells.clone(),
            });
        }
    }

    None
}

fn reconstruct_bidirectional_path(
    meet_idx: usize,
    came_from_fwd: &[u32],
    came_from_bwd: &[u32],
    x_len: usize,
) -> Vec<(usize, usize)> {
    let mut path = Vec::new();

    // 1. 从相遇点回溯到起点
    let mut curr = meet_idx;
    path.push(idx_to_pos(curr, x_len));
    while came_from_fwd[curr] != u32::MAX {
        curr = came_from_fwd[curr] as usize;
        path.push(idx_to_pos(curr, x_len));
    }
    path.reverse();

    // 2. 从相遇点回溯到终点 (came_from_bwd 记录的是从终点反向搜索的父节点)
    curr = meet_idx;
    while came_from_bwd[curr] != u32::MAX {
        curr = came_from_bwd[curr] as usize;
        path.push(idx_to_pos(curr, x_len));
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

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use lol_base::grid::{ConfigNavigationGridCell, GridFlagsVisionPathing};

    use super::*;

    fn make_test_grid(size: usize, cell_size: f32) -> ConfigNavigationGrid {
        let cell = ConfigNavigationGridCell {
            heuristic: 1.0,
            vision_pathing_flags: GridFlagsVisionPathing::Walkable,
            ..default()
        };

        ConfigNavigationGrid {
            min_position: Vec2::ZERO,
            cell_size,
            x_len: size,
            y_len: size,
            cells: vec![vec![cell; size]; size],
            height_x_len: size,
            height_y_len: size,
            height_samples: vec![vec![0.0; size]; size],
            occupied_cells: Default::default(),
            exclude_cells: Default::default(),
        }
    }

    #[test]
    fn test_astar_same_start_and_end() {
        let grid = make_test_grid(50, 10.0);
        let start = Vec2::new(100.0, 100.0);
        let result = find_grid_path_with_result(&grid, &start, &start);
        assert!(result.is_some());
        let res = result.unwrap();
        assert_eq!(res.path.len(), 1);
    }

    #[test]
    fn test_astar_straight_line() {
        let grid = make_test_grid(50, 10.0);
        let start = Vec2::new(10.0, 10.0);
        let end = Vec2::new(100.0, 10.0);
        let result = find_grid_path(&grid, &start, &end);
        assert!(result.is_some());
        let path = result.unwrap();
        assert_eq!(path.first(), Some(&(1, 1)));
        assert_eq!(path.last(), Some(&(10, 1)));
    }

    #[test]
    fn test_astar_wall_bypass() {
        let mut grid = make_test_grid(50, 10.0);
        // 在 x = 5, y = 0..10 放置一堵墙
        for y in 0..10 {
            grid.cells[y][5].vision_pathing_flags = GridFlagsVisionPathing::Wall;
        }

        let start = Vec2::new(20.0, 50.0); // (2, 5)
        let end = Vec2::new(80.0, 50.0); // (8, 5)
        let result = find_grid_path(&grid, &start, &end);
        assert!(result.is_some());
        let path = result.unwrap();
        // 验证路径绕过了墙体
        for &(x, y) in &path {
            assert!(x != 5 || y >= 10, "路径不应穿过墙体: ({}, {})", x, y);
        }
    }

    #[test]
    fn test_real_sr_map_spawn_positions() {
        let path = std::path::Path::new("../../assets/maps/sr_seasonal_map/navgrid.bin");
        if !path.exists() {
            println!("navgrid.bin not found at {:?}", path);
            return;
        }
        let data = std::fs::read(path).expect("read navgrid.bin");
        let grid: ConfigNavigationGrid = bincode::deserialize(&data).expect("deserialize navgrid");

        println!(
            "Grid info: min_pos={:?}, max_pos={:?}, cell_size={}, x_len={}, y_len={}",
            grid.min_position,
            grid.get_max_position(),
            grid.cell_size,
            grid.x_len,
            grid.y_len
        );

        let order_spawn = Vec2::new(1000.0, 1000.0);
        let chaos_spawn = Vec2::new(14000.0, 14000.0);

        let order_grid_xy = grid.get_cell_xy_by_position(&order_spawn).unwrap();
        let chaos_grid_xy = grid.get_cell_xy_by_position(&chaos_spawn).unwrap();

        let order_walkable = grid.is_walkable_by_xy(order_grid_xy);
        let chaos_walkable = grid.is_walkable_by_xy(chaos_grid_xy);

        let order_cell = grid.get_cell_by_xy(order_grid_xy);
        let chaos_cell = grid.get_cell_by_xy(chaos_grid_xy);

        println!(
            "Order spawn (1000, 1000): grid_xy={:?}, walkable={}, flags={:?}, height={}",
            order_grid_xy,
            order_walkable,
            order_cell.vision_pathing_flags,
            grid.get_height_by_position(&order_spawn)
        );

        println!(
            "Chaos spawn (14000, 14000): grid_xy={:?}, walkable={}, flags={:?}, height={}",
            chaos_grid_xy,
            chaos_walkable,
            chaos_cell.vision_pathing_flags,
            grid.get_height_by_position(&chaos_spawn)
        );

        // 1. 短距离寻路（从泉水走 50 码）
        let short_target = Vec2::new(1050.0, 1050.0);
        let t0 = Instant::now();
        let short_path = find_grid_path(&grid, &order_spawn, &short_target);
        let elapsed_short = t0.elapsed();

        // 2. 中距离寻路（从泉水走到下路一塔附近，约 3500 码）
        let mid_target = Vec2::new(3500.0, 1500.0);
        let t1 = Instant::now();
        let mid_path = find_grid_path(&grid, &order_spawn, &mid_target);
        let elapsed_mid = t1.elapsed();

        // 3. 全图超长距离寻路（蓝方泉水到红方泉水，跨越约 18000 码）
        let t2 = Instant::now();
        let long_path = find_grid_path(&grid, &order_spawn, &chaos_spawn);
        let elapsed_long = t2.elapsed();

        println!("------------------------------------------------------------------");
        println!(
            "真实地图测试 (网格 {}x{}，CellSize {}):",
            grid.x_len, grid.y_len, grid.cell_size
        );
        println!(
            "- 短距离 (50 码):   找到={}, 路径长度={}, 耗时={:?}",
            short_path.is_some(),
            short_path.as_ref().map(|p| p.len()).unwrap_or(0),
            elapsed_short
        );
        println!(
            "- 中距离 (3500 码): 找到={}, 路径长度={}, 耗时={:?}",
            mid_path.is_some(),
            mid_path.as_ref().map(|p| p.len()).unwrap_or(0),
            elapsed_mid
        );
        println!(
            "- 全图长距离 (18000 码): 找到={}, 路径长度={}, 耗时={:?}",
            long_path.is_some(),
            long_path.as_ref().map(|p| p.len()).unwrap_or(0),
            elapsed_long
        );
        println!("------------------------------------------------------------------");
    }
}
