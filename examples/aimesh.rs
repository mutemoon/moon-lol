use bevy::prelude::*;
use binrw::{io::BufReader, BinRead};
use moon_lol::{
    combat::PluginCombat, entities::PluginEntities, league::AiMeshNGrid, render::PluginRender,
};
use std::fs::File;

fn main() {
    let file = File::open("assets/bloom.aimesh_ngrid").expect("找不到 aimesh_ngrid 文件！");
    let mut reader = BufReader::new(file);
    let aimesh = AiMeshNGrid::read(&mut reader).expect("解析 aimesh_ngrid 文件失败！");

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PluginCombat)
        .add_plugins(PluginEntities)
        .add_plugins(PluginRender)
        .insert_resource(aimesh)
        .add_systems(Startup, spawn_nav_grid)
        .run();
}

fn spawn_nav_grid(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    aimesh: Res<AiMeshNGrid>,
) {
    let min_pos = aimesh.header.min_grid_pos.as_ref().unwrap();
    let cell_size = aimesh.header.cell_size.unwrap();

    let walkable_material = materials.add(Color::srgb(0.0, 1.0, 0.0));
    let cube_mesh = meshes.add(Sphere::new(cell_size / 2.0));

    println!(
        "开始生成导航网格，共 {} 个单元格...",
        aimesh.navigation_grid.len()
    );

    for nav_cell in aimesh.navigation_grid.iter() {
        let x = nav_cell.get_x();
        let z = nav_cell.get_z();
        let height = nav_cell.get_height();

        if x < 0 || z < 0 {
            continue;
        }

        let world_x = min_pos.0.x + (x as f32 * cell_size);
        let world_y = height;
        let world_z = -(min_pos.0.z + (z as f32 * cell_size));

        commands.spawn((
            Mesh3d(cube_mesh.clone()),
            MeshMaterial3d(walkable_material.clone()),
            Transform::from_xyz(world_x, world_y, world_z),
        ));
    }

    println!("导航网格生成完毕！");
}
