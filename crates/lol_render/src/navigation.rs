use bevy::asset::RenderAssetUsages;
use bevy::color::palettes;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use lol_base::grid::{CELL_COST_IMPASSABLE, ConfigNavigationGrid};
use lol_core::navigation::grid::ResourceGrid;
use lol_core::navigation::navigation::{NavigationDebug, NavigationDebugState};

use crate::map::{Map, on_click_map};

// 纹理中每个单元格的像素尺寸（越大线条越精细）
const CELL_TEX_SIZE: u32 = 24;

#[derive(Default)]
pub struct PluginRenderNavigation;

impl Plugin for PluginRenderNavigation {
    fn build(&self, app: &mut App) {
        app.insert_resource(FlagFilters {
            // enabled_flags: todo!(),
            show_all: true,
        });
        // 去掉了 run_once，使网格线持续绘制
        app.add_systems(
            Update,
            setup_grid_visualization.run_if(
                resource_exists::<ResourceGrid>.and_then(resource_exists::<NavigationDebug>), // .and_then(run_once),
            ),
        );
        app.add_systems(
            Update,
            draw_grid.run_if(
                resource_exists::<ResourceGrid>.and_then(resource_exists::<NavigationDebug>),
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
    let w = grid.x_len as u32 * CELL_TEX_SIZE;
    let h = grid.y_len as u32 * CELL_TEX_SIZE;
    let mut data = vec![0u8; (w * h * 4) as usize];

    for row in 0..grid.y_len as u32 {
        for col in 0..grid.x_len as u32 {
            let x0 = col * CELL_TEX_SIZE;
            let y0 = row * CELL_TEX_SIZE;

            for dy in 0..CELL_TEX_SIZE {
                for dx in 0..CELL_TEX_SIZE {
                    let px = x0 + dx;
                    let py = y0 + dy;
                    let idx = ((py * w + px) * 4) as usize;

                    // 网格线：在左/下边界（或最右/最上）画线，颜色灰色，不透明
                    let is_border =
                        dx == 0 || dy == 0 || dx == CELL_TEX_SIZE - 1 || dy == CELL_TEX_SIZE - 1;
                    if is_border {
                        // data[idx..idx + 4].copy_from_slice(&[60, 60, 60, 128]);
                        data[idx..idx + 4].copy_from_slice(&[0, 0, 0, 0]);
                    } else {
                        // 非线区域：完全透明，颜色任意
                        data[idx..idx + 4].copy_from_slice(&[0, 0, 0, 0]);
                        // data[idx..idx + 4].copy_from_slice(&[60, 60, 60, 128]);
                    }
                }
            }
        }
    }

    let mut image = Image::new(
        Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::default(),
    );
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
    q_map: Query<&Map>,
) {
    if !q_map.is_empty() {
        return;
    }

    let Some(grid) = assets_grid.get(&res_grid.0) else {
        info!("没有网格");
        return;
    };
    info!("生成网格");

    flag_filters.show_all = true;

    let mesh = build_merged_grid_mesh(grid, 0.0);
    let texture = build_grid_texture(grid, &mut images);

    let material = materials.add(StandardMaterial {
        base_color_texture: Some(texture),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
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

fn draw_grid(
    res_grid: Res<ResourceGrid>,
    assets_grid: Res<Assets<ConfigNavigationGrid>>,
    mut gizmos: Gizmos,
) {
    let Some(grid) = assets_grid.get(&res_grid.0) else {
        return;
    };

    let height_offset = 0.1;
    let cell_size = grid.cell_size;
    let x_len = grid.x_len;
    let y_len = grid.y_len;

    let thin_color = Color::srgb(0.3, 0.3, 0.3);
    let thick_color = Color::srgb(0.8, 0.8, 0.8);

    // 遍历每个格子，绘制其左边线和下边线（高度使用该格子的中心高度）
    for y in 0..y_len {
        for x in 0..x_len {
            let center = grid.get_cell_center_position_by_xy((x, y));
            let half = cell_size / 2.0;
            let height = center.y + height_offset; // 可以让线条略高于格子表面

            let left = center.x - half;
            let right = center.x + half;
            let bottom = center.z - half;
            let top = center.z + half;

            // 左边线（垂直方向）
            let start_left = Vec3::new(left, height, bottom);
            let end_left = Vec3::new(left, height, top);
            let color_left = if x % 10 == 0 { thick_color } else { thin_color };
            gizmos.line(start_left, end_left, color_left);

            // 下边线（水平方向）
            let start_bottom = Vec3::new(left, height, bottom);
            let end_bottom = Vec3::new(right, height, bottom);
            let color_bottom = if y % 10 == 0 { thick_color } else { thin_color };
            gizmos.line(start_bottom, end_bottom, color_bottom);
        }
    }

    // 补充最右边边界线（右侧无格子再画左边线，需单独绘制）
    for y in 0..y_len {
        let center = grid.get_cell_center_position_by_xy((x_len - 1, y));
        let half = cell_size / 2.0;
        let height = center.y + height_offset;
        let right_x = center.x + half;
        let z_start = center.z - half;
        let z_end = center.z + half;
        let color = if x_len % 10 == 0 {
            thick_color
        } else {
            thin_color
        };
        gizmos.line(
            Vec3::new(right_x, height, z_start),
            Vec3::new(right_x, height, z_end),
            color,
        );
    }

    // 补充最上边边界线
    for x in 0..x_len {
        let center = grid.get_cell_center_position_by_xy((x, y_len - 1));
        let half = cell_size / 2.0;
        let height = center.y + height_offset;
        let top_z = center.z + half;
        let x_start = center.x - half;
        let x_end = center.x + half;
        let color = if y_len % 10 == 0 {
            thick_color
        } else {
            thin_color
        };
        gizmos.line(
            Vec3::new(x_start, height, top_z),
            Vec3::new(x_end, height, top_z),
            color,
        );
    }
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
