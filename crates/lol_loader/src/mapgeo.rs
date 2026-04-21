use bevy::asset::{AssetLoader, LoadContext};
use bevy::reflect::TypePath;
use lol_base::mapgeo::ConfigMapGeo;

use super::error::Error;

#[derive(Default, TypePath)]
pub struct LeagueLoaderMapgeo;

impl AssetLoader for LeagueLoaderMapgeo {
    type Asset = ConfigMapGeo;

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
        let league_mapgeo = league_to_lol::mapgeo::parse_mapgeo(&buf).unwrap();
        Ok(league_mapgeo)
    }

    fn extensions(&self) -> &[&str] {
        &["mapgeo"]
    }
}
