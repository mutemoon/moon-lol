use bevy::prelude::*;
use bevy::render::{
    settings::{Backends, RenderCreation, WgpuSettings},
    RenderPlugin,
};
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiPrimaryContextPass};
use moon_lol::core::{Configs, PluginCamera, PluginResource};
use moon_lol::logging::PluginLogging;

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
            PluginCamera,
            PluginResource,
        ))
        .init_resource::<FlagFilters>()
        .add_systems(Startup, setup)
        .add_systems(EguiPrimaryContextPass, ui_system)
        .add_systems(Update, update_grid_visibility)
        .run();
}

// 常见的导航网格 flags
const COMMON_FLAGS: &[u32] = &[1, 2, 4, 66, 256, 514, 1025, 1088, 2049, 2112, 3, 5184, 6208];

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
    configs: Res<Configs>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut flag_filters: ResMut<FlagFilters>,
) {
    let navigation_grid = &configs.navigation_grid;

    // 初始化显示所有网格点
    flag_filters.show_all = true;

    let mesh = meshes.add(Sphere::new(navigation_grid.cell_size / 2.0));

    for (x, row) in navigation_grid.cells.iter().enumerate() {
        for (y, cell) in row.iter().enumerate() {
            if cell.heuristic > 0.0 {
                continue;
            }

            // 根据 flags 设置不同颜色
            let color = match cell.flags {
                1 => Color::srgb(0.0, 1.0, 0.0),   // 绿色 - 可行走
                2 => Color::srgb(1.0, 0.0, 0.0),   // 红色 - 不可行走
                4 => Color::srgb(0.0, 0.0, 1.0),   // 蓝色 - 特殊区域
                66 => Color::srgb(1.0, 1.0, 0.0),  // 黄色
                256 => Color::srgb(1.0, 0.0, 1.0), // 紫色
                _ => Color::srgb(0.5, 0.5, 0.5),   // 灰色 - 其他
            };

            let material = materials.add(color);
            commands.spawn((
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material),
                Transform::from_translation(navigation_grid.get_cell_pos(x, y)),
                GridCell {
                    flags: cell.flags as u32,
                },
                Visibility::Visible,
            ));
        }
    }
}

fn ui_system(mut contexts: EguiContexts, mut flag_filters: ResMut<FlagFilters>) {
    egui::Window::new("网格点过滤器").default_width(300.0).show(
        contexts.ctx_mut().unwrap(),
        |ui| {
            ui.heading("显示控制");

            ui.checkbox(&mut flag_filters.show_all, "显示所有网格点");

            if !flag_filters.show_all {
                ui.separator();
                ui.heading("按 Flag 过滤");

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
                    if ui.button("全选").clicked() {
                        for &flag in COMMON_FLAGS {
                            flag_filters.enabled_flags.insert(flag);
                        }
                    }

                    if ui.button("全不选").clicked() {
                        flag_filters.enabled_flags.clear();
                    }
                });
            }

            ui.separator();
            ui.label(format!(
                "当前显示的 flags: {:?}",
                if flag_filters.show_all {
                    "全部".to_string()
                } else {
                    format!("{:?}", flag_filters.enabled_flags)
                }
            ));
        },
    );
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
            *visibility = if flag_filters.enabled_flags.contains(&grid_cell.flags) {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
        }
    }
}
