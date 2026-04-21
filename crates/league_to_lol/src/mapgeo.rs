use league_core::mapgeo::EnvironmentVisibility;
use league_file::mapgeo::LeagueMapGeo;
use lol_base::mapgeo::ConfigMapGeo;

use crate::sub_mesh::{parse_vertex_data, submesh_to_intermediate};
use crate::utils::Error;

pub fn parse_mapgeo(buf: &[u8]) -> Result<ConfigMapGeo, Error> {
    let (_, league_mapgeo) =
        LeagueMapGeo::parse(&buf).map_err(|e| Error::Parse("parse mapgeo error".to_string()))?;

    let mut submeshes = Vec::new();

    for (_i, map_mesh) in league_mapgeo.meshes.iter().enumerate() {
        // if map_mesh.layer_transition_behavior != LayerTransitionBehavior::Unaffected {
        //     continue;
        // }

        if !map_mesh
            .environment_visibility
            .contains(EnvironmentVisibility::Layer1)
        {
            continue;
        }

        let (all_positions, all_normals, all_uvs) = parse_vertex_data(&league_mapgeo, map_mesh);

        for (_j, submesh) in map_mesh.submeshes.iter().enumerate() {
            let _intermediate_meshes = submesh_to_intermediate(
                &submesh,
                &league_mapgeo,
                map_mesh,
                &all_positions,
                &all_normals,
                &all_uvs,
            );

            // submeshes.push((
            //     load_context.add_labeled_asset(format!("{i}-{j}"), intermediate_meshes.into()),
            //     submesh.material_name.text.clone(),
            //     Aabb3d {
            //         min: map_mesh.bounding_box.min.into(),
            //         max: map_mesh.bounding_box.max.into(),
            //     },
            // ));
        }
    }

    Ok(ConfigMapGeo { submeshes })
}
