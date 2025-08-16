use bevy::prelude::*;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

use crate::core::{ConfigMap, Movement};
use crate::league::VisionPathingFlags;

pub struct PluginNavigaton;

impl Plugin for PluginNavigaton {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(FixedPreUpdate, update);
    }
}

fn setup(
    mut commands: Commands,
    configs: Res<ConfigMap>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let navigation_grid = &configs.navigation_grid;

    let mesh = meshes.add(Plane3d::new(
        vec3(0.0, 1.0, 0.0),
        Vec2::splat(navigation_grid.cell_size / 2.0 - 5.0),
    ));
    let red_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.0, 0.0),
        unlit: true,
        depth_bias: 100.0,
        ..default()
    });

    let green_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.0, 1.0, 0.0),
        unlit: true,
        depth_bias: 100.0,
        ..default()
    });

    for (x, row) in navigation_grid.cells.iter().enumerate() {
        for (y, cell) in row.iter().enumerate() {
            commands.spawn((
                Mesh3d(mesh.clone()),
                MeshMaterial3d(
                    if cell.vision_pathing_flags.contains(VisionPathingFlags::Wall) {
                        red_material.clone()
                    } else {
                        green_material.clone()
                    },
                ),
                Transform::from_translation(navigation_grid.get_cell_pos(x, y)),
                Visibility::Visible,
                Pickable::IGNORE,
            ));
        }
    }
}

fn update(configs: Res<ConfigMap>, mut q_movement: Query<&mut Transform, With<Movement>>) {
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

pub fn find_path(configs: &ConfigMap, start: Vec3, end: Vec3) -> Option<Vec<Vec2>> {
    let grid = &configs.navigation_grid;

    let start_pos = world_to_grid(grid, start);
    let end_pos = world_to_grid(grid, end);

    debug!(
        "A* pathfinding: start=({}, {}) end=({}, {})",
        start_pos.x, start_pos.y, end_pos.x, end_pos.y
    );

    if !is_valid_pos(grid, start_pos) || !is_valid_pos(grid, end_pos) {
        warn!("A* pathfinding: Invalid start or end position");
        return None;
    }

    let mut open_set = BinaryHeap::new();
    let mut closed_set = HashMap::new();
    let mut came_from = HashMap::new();
    let mut g_scores = HashMap::new(); // 跟踪每个位置的最佳g_cost

    let start_node = AStarNode {
        pos: start_pos,
        g_cost: 0.0,
        h_cost: heuristic_cost(grid, start_pos, end_pos),
    };

    open_set.push(start_node);
    g_scores.insert(start_pos, 0.0);

    let mut iterations = 0;
    const MAX_ITERATIONS: usize = 10000; // 防止无限循环

    while let Some(current) = open_set.pop() {
        iterations += 1;

        if iterations > MAX_ITERATIONS {
            error!("A* pathfinding: Exceeded maximum iterations ({}), breaking to prevent infinite loop", MAX_ITERATIONS);
            return None;
        }

        if iterations % 1000 == 0 {
            debug!(
                "A* pathfinding: Iteration {}, current=({}, {}), f_cost={:.2}",
                iterations,
                current.pos.x,
                current.pos.y,
                current.f_cost()
            );
        }

        if current.pos == end_pos {
            debug!("A* pathfinding: Found path in {} iterations", iterations);
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
            debug!("A* pathfinding: Generated path with {} points", path.len());
            return Some(path);
        }

        // 如果这个节点已经在closed_set中且有更好的g_cost，跳过
        if let Some(&existing_g_cost) = closed_set.get(&current.pos) {
            if current.g_cost > existing_g_cost {
                continue;
            }
        }

        closed_set.insert(current.pos, current.g_cost);

        // 检查邻居
        for neighbor_pos in get_neighbors(grid, current.pos) {
            // 1. 如果邻居已在closed_set中，直接跳过。
            if closed_set.contains_key(&neighbor_pos) {
                continue;
            }

            let tentative_g_cost = current.g_cost + distance_cost(grid, current.pos, neighbor_pos);

            // 2. 检查是否需要更新 g_score
            if let Some(&existing_g_cost) = g_scores.get(&neighbor_pos) {
                if tentative_g_cost >= existing_g_cost {
                    continue; // 不是更优的路径，跳过
                }
            }

            // 找到了更优的路径，或第一次访问该节点
            let neighbor_node = AStarNode {
                pos: neighbor_pos,
                g_cost: tentative_g_cost,
                h_cost: heuristic_cost(grid, neighbor_pos, end_pos),
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
    None
}

fn world_to_grid(grid: &crate::core::ConfigNavigationGrid, world_pos: Vec3) -> GridPos {
    let (x, y) = grid.get_cell_xy_by_pos(world_pos);

    GridPos { x, y }
}

fn grid_to_world(grid: &crate::core::ConfigNavigationGrid, grid_pos: GridPos) -> Vec3 {
    grid.get_cell_pos(grid_pos.x, grid_pos.y)
}

fn is_valid_pos(grid: &crate::core::ConfigNavigationGrid, pos: GridPos) -> bool {
    pos.x < grid.x_len && pos.y < grid.y_len
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

        if new_x < 0 || new_y < 0 {
            continue;
        }

        let pos = GridPos {
            x: new_x as usize,
            y: new_y as usize,
        };

        if !is_valid_pos(grid, pos) {
            continue;
        }

        if grid.cells[pos.x][pos.y]
            .vision_pathing_flags
            .contains(VisionPathingFlags::Wall)
        {
            continue;
        }

        neighbors.push(pos);
    }

    neighbors
}

fn distance_cost(grid: &crate::core::ConfigNavigationGrid, from: GridPos, to: GridPos) -> f32 {
    let dx = (to.x as i32 - from.x as i32).abs() as f32;
    let dy = (to.y as i32 - from.y as i32).abs() as f32;

    // 对角线移动成本更高
    if dx == 1.0 && dy == 1.0 {
        1.414 * grid.cell_size
    } else {
        grid.cell_size
    }
}

fn heuristic_cost(grid: &crate::core::ConfigNavigationGrid, from: GridPos, to: GridPos) -> f32 {
    let dx = (to.x as i32 - from.x as i32).abs() as f32;
    let dy = (to.y as i32 - from.y as i32).abs() as f32;
    let euclidean_distance = (dx * dx + dy * dy).sqrt() * grid.cell_size;

    // 引入一个非常小的权重来打破平局，p 应该很小
    // 例如 1.0 / (地图最大距离)
    const P: f32 = 1.0 / (300.0 * 300.0);
    return euclidean_distance * (1.0 + P);
}
