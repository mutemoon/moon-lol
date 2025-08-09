mod enums;
mod mesh_creation;
mod skinned;
mod static_mesh;
mod texture_loading;
mod types;
mod vertex_parsing;

pub use enums::*;
pub use mesh_creation::*;
pub use skinned::*;
pub use static_mesh::*;
pub use texture_loading::*;
pub use types::*;
pub use vertex_parsing::*;

use crate::league::{LeagueLoader, LeagueMapGeo};
use bevy::prelude::{Image, Mesh};
use cdragon_prop::PropFile;

use self::mesh_creation::create_bevy_mesh_for_submesh;
use self::texture_loading::find_and_load_image_for_submesh;
use self::vertex_parsing::parse_vertex_data;

pub fn process_map_geo_mesh(
    map_materials: &PropFile,
    map_file: &LeagueMapGeo,
    map_mesh: &LeagueMapGeoMesh,
    league_loader: &LeagueLoader,
) -> Vec<(Mesh, Image)> {
    let (all_positions, all_normals, all_uvs) = parse_vertex_data(map_file, map_mesh);

    let index_buffer = &map_file.index_buffers[map_mesh.index_buffer_id as usize];
    let all_indices = &index_buffer.buffer;

    let result_bundles: Vec<(Mesh, Image)> = map_mesh
        .submeshes
        .iter()
        .filter_map(|submesh| {
            let start = submesh.start_index as usize;
            let end = start + submesh.submesh_index_count as usize;
            if end > all_indices.len() {
                return None;
            }
            let global_indices_slice = &all_indices[start..end];

            let bevy_mesh = create_bevy_mesh_for_submesh(
                global_indices_slice,
                &all_positions,
                &all_normals,
                &all_uvs,
            );

            let bevy_image =
                find_and_load_image_for_submesh(submesh, map_materials, league_loader).unwrap();

            Some((bevy_mesh, bevy_image))
        })
        .collect();

    result_bundles
}
