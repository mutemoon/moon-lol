use std::collections::HashMap;

use bevy::asset::{AssetLoader, LoadContext};
use league_property::PropFile;
use lol_config::{ASSET_LOADER_REGISTRY, LeagueProperties};

use super::error::Error;

#[derive(Default)]
pub struct LeagueLoaderProperty;

impl AssetLoader for LeagueLoaderProperty {
    type Asset = LeagueProperties;

    type Settings = ();

    type Error = Error;

    async fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        _settings: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf).await?;
        let (_, prop_bin) = PropFile::parse(&buf).map_err(|e| Error::Parse(e.to_string()))?;

        let mut handles = HashMap::new();
        for (entry_hash, entry) in prop_bin.iter_class_hash_and_entry() {
            let Some((_, loader)) = ASSET_LOADER_REGISTRY.loaders.get(&entry_hash) else {
                continue;
            };

            let handle = loader.load_and_add(load_context, entry);

            if !handles.contains_key(&entry_hash) {
                handles.insert(entry_hash, HashMap::new());
            };

            let store = handles.get_mut(&entry_hash).unwrap();

            store.insert(entry.hash, handle);
        }

        let paths = prop_bin.links.into_iter().map(|v| v.text).collect();

        Ok(LeagueProperties(handles, paths))
    }

    fn extensions(&self) -> &[&str] {
        &["bin"]
    }
}
