use bevy::asset::RenderAssetUsages;
use bevy::gltf::GltfLoaderSettings;
use bevy::light::CascadeShadowConfigBuilder;
use bevy::prelude::*;
use bevy_gltf_draco::GltfDracoDecoderPlugin;
use lol_core::action::{Action, CommandAction};
use lol_core::map::MapName;

use crate::controller::Controller;

#[derive(Default)]
pub struct PluginRenderMap;

impl Plugin for PluginRenderMap {
    fn build(&self, app: &mut App) {
        app.add_plugins(GltfDracoDecoderPlugin);
        app.add_plugins(MeshPickingPlugin);
        app.add_systems(Startup, setup);
        // app.insert_resource(DefaultOpaqueRendererMethod::deferred());
    }
}

#[derive(Component)]
pub struct Map;

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    res_map_name: Res<MapName>,
    mut ambient_light: ResMut<GlobalAmbientLight>,
) {
    let handle = asset_server
        .load_builder()
        .with_settings(|s: &mut GltfLoaderSettings| {
            s.validate = false;
            s.load_materials = RenderAssetUsages::RENDER_WORLD;
        })
        .load(GltfAssetLabel::Scene(0).from_asset(format!("maps/{}_mapgeo.glb", res_map_name.0)));

    commands.spawn(WorldAssetRoot(handle));

    ambient_light.brightness = 1000.0;

    commands.spawn((
        DirectionalLight {
            color: Color::WHITE,
            illuminance: 5_000.,
            shadow_maps_enabled: true,
            ..default()
        },
        CascadeShadowConfigBuilder {
            first_cascade_far_bound: 200.0,
            maximum_distance: 10000.0,
            ..default()
        }
        .build(),
        Transform::from_xyz(5.0, 10.0, -5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
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
