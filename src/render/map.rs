use crate::combat::{AttackState, Movement, MovementDestination, Target};
use crate::map::Map;
// use crate::render::WadRes;
use crate::league::{
    get_barrack_by_bin, neg_mat_z, process_map_geo_mesh, EnvironmentVisibility,
    LayerTransitionBehavior, LeagueBinMaybeCharacterMapRecord, LeagueLoader,
};
use crate::system_info;
use bevy::render::mesh::skinning::SkinnedMeshInverseBindposes;
use bevy::{color::palettes, prelude::*};
use cdragon_prop::{BinHash, BinMap, BinStruct};
use std::cmp::Ordering;

pub struct PluginMap;

impl Plugin for PluginMap {
    fn build(&self, app: &mut App) {
        app.insert_resource(CurrentVisibilityLayers(EnvironmentVisibility::Layer1));
        // app.add_systems(Startup, setup_map);
        // app.add_systems(Startup, setup_map_placeble);
    }
}

// 用于存储全局选择的可见性图层的 Resource
#[derive(Resource, Debug)]
pub struct CurrentVisibilityLayers(pub EnvironmentVisibility);

// 用于标记每个地图网格实体所属图层的 Component
#[derive(Component, Debug)]
pub struct MapMeshLayer(pub EnvironmentVisibility);

// fn setup_map(
//     mut commands: Commands,
//     res_wad: Res<WadRes>,
//     mut res_mesh: ResMut<Assets<Mesh>>,
//     mut res_image: ResMut<Assets<Image>>,
//     mut res_materials: ResMut<Assets<StandardMaterial>>,
// ) {
//     let start_time = std::time::Instant::now();

//     for map_mesh in res_wad.loader.mapgeo.meshes.iter() {
//         if map_mesh.layer_transition_behavior != LayerTransitionBehavior::Unaffected {
//             continue;
//         }

//         let bevy_meshes = process_map_geo_mesh(
//             &res_wad.loader.materials_bin.0,
//             &res_wad.loader.mapgeo,
//             map_mesh,
//             &res_wad.loader,
//         );

//         for (mesh, material_image) in bevy_meshes {
//             let _format = material_image.texture_descriptor.format.clone();

//             commands.spawn((
//                 Mesh3d(res_mesh.add(mesh)),
//                 MeshMaterial3d(res_materials.add(StandardMaterial {
//                     base_color_texture: Some(res_image.add(material_image)),
//                     unlit: true,
//                     alpha_mode: AlphaMode::Mask(0.3),
//                     ..default()
//                 })),
//                 Visibility::Visible,
//                 MapMeshLayer(map_mesh.environment_visibility),
//                 Map,
//             ));
//         }
//     }

//     system_info!("setup_map", "Map loaded in {:?}", start_time.elapsed());
// }

pub fn draw_attack(
    mut gizmos: Gizmos,
    q_attack: Query<(&Transform, &AttackState)>,
    q_movement_destination: Query<(&Transform, &MovementDestination)>,
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

    for (transform, movement_destination) in q_movement_destination.iter() {
        let destination = movement_destination.0;

        gizmos.line(
            transform.translation,
            transform
                .translation
                .clone()
                .with_x(destination.x)
                .with_z(destination.y),
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

// fn setup_map_placeble(
//     res_wad: Res<WadRes>,
//     mut commands: Commands,
//     mut character_cache: ResMut<crate::render::CharacterResourceCache>,
//     mut res_animation_clips: ResMut<Assets<AnimationClip>>,
//     mut res_animation_graphs: ResMut<Assets<AnimationGraph>>,
//     mut res_image: ResMut<Assets<Image>>,
//     mut res_materials: ResMut<Assets<StandardMaterial>>,
//     mut res_meshes: ResMut<Assets<Mesh>>,
//     mut res_skinned_mesh_inverse_bindposes: ResMut<Assets<SkinnedMeshInverseBindposes>>,
// ) {
//     let start_time = std::time::Instant::now();
//     let bin = &res_wad.loader.materials_bin.0;

//     bin.entries
//         .iter()
//         .filter(|v| v.ctype.hash == LeagueLoader::hash_bin("MapPlaceableContainer"))
//         .filter_map(|v| v.getv::<BinMap>(LeagueLoader::hash_bin("items").into()))
//         .filter_map(|v| v.downcast::<BinHash, BinStruct>())
//         .flatten()
//         .for_each(|v| match v.1.ctype.hash {
//             0x1e1cce2 => {
//                 let mut character_map_record = LeagueBinMaybeCharacterMapRecord::from(&v.1);

//                 neg_z(&mut character_map_record.transform);

//                 crate::render::spawn_character_cached(
//                     &mut commands,
//                     &mut character_cache,
//                     &mut res_animation_clips,
//                     &mut res_animation_graphs,
//                     &mut res_image,
//                     &mut res_materials,
//                     &mut res_meshes,
//                     &mut res_skinned_mesh_inverse_bindposes,
//                     &res_wad.loader,
//                     character_map_record.transform,
//                     &character_map_record.definition.skin,
//                 );
//             }
//             0x71d0eabd => {
//                 commands.spawn(get_barrack_by_bin(&bin, &v.1));
//             }
//             _ => {}
//         });

//     system_info!(
//         "setup_map_placeble",
//         "map objects loaded in {:?}",
//         start_time.elapsed()
//     );
// }
