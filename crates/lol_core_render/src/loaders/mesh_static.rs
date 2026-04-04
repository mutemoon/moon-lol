use bevy::asset::{AssetLoader, LoadContext};
use bevy::prelude::Mesh;
use league_file::mesh_static::LeagueMeshStatic;
use league_to_lol::mesh_static::mesh_static_to_bevy_mesh;

use crate::error::Error;

#[derive(Default)]
pub struct LeagueLoaderMeshStatic;

impl AssetLoader for LeagueLoaderMeshStatic {
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
        let (_, mesh) = LeagueMeshStatic::parse(&buf).map_err(|e| Error::Parse(e.to_string()))?;
        Ok(mesh_static_to_bevy_mesh(mesh))
    }

    fn extensions(&self) -> &[&str] {
        &["scb"]
    }
}
