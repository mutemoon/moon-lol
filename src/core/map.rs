use std::f32;

use bevy::{math::bounding::Aabb3d, prelude::*};

use league_file::LeagueMapGeoMesh;
use lol_config::ConfigMap;

use crate::{spawn_geometry_object, Action, CommandAction, CommandSkinSpawn, Controller};

pub const MAP_WIDTH: f32 = 14400.0;
pub const MAP_HEIGHT: f32 = 14765.0;

pub const MAP_OFFSET_X: f32 = 300.0;
pub const MAP_OFFSET_Y: f32 = 520.0;

#[derive(Component)]
pub struct Map;

#[derive(Component)]
pub struct MapGeometry {
    pub bounding_box: Aabb3d,
    pub config: LeagueMapGeoMesh,
}

#[derive(Default)]
pub struct PluginMap;

impl Plugin for PluginMap {
    fn build(&self, app: &mut App) {
        app.add_plugins(MeshPickingPlugin);
        app.add_systems(Startup, setup);
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, configs: Res<ConfigMap>) {
    let geo_entity = spawn_geometry_objects_from_configs(&mut commands, &asset_server, &configs);

    commands
        .entity(geo_entity)
        .insert(Map)
        .observe(on_click_map);

    let environment_entities = spawn_environment_objects_from_configs(&mut commands, &configs);

    for entity in environment_entities {
        commands.entity(entity).insert(Pickable::IGNORE);
    }
}

pub fn spawn_environment_objects_from_configs(
    commands: &mut Commands,
    configs: &ConfigMap,
) -> Vec<Entity> {
    let mut entities = Vec::new();

    for (_, environment_object) in &configs.environment_objects {
        let entity = commands
            .spawn(Transform::from_matrix(environment_object.transform))
            .id();
        commands.trigger_targets(
            CommandSkinSpawn {
                skin_path: environment_object.definition.skin.clone(),
            },
            entity,
        );
        entities.push(entity);
    }

    entities
}

pub fn spawn_geometry_objects_from_configs(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    configs: &ConfigMap,
) -> Entity {
    let geo_entity = commands
        .spawn((Transform::default(), Visibility::default()))
        .id();

    for config_geo_object in &configs.geometry_objects {
        let entity = spawn_geometry_object(commands, asset_server, config_geo_object);

        commands.entity(geo_entity).add_child(entity);
    }

    geo_entity
}

pub fn on_click_map(
    click: Trigger<Pointer<Pressed>>,
    mut commands: Commands,
    q_move: Query<Entity, With<Controller>>,
    // q_map_geo: Query<&MapGeometry>,
) {
    let Some(position) = click.hit.position else {
        return;
    };
    let targets = q_move.iter().collect::<Vec<Entity>>();

    // let map_geo_entity = click.target;
    // if let Ok(map_geo) = q_map_geo.get(map_geo_entity) {
    //     println!("map_geo: {:?}", map_geo.config);
    // }

    commands.trigger_targets(
        CommandAction {
            action: Action::Move(position.xz()),
        },
        targets,
    );
}
