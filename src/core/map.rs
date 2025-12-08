use std::{collections::HashMap, f32};

use bevy::{math::bounding::Aabb3d, prelude::*};

use league_core::{
    EnvironmentVisibility, MapContainer, MapPlaceableContainer, MissileSpecificationBehaviors,
    StaticMaterialDef,
};
use league_file::{LeagueMapGeo, LeagueMapGeoMesh};
use league_to_lol::{parse_vertex_data, submesh_to_intermediate};
use league_utils::{get_asset_id_by_hash, get_asset_id_by_path};
use lol_core::{Lane, Team};

use crate::{get_standard, Action, CommandAction, CommandCharacterSpawn, Controller, Turret};

pub const MAP_WIDTH: f32 = 14400.0;
pub const MAP_HEIGHT: f32 = 14765.0;

pub const MAP_OFFSET_X: f32 = 300.0;
pub const MAP_OFFSET_Y: f32 = 520.0;

#[derive(Default)]
pub struct PluginMap;

impl Plugin for PluginMap {
    fn build(&self, app: &mut App) {
        app.add_plugins(MeshPickingPlugin);
        app.add_systems(Startup, startup_spawn_map_character);
        app.add_systems(Startup, startup_spawn_map_geometry);
    }
}

#[derive(Component)]
pub struct Map;

#[derive(Component)]
pub struct MapGeometry {
    pub bounding_box: Aabb3d,
    pub config: LeagueMapGeoMesh,
}

#[derive(Resource)]
pub struct MapName(pub String);

impl Default for MapName {
    fn default() -> Self {
        Self("Maps/MapGeometry/Map11/Base_SRX".to_string())
    }
}

#[derive(Resource)]
pub struct MinionPath(pub HashMap<Lane, Vec<Vec2>>);

fn startup_spawn_map_character(
    mut commands: Commands,
    map_name: Res<MapName>,
    res_assets_map_container: Res<Assets<MapContainer>>,
    res_assets_map_placeable_container: Res<Assets<MapPlaceableContainer>>,
) {
    let map_container = res_assets_map_container
        .get(get_asset_id_by_path(&map_name.0))
        .unwrap();

    for (_, &link) in &map_container.chunks {
        let Some(map_placeable_container) =
            res_assets_map_placeable_container.get(get_asset_id_by_hash(link))
        else {
            continue;
        };

        for (_, value) in &map_placeable_container.items {
            match value {
                MissileSpecificationBehaviors::Unk0xad65d8c4(unk0xad65d8c4) => {
                    let transform = Transform::from_matrix(unk0xad65d8c4.transform);
                    let entity = commands
                        .spawn((
                            transform,
                            Team::from(unk0xad65d8c4.definition.team),
                            Pickable::IGNORE,
                        ))
                        .id();

                    if matches!(unk0xad65d8c4.definition.r#type, Some(0)) {
                        commands.entity(entity).insert(Turret);
                    }

                    commands.trigger(CommandCharacterSpawn {
                        entity,
                        character_record_key: get_asset_id_by_path(
                            &unk0xad65d8c4.definition.character_record,
                        ),
                        skin_key: get_asset_id_by_path(&unk0xad65d8c4.definition.skin),
                    });
                }
                _ => {}
            }
        }
    }
}

fn startup_spawn_map_geometry(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    map_name: Res<MapName>,
    res_assets_map_geo: Res<Assets<LeagueMapGeo>>,
    res_assets_static_material_def: Res<Assets<StaticMaterialDef>>,
    mut res_assets_mesh: ResMut<Assets<Mesh>>,
    mut res_assets_standard_material: ResMut<Assets<StandardMaterial>>,
) {
    let geo_entity = commands
        .spawn((Transform::default(), Visibility::default(), Map))
        .id();

    let map_geo = res_assets_map_geo
        .get(get_asset_id_by_path(&map_name.0))
        .unwrap();

    for map_mesh in map_geo.meshes.iter() {
        // if map_mesh.layer_transition_behavior != LayerTransitionBehavior::Unaffected {
        //     continue;
        // }

        if !map_mesh
            .environment_visibility
            .contains(EnvironmentVisibility::Layer1)
        {
            continue;
        }

        let (all_positions, all_normals, all_uvs) = parse_vertex_data(&map_geo, map_mesh);

        for submesh in map_mesh.submeshes.iter() {
            let intermediate_meshes = submesh_to_intermediate(
                &submesh,
                &map_geo,
                map_mesh,
                &all_positions,
                &all_normals,
                &all_uvs,
            );

            let static_material_def = res_assets_static_material_def
                .get(get_asset_id_by_path(&submesh.material_name.text))
                .unwrap();

            let base_color_texture = static_material_def.sampler_values.as_ref().and_then(|v| {
                v.into_iter().find_map(|sampler_item| {
                    let texture_name = &sampler_item.texture_name;
                    if texture_name == "DiffuseTexture" || texture_name == "Diffuse_Texture" {
                        sampler_item
                            .texture_path
                            .as_ref()
                            .map(|path| format!("{}#srgb", path))
                    } else {
                        None
                    }
                })
            });

            let mesh_handle = res_assets_mesh.add(intermediate_meshes);

            let material_handle = get_standard(
                &mut res_assets_standard_material,
                &asset_server,
                base_color_texture,
            );

            commands
                .spawn((
                    Mesh3d(mesh_handle),
                    MeshMaterial3d(material_handle),
                    MapGeometry {
                        bounding_box: Aabb3d {
                            min: map_mesh.bounding_box.min.into(),
                            max: map_mesh.bounding_box.max.into(),
                        },
                        config: map_mesh.clone(),
                    },
                    ChildOf(geo_entity),
                ))
                .observe(on_click_map);
        }
    }
}

pub fn on_click_map(
    click: On<Pointer<Press>>,
    mut commands: Commands,
    q_move: Query<Entity, With<Controller>>,
    // q_map_geo: Query<&MapGeometry>,
) {
    let Some(position) = click.hit.position else {
        return;
    };
    let targets = q_move.iter().collect::<Vec<Entity>>();

    // let map_geo_entity = click.entity;
    // if let Ok(map_geo) = q_map_geo.get(map_geo_entity) {
    //     println!("map_geo: {:?}", map_geo.config);
    // } else {
    //     println!("map_geo_entity: {:?}", map_geo_entity);
    // }

    for entity in targets {
        commands.trigger(CommandAction {
            entity,
            action: Action::Move(position.xz()),
        });
    }
}
