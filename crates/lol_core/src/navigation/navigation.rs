use std::collections::{HashSet, VecDeque};
use std::time::Duration;

use bevy::prelude::*;
use lol_base::grid::{CELL_COST_IMPASSABLE, ConfigNavigationGrid};

use crate::base::bounding::Bounding;
use crate::character::Character;
use crate::loaders::navgrid::NavGridLoader;
pub use crate::navigation::astar::AStarCache;
use crate::navigation::astar::find_grid_path_with_result_cache;
use crate::navigation::grid::ResourceGrid;

#[derive(Default)]
pub struct PluginNavigaton;

impl Plugin for PluginNavigaton {
    fn build(&self, app: &mut App) {
        app.init_asset::<ConfigNavigationGrid>();
        app.init_asset_loader::<NavGridLoader>();

        app.init_resource::<NavigationStats>();
        app.init_resource::<NavigationDebugState>();
        app.init_resource::<AStarCache>();

        app.add_systems(First, |mut res_stats: ResMut<NavigationStats>| {
            *res_stats = Default::default();
        });
        app.add_systems(Last, |res_stats: Res<NavigationStats>| {
            if res_stats.get_nav_path_time > Duration::from_millis(10) {
                info!("{:#?}", res_stats);
            }
        });
        app.add_systems(Update, update_y.run_if(resource_exists::<ResourceGrid>));
    }
}

#[derive(Resource, Default, Debug)]
pub struct NavigationStats {
    pub find_nearest_walkable_cell_count: u32,
    pub find_nearest_walkable_cell_time: Duration,

    pub get_nav_path_count: u32,
    pub get_nav_path_time: Duration,

    pub occupied_grid_cells_num: u32,

    pub calculate_occupied_grid_cells_count: u32,
    pub calculate_occupied_grid_cells_time: Duration,

    pub exclude_count: u32,
    pub exclude_time: Duration,

    pub check_path_count: u32,
    pub check_path_time: Duration,
}

/// A* 可视化 debug 标记资源
#[derive(Resource)]
pub struct NavigationDebug;

/// A* 可视化 debug 状态资源
#[derive(Resource, Default)]
pub struct NavigationDebugState {
    pub visited_cells: Vec<(usize, usize)>,
    pub path_cells: Vec<(usize, usize)>,
    pub unoptimized_path: Vec<Vec2>,
    pub optimized_path: Vec<Vec2>,
}

fn update_y(
    res_grid: Res<ResourceGrid>,
    assets_grid: Res<Assets<ConfigNavigationGrid>>,
    mut q_movement: Query<&mut Transform, With<Character>>,
) {
    let Some(grid) = assets_grid.get(&res_grid.0) else {
        return;
    };
    for mut transform in q_movement.iter_mut() {
        transform.translation = grid.get_world_position_by_position(&transform.translation.xz());
    }
}

pub fn get_nav_path(
    start_pos: &Vec2,
    end_pos: &Vec2,
    grid: &ConfigNavigationGrid,
    stats: &mut NavigationStats,
) -> Option<Vec<Vec2>> {
    get_nav_path_with_debug(start_pos, end_pos, grid, stats, None, None)
}

