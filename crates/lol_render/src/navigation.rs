use bevy::asset::RenderAssetUsages;
use bevy::color::palettes;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use lol_base::grid::{CELL_COST_IMPASSABLE, ConfigNavigationGrid, GridFlagsVisionPathing};
use lol_core::navigation::grid::ResourceGrid;
use lol_core::navigation::navigation::{NavigationDebug, NavigationDebugState};

use crate::map::{Map, on_click_map};

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
            setup_grid_visualization.run_if(
                resource_exists::<ResourceGrid>
                    .and_then(resource_exists::<NavigationDebug>)
                    .and_then(run_once),
            ),
        );
        app.add_systems(
            Update,
            (update_visualization_astar, update_visualization_move_path).run_if(
                resource_exists::<ResourceGrid>.and_then(resource_exists::<NavigationDebug>),
            ),
        );
    }
}

#[derive(Component)]
struct AStarCell;

#[derive(Component)]
struct AStarPathCell;

#[derive(Component)]
struct ObstacleCell;

#[derive(Resource, Default)]
pub struct FlagFilters {
    // pub enabled_flags: std::collections::HashSet<u32>,
    pub show_all: bool,
}

fn build_merged_grid_mesh(grid: &ConfigNavigationGrid, gap: f32) -> Mesh {
    let x_len = grid.x_len;
    let y_len = grid.y_len;
    let half = (grid.cell_size / 2.0) - gap;

    let mut positions = Vec::new();
    let mut uvs = Vec::new();
    let mut normals = Vec::new();
    let mut indices = Vec::new();

    for y in 0..y_len {
        let v_min = y as f32 / y_len as f32;
        let v_max = (y + 1) as f32 / y_len as f32;

        for x in 0..x_len {
            let center = grid.get_cell_center_position_by_xy((x, y));
            let u_min = x as f32 / x_len as f32;
            let u_max = (x + 1) as f32 / x_len as f32;

            let v0 = center + Vec3::new(-half, 0.0, -half);
            let v1 = center + Vec3::new(half, 0.0, -half);
            let v2 = center + Vec3::new(half, 0.0, half);
            let v3 = center + Vec3::new(-half, 0.0, half);

            let base = positions.len() as u32;
            positions.extend_from_slice(&[v0, v1, v2, v3]);

            normals.extend_from_slice(&[Vec3::Y, Vec3::Y, Vec3::Y, Vec3::Y]);

            uvs.extend_from_slice(&[
                [u_min, v_min],
                [u_max, v_min],
                [u_max, v_max],
                [u_min, v_max],
            ]);

            indices.extend_from_slice(&[base, base + 2, base + 1, base, base + 3, base + 2]);
        }
    }

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(Indices::U32(indices))
}

fn build_grid_texture(grid: &ConfigNavigationGrid, images: &mut Assets<Image>) -> Handle<Image> {
    let width = grid.x_len as u32;
    let height = grid.y_len as u32;
    let mut data = vec![0u8; (width * height * 4) as usize];

    for (y, row) in grid.cells.iter().enumerate() {
        for (x, cell) in row.iter().enumerate() {
            let color = if cell
                .vision_pathing_flags
                .contains(GridFlagsVisionPathing::BlueTeamOnly)
            {
                [0u8, 0, 255, 255]
            } else if cell
                .vision_pathing_flags
                .contains(GridFlagsVisionPathing::Wall)
            {
                [255, 0, 0, 255]
            } else {
                [0, 255, 0, 255]
            };

            let idx = ((y as u32 * width + x as u32) * 4) as usize;
            data[idx..idx + 4].copy_from_slice(&color);
        }
    }

    let mut image = Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::default(),
    );

    // Set nearest neighbor sampling to prevent blurring between cells
    image.sampler = bevy::image::ImageSampler::nearest();

    images.add(image)
}

fn setup_grid_visualization(
    mut commands: Commands,
    res_grid: Res<ResourceGrid>,
    assets_grid: Res<Assets<ConfigNavigationGrid>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut flag_filters: ResMut<FlagFilters>,
) {
    let Some(grid) = assets_grid.get(&res_grid.0) else {
        info!("网格未加载");
        return;
    };
    info!("生成网格");

    flag_filters.show_all = true;

    let mesh = build_merged_grid_mesh(grid, 5.0);
    let texture = build_grid_texture(grid, &mut images);

    let material = materials.add(StandardMaterial {
        base_color_texture: Some(texture),
        unlit: true,
        depth_bias: 100.0,
        ..default()
    });

    commands
        .spawn((
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(material),
            Transform::default(),
            Visibility::Visible,
            Map,
        ))
        .observe(on_click_map);
}

fn update_visualization_astar(
    mut commands: Commands,
    res_grid: Res<ResourceGrid>,
    assets_grid: Res<Assets<ConfigNavigationGrid>>,
    nav_debug: Res<NavigationDebugState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    visited_query: Query<Entity, With<AStarCell>>,
    path_query: Query<Entity, With<AStarPathCell>>,
    obstacle_query: Query<Entity, With<ObstacleCell>>,
) {
    let Some(grid) = assets_grid.get(&res_grid.0) else {
        return;
    };

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
        ..default()
    });

    let mut blue_materials = Vec::new();
    for i in 0..11 {
        blue_materials.push(materials.add(StandardMaterial {
            base_color: Color::srgb(0.1, 0.1, i as f32 / 10.0),
            unlit: true,
            depth_bias: 40.0,
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
    nav_debug: Res<NavigationDebugState>,
) {
    let Some(grid) = assets_grid.get(&res_grid.0) else {
        return;
    };

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
