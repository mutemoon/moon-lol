use bevy::asset::{AssetLoader, LoadContext};
use bevy::reflect::TypePath;
use lol_base::spell::Spell;

use crate::error::Error;

#[derive(Default, TypePath)]
pub struct LoaderSpell;

impl AssetLoader for LoaderSpell {
    type Asset = Spell;

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
        let content = String::from_utf8(buf).map_err(|e| Error::Parse(e.to_string()))?;
        let spell: Spell = ron::from_str(&content).map_err(|e| Error::Parse(e.to_string()))?;
        Ok(spell)
    }

    fn extensions(&self) -> &[&str] {
        &["ron"]
    }
}
