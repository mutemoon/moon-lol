use bevy::prelude::*;

use crate::core::{Configs, Movement};

pub struct PluginNavigaton;

impl Plugin for PluginNavigaton {
    fn build(&self, app: &mut App) {
        // 可视化导航网格
        // app.add_systems(Startup, setup);
        app.add_systems(FixedPreUpdate, update);
    }
}

// fn setup(
//     mut commands: Commands,
//     configs: Res<Configs>,
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut materials: ResMut<Assets<StandardMaterial>>,
// ) {
//     let navigation_grid = &configs.navigation_grid;

//     let mut max_heuristic = 0.0;

//     for row in navigation_grid.cells.iter() {
//         for cell in row.iter() {
//             if let Some(cell) = cell {
//                 if cell.heuristic > max_heuristic {
//                     max_heuristic = cell.heuristic;
//                 }
//             }
//         }
//     }

//     let mesh = meshes.add(Sphere::new(navigation_grid.cell_size / 2.0));

//     for (x, row) in navigation_grid.cells.iter().enumerate() {
//         for (y, cell) in row.iter().enumerate() {
//             if let Some(cell) = cell {
//                 let material = materials.add(Color::srgb(
//                     f32::min(cell.heuristic / (max_heuristic / 5.0), 1.0),
//                     0.0,
//                     0.0,
//                 ));
//                 commands.spawn((
//                     Mesh3d(mesh.clone()),
//                     MeshMaterial3d(material.clone()),
//                     Transform::from_translation(navigation_grid.get_cell_pos(x, y)),
//                 ));
//             }
//         }
//     }
// }

fn update(configs: Res<Configs>, mut q_movement: Query<&mut Transform, With<Movement>>) {
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
