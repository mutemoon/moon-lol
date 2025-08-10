use crate::league::{IntermediateMesh, LeagueLoaderError};
use bevy::{
    asset::{AssetLoader, LoadContext},
    render::mesh::Mesh,
};
use binrw::BinRead;
use std::io::Cursor;

#[derive(Default)]
pub struct LeagueLoaderMesh;

impl AssetLoader for LeagueLoaderMesh {
    type Asset = Mesh;

    type Settings = ();

    type Error = LeagueLoaderError;

    async fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf).await?;
        let mut reader = Cursor::new(buf);
        let mesh = IntermediateMesh::read(&mut reader)?;

        Ok(mesh.to_bevy_mesh())
    }
}
