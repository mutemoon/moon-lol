use std::collections::HashMap;
use std::f32;

use bevy::math::bounding::Aabb3d;
use bevy::prelude::*;
use league_core::extract::{EnumMap, MapContainer, MapPlaceableContainer};
use lol_config::prop::{HashKey, LoadHashKeyTrait};

use crate::character::CommandCharacterSpawn;
use crate::entities::turret::Turret;
use crate::lane::Lane;
use crate::resource::prop_bin::{CommandLoadPropBin, PropPath};
use crate::team::Team;

pub const MAP_WIDTH: f32 = 14400.0;
pub const MAP_HEIGHT: f32 = 14765.0;

pub const MAP_OFFSET_X: f32 = 300.0;
pub const MAP_OFFSET_Y: f32 = 520.0;

#[derive(Default)]
pub struct PluginMap;

impl Plugin for PluginMap {
    fn build(&self, app: &mut App) {
        app.init_resource::<MapName>();
        app.init_resource::<MinionPath>();

        app.init_state::<MapState>();

        app.add_systems(Startup, startup_load_map_geometry);
        app.add_systems(
            Update,
            (update_spawn_map_character.run_if(in_state(MapState::Loading)),),
        );
    }
}

#[derive(States, Default, Debug, Hash, Eq, Clone, PartialEq)]
pub enum MapState {
    #[default]
    Loading,
    Loaded,
}

#[derive(Component)]
pub struct MapGeometry {
    pub bounding_box: Aabb3d,
}

#[derive(Resource)]
pub struct MapName(String);

impl Default for MapName {
    fn default() -> Self {
        Self("sr_seasonal_map".to_string())
    }
}

impl MapName {
    pub fn get_materials_path(&self) -> String {
        format!("Maps/MapGeometry/Map11/{}", &self.0)
    }
}

#[derive(Resource, Default)]
pub struct MinionPath(pub HashMap<Lane, Vec<Vec2>>);

fn startup_load_map_geometry(
    mut commands: Commands,
    res_map_name: Res<MapName>,
) {
    let paths = vec![
        format!("data/{}.materials.bin", &res_map_name.get_materials_path()),
        "data/maps/shipping/map11/map11.bin".to_string(),
    ];

    commands.trigger(CommandLoadPropBin {
        path: PropPath::Path(paths),
        label: None,
    });
}

fn update_spawn_map_character(
    mut commands: Commands,
    map_name: Res<MapName>,
    res_assets_map_container: Res<Assets<MapContainer>>,
    res_assets_map_placeable_container: Res<Assets<MapPlaceableContainer>>,
) {
    let Some(map_container) =
        res_assets_map_container.get(HashKey::from(&map_name.get_materials_path()))
    else {
        return;
    };

    for (_, &link) in &map_container.chunks {
        let Some(map_placeable_container) = res_assets_map_placeable_container.load_hash(link)
        else {
            continue;
        };

        let Some(items) = map_placeable_container.items.as_ref() else {
            continue;
        };

        for (_, value) in items {
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
