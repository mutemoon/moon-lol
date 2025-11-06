use bevy::math::bounding::Aabb3d;
use league_core::EnvironmentVisibility;
use league_loader::LeagueWadMapLoader;
use lol_config::ConfigGeometryObject;

use crate::{
    get_bin_path, parse_vertex_data, save_struct_to_file, save_wad_entry_to_file,
    submesh_to_intermediate, Error,
};

pub async fn save_mapgeo(loader: &LeagueWadMapLoader) -> Result<Vec<ConfigGeometryObject>, Error> {
    let mut geometry_objects = Vec::new();

    for (i, map_mesh) in loader.map_geo.meshes.iter().enumerate() {
        // if map_mesh.layer_transition_behavior != LayerTransitionBehavior::Unaffected {
        //     continue;
        // }

        if !map_mesh
            .environment_visibility
            .contains(EnvironmentVisibility::Layer1)
        {
            continue;
        }

        let (all_positions, all_normals, all_uvs) = parse_vertex_data(&loader.map_geo, map_mesh);

        for (j, submesh) in map_mesh.submeshes.iter().enumerate() {
            let intermediate_meshes = submesh_to_intermediate(
                &submesh,
                &loader.map_geo,
                map_mesh,
                &all_positions,
                &all_normals,
                &all_uvs,
            );
            let material = loader
                .load_image_for_submesh(&submesh.material_name.text)
                .unwrap();

            let mesh_path = format!("mapgeo/meshes/{}_{}.mesh", i, j);
            save_struct_to_file(&mesh_path, &intermediate_meshes).await?;

            let material_path = get_bin_path(&submesh.material_name.text.clone());
            save_struct_to_file(&material_path, &material).await?;

            let texture_path = material.texture_path.clone();

            save_wad_entry_to_file(&loader.wad_loader, &texture_path).await?;

            geometry_objects.push(ConfigGeometryObject {
                mesh_path,
                material_path,
                bounding_box: Aabb3d {
                    min: map_mesh.bounding_box.min.into(),
                    max: map_mesh.bounding_box.max.into(),
                },
                geo_mesh: map_mesh.clone(),
            });
        }
    }

    Ok(geometry_objects)
}
