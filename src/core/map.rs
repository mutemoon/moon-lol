use std::collections::HashMap;
use std::f32;
use std::ops::Deref;

use bevy::math::bounding::Aabb3d;
use bevy::prelude::*;
use league_core::{EnumMap, MapContainer, MapPlaceableContainer, StaticMaterialDef};
use lol_config::{ConfigMapGeo, HashKey, LoadHashKeyTrait};
use lol_core::{Lane, Team};

use crate::{
    get_standard, Action, CommandAction, CommandCharacterSpawn, CommandLoadPropBin, Controller,
    Loading, Turret,
};

pub const MAP_WIDTH: f32 = 14400.0;
pub const MAP_HEIGHT: f32 = 14765.0;

pub const MAP_OFFSET_X: f32 = 300.0;
pub const MAP_OFFSET_Y: f32 = 520.0;

#[derive(Default)]
pub struct PluginMap;

impl Plugin for PluginMap {
    fn build(&self, app: &mut App) {
        app.add_plugins(MeshPickingPlugin);

        app.init_resource::<MapName>();
        app.init_resource::<MinionPath>();

        app.init_state::<MapState>();

        app.add_systems(Startup, startup_load_map_geometry);
        app.add_systems(
            Update,
            (
                update_spawn_map_character.run_if(in_state(MapState::Loading)),
                update_spawn_map_geometry.run_if(
                    resource_exists::<Loading<Handle<ConfigMapGeo>>>
                        .and(in_state(MapState::Loaded)),
                ),
            ),
        );
        // app.add_systems(Startup, startup_spawn_map_geometry);
    }
}

#[derive(States, Default, Debug, Hash, Eq, Clone, PartialEq)]
pub enum MapState {
    #[default]
    Loading,
    Loaded,
}

#[derive(Component)]
pub struct Map;

#[derive(Component)]
pub struct MapGeometry {
    pub bounding_box: Aabb3d,
}

#[derive(Resource)]
pub struct MapName(pub String);

impl Default for MapName {
    fn default() -> Self {
        Self("Maps/MapGeometry/Map11/Base_SRX".to_string())
    }
}

#[derive(Resource, Default)]
pub struct MinionPath(pub HashMap<Lane, Vec<Vec2>>);

fn startup_load_map_geometry(
    mut commands: Commands,
    res_asset_server: Res<AssetServer>,
    res_map_name: Res<MapName>,
) {
    let paths = vec![
        "data/maps/mapgeometry/map11/base_srx.materials.bin".to_string(),
        "data/maps/shipping/map11/map11.bin".to_string(),
    ];

    commands.trigger(CommandLoadPropBin { paths });

    commands.insert_resource(Loading::new(
        res_asset_server.load::<ConfigMapGeo>(format!("data/{}.mapgeo", &res_map_name.0)),
    ));
}

fn update_spawn_map_character(
    mut commands: Commands,
    map_name: Res<MapName>,
    res_assets_map_container: Res<Assets<MapContainer>>,
    res_assets_map_placeable_container: Res<Assets<MapPlaceableContainer>>,
) {
    let Some(map_container) = res_assets_map_container.get(HashKey::from(&map_name.0)) else {
        return;
    };

    for (_, &link) in &map_container.chunks {
        let Some(map_placeable_container) = res_assets_map_placeable_container.load_hash(link)
        else {
            continue;
        };

        for (_, value) in map_placeable_container.items.as_ref().unwrap() {
            match value {
                EnumMap::Unk0xad65d8c4(unk0xad65d8c4) => {
                    let transform = Transform::from_matrix(unk0xad65d8c4.transform.unwrap());
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
                        character_record: (&unk0xad65d8c4.definition.character_record).into(),
                        skin: (&unk0xad65d8c4.definition.skin).into(),
                    });
                }
                _ => {}
            }
        }
    }

    commands.set_state(MapState::Loaded);
}

fn update_spawn_map_geometry(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut res_assets_standard_material: ResMut<Assets<StandardMaterial>>,
    res_assets_map_geo: Res<Assets<ConfigMapGeo>>,
    res_assets_static_material_def: Res<Assets<StaticMaterialDef>>,
    res_loading_map_geo: Res<Loading<Handle<ConfigMapGeo>>>,
) {
    let Some(config_map_geo) = res_assets_map_geo.get(res_loading_map_geo.deref().deref()) else {
        return;
    };

    commands.remove_resource::<Loading<Handle<ConfigMapGeo>>>();

    let geo_entity = commands
        .spawn((Transform::default(), Visibility::default(), Map))
        .id();

    println!("地图网格数量: {:?}", config_map_geo.submeshes.len());

    for (mesh_handle, mat_name, bounding_box) in &config_map_geo.submeshes {
        let static_material_def = res_assets_static_material_def.load_hash(mat_name).unwrap();

        let base_color_texture = static_material_def.sampler_values.as_ref().and_then(|v| {
            v.into_iter().find_map(|sampler_item| {
                let texture_name = &sampler_item.texture_name;
                if texture_name == "DiffuseTexture" || texture_name == "Diffuse_Texture" {
                    sampler_item.texture_path.as_ref()
                } else {
                    None
                }
            })
        });

        let material_handle = get_standard(
            &mut res_assets_standard_material,
            &asset_server,
            base_color_texture.cloned(),
        );

        commands
            .spawn((
                Mesh3d(mesh_handle.clone()),
                MeshMaterial3d(material_handle),
                MapGeometry {
                    bounding_box: bounding_box.clone(),
                },
                ChildOf(geo_entity),
            ))
            .observe(on_click_map);
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
