use bevy::asset::{AssetLoader, LoadContext};
use league_file::skeleton::LeagueSkeleton;

use super::error::Error;

#[derive(Default)]
pub struct LeagueLoaderSkeleton;

impl AssetLoader for LeagueLoaderSkeleton {
    type Asset = LeagueSkeleton;

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
        let (_, league_skeleton) =
            LeagueSkeleton::parse(&buf).map_err(|e| Error::Parse(e.to_string()))?;

        Ok(league_skeleton)
    }

    fn extensions(&self) -> &[&str] {
        &["skl"]
    }
}
