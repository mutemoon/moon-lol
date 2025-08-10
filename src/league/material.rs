use bevy::{
    asset::{AssetLoader, LoadContext},
    pbr::StandardMaterial,
    render::alpha::AlphaMode,
    scene::ron::de::from_bytes,
    utils::default,
};

use crate::league::{LeagueLoaderError, LeagueMaterial};

#[derive(Default)]
pub struct LeagueLoaderMaterial;

impl AssetLoader for LeagueLoaderMaterial {
    type Asset = StandardMaterial;

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
        let material: LeagueMaterial = from_bytes(&buf)?;
        let image = _load_context.load(material.texture_path);
        Ok(StandardMaterial {
            base_color_texture: Some(image),
            unlit: true,
            alpha_mode: AlphaMode::Mask(0.3),
            ..default()
        })
    }
}
