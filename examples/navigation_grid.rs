use bevy::prelude::*;
use bevy::render::render_resource::Face;
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiPrimaryContextPass};

use league_core::VisionPathingFlags;
use lol_config::ConfigNavigationGrid;

use moon_lol::{
    on_click_map, CameraState, Map, NavigationDebug, PluginBarrack, PluginCore, PluginResource,
};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.build().set(WindowPlugin {
                primary_window: Some(Window {
                    title: "navigation".to_string(),
                    resolution: (300, 300).into(),
                    position: WindowPosition::At((0, 1000).into()),
                    ..default()
                }),
                ..default()
            }),
            EguiPlugin::default(),
            PluginCore
                .build()
                .disable::<PluginBarrack>()
                .set(PluginResource {
                    game_config_path: "games/attack.ron".to_string(),
                }),
        ))
        .init_resource::<FlagFilters>()
        .insert_resource(NavigationDebug {
            enabled: true,
            ..default()
        })
        .add_systems(Startup, setup)
        .add_systems(EguiPrimaryContextPass, ui_system)
        .add_systems(Update, update_grid_visibility)
        .add_systems(Update, on_key_space)
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
    mut nav_debug: ResMut<NavigationDebug>,
) {
    // 启用 A* 可视化
    nav_debug.enabled = true;

    // Initialize to show all grid points
    flag_filters.show_all = true;

    // let mesh = meshes.add(Plane3d::new(
    //     vec3(0.0, 1.0, 0.0),
    //     Vec2::splat(grid.cell_size / 2.0 - 5.0),
    // ));

    // let red_material = materials.add(StandardMaterial {
    //     base_color: Color::srgb(1.0, 0.0, 0.0),
    //     unlit: true,
    //     cull_mode: Some(Face::Front),
    //     depth_bias: 100.0,
    //     ..default()
    // });

    // let green_material = materials.add(StandardMaterial {
    //     base_color: Color::srgb(0.0, 1.0, 0.0),
    //     unlit: true,
    //     cull_mode: Some(Face::Front),
    //     depth_bias: 100.0,
    //     ..default()
    // });

    // let blue_material = materials.add(StandardMaterial {
    //     base_color: Color::srgb(0.0, 0.0, 1.0),
    //     unlit: true,
    //     cull_mode: Some(Face::Front),
    //     depth_bias: 100.0,
    //     ..default()
    // });

    // for (y, row) in grid.cells.iter().enumerate() {
    //     for (x, cell) in row.iter().enumerate() {
    //         let position = grid.get_cell_center_position_by_xy((x, y));
    //         commands
    //             .spawn((
    //                 Mesh3d(mesh.clone()),
    //                 MeshMaterial3d(
    //                     if cell
    //                         .vision_pathing_flags
    //                         .contains(VisionPathingFlags::BlueTeamOnly)
    //                     {
    //                         blue_material.clone()
    //                     } else if cell.vision_pathing_flags.contains(VisionPathingFlags::Wall) {
    //                         red_material.clone()
    //                     } else {
    //                         green_material.clone()
    //                     },
    //                 ),
    //                 Transform::from_translation(position),
    //                 GridCell {
    //                     flags: cell.vision_pathing_flags.bits() as u32,
    //                 },
    //                 Visibility::Visible,
    //             ))
    //             .observe(on_click_map);
    //     }
    // }
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

fn ui_system(
    mut contexts: EguiContexts,
    mut flag_filters: ResMut<FlagFilters>,
    mut nav_debug: ResMut<NavigationDebug>,
) {
    egui::Window::new("Grid Point Filter")
        .default_width(300.0)
        .show(contexts.ctx_mut().unwrap(), |ui| {
            ui.heading("Display Control");

            ui.checkbox(&mut flag_filters.show_all, "Show All Grid Points");

            // A* debug 开关
            if ui
                .checkbox(&mut nav_debug.enabled, "A* Visualization")
                .changed()
            {
                // 当状态改变时触发更新
            }

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
                nav_debug.visited_cells.len()
            ));

            ui.label(format!("Path length: {} cells", nav_debug.path_cells.len()));
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
