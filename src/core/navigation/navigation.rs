use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use bevy::render::render_resource::Face;
use std::collections::{HashMap, HashSet, VecDeque};
use std::time::{Duration, Instant};

use lol_config::{ConfigNavigationGrid, CELL_COST_IMPASSABLE};

use crate::{find_grid_path_with_result, system_debug, Bounding, Character};

#[derive(Default)]
pub struct PluginNavigaton;

impl Plugin for PluginNavigaton {
    fn build(&self, app: &mut App) {
        app.init_resource::<NavigationStats>();
        app.init_resource::<NavigationDebug>();

        app.add_systems(First, |mut res_stats: ResMut<NavigationStats>| {
            *res_stats = Default::default();
        });
        // app.add_systems(Last, |res_stats: Res<NavigationStats>| {
        //     if res_stats.get_nav_path_time > Duration::from_millis(10) {
        //         info!("{:#?}", res_stats);
        //     }

        //     if res_stats.occupied_grid_cells_num > 0 {
        //         info!("{:#?}", res_stats.occupied_grid_cells_num);
        //     }
        // });
        app.add_systems(PreUpdate, pre_update_global_occupied_cells);
        app.add_systems(Update, update_y);
        app.add_systems(Update, update_visualization_astar);
        app.add_systems(Update, update_visualization_move_path);
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
}

/// A* 可视化 debug 资源
#[derive(Resource, Default)]
pub struct NavigationDebug {
    pub enabled: bool,
    pub visited_cells: Vec<(usize, usize)>,
    pub path_cells: Vec<(usize, usize)>,
    pub unoptimized_path: Vec<Vec2>,
    pub optimized_path: Vec<Vec2>,
}

#[derive(Component)]
struct AStarCell;

#[derive(Component)]
struct AStarPathCell;

#[derive(Component)]
struct ObstacleCell;

fn update_y(
    grid: Res<ConfigNavigationGrid>,
    mut q_movement: Query<&mut Transform, With<Character>>,
) {
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
    get_nav_path_with_debug(start_pos, end_pos, grid, stats, None)
}

pub fn get_nav_path_with_debug(
    start_pos: &Vec2,
    end_pos: &Vec2,
    grid: &ConfigNavigationGrid,
    stats: &mut NavigationStats,
    mut debug: Option<&mut NavigationDebug>,
) -> Option<Vec<Vec2>> {
    let start = Instant::now();

    let start_grid_pos = grid.get_cell_xy_by_position(start_pos);
    let adjusted_start_pos = if !grid.is_walkable_by_xy(start_grid_pos) {
        let start_time = Instant::now();
        if let Some(new_start_grid_pos) = find_nearest_walkable_cell(grid, start_grid_pos) {
            debug!(
                "get_nav_path: Start position ({}, {}) is not walkable, using nearest walkable cell ({}, {})",
                start_grid_pos.0, start_grid_pos.1, new_start_grid_pos.0, new_start_grid_pos.1
            );
            {
                stats.find_nearest_walkable_cell_count += 1;
                stats.find_nearest_walkable_cell_time += start_time.elapsed();
            }
            grid.get_cell_center_position_by_xy(new_start_grid_pos).xz()
        } else {
            warn!("get_nav_path: No walkable cell found near start position");
            return None;
        }
    } else {
        *start_pos
    };

    // 检查终点是否可行走，如果不可行，找到最近的可达格子
    let end_grid_pos = grid.get_cell_xy_by_position(end_pos);
    let adjusted_end_pos = if !grid.is_walkable_by_xy(end_grid_pos) {
        let start_time = Instant::now();
        if let Some(new_end_grid_pos) = find_nearest_walkable_cell(grid, end_grid_pos) {
            debug!(
                "get_nav_path: End position ({}, {}) is not walkable, using nearest walkable cell ({}, {})",
                end_grid_pos.0, end_grid_pos.1, new_end_grid_pos.0, new_end_grid_pos.1
            );
            {
                stats.find_nearest_walkable_cell_count += 1;
                stats.find_nearest_walkable_cell_time += start_time.elapsed();
            }
            grid.get_cell_center_position_by_xy(new_end_grid_pos).xz()
        } else {
            warn!("get_nav_path: No walkable cell found near end position");
            return None;
        }
    } else {
        *end_pos
    };

    // 检查起点和终点是否可直达
    let adjusted_start_grid_pos = (adjusted_start_pos - grid.min_position) / grid.cell_size;
    let adjusted_end_grid_pos = (adjusted_end_pos - grid.min_position) / grid.cell_size;

    if has_line_of_sight(&grid, adjusted_start_grid_pos, adjusted_end_grid_pos) {
        system_debug!(
            "command_movement_move_to",
            "Direct path found in {:.6}ms",
            start.elapsed().as_millis()
        );
        {
            stats.get_nav_path_count += 1;
            stats.get_nav_path_time += start.elapsed();
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
    let result = find_path_with_result(&grid, &adjusted_start_pos, &adjusted_end_pos);

    system_debug!(
        "command_movement_move_to",
        "A* path found in {:.6}ms",
        start.elapsed().as_millis()
    );

    {
        stats.get_nav_path_count += 1;
        stats.get_nav_path_time += start.elapsed();
    }

    match result {
        Some(find_result) => {
            if let Some(ref mut nav_debug) = debug {
                nav_debug.visited_cells = find_result.visited_cells;
                nav_debug.path_cells = find_result.path_cells;
                nav_debug.unoptimized_path = find_result.unoptimized_path;
                nav_debug.optimized_path = find_result.path.clone();
            }
            Some(find_result.path)
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
    let astar_result = find_grid_path_with_result(grid, start, end)?;

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
pub fn is_path_blocked(grid: &ConfigNavigationGrid, path: &[Vec3], current_index: usize) -> bool {
    if path.is_empty() || current_index >= path.len() {
        return false;
    }

    // 检测从当前点到路径终点的每一段是否被阻挡
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
    let x = ((world_pos.x - grid.min_position.x) / grid.cell_size).floor() as usize;
    let y = ((world_pos.y - grid.min_position.y) / grid.cell_size).floor() as usize;
    (x, y)
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

fn pre_update_global_occupied_cells(
    mut grid: ResMut<ConfigNavigationGrid>,
    entities_with_bounding: Query<(Entity, &GlobalTransform, &Bounding)>,
    mut stats: ResMut<NavigationStats>,
) {
    let start = Instant::now();

    // 计算所有实体的 occupied_cells（不排除任何实体）
    let occupied_cells = calculate_occupied_grid_cells(&grid, &entities_with_bounding, &[]);
    grid.occupied_cells = occupied_cells;

    stats.calculate_occupied_grid_cells_time += start.elapsed();
    stats.calculate_occupied_grid_cells_count += 1;
    stats.occupied_grid_cells_num = grid.occupied_cells.len() as u32;
}

/// 根据所有带Bounding组件的实体，计算被占据的网格格子及其通行成本
///
/// # 参数
/// - `grid`: 导航网格
/// - `entities_with_bounding`: 查询所有带Transform和Bounding组件的实体
/// - `exclude_entities`: 要排除的实体ID列表（不将其作为障碍物），例如当前移动的实体自身
///
/// # 返回值
/// - 格子坐标到通行成本的映射，成本越高表示通行代价越大
pub fn calculate_occupied_grid_cells(
    grid: &ConfigNavigationGrid,
    entities_with_bounding: &Query<(Entity, &GlobalTransform, &Bounding)>,
    exclude_entities: &[Entity],
) -> HashMap<(usize, usize), f32> {
    use lol_config::CELL_COST_IMPASSABLE;
    use std::collections::HashMap;

    let mut occupied_cells: HashMap<(usize, usize), f32> = HashMap::new();
    let exclude_set: std::collections::HashSet<Entity> = exclude_entities.iter().copied().collect();

    for (entity, transform, bounding) in entities_with_bounding.iter() {
        if exclude_set.contains(&entity) {
            continue;
        }

        let entity_pos = transform.translation().xz();
        let entity_grid_pos = world_pos_to_grid_xy(grid, entity_pos);
        let radius_in_cells = (bounding.radius / grid.cell_size).ceil() as i32;

        for dx in -radius_in_cells..=radius_in_cells {
            for dy in -radius_in_cells..=radius_in_cells {
                let new_x = entity_grid_pos.0 as i32 + dx;
                let new_y = entity_grid_pos.1 as i32 + dy;

                if new_x < 0 || new_y < 0 {
                    continue;
                }

                let new_pos = (new_x as usize, new_y as usize);
                if new_pos.0 >= grid.x_len || new_pos.1 >= grid.y_len {
                    continue;
                }

                // 计算格子中心到实体中心的距离（以格子为单位）
                let distance = ((dx * dx + dy * dy) as f32).sqrt();

                // 在半径内的格子视为不可通行，边缘格子给予较高成本
                let cost = if distance <= radius_in_cells as f32 * 0.7 {
                    // 核心区域：不可通行
                    CELL_COST_IMPASSABLE
                } else {
                    // 边缘区域：基于距离的成本衰减
                    let t =
                        (distance - radius_in_cells as f32 * 0.7) / (radius_in_cells as f32 * 0.3);
                    let t = t.clamp(0.0, 1.0);
                    // 从高成本衰减到较低成本
                    (1.0 - t) * 100.0 + 10.0
                };

                // 多个实体重叠时保留最高成本
                occupied_cells
                    .entry(new_pos)
                    .and_modify(|c| *c = c.max(cost))
                    .or_insert(cost);
            }
        }
    }

    occupied_cells
}

fn update_visualization_astar(
    mut commands: Commands,
    grid: Res<ConfigNavigationGrid>,
    nav_debug: Res<NavigationDebug>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    visited_query: Query<Entity, With<AStarCell>>,
    path_query: Query<Entity, With<AStarPathCell>>,
    obstacle_query: Query<Entity, With<ObstacleCell>>,
) {
    if !nav_debug.enabled {
        return;
    }

    if !nav_debug.is_changed() && !grid.is_changed() {
        return;
    }

    // 删除旧的 A* 可视化单元格
    for entity in visited_query.iter() {
        commands.entity(entity).despawn();
    }

    for entity in path_query.iter() {
        commands.entity(entity).despawn();
    }

    for entity in obstacle_query.iter() {
        commands.entity(entity).despawn();
    }

    let mesh = meshes.add(Plane3d::new(
        vec3(0.0, 1.0, 0.0),
        Vec2::splat(grid.cell_size / 2.0 - 3.0),
    ));

    // 障碍物格子（红色正五边形）
    let red_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.3, 0.3),
        unlit: true,
        depth_bias: 40.0,
        cull_mode: Some(Face::Front),
        ..default()
    });

    let mut blue_materials = Vec::new();
    for i in 0..11 {
        blue_materials.push(materials.add(StandardMaterial {
            base_color: Color::srgb(0.1, 0.1, i as f32 / 10.0),
            unlit: true,
            depth_bias: 40.0,
            cull_mode: Some(Face::Front),
            ..default()
        }));
    }

    for (&(x, y), cost) in grid.occupied_cells.iter() {
        commands.spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(if *cost == CELL_COST_IMPASSABLE {
                red_material.clone()
            } else {
                blue_materials[(cost / 100.0 * 10.0).floor() as usize].clone()
            }),
            Transform::from_translation(
                grid.get_cell_center_position_by_xy((x, y)) + Vec3::new(0.0, 1.5, 0.0),
            ),
            ObstacleCell,
            Visibility::Visible,
        ));
    }

    // 访问的单元格（黄色）
    let yellow_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 1.0, 0.0),
        unlit: true,
        depth_bias: 50.0,
        cull_mode: Some(Face::Front),
        ..default()
    });

    for &(x, y) in &nav_debug.visited_cells {
        commands.spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(yellow_material.clone()),
            Transform::from_translation(
                grid.get_cell_center_position_by_xy((x, y)) + Vec3::new(0.0, 2.0, 0.0),
            ),
            AStarCell,
            Visibility::Visible,
        ));
    }

    // 路径单元格（白色）
    let white_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 1.0, 1.0),
        unlit: true,
        depth_bias: 60.0,
        cull_mode: Some(Face::Front),
        ..default()
    });

    for &(x, y) in &nav_debug.path_cells {
        commands.spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(white_material.clone()),
            Transform::from_translation(
                grid.get_cell_center_position_by_xy((x, y)) + Vec3::new(0.0, 3.0, 0.0),
            ),
            AStarPathCell,
            Visibility::Visible,
        ));
    }
}

/// 绘制移动路径（粉色为未优化路径，蓝色为优化后路径）
fn update_visualization_move_path(
    grid: Res<ConfigNavigationGrid>,
    mut gizmos: Gizmos,
    nav_debug: Res<NavigationDebug>,
) {
    use bevy::color::palettes;

    if !nav_debug.enabled {
        return;
    }

    for path_point in nav_debug.unoptimized_path.windows(2) {
        gizmos.line(
            grid.get_world_position_by_position(&path_point[0]) + vec3(0.0, 4.0, 0.0),
            grid.get_world_position_by_position(&path_point[1]) + vec3(0.0, 4.0, 0.0),
            Color::Srgba(palettes::tailwind::PINK_500),
        );
    }

    for path_point in nav_debug.optimized_path.windows(2) {
        gizmos.line(
            grid.get_world_position_by_position(&path_point[0]) + vec3(0.0, 5.0, 0.0),
            grid.get_world_position_by_position(&path_point[1]) + vec3(0.0, 5.0, 0.0),
            Color::Srgba(palettes::tailwind::BLUE_500),
        );
    }
}
