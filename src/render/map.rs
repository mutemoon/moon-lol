use crate::combat::{AttackInfo, MoveDestination, Target};
use crate::render::{
    load_character, process_map_geo_mesh, EnvironmentVisibility, LayerTransitionBehavior,
    LeagueLoader, LeagueMinionPath, WadRes,
};
use bevy::animation::{animated_field, AnimationTarget, AnimationTargetId};
use bevy::asset::RenderAssetUsages;
use bevy::render::mesh::skinning::{SkinnedMesh, SkinnedMeshInverseBindposes};
use bevy::render::mesh::PrimitiveTopology;
use bevy::{color::palettes, prelude::*};
use bevy_egui::EguiPlugin;
use cdragon_prop::{BinHash, BinMap, BinStruct};

pub struct PluginMap;

impl Plugin for PluginMap {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<EguiPlugin>() {
            app.add_plugins(EguiPlugin::default());
        }

        app.insert_resource(CurrentVisibilityLayers(EnvironmentVisibility::Layer1));
        app.add_systems(Startup, setup_map);
        app.add_systems(Startup, setup_map_placeble);
        // app.add_systems(Update, draw_attack);
        // app.add_systems(Update, update_map_visibility);

        // app.add_systems(EguiPrimaryContextPass, visibility_ui_system);
        // app.add_systems(Update, update_z);
    }
}

// 用于存储全局选择的可见性图层的 Resource
#[derive(Resource, Debug)]
pub struct CurrentVisibilityLayers(pub EnvironmentVisibility);

// 用于标记每个地图网格实体所属图层的 Component
#[derive(Component, Debug)]
pub struct MapMeshLayer(pub EnvironmentVisibility);

fn setup_map(
    mut commands: Commands,
    res_wad: Res<WadRes>,
    mut res_mesh: ResMut<Assets<Mesh>>,
    mut res_image: ResMut<Assets<Image>>,
    mut res_materials: ResMut<Assets<StandardMaterial>>,
) {
    let start_time = std::time::Instant::now();
    // Process fewer meshes initially to reduce load
    for map_mesh in res_wad.loader.map_geo.meshes.iter() {
        if map_mesh.layer_transition_behavior != LayerTransitionBehavior::Unaffected {
            continue;
        }

        let bevy_meshes = process_map_geo_mesh(
            &res_wad.loader.map_materials.0,
            &res_wad.loader.map_geo,
            map_mesh,
            &res_wad.loader,
        );

        for (mesh, material_image) in bevy_meshes {
            let _format = material_image.texture_descriptor.format.clone();

            commands.spawn((
                Mesh3d(res_mesh.add(mesh)),
                MeshMaterial3d(res_materials.add(StandardMaterial {
                    base_color_texture: Some(res_image.add(material_image)),
                    unlit: true,
                    alpha_mode: AlphaMode::Mask(0.3),
                    // alpha_mode: match format {
                    //     bevy::render::render_resource::TextureFormat::Bc1RgbaUnorm
                    //     | bevy::render::render_resource::TextureFormat::Bc1RgbaUnormSrgb => {
                    //         AlphaMode::Mask(0.3)
                    //     }
                    //     _ => AlphaMode::Blend,
                    // },
                    ..default()
                })),
                Visibility::Visible,
                MapMeshLayer(map_mesh.environment_visibility),
            ));
        }
    }

    println!("Map loaded in {:?}", start_time.elapsed());

    // commands.spawn((
    //     DirectionalLight {
    //         ..Default::default()
    //     },
    //     Transform::default()
    //         .with_translation(vec3(0.0, 1000.0, 100.0))
    //         .looking_at(vec3(0.0, 0.0, 0.0), Dir3::Z),
    // ));

    commands.insert_resource(AmbientLight::default());
}

// 根据全局选择更新地图网格的可见性
/* fn update_map_visibility(
    // 监听资源变化，只有在选项改变时才执行逻辑
    layers_state: Res<CurrentVisibilityLayers>,
    // 查询所有带图层标记的地图实体
    mut query: Query<(&MapMeshLayer, &mut Visibility)>,
) {
    // 如果资源没有变化，则无需执行，提高效率
    if !layers_state.is_changed() {
        return;
    }

    let selected_layers = layers_state.0;
    info!("Updating visibility based on layers: {:?}", selected_layers);

    for (map_mesh_layer, mut visibility) in query.iter_mut() {
        // 检查该实体的图层(map_mesh_layer.0)与当前选择的图层(selected_layers)是否有交集
        // .intersects() 方法会判断两个 bitflags 是否有任何共同的位被设置
        let is_visible = map_mesh_layer.0.intersects(selected_layers);

        if is_visible {
            *visibility = Visibility::Visible;
        } else {
            *visibility = Visibility::Hidden;
        }
    }
} */

// use bevy_egui::{egui, EguiContexts};

