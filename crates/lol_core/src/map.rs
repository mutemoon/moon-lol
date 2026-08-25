use std::collections::BTreeMap;
use std::f32;

use bevy::math::bounding::Aabb3d;
use bevy::prelude::*;
use bevy::world_serialization::WorldInstanceReady;
use lol_base::character::ConfigCharacterRecord;
use lol_base::map::MapPaths;

use crate::game::{WaitCharacterReady, WaitSceneReady};
use crate::lane::Lane;

pub const MAP_WIDTH: f32 = 14400.0;
pub const MAP_HEIGHT: f32 = 14765.0;

pub const MAP_OFFSET_X: f32 = 300.0;
pub const MAP_OFFSET_Y: f32 = 520.0;

#[derive(Default)]
pub struct PluginMap;

impl Plugin for PluginMap {
    fn build(&self, app: &mut App) {
        app.register_type::<MapRoot>();
        app.init_resource::<MapPaths>();
        app.init_resource::<MinionPath>();

        app.add_systems(Startup, startup_load_map_geometry);
    }
}

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
pub struct MapRoot;

#[derive(Component)]
pub struct MapGeometry {
    pub bounding_box: Aabb3d,
}

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct MinionPath(pub BTreeMap<Lane, Vec<Vec2>>);

#[derive(Resource)]
pub struct DynamicWorldHandle(pub Handle<DynamicWorld>);

fn startup_load_map_geometry(
    mut commands: Commands,
    res_map_paths: Res<MapPaths>,
    res_asset_server: Res<AssetServer>,
) {
    commands
        .spawn((
            MapRoot,
            WaitSceneReady,
            DynamicWorldRoot(res_asset_server.load(res_map_paths.scene_ron())),
        ))
        .observe(
            |trigger: On<WorldInstanceReady>,
             mut commands: Commands,
             q_children: Query<&Children, With<MapRoot>>,
             q_character_records: Query<&ConfigCharacterRecord>| {
                commands
                    .entity(trigger.event_target())
                    .remove::<WaitSceneReady>();
                info!("地图加载完成");

                if let Ok(children) = q_children.get(trigger.event_target()) {
                    info!("儿子组件个数: {}", children.len());
                    for child in children.iter() {
                        if let Ok(record) = q_character_records.get(child) {
                            debug!(
                                "儿子实体 {:?} 的 character_record handle: {:?}",
                                child, record.character_record
                            );
                            commands.entity(child).insert(WaitCharacterReady);
                        }
                    }
                }
            },
        );
}
