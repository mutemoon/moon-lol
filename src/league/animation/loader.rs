use crate::{
    core::ConfigSkinnedMeshInverseBindposes,
    league::{AnimationData, AnimationFile, LeagueLoaderError},
};
use bevy::{
    animation::AnimationClip,
    asset::{AssetLoader, LoadContext},
    render::mesh::skinning::SkinnedMeshInverseBindposes,
};
use binrw::BinRead;
use std::io::Cursor;

#[derive(Default)]
pub struct LeagueLoaderAnimationClip;

impl AssetLoader for LeagueLoaderAnimationClip {
    type Asset = AnimationClip;

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
        let animation = AnimationFile::read(&mut reader)?;

        Ok(AnimationData::from(animation).into())
    }
}

#[derive(Default)]
pub struct LeagueLoaderSkinnedMeshInverseBindposes;

impl AssetLoader for LeagueLoaderSkinnedMeshInverseBindposes {
    type Asset = SkinnedMeshInverseBindposes;

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
        let config: ConfigSkinnedMeshInverseBindposes = bincode::deserialize(&buf)?;
        Ok(SkinnedMeshInverseBindposes::from(config.inverse_bindposes))
    }
}
