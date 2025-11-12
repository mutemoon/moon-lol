use bevy::prelude::*;

use lol_config::ConfigGeometryObject;

use crate::MapGeometry;

pub fn spawn_geometry_object(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    config_geo_object: &ConfigGeometryObject,
) -> Entity {
    let material_handle: Handle<StandardMaterial> =
        asset_server.load(config_geo_object.material_path.clone());

    let mesh_handle = asset_server.load(config_geo_object.mesh_path.clone());

    commands
        .spawn((
            Mesh3d(mesh_handle),
            MeshMaterial3d(material_handle),
            MapGeometry {
                bounding_box: config_geo_object.bounding_box.clone(),
                config: config_geo_object.geo_mesh.clone(),
            },
        ))
        .id()
}
