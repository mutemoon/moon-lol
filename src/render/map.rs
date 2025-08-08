use crate::combat::{AttackState, MovementState, Target};
use crate::render::{
    get_barrack_by_bin, neg_z, process_map_geo_mesh, EnvironmentVisibility,
    LayerTransitionBehavior, LeagueBinMaybeCharacterMapRecord, LeagueLoader, LeagueMinionPath,
    WadRes,
};
use crate::system_info;
use bevy::asset::RenderAssetUsages;
use bevy::render::mesh::skinning::SkinnedMeshInverseBindposes;
use bevy::render::mesh::PrimitiveTopology;
use bevy::{color::palettes, prelude::*};
use cdragon_prop::{BinHash, BinMap, BinStruct};

pub struct PluginMap;

impl Plugin for PluginMap {
    fn build(&self, app: &mut App) {
        app.insert_resource(CurrentVisibilityLayers(EnvironmentVisibility::Layer1));
        app.add_systems(Startup, setup_map);
        app.add_systems(Startup, setup_map_placeble);
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
                    ..default()
                })),
                Visibility::Visible,
                MapMeshLayer(map_mesh.environment_visibility),
            ));
        }
    }

    system_info!("setup_map", "Map loaded in {:?}", start_time.elapsed());

    // commands.insert_resource(AmbientLight::default());
}

pub fn draw_attack(
    mut gizmos: Gizmos,
    q_attack: Query<(&Transform, &AttackState)>,
    q_movement_state: Query<(&Transform, &MovementState)>,
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

    for (transform, movement_state) in q_movement_state.iter() {
        let Some(destination) = movement_state.destination else {
            continue;
        };

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

fn setup_map_placeble(
    res_wad: Res<WadRes>,
    mut commands: Commands,
    mut character_cache: ResMut<crate::render::CharacterResourceCache>,
    mut res_animation_clips: ResMut<Assets<AnimationClip>>,
    mut res_animation_graphs: ResMut<Assets<AnimationGraph>>,
    mut res_image: ResMut<Assets<Image>>,
    mut res_materials: ResMut<Assets<StandardMaterial>>,
    mut res_meshes: ResMut<Assets<Mesh>>,
    mut res_skinned_mesh_inverse_bindposes: ResMut<Assets<SkinnedMeshInverseBindposes>>,
) {
    let start_time = std::time::Instant::now();
    let bin = &res_wad.loader.map_materials.0;

    bin.entries
        .iter()
        .filter(|v| v.ctype.hash == LeagueLoader::hash_bin("MapPlaceableContainer"))
        .filter_map(|v| v.getv::<BinMap>(LeagueLoader::hash_bin("items").into()))
        .filter_map(|v| v.downcast::<BinHash, BinStruct>())
        .flatten()
        .for_each(|v| match v.1.ctype.hash {
            0x1e1cce2 => {
                let mut character_map_record = LeagueBinMaybeCharacterMapRecord::from(&v.1);

                neg_z(&mut character_map_record.transform);

                crate::render::spawn_character_cached(
                    &mut commands,
                    &mut character_cache,
                    &mut res_animation_clips,
                    &mut res_animation_graphs,
                    &mut res_image,
                    &mut res_materials,
                    &mut res_meshes,
                    &mut res_skinned_mesh_inverse_bindposes,
                    &res_wad.loader,
                    character_map_record.transform,
                    &character_map_record.definition.skin,
                );
            }
            0x3c995caf => {
                let v: LeagueMinionPath = (&v.1).into();

                let mut transform = Transform::from_matrix(v.transform);

                transform.translation.y = transform.translation.y;
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
            0x71d0eabd => {
                commands.spawn((
                    get_barrack_by_bin(&bin, &v.1),
                    Mesh3d(res_meshes.add(Sphere::new(30.0))),
                    MeshMaterial3d(res_materials.add(StandardMaterial {
                        base_color: palettes::tailwind::PURPLE_500.into(),
                        ..default()
                    })),
                ));
            }
            _ => {}
        });

    system_info!(
        "setup_map_placeble",
        "map objects loaded in {:?}",
        start_time.elapsed()
    );
}