pub fn get_nav_path_with_debug(
    start_pos: &Vec2,
    end_pos: &Vec2,
    grid: &ConfigNavigationGrid,
    stats: &mut NavigationStats,
    mut debug: Option<&mut NavigationDebugState>,
    mut cache: Option<&mut AStarCache>,
) -> Option<Vec<Vec2>> {
    // let start = Instant::now();

    let start_grid_pos = grid
        .get_cell_xy_by_position(start_pos)
        .unwrap_or_else(|| grid.clamp_position_to_grid_xy(start_pos));
    let adjusted_start_pos = if !grid.is_walkable_by_xy(start_grid_pos) {
        // let start_time = Instant::now();
        if let Some(new_start_grid_pos) = find_nearest_walkable_cell(grid, start_grid_pos) {
            debug!(
                "寻路: 起点 ({}, {}) 不可行走，使用最近的可行走格子 ({}, {})",
                start_grid_pos.0, start_grid_pos.1, new_start_grid_pos.0, new_start_grid_pos.1
            );
            {
                stats.find_nearest_walkable_cell_count += 1;
                // stats.find_nearest_walkable_cell_time += start_time.elapsed();
            }
            grid.get_cell_center_position_by_xy(new_start_grid_pos).xz()
        } else {
            warn!("寻路: 起点附近未找到可行走格子");
            return None;
        }
    } else {
        *start_pos
    };

    // 检查终点是否可行走，如果不可行，找到最近的可达格子（若点击在地图外则投影到最近边缘）
    let end_grid_pos = grid
        .get_cell_xy_by_position(end_pos)
        .unwrap_or_else(|| grid.clamp_position_to_grid_xy(end_pos));
    let adjusted_end_pos = if !grid.is_walkable_by_xy(end_grid_pos) {
        // let start_time = Instant::now();
        if let Some(new_end_grid_pos) = find_nearest_walkable_cell(grid, end_grid_pos) {
            debug!(
                "寻路: 终点 ({}, {}) 不可行走，使用最近的可行走格子 ({}, {})",
                end_grid_pos.0, end_grid_pos.1, new_end_grid_pos.0, new_end_grid_pos.1
            );
            {
                stats.find_nearest_walkable_cell_count += 1;
                // stats.find_nearest_walkable_cell_time += start_time.elapsed();
            }
            grid.get_cell_center_position_by_xy(new_end_grid_pos).xz()
        } else {
            warn!("寻路: 终点附近未找到可行走格子");
            return None;
        }
    } else {
        *end_pos
    };

    // 检查起点和终点是否可直达
    let adjusted_start_grid_pos = (adjusted_start_pos - grid.min_position) / grid.cell_size;
    let adjusted_end_grid_pos = (adjusted_end_pos - grid.min_position) / grid.cell_size;

    if has_line_of_sight(&grid, adjusted_start_grid_pos, adjusted_end_grid_pos) {
        // debug!("直接路径找到，耗时 {:.6}ms", start.elapsed().as_millis());
        {
            stats.get_nav_path_count += 1;
            // stats.get_nav_path_time += start.elapsed();
        }

        // 直线路径的 debug 信息
        if let Some(ref mut nav_debug) = debug {
            nav_debug.visited_cells.clear();
            nav_debug.path_cells.clear();
            nav_debug.unoptimized_path = vec![adjusted_start_pos, adjusted_end_pos];
            nav_debug.optimized_path = vec![adjusted_start_pos, adjusted_end_pos];
        }

        return Some(vec![adjusted_start_pos, adjusted_end_pos]);
    }

    // 如果不可直达，则使用A*算法规划路径（包含 debug 信息）
    let result = if let Some(ref mut c) = cache {
        find_path_with_result_cache(&grid, &adjusted_start_pos, &adjusted_end_pos, c)
    } else {
        find_path_with_result(&grid, &adjusted_start_pos, &adjusted_end_pos)
    };

    // debug!("A* 路径找到，耗时 {:.6}ms", start.elapsed().as_millis());

    {
        stats.get_nav_path_count += 1;
        // stats.get_nav_path_time += start.elapsed();
    }

    match result {
        Some(find_result) => {
            if let Some(ref mut nav_debug) = debug {
                nav_debug.visited_cells = find_result.visited_cells;
                nav_debug.path_cells = find_result.path_cells;
                nav_debug.unoptimized_path = find_result.unoptimized_path;
                nav_debug.optimized_path = find_result.path.clone();
            }
            if find_result.path.is_empty() {
                None
            } else {
                Some(find_result.path)
            }
        }
        None => None,
    }
}

/// 寻路结果，包含路径和 debug 信息
pub struct FindPathResult {
    pub path: Vec<Vec2>,
    pub visited_cells: Vec<(usize, usize)>,
    pub path_cells: Vec<(usize, usize)>,
    pub unoptimized_path: Vec<Vec2>,
}

/// 主要的寻路函数，结合A*和漏斗算法
pub fn find_path(grid: &ConfigNavigationGrid, start: &Vec2, end: &Vec2) -> Option<Vec<Vec2>> {
    find_path_with_result(grid, start, end).map(|result| result.path)
}

/// 主要的寻路函数，返回完整的 debug 信息
pub fn find_path_with_result(
    grid: &ConfigNavigationGrid,
    start: &Vec2,
    end: &Vec2,
) -> Option<FindPathResult> {
    let mut cache = AStarCache::default();
    find_path_with_result_cache(grid, start, end, &mut cache)
}

