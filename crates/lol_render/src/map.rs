use std::ops::Deref;

use bevy::prelude::*;
use league_core::extract::StaticMaterialDef;
use lol_config::mapgeo::ConfigMapGeo;
use lol_config::prop::LoadHashKeyTrait;
use lol_core::action::{Action, CommandAction};
use lol_core::map::{MapGeometry, MapState};
use lol_core::resource::loading::Loading;

use crate::controller::Controller;
use crate::skin::mesh::get_standard;

#[derive(Default)]
pub struct PluginRenderMap;

impl Plugin for PluginRenderMap {
    fn build(&self, app: &mut App) {
        app.add_plugins(MeshPickingPlugin);

        app.add_systems(
            Update,
            (update_spawn_map_geometry.run_if(
                resource_exists::<Loading<Handle<ConfigMapGeo>>>.and(in_state(MapState::Loaded)),
            ),),
        );
    }
}

#[derive(Component)]
pub struct Map;

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

    debug!("地图网格数量: {:?}", config_map_geo.submeshes.len());

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
