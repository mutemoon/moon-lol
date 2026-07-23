use bevy::asset::{AssetLoader, LoadContext};
use bevy::prelude::Mesh;
use bevy::reflect::TypePath;
use league_file::mesh_static::LeagueMeshStatic;
use league_to_lol::mesh_static::mesh_static_to_bevy_mesh;

use crate::error::Error;

#[derive(Default, TypePath)]
pub struct ScbMeshLoader;

impl AssetLoader for ScbMeshLoader {
    type Asset = Mesh;

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

        let (_, league_mesh) = LeagueMeshStatic::parse(&buf)
            .map_err(|e| Error::Parse(format!("Failed to parse scb mesh: {:?}", e)))?;

        let bevy_mesh = mesh_static_to_bevy_mesh(league_mesh);
        Ok(bevy_mesh)
    }

    fn extensions(&self) -> &[&str] {
        &["scb"]
    }
}
