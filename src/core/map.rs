use std::f32;

use bevy::{math::bounding::Aabb3d, prelude::*};

use league_file::LeagueMapGeoMesh;
use league_utils::neg_mat_z;
use lol_config::ConfigMap;

use crate::core::{spawn_geometry_object, spawn_skin_entity, Action, CommandAction, Controller};

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

fn setup(
    mut commands: Commands,
    mut res_animation_graph: ResMut<Assets<AnimationGraph>>,
    asset_server: Res<AssetServer>,
    configs: Res<ConfigMap>,
) {
    let geo_entity = spawn_geometry_objects_from_configs(&mut commands, &asset_server, &configs);

    commands
        .entity(geo_entity)
        .insert((Visibility::Visible, Map))
        .observe(on_click_map);

    let environment_entities = spawn_environment_objects_from_configs(
        &mut commands,
        &mut res_animation_graph,
        &asset_server,
        &configs,
    );

    for entity in environment_entities {
        commands
            .entity(entity)
            .insert((Visibility::Visible, Map, Pickable::IGNORE));
    }
}

pub fn spawn_environment_objects_from_configs(
    commands: &mut Commands,
    res_animation_graph: &mut ResMut<Assets<AnimationGraph>>,
    asset_server: &Res<AssetServer>,
    configs: &ConfigMap,
) -> Vec<Entity> {
    let mut entities = Vec::new();

    for (_, environment_object) in &configs.environment_objects {
        let entity = spawn_skin_entity(
            commands,
            res_animation_graph,
            asset_server,
            Transform::from_matrix(neg_mat_z(&environment_object.transform)),
            configs
                .skins
                .get(&environment_object.definition.skin)
                .unwrap(),
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
    let geo_entity = commands.spawn(Transform::default()).id();

    for config_geo_object in &configs.geometry_objects {
        let entity = spawn_geometry_object(commands, asset_server, config_geo_object);

        commands.entity(entity).insert(ChildOf(geo_entity));
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
