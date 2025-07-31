use bevy::{
    // 引入新的 Mesh3d 和 MeshMaterial3d 组件
    // 注意：根据 Bevy 版本，路径可能是 bevy::pbr 或 bevy::sprite
    pbr::MeshMaterial3d,
    prelude::*,
};
use binrw::BinRead;
use moon_lol::render::LeagueSkeleton;
use std::fs::File;
use std::io::BufReader;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // --- 场景设置 (使用“必需组件”范式) ---

    // 【1. 生成相机】
    // 只需提供 Camera3d 组件和 Transform。
    // 其他如 Projection, Frustum 等都会被自动添加。
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-250.0, 1000.0, 1000.0).looking_at(Vec3::new(0.0, 50.0, 0.0), Vec3::Y),
    ));

    // 【2. 生成光源】
    // 同理，只需提供 DirectionalLight 组件和 Transform。
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::YXZ, -0.8, -0.5, 0.0)),
    ));

    // --- 骨骼可视化 (使用“必需组件”范式) ---
    println!("正在解析骨骼文件: assets/turret.skl");
    let path = "assets/turret.skl";
    let skeleton = match File::open(path) {
        Ok(file) => {
            let mut reader = BufReader::new(file);
            LeagueSkeleton::read(&mut reader).expect("解析 .skl 文件失败")
        }
        Err(e) => {
            eprintln!("错误: 无法打开文件 '{}'。", path);
            eprintln!("具体错误: {}", e);
            return;
        }
    };
    let joints = &skeleton.modern_data.joints;

    let sphere_mesh_handle = meshes.add(Sphere::new(3.0));
    let sphere_material_handle = materials.add(Color::srgb(1.0, 0.2, 0.2));
    let mut joint_entities = vec![None; joints.len()];

    for (i, joint) in joints.iter().enumerate() {
        // 【4. 生成每个关节小球】
        // 同样使用最简洁的方式
        let current_entity = commands
            .spawn((
                Mesh3d(sphere_mesh_handle.clone()),
                MeshMaterial3d(sphere_material_handle.clone()),
                Transform::from_matrix(joint.local_transform),
                Name::new(joint.name.clone()),
            ))
            .id();

        joint_entities[i] = Some(current_entity);

        if joint.parent_id >= 0 {
            if let Some(parent_entity) = joint_entities[joint.parent_id as usize] {
                commands.entity(parent_entity).add_child(current_entity);
            }
        }
    }
}
