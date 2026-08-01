use bevy::asset::{AssetLoader, LoadContext};
use bevy::prelude::*;
use bevy::reflect::TypePath;
use lol_base::audio::ConfigAudio;

use crate::error::Error;

/// 加载 `characters/{champ}/skins/{skin}_audio.ron`（纯 serde RON，无 bevy 句柄）。
#[derive(Default, TypePath)]
pub struct LoaderConfigAudioLoader;

impl AssetLoader for LoaderConfigAudioLoader {
    type Asset = ConfigAudio;

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
        let config: ConfigAudio =
            ron::from_str(&content).map_err(|e| Error::Parse(e.to_string()))?;

        Ok(config)
    }

    fn extensions(&self) -> &[&str] {
        &[".ron"]
    }
}