// UI 系统，用于显示和修改可见性图层
/* fn visibility_ui_system(
    mut contexts: EguiContexts,
    mut layers_state: ResMut<CurrentVisibilityLayers>,
) {
    egui::Window::new("图层可见性 (Visibility)").show(contexts.ctx_mut().unwrap(), |ui| {
        ui.heading("选择要显示的图层");

        // 创建一个包含所有图层的列表以便于遍历
        let all_layers = [
            (EnvironmentVisibility::Layer1, "Layer 1"),
            (EnvironmentVisibility::Layer2, "Layer 2"),
            (EnvironmentVisibility::Layer3, "Layer 3"),
            (EnvironmentVisibility::Layer4, "Layer 4"),
            (EnvironmentVisibility::Layer5, "Layer 5"),
            (EnvironmentVisibility::Layer6, "Layer 6"),
            (EnvironmentVisibility::Layer7, "Layer 7"),
            (EnvironmentVisibility::Layer8, "Layer 8"),
        ];

        for (layer_flag, name) in all_layers.iter() {
            // `toggle_value` 是一个非常有用的 egui 功能，它直接修改 bitflags
            let mut is_selected = layers_state.0.contains(*layer_flag);
            if ui.checkbox(&mut is_selected, *name).changed() {
                // 当复选框状态改变时，更新我们的 Resource
                layers_state.0.toggle(*layer_flag);
            }
        }

        ui.separator();

        if ui.button("显示全部").clicked() {
            layers_state.0 = EnvironmentVisibility::AllLayers;
        }
        if ui.button("隐藏全部").clicked() {
            layers_state.0 = EnvironmentVisibility::NoLayer;
        }
    });
} */

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

