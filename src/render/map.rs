use crate::combat::{AttackInfo, MoveDestination, Target};
use crate::render::{process_map_geo_mesh, EnvironmentVisibility, WadRes};
use bevy::math::vec3;
use bevy::{color::palettes, prelude::*};

pub struct PluginMap;

impl Plugin for PluginMap {
    fn build(&self, app: &mut App) {
        // app.add_systems(Startup, setup_map);
        app.add_systems(Update, draw_attack);
        // app.add_systems(Update, update_z);
    }
}

fn setup_map(
    mut commands: Commands,
    res_wad: Res<WadRes>,
    mut res_mesh: ResMut<Assets<Mesh>>,
    mut res_image: ResMut<Assets<Image>>,
    mut res_materials: ResMut<Assets<StandardMaterial>>,
) {
    // Add error handling to prevent panics
    let map_geo = match res_wad
        .loader
        .get_map_geo_by_path("data/maps/mapgeometry/map11/boba_srs_act2a.mapgeo")
    {
        Ok(geo) => geo,
        Err(e) => {
            error!("Failed to load map geometry: {}", e);
            return;
        }
    };

    let map_materials = match res_wad
        .loader
        .get_prop_bin_by_path("data/maps/mapgeometry/map11/boba_srs_act2a.materials.bin")
    {
        Ok(materials) => materials,
        Err(e) => {
            error!("Failed to load map materials: {}", e);
            return;
        }
    };

    let start_time = std::time::Instant::now();
    // Process fewer meshes initially to reduce load
    for map_mesh in map_geo.meshes.iter() {
        if !map_mesh
            .environment_visibility
            .contains(EnvironmentVisibility::Layer1)
        {
            continue;
        }

        let bevy_meshes = process_map_geo_mesh(&map_materials, &map_geo, map_mesh, &res_wad.loader);

        for (mesh, material_image) in bevy_meshes {
            // Validate mesh before adding
            if mesh.attribute(Mesh::ATTRIBUTE_POSITION).is_none() {
                warn!("Skipping mesh with no position data");
                continue;
            }

            let format = material_image.texture_descriptor.format.clone();

            commands.spawn((
                Mesh3d(res_mesh.add(mesh)),
                MeshMaterial3d(res_materials.add(StandardMaterial {
                    base_color_texture: Some(res_image.add(material_image)),
                    // Use more conservative material settings
                    metallic: 0.0,
                    reflectance: 0.0,
                    alpha_mode: match format {
                        bevy::render::render_resource::TextureFormat::Bc1RgbaUnorm
                        | bevy::render::render_resource::TextureFormat::Bc1RgbaUnormSrgb => {
                            AlphaMode::Mask(0.3)
                        }
                        _ => AlphaMode::Blend,
                    },
                    ..default()
                })),
            ));
        }
    }

    println!("Map loaded in {:?}", start_time.elapsed());

    commands.spawn((
        DirectionalLight {
            ..Default::default()
        },
        Transform::default()
            .with_translation(vec3(0.0, 1000.0, 100.0))
            .looking_at(vec3(0.0, 0.0, 0.0), Dir3::Z),
    ));

    commands.insert_resource(AmbientLight::default());
}

// TODO: Fix picking system integration
// pub fn on_click_map(
//     click: Trigger<Pointer<Pressed>>,
//     mut commands: Commands,
//     q_move: Query<Entity, With<Controller>>,
// ) {
//     let Some(position) = click.hit.position else {
//         return;
//     };

//     let targets = q_move.iter().collect::<Vec<Entity>>();

//     system_debug!(
//         "on_click_map",
//         "Received click at position ({:.1}, {:.1}, {:.1}) move len: {}",
//         position.x,
//         position.y,
//         position.z,
//         targets.len()
//     );

//     commands.trigger_targets(CommandMove { target: position }, targets);
// }

pub fn draw_attack(
    mut gizmos: Gizmos,
    q_attack: Query<(&Transform, &AttackInfo)>,
    q_move_destination: Query<(&Transform, &MoveDestination)>,
    q_target: Query<(&Transform, &Target)>,
    q_transform: Query<&Transform>,
) {
    for (transform, attack_info) in q_attack.iter() {
        let Some(target) = attack_info.target else {
            continue;
        };
        let Ok(target_transform) = q_transform.get(target) else {
            continue;
        };
        gizmos.line(
            transform.translation,
            target_transform.translation,
            Color::Srgba(palettes::tailwind::RED_500),
        );
    }

    for (transform, move_destination) in q_move_destination.iter() {
        let destination = move_destination.0;

        gizmos.line(
            transform.translation,
            destination.extend(transform.translation.z),
            Color::Srgba(palettes::tailwind::GREEN_500),
        );
    }

    for (transform, target) in q_target.iter() {
        let Ok(target_transform) = q_transform.get(target.0) else {
            continue;
        };
        gizmos.line(
            transform.translation,
            target_transform.translation,
            Color::Srgba(palettes::tailwind::YELLOW_500),
        );
    }
}

// fn update_z(
//     mut ray_cast: MeshRayCast,
//     map_query: Query<(), With<Map>>,
//     // 新增一个查询，用于获取实体的父级
//     parent_query: Query<&ChildOf>,
//     mut q_champion: Query<(Entity, &mut Transform), With<Champion>>,
// ) {
//     for (champion_entity, mut transform) in q_champion.iter_mut() {
//         let ray_origin = Vec3::new(transform.translation.x, transform.translation.y, 10000.0);
//         let ray = Ray3d::new(ray_origin, -Dir3::Z);

//         // 在闭包中，我们需要检查实体自身或其任何父级是否是地图
//         let filter = |entity: Entity| {
//             // 首先，仍然要确保不与自己碰撞
//             if entity == champion_entity {
//                 return false;
//             }

//             let mut current_entity = entity;
//             // 循环向上查找
//             loop {
//                 // 检查当前实体是否是 Map
//                 if map_query.contains(current_entity) {
//                     return true; // 找到了！这个碰撞有效
//                 }

//                 // 尝试获取当前实体的父级
//                 match parent_query.get(current_entity) {
//                     // 如果有父级，下一轮循环就检查父级
//                     Ok(parent) => current_entity = parent.parent(),
//                     // 如果没有父级了（已经到了层级顶端），说明不是地图的一部分
//                     Err(_) => return false,
//                 }
//             }
//         };

//         let settings = MeshRayCastSettings::default().with_filter(&filter);
//         let hits = ray_cast.cast_ray(ray, &settings);

//         // ... 后续逻辑保持不变 ...
//         let highest_hit = hits.iter().max_by(|a, b| {
//             a.1.point
//                 .z
//                 .partial_cmp(&b.1.point.z)
//                 .unwrap_or(Ordering::Equal)
//         });

//         if let Some(intersection) = highest_hit {
//             transform.translation.z = intersection.1.point.z;
//         }
//     }
// }
