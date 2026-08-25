use bevy::asset::{AssetLoader, LoadContext};
use bevy::reflect::TypePath;
use lol_base::barrack::ConfigBarracks;

use crate::error::Error;

#[derive(Default, TypePath)]
pub struct ConfigBarracksLoader;

impl AssetLoader for ConfigBarracksLoader {
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

        let content = String::from_utf8(buf).map_err(|e| Error::Parse(e.to_string()))?;
        let content = content.trim_start_matches('\u{feff}');
        let barrack: ConfigBarracks =
            ron::from_str(content).map_err(|e| Error::Parse(e.to_string()))?;

        Ok(barrack)
    }

    fn extensions(&self) -> &[&str] {
        &["ron"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_barrack_loader() {
        use lol_base::barrack::*;
        let content = std::fs::read_to_string("d:/Users/admin/workspace/moon-lol/assets/maps/sr_seasonal_map/barracks/147211fb.ron").unwrap();
        let trimmed = content.trim_start_matches('\u{feff}');
        let res: Result<ConfigBarracks, _> = ron::from_str(trimmed);
        assert!(res.is_ok());
    }
}
