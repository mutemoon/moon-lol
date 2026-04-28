use bevy::color::palettes;
use bevy::prelude::*;
use bevy::render::render_resource::Face;
use lol_base::grid::{CELL_COST_IMPASSABLE, ConfigNavigationGrid, GridFlagsVisionPathing};
use lol_core::navigation::grid::ResourceGrid;
use lol_core::navigation::navigation::NavigationDebug;

#[derive(Default)]
pub struct PluginRenderNavigation;

impl Plugin for PluginRenderNavigation {
    fn build(&self, app: &mut App) {
        app.insert_resource(FlagFilters {
            // enabled_flags: todo!(),
            show_all: true,
        });
        app.add_systems(
            Update,
            (
                setup_grid_visualization,
                update_grid_visibility,
                update_visualization_astar,
                update_visualization_move_path,
            )
                .run_if(resource_exists::<ResourceGrid>),
        );
    }
}

#[derive(Component)]
struct AStarCell;

#[derive(Component)]
struct AStarPathCell;

#[derive(Component)]
struct ObstacleCell;

#[derive(Component)]
struct GridCell {
    flags: u32,
}

#[derive(Resource, Default)]
pub struct FlagFilters {
    // pub enabled_flags: std::collections::HashSet<u32>,
    pub show_all: bool,
}

fn setup_grid_visualization(
    mut commands: Commands,
    res_grid: Res<ResourceGrid>,
    assets_grid: Res<Assets<ConfigNavigationGrid>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut flag_filters: ResMut<FlagFilters>,
    q_cells: Query<Entity, With<GridCell>>,
) {
    if !q_cells.is_empty() {
        return;
    }
    let Some(grid) = assets_grid.get(&res_grid.0) else {
        return;
    };

    info!("{:?}", grid);

    // Initialize to show all grid points
    flag_filters.show_all = true;

    let mesh = meshes.add(Plane3d::new(
        vec3(0.0, 1.0, 0.0),
        Vec2::splat(grid.cell_size / 2.0 - 5.0),
    ));

    let red_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.0, 0.0),
        unlit: true,
        cull_mode: Some(Face::Front),
        depth_bias: 100.0,
        ..default()
    });

    let green_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.0, 1.0, 0.0),
        unlit: true,
        cull_mode: Some(Face::Front),
        depth_bias: 100.0,
        ..default()
    });

    let blue_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.0, 0.0, 1.0),
        unlit: true,
        cull_mode: Some(Face::Front),
        depth_bias: 100.0,
        ..default()
    });

    for (y, row) in grid.cells.iter().enumerate() {
        for (x, cell) in row.iter().enumerate() {
            let position = grid.get_cell_center_position_by_xy((x, y));
            commands.spawn((
                Mesh3d(mesh.clone()),
                MeshMaterial3d(
                    if cell
                        .vision_pathing_flags
                        .contains(GridFlagsVisionPathing::BlueTeamOnly)
                    {
                        blue_material.clone()
                    } else if cell
                        .vision_pathing_flags
                        .contains(GridFlagsVisionPathing::Wall)
                    {
                        red_material.clone()
                    } else {
                        green_material.clone()
                    },
                ),
                Transform::from_translation(position),
                GridCell {
                    flags: cell.vision_pathing_flags.bits() as u32,
                },
                Visibility::Visible,
            ));
        }
    }
}

fn update_grid_visibility(
    flag_filters: Res<FlagFilters>,
    mut query: Query<(&GridCell, &mut Visibility)>,
) {
    if !flag_filters.is_changed() {
        return;
    }

    for (grid_cell, mut visibility) in query.iter_mut() {
        if flag_filters.show_all {
            *visibility = Visibility::Visible;
        } else {
            *visibility = Visibility::Visible;
            // *visibility = if flag_filters
            //     .enabled_flags
            //     .iter()
            //     .all(|&flag| (grid_cell.flags & flag) != 0)
            // {
            //     Visibility::Visible
            // } else {
            //     Visibility::Hidden
            // };
        }
    }
}

fn update_visualization_astar(
    mut commands: Commands,
    res_grid: Res<ResourceGrid>,
    assets_grid: Res<Assets<ConfigNavigationGrid>>,
    nav_debug: Res<NavigationDebug>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    visited_query: Query<Entity, With<AStarCell>>,
    path_query: Query<Entity, With<AStarPathCell>>,
    obstacle_query: Query<Entity, With<ObstacleCell>>,
) {
    let Some(grid) = assets_grid.get(&res_grid.0) else {
        return;
    };

    if !nav_debug.enabled {
        return;
    }

    if !nav_debug.is_changed() && !assets_grid.is_changed() {
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
    res_grid: Res<ResourceGrid>,
    assets_grid: Res<Assets<ConfigNavigationGrid>>,
    mut gizmos: Gizmos,
    nav_debug: Res<NavigationDebug>,
) {
    let Some(grid) = assets_grid.get(&res_grid.0) else {
        return;
    };

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
