use std::collections::HashMap;

use bevy::asset::{AssetLoader, LoadContext};
use bevy::prelude::Shader;
use bevy::reflect::TypePath;
use league_utils::LeagueShader;

use crate::error::Error;

#[derive(bevy::asset::Asset, TypePath)]
pub struct ShaderMapAsset {
    pub entries: HashMap<LeagueShader, HashMap<u64, Shader>>,
}

#[derive(Default, TypePath)]
pub struct LeagueLoaderShaderMap;

impl AssetLoader for LeagueLoaderShaderMap {
    type Asset = ShaderMapAsset;
    type Settings = ();
    type Error = Error;

    async fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        _settings: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut ron_bytes = Vec::new();
        reader
            .read_to_end(&mut ron_bytes)
            .await
            .map_err(|e| Error::Parse(format!("Failed to read map.ron: {}", e)))?;

        #[derive(serde::Deserialize)]
        struct RawShaderMap {
            pub entries: HashMap<LeagueShader, HashMap<u64, String>>,
        }

        let raw_map: RawShaderMap = ron::de::from_bytes(&ron_bytes)
            .map_err(|e| Error::Parse(format!("Failed to deserialize map.ron: {}", e)))?;

        let mut entries = HashMap::new();

        for (shader_type, inner_map) in raw_map.entries {
            let mut shaders = HashMap::new();
            for (u64_hash, spv_relative_path) in inner_map {
                let spv_bytes = load_context
                    .read_asset_bytes(&spv_relative_path)
                    .await
                    .map_err(|e| {
                        Error::Parse(format!("Failed to read {}: {}", spv_relative_path, e))
                    })?;

                let shader = Shader::from_spirv(spv_bytes, spv_relative_path.clone());
                shaders.insert(u64_hash, shader);
            }
            entries.insert(shader_type, shaders);
        }

        Ok(ShaderMapAsset { entries })
    }

    fn extensions(&self) -> &[&str] {
        &["ron"]
    }
}
