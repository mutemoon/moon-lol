use bevy::color::palettes;
use bevy::prelude::*;
use bevy::render::{
    settings::{Backends, RenderCreation, WgpuSettings},
    RenderPlugin,
};
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiPrimaryContextPass};
use league_core::VisionPathingFlags;
use lol_config::ConfigNavigationGrid;

use moon_lol::{
    on_click_map, CameraState, Map, Movement, PluginCore, PluginLogging, PluginNavigaton,
};

fn main() {
    App::new()
        .add_plugins((
            PluginLogging,
            DefaultPlugins
                .build()
                .disable::<bevy::log::LogPlugin>()
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Navigation Grid with Flag Controls".to_string(),
                        ..default()
                    }),
                    ..default()
                })
                .set(RenderPlugin {
                    render_creation: RenderCreation::Automatic(WgpuSettings {
                        backends: Some(Backends::VULKAN),
                        ..default()
                    }),
                    ..default()
                }),
            EguiPlugin::default(),
            PluginCore.build().disable::<PluginNavigaton>(),
        ))
        .init_resource::<FlagFilters>()
        .add_systems(Startup, setup)
        .add_systems(EguiPrimaryContextPass, ui_system)
        .add_systems(Update, update_grid_visibility)
        .add_systems(Update, on_key_space)
        // override navigation plugin
        .init_resource::<AStarVisualization>()
        .add_systems(Update, update_height)
        .add_systems(Update, draw_move_path)
        .add_systems(Update, update_astar_visualization)
        // .add_observer(command_navigation_to)
        // sample height
        // .add_systems(Startup, setup_sample_height_textured)
        // .add_systems(Update, lock_camera_full_map)
        .add_systems(Update, on_key_m)
        .run();
}

// Common navigation grid flags
const COMMON_FLAGS: &[u32] = &[
    0 << 0,
    1 << 0,
    1 << 1,
    1 << 2,
    1 << 3,
    1 << 4,
    1 << 5,
    1 << 6,
    1 << 7,
    1 << 8,
    1 << 9,
    1 << 10,
    1 << 11,
    1 << 12,
    1 << 13,
    1 << 14,
    1 << 15,
    1 << 16,
    1 << 17,
];

#[derive(Resource, Default)]
struct FlagFilters {
    enabled_flags: std::collections::HashSet<u32>,
    show_all: bool,
}

#[derive(Component)]
struct GridCell {
    flags: u32,
}

fn setup(
    mut commands: Commands,
    grid: Res<ConfigNavigationGrid>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut flag_filters: ResMut<FlagFilters>,
) {
    // Initialize to show all grid points
    flag_filters.show_all = true;

    let mesh = meshes.add(Plane3d::new(
        vec3(0.0, 1.0, 0.0),
        Vec2::splat(grid.cell_size / 2.0 - 5.0),
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

    let blue_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.0, 0.0, 1.0),
        unlit: true,
        depth_bias: 100.0,
        ..default()
    });

    for (y, row) in grid.cells.iter().enumerate() {
        for (x, cell) in row.iter().enumerate() {
            let position = grid.get_cell_center_position_by_xy((x, y));
            commands
                .spawn((
                    Mesh3d(mesh.clone()),
                    MeshMaterial3d(
                        if cell
                            .vision_pathing_flags
                            .contains(VisionPathingFlags::BlueTeamOnly)
                        {
                            blue_material.clone()
                        } else if cell.vision_pathing_flags.contains(VisionPathingFlags::Wall) {
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
                ))
                .observe(on_click_map);
        }
    }
}

fn on_key_space(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut q_map: Query<&mut Visibility, With<Map>>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        for mut visibility in q_map.iter_mut() {
            visibility.toggle_visible_hidden();
        }
    }
}

fn update_height(
    grid: Res<ConfigNavigationGrid>,
    mut q_movement: Query<&mut Transform, With<Movement>>,
) {
    for mut transform in q_movement.iter_mut() {
        transform.translation = grid.get_world_position_by_position(&transform.translation.xz());
    }
}

fn draw_move_path(
    grid: Res<ConfigNavigationGrid>,
    mut gizmos: Gizmos,
    astar_vis: Res<AStarVisualization>,
) {
    for path_point in astar_vis.unoptimized_path.windows(2) {
        gizmos.line(
            grid.get_world_position_by_position(&path_point[0]) + vec3(0.0, 4.0, 0.0),
            grid.get_world_position_by_position(&path_point[1]) + vec3(0.0, 4.0, 0.0),
            Color::Srgba(palettes::tailwind::PINK_500),
        );
    }

    for path_point in astar_vis.optimized_path.windows(2) {
        gizmos.line(
            grid.get_world_position_by_position(&path_point[0]) + vec3(0.0, 5.0, 0.0),
            grid.get_world_position_by_position(&path_point[1]) + vec3(0.0, 5.0, 0.0),
            Color::Srgba(palettes::tailwind::BLUE_500),
        );
    }
}