/// 主要的寻路函数（带 AStarCache 复用）
pub fn find_path_with_result_cache(
    grid: &ConfigNavigationGrid,
    start: &Vec2,
    end: &Vec2,
    cache: &mut AStarCache,
) -> Option<FindPathResult> {
    let astar_result = find_grid_path_with_result_cache(grid, start, end, cache)?;

    let unoptimized_path = astar_result
        .path
        .iter()
        .map(|&(x, y)| grid.get_position_by_float_xy(&vec2(x as f32 + 0.5, y as f32 + 0.5)))
        .collect::<Vec<_>>();

    let optimized_path = post_process_path(grid, &astar_result.path, start, end);

    Some(FindPathResult {
        path: optimized_path,
        visited_cells: astar_result.visited_cells,
        path_cells: astar_result.path.clone(),
        unoptimized_path,
    })
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

    let path = optimize_path(&grid, &path);

    let path = path
        .into_iter()
        .map(|v| grid.get_position_by_float_xy(&v))
        .collect::<Vec<_>>();

    return path;
}

fn optimize_path(grid: &ConfigNavigationGrid, path: &Vec<Vec2>) -> Vec<Vec2> {
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

            // 只要找到一个可见的，那它一定是从后往前看的"最远"的点
            if has_line_of_sight(grid, start_pos, end_pos) {
                furthest_visible_index = lookahead_index;
                break;
            }
        }

        optimized_path.push(path[furthest_visible_index]);
        current_index = furthest_visible_index;
    }

    optimized_path
}

/// 检测给定路径上是否有障碍物阻挡
/// 从当前位置开始，检测路径的剩余部分是否仍然可通行
pub fn is_path_blocked(
    grid: &ConfigNavigationGrid,
    path: &[Vec3],
    current_index: usize,
    current_pos: Option<Vec2>,
) -> bool {
    if path.is_empty() || current_index >= path.len() {
        return false;
    }

    // 首先检测从实体当前物理位置到目标路点的视线
    if let Some(curr) = current_pos {
        let target = path[current_index].xz();
        let curr_grid = (curr - grid.min_position) / grid.cell_size;
        let target_grid = (target - grid.min_position) / grid.cell_size;
        if !has_line_of_sight(grid, curr_grid, target_grid) {
            return true;
        }
    }

    // 检测从当前路点到后续路点的每一段是否被阻挡
    for i in current_index..path.len().saturating_sub(1) {
        let start = path[i].xz();
        let end = path[i + 1].xz();

        // 转换为网格坐标
        let start_grid = (start - grid.min_position) / grid.cell_size;
        let end_grid = (end - grid.min_position) / grid.cell_size;

        if !has_line_of_sight(grid, start_grid, end_grid) {
            return true;
        }
    }

    false
}

pub fn has_line_of_sight(grid: &ConfigNavigationGrid, start: Vec2, end: Vec2) -> bool {
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
        if !grid.is_walkable_by_xy((current_grid_x as usize, current_grid_y as usize)) {
            return false;
        }

        // 检查是否到达终点
        if current_grid_x == end_grid_x && current_grid_y == end_grid_y {
            return true;
        }
    }

    true
}

/// 将世界坐标转换为网格坐标的辅助函数
pub fn world_pos_to_grid_xy(grid: &ConfigNavigationGrid, world_pos: Vec2) -> (usize, usize) {
    grid.clamp_position_to_grid_xy(&world_pos)
}

/// 找到最近的可达格子
pub fn find_nearest_walkable_cell(
    grid: &ConfigNavigationGrid,
    start: (usize, usize),
) -> Option<(usize, usize)> {
    if grid.is_walkable_by_xy(start) {
        return Some(start);
    }

    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();

    visited.insert(start);
    queue.push_back(start);

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

    while let Some((x, y)) = queue.pop_front() {
        for (dx, dy) in directions {
            let new_x = x as i32 + dx;
            let new_y = y as i32 + dy;

            if new_x < 0 || new_y < 0 {
                continue;
            }

            let new_pos = (new_x as usize, new_y as usize);

            if new_pos.0 >= grid.x_len || new_pos.1 >= grid.y_len {
                continue;
            }

            if visited.contains(&new_pos) {
                continue;
            }

            if grid.is_walkable_by_xy(new_pos) {
                return Some(new_pos);
            }

            visited.insert(new_pos);
            queue.push_back(new_pos);
        }
    }

    None
}

