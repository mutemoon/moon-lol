use crate::league::{AnimationData, AnimationFile, LeagueLoaderError};
use bevy::{
    animation::AnimationClip,
    asset::{AssetLoader, LoadContext},
};
use binrw::BinRead;
use std::io::Cursor;

#[derive(Default)]
pub struct LeagueLoaderAnimation;

impl AssetLoader for LeagueLoaderAnimation {
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
