use bevy::asset::{AssetLoader, LoadContext};
use bevy::reflect::TypePath;
use lol_base::grid::ConfigNavigationGrid;

use super::error::Error;

#[derive(Default, TypePath)]
pub struct NavGridLoader;

impl AssetLoader for NavGridLoader {
    type Asset = ConfigNavigationGrid;

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

        let nav_grid: ConfigNavigationGrid =
            bincode::deserialize(&buf).map_err(|e| Error::Parse(e.to_string()))?;

        Ok(nav_grid)
    }

    fn extensions(&self) -> &[&str] {
        &["nav_grid"]
    }
}