pub fn update_occupied_cells_flat(
    grid: &mut ConfigNavigationGrid,
    entities_with_bounding: &Query<(Entity, &GlobalTransform, &Bounding)>,
    stats: &mut NavigationStats,
) {
    let total = grid.x_len * grid.y_len;
    if grid.occupied_cells.len() != total {
        grid.occupied_cells = vec![0.0; total];
    } else {
        grid.occupied_cells.fill(0.0);
    }

    let mut occupied_count = 0u32;
    let x_len = grid.x_len;
    let y_len = grid.y_len;
    let cell_size = grid.cell_size;

    for (_entity, transform, bounding) in entities_with_bounding.iter() {
        let entity_pos = transform.translation().xz();
        let entity_grid_pos = world_pos_to_grid_xy(grid, entity_pos);
        let radius_in_cells = (bounding.radius / cell_size).ceil() as i32;

        process_entity_cells_flat(
            &mut grid.occupied_cells,
            x_len,
            y_len,
            entity_grid_pos,
            radius_in_cells,
            &mut occupied_count,
        );
    }

    stats.calculate_occupied_grid_cells_count += 1;
    stats.occupied_grid_cells_num = occupied_count;
}

/// 根据所有带Bounding组件的实体，计算被占据的网格格子及其通行成本
///
/// # 参数
/// - `grid`: 导航网格
/// - `entities_with_bounding`: 查询所有带Transform和Bounding组件的实体
/// - `exclude_entities`: 要排除的实体ID列表（不将其作为障碍物），例如当前移动的实体自身
///
/// # 返回值
/// - 扁平列表（索引为 y * x_len + x），值为通行成本
pub fn calculate_occupied_grid_cells(
    grid: &ConfigNavigationGrid,
    entities_with_bounding: &Query<(Entity, &GlobalTransform, &Bounding)>,
    exclude_entities: &[Entity],
) -> Vec<f32> {
    let total = grid.x_len * grid.y_len;
    let mut occupied_cells = vec![0.0; total];
    let exclude_set: HashSet<Entity> = exclude_entities.iter().copied().collect();
    let mut count = 0u32;

    for (entity, transform, bounding) in entities_with_bounding.iter() {
        if exclude_set.contains(&entity) {
            continue;
        }

        let entity_pos = transform.translation().xz();
        let entity_grid_pos = world_pos_to_grid_xy(grid, entity_pos);
        let radius_in_cells = (bounding.radius / grid.cell_size).ceil() as i32;

        process_entity_cells_flat(
            &mut occupied_cells,
            grid.x_len,
            grid.y_len,
            entity_grid_pos,
            radius_in_cells,
            &mut count,
        );
    }

    occupied_cells
}

fn process_entity_cells_flat(
    occupied_cells: &mut [f32],
    x_len: usize,
    y_len: usize,
    entity_grid_pos: (usize, usize),
    radius_in_cells: i32,
    occupied_count: &mut u32,
) {
    for dx in -radius_in_cells..=radius_in_cells {
        for dy in -radius_in_cells..=radius_in_cells {
            let new_x = entity_grid_pos.0 as i32 + dx;
            let new_y = entity_grid_pos.1 as i32 + dy;

            if new_x < 0 || new_y < 0 {
                continue;
            }

            let nx = new_x as usize;
            let ny = new_y as usize;
            if nx >= x_len || ny >= y_len {
                continue;
            }

            let distance = ((dx * dx + dy * dy) as f32).sqrt();
            let cost = calculate_cell_cost(distance, radius_in_cells);

            let idx = ny * x_len + nx;
            if occupied_cells[idx] == 0.0 && cost > 0.0 {
                *occupied_count += 1;
            }
            if cost > occupied_cells[idx] {
                occupied_cells[idx] = cost;
            }
        }
    }
}

fn calculate_cell_cost(distance: f32, radius_in_cells: i32) -> f32 {
    if distance <= radius_in_cells as f32 * 0.7 {
        return CELL_COST_IMPASSABLE;
    }

    let t = (distance - radius_in_cells as f32 * 0.7) / (radius_in_cells as f32 * 0.3);
    let t = t.clamp(0.0, 1.0);
    (1.0 - t) * 100.0 + 10.0
}
