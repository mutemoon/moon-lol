use bevy::asset::{AssetLoader, LoadContext};
use bevy::reflect::TypePath;
use lol_base::barrack::ConfigBarracks;

use super::error::Error;

#[derive(Default, TypePath)]
pub struct BarracksLoader;

impl AssetLoader for BarracksLoader {
    type Asset = ConfigBarracks;

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

        let barrack: ConfigBarracks =
            ron::de::from_bytes(&buf).map_err(|e| Error::Parse(e.to_string()))?;

        Ok(barrack)
    }

    fn extensions(&self) -> &[&str] {
        &["barracks", "ron"]
    }
}
