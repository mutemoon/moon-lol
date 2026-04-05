use bevy::asset::{AssetLoader, LoadContext};
use bevy::reflect::TypePath;
use league_file::mesh_skinned::LeagueSkinnedMesh;
use league_to_lol::skin_mesh::skinned_mesh_to_intermediate;

use crate::error::Error;
use crate::skin::LeagueSkinMesh;

#[derive(Default, TypePath)]
pub struct LeagueLoaderMesh;

impl AssetLoader for LeagueLoaderMesh {
    type Asset = LeagueSkinMesh;

    type Settings = ();

    type Error = Error;

    async fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf).await?;
        let (_, league_skinned_mesh) =
            LeagueSkinnedMesh::parse(&buf).map_err(|e| Error::Parse(e.to_string()))?;

        let mut submeshes = Vec::new();
        for (i, _) in league_skinned_mesh.ranges.iter().enumerate() {
            let mesh = skinned_mesh_to_intermediate(&league_skinned_mesh, i);
            submeshes.push(_load_context.add_labeled_asset(i.to_string(), mesh.into()));
        }

        Ok(LeagueSkinMesh { submeshes })
    }

    fn extensions(&self) -> &[&str] {
        &["skn"]
    }
}