fn ui_system(
    mut contexts: EguiContexts,
    mut flag_filters: ResMut<FlagFilters>,
    astar_vis: Res<AStarVisualization>,
) {
    egui::Window::new("Grid Point Filter")
        .default_width(300.0)
        .show(contexts.ctx_mut().unwrap(), |ui| {
            ui.heading("Display Control");

            ui.checkbox(&mut flag_filters.show_all, "Show All Grid Points");

            if !flag_filters.show_all {
                ui.separator();
                ui.heading("Filter by Flag");

                for &flag in COMMON_FLAGS {
                    let mut enabled = flag_filters.enabled_flags.contains(&flag);
                    if ui
                        .checkbox(&mut enabled, format!("Flag {}", flag))
                        .changed()
                    {
                        if enabled {
                            flag_filters.enabled_flags.insert(flag);
                        } else {
                            flag_filters.enabled_flags.remove(&flag);
                        }
                    }
                }

                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("Select All").clicked() {
                        for &flag in COMMON_FLAGS {
                            flag_filters.enabled_flags.insert(flag);
                        }
                    }

                    if ui.button("Deselect All").clicked() {
                        flag_filters.enabled_flags.clear();
                    }
                });
            }

            ui.separator();
            ui.label(format!(
                "Currently displayed flags: {:?}",
                if flag_filters.show_all {
                    "All".to_string()
                } else {
                    format!("{:?}", flag_filters.enabled_flags)
                }
            ));

            ui.label(format!(
                "Last search visited {} cells",
                astar_vis.visited_cells.len()
            ));

            ui.label(format!("Path length: {} cells", astar_vis.path_cells.len()));
        });
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
            *visibility = if flag_filters
                .enabled_flags
                .iter()
                .all(|&flag| (grid_cell.flags & flag) != 0)
            {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        }
    }
}

// A* 可视化相关组件和资源
#[derive(Component)]
struct AStarCell;

#[derive(Component)]
struct AStarPathCell;

#[derive(Resource, Default)]
pub struct AStarVisualization {
    pub visited_cells: Vec<(usize, usize)>,
    pub path_cells: Vec<(usize, usize)>,
    pub unoptimized_path: Vec<Vec2>,
    pub optimized_path: Vec<Vec2>,
}

// fn command_navigation_to(
//     trigger: Trigger<CommandNavigationTo>,
//     mut commands: Commands,
//     grid: Res<ConfigNavigationGrid>,
//     mut q_transform: Query<&Transform>,
//     mut astar_vis: ResMut<AStarVisualization>,
// ) {
//     let entity = trigger.target();
//     let destination = trigger.target;

//     // 获取当前位置
//     if let Ok(transform) = q_transform.get_mut(entity) {
//         let start_pos = transform.translation.xz();
//         let end_pos = destination;

//         let start = Instant::now();
//         // 使用A*算法规划路径，对于单点移动，创建长度为1的路径
//         if let Some(result) = find_path_with_visualization(&grid, &start_pos, &end_pos) {
//             // 更新 A* 可视化数据
//             astar_vis.visited_cells = result.visited_cells;
//             astar_vis.path_cells = result.path.clone();

//             if result.path.is_empty() {
//                 return;
//             }

//             let world_path = post_process_path(&grid, &result.path, &start_pos, &end_pos);

//             system_debug!(
//                 "command_navigation_to",
//                 "Path found in {:.6}ms",
//                 start.elapsed().as_millis()
//             );

//             astar_vis.optimized_path = world_path.clone();

//             commands.trigger_targets(
//                 CommandMovement {
//                     priority: 0,
//                     action: MovementAction::Start {
//                         way: MovementWay::Path(world_path),
//                         speed: None,
//                     },
//                 },
//                 entity,
//             );
//         }
//     }
// }

// fn find_path_with_visualization(
//     grid: &ConfigNavigationGrid,
//     start: &Vec2,
//     end: &Vec2,
// ) -> Option<AStarResult> {
//     if let Some(astar_result) = find_grid_path_with_result(grid, start, end) {
//         // 注意：这里不对路径进行简化，保持原始的A*网格路径
//         // 简化路径只在最终转换为世界坐标时使用
//         Some(astar_result)
//     } else {
//         None
//     }
// }

/// 更新 A* 可视化系统
fn update_astar_visualization(
    mut commands: Commands,
    grid: Res<ConfigNavigationGrid>,
    astar_vis: Res<AStarVisualization>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    visited_query: Query<Entity, With<AStarCell>>,
    path_query: Query<Entity, With<AStarPathCell>>,
) {
    if !astar_vis.is_changed() {
        return;
    }

    // 删除旧的 A* 可视化单元格
    for entity in visited_query.iter() {
        commands.entity(entity).despawn();
    }

    for entity in path_query.iter() {
        commands.entity(entity).despawn();
    }

    let mesh = meshes.add(Plane3d::new(
        vec3(0.0, 1.0, 0.0),
        Vec2::splat(grid.cell_size / 2.0 - 3.0),
    ));

    // 如果需要显示访问的单元格（黄色）
    let yellow_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 1.0, 0.0), // 黄色
        unlit: true,
        depth_bias: 50.0,
        ..default()
    });

    // 创建访问过的单元格
    for &(x, y) in &astar_vis.visited_cells {
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

    // 如果需要显示路径单元格（白色）
    let white_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 1.0, 1.0), // 白色
        unlit: true,
        depth_bias: 60.0, // 比黄色稍高一点，确保显示在上面
        ..default()
    });

    // 创建路径单元格
    for &(x, y) in &astar_vis.path_cells {
        commands.spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(white_material.clone()),
            Transform::from_translation(
                grid.get_cell_center_position_by_xy((x, y)) + Vec3::new(0.0, 3.0, 0.0), // 比黄色更高
            ),
            AStarPathCell,
            Visibility::Visible,
        ));
    }
}

fn on_key_m(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    grid: Res<ConfigNavigationGrid>,
    mut camera: Query<&mut CameraState, With<Camera3d>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyM) {
        let center_pos = grid.get_map_center_position();

        if let Ok(mut camera_state) = camera.single_mut() {
            camera_state.position = center_pos;
            camera_state.scale = 10.0;
        }
    }
}
