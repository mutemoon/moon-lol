use bevy::asset::{AssetLoader, LoadContext};
use bevy::math::bounding::Aabb3d;
use league_core::mapgeo::EnvironmentVisibility;
use league_file::mapgeo::LeagueMapGeo;
use league_to_lol::sub_mesh::{parse_vertex_data, submesh_to_intermediate};
use lol_base::mapgeo::ConfigMapGeo;

use super::error::Error;

#[derive(Default)]
pub struct LeagueLoaderMapgeo;

impl AssetLoader for LeagueLoaderMapgeo {
    type Asset = ConfigMapGeo;

    type Settings = ();

    type Error = Error;

    async fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        _settings: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf).await?;
        let (_, league_mapgeo) =
            LeagueMapGeo::parse(&buf).map_err(|e| Error::Parse(e.to_string()))?;

        let mut submeshes = Vec::new();

        for (i, map_mesh) in league_mapgeo.meshes.iter().enumerate() {
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

            for (j, submesh) in map_mesh.submeshes.iter().enumerate() {
                let intermediate_meshes = submesh_to_intermediate(
                    &submesh,
                    &league_mapgeo,
                    map_mesh,
                    &all_positions,
                    &all_normals,
                    &all_uvs,
                );

                submeshes.push((
                    load_context.add_labeled_asset(format!("{i}-{j}"), intermediate_meshes.into()),
                    submesh.material_name.text.clone(),
                    Aabb3d {
                        min: map_mesh.bounding_box.min.into(),
                        max: map_mesh.bounding_box.max.into(),
                    },
                ));
            }
        }

        Ok(ConfigMapGeo { submeshes })
    }

    fn extensions(&self) -> &[&str] {
        &["mapgeo"]
    }
}