fn setup_map_placeble(
    res_wad: Res<WadRes>,
    mut commands: Commands,
    mut res_meshes: ResMut<Assets<Mesh>>,
    mut res_materials: ResMut<Assets<StandardMaterial>>,
    mut res_image: ResMut<Assets<Image>>,
    mut skinned_mesh_inverse_bindposes: ResMut<Assets<SkinnedMeshInverseBindposes>>,
    mut animation_clips: ResMut<Assets<AnimationClip>>,
    mut animation_graphs: ResMut<Assets<AnimationGraph>>,
) {
    let start_time = std::time::Instant::now();
    res_wad
        .loader
        .map_materials
        .0
        .entries
        .iter()
        .filter(|v| v.ctype.hash == LeagueLoader::hash_bin("MapPlaceableContainer"))
        .filter_map(|v| v.getv::<BinMap>(LeagueLoader::hash_bin("items").into()))
        .filter_map(|v| v.downcast::<BinHash, BinStruct>())
        .flatten()
        .for_each(|v| match v.1.ctype.hash {
            0x1e1cce2 => {
                {
                    let (
                        league_bin_maybe_character_map_record,
                        league_bin_character_record,
                        image,
                        league_skinned_mesh,
                        league_skeleton,
                        animation_data,
                    ) = load_character(&res_wad.loader, &v.1);

                    let mut clip = AnimationClip::default();

                    let mut index_to_entity =
                        vec![Entity::PLACEHOLDER; league_skeleton.modern_data.joints.len()];
                    let mut joint_inverse_matrix =
                        vec![Mat4::default(); league_skeleton.modern_data.joints.len()];

                    let mut transform =
                        Transform::from_matrix(league_bin_maybe_character_map_record.transform);

                    transform.translation.z = -transform.translation.z;

                    let player_entity = commands.spawn(transform).id();

                    let sphere = res_meshes.add(Sphere::new(50.0));

                    let mat = res_materials.add(Color::srgb(1.0, 0.2, 0.2));

                    for (i, joint) in league_skeleton.modern_data.joints.iter().enumerate() {
                        let joint_name_str = joint.name.clone();
                        let name = Name::new(joint_name_str.clone());
                        let hash = LeagueLoader::compute_joint_hash(&joint.name);

                        let target_id = AnimationTargetId::from_name(&name);

                        match animation_data {
                            Some(ref animation_data) => {
                                if let Some(anim_track_index) =
                                    animation_data.joint_hashes.iter().position(|v| *v == hash)
                                {
                                    if let Some(data) =
                                        animation_data.translates.get(anim_track_index)
                                    {
                                        clip.add_curve_to_target(
                                            target_id,
                                            AnimatableCurve::new(
                                                animated_field!(Transform::translation),
                                                AnimatableKeyframeCurve::new(
                                                    data.clone().into_iter(),
                                                )
                                                .unwrap(),
                                            ),
                                        );
                                    }

                                    if let Some(data) =
                                        animation_data.rotations.get(anim_track_index)
                                    {
                                        clip.add_curve_to_target(
                                            target_id,
                                            AnimatableCurve::new(
                                                animated_field!(Transform::rotation),
                                                AnimatableKeyframeCurve::new(
                                                    data.clone().into_iter(),
                                                )
                                                .unwrap(),
                                            ),
                                        );
                                    }

                                    if let Some(data) = animation_data.scales.get(anim_track_index)
                                    {
                                        clip.add_curve_to_target(
                                            target_id,
                                            AnimatableCurve::new(
                                                animated_field!(Transform::scale),
                                                AnimatableKeyframeCurve::new(
                                                    data.clone().into_iter(),
                                                )
                                                .unwrap(),
                                            ),
                                        );
                                    }
                                }
                            }
                            None => {}
                        }

                        let ent = commands
                            .spawn((
                                // Mesh3d(sphere.clone()),
                                // MeshMaterial3d(mat.clone()),
                                Transform::from_matrix(joint.local_transform),
                                name,
                                AnimationTarget {
                                    id: target_id,
                                    player: player_entity,
                                },
                            ))
                            .id();
                        index_to_entity[i] = ent;
                        joint_inverse_matrix[i] = joint.inverse_bind_transform;
                    }

                    for (i, joint) in league_skeleton.modern_data.joints.iter().enumerate() {
                        if joint.parent_id >= 0 {
                            let parent_entity = index_to_entity[joint.parent_id as usize];
                            commands.entity(parent_entity).add_child(index_to_entity[i]);
                        } else {
                            commands.entity(player_entity).add_child(index_to_entity[i]);
                        }
                    }

                    let texu = res_image.add(image);

                    let clip_handle = animation_clips.add(clip);

                    let (graph, animation_node_index) = AnimationGraph::from_clip(clip_handle);
                    let graph_handle = animation_graphs.add(graph);

                    let mut player = AnimationPlayer::default();
                    player.play(animation_node_index).repeat();

                    commands
                        .entity(player_entity)
                        .insert((player, AnimationGraphHandle(graph_handle)));

                    for i in 0..league_skinned_mesh.ranges.len() {
                        let mesh = league_skinned_mesh.to_bevy_mesh(i).unwrap();

                        let child = commands
                            .spawn((
                                Transform::default(),
                                Mesh3d(res_meshes.add(mesh)),
                                MeshMaterial3d(res_materials.add(StandardMaterial {
                                    base_color_texture: Some(texu.clone()),
                                    unlit: true,
                                    cull_mode: None,
                                    alpha_mode: AlphaMode::Opaque,
                                    ..Default::default()
                                })),
                                SkinnedMesh {
                                    inverse_bindposes: skinned_mesh_inverse_bindposes.add(
                                        SkinnedMeshInverseBindposes::from(
                                            league_skeleton
                                                .modern_data
                                                .influences
                                                .iter()
                                                .map(|v| joint_inverse_matrix[*v as usize])
                                                .collect::<Vec<_>>(),
                                        ),
                                    ),
                                    joints: league_skeleton
                                        .modern_data
                                        .influences
                                        .iter()
                                        .map(|v| index_to_entity[*v as usize])
                                        .collect::<Vec<_>>(),
                                },
                            ))
                            .id();
                        commands.entity(player_entity).add_child(child);
                    }
                }
            }
            0x3c995caf => {
                let v: LeagueMinionPath = (&v.1).into();
                println!("{:#?}", v);

                let mut transform = Transform::from_matrix(v.transform);

                transform.translation.y = transform.translation.y + 200.0;
                transform.translation.z = -transform.translation.z;

                // 这个部分保持不变，用于在路径起点生成一个红色的球体
                let root = commands
                    .spawn((
                        transform,
                        Mesh3d(res_meshes.add(Sphere::new(30.0))),
                        MeshMaterial3d(res_materials.add(StandardMaterial {
                            base_color: palettes::tailwind::RED_500.into(),
                            ..default()
                        })),
                    ))
                    .id();

                if v.segments.len() > 1 {
                    // 1. 创建一个新的 Mesh，并指定其拓扑为 LineStrip
                    // LineStrip 会按顺序连接所有顶点，例如 0-1, 1-2, 2-3, ...
                    let mut line_mesh =
                        Mesh::new(PrimitiveTopology::LineStrip, RenderAssetUsages::default());

                    // 2. 将路径的所有点作为顶点位置属性插入到 Mesh 中
                    line_mesh.insert_attribute(
                        Mesh::ATTRIBUTE_POSITION,
                        v.segments
                            .iter()
                            .map(|v| {
                                let mut v = v.clone();
                                v.z = -v.z;
                                v
                            })
                            .collect::<Vec<Vec3>>()
                            .clone(),
                    );

                    let child = commands
                        .spawn((
                            Mesh3d(res_meshes.add(line_mesh)),
                            MeshMaterial3d(res_materials.add(StandardMaterial {
                                base_color: palettes::tailwind::GREEN_500.into(),
                                // 对于调试线条，通常设置为 unlit 效果更好，使其不受光照影响
                                unlit: true,
                                ..default()
                            })),
                            // 由于顶点坐标已经是世界坐标，所以 Transform 可以是默认的
                            Transform::default(),
                        ))
                        .id();

                    // 3. 生成包含这条完整路径的单个实体
                    commands.entity(root).add_child(child);
                }
            }
            _ => {}
        });

    println!("map objects loaded in {:?}", start_time.elapsed());
}
