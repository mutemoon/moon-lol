use bevy::asset::{Asset, Handle};
use bevy::prelude::*;
use league_utils::hash_wad;
use lol_core::utils::HashPath;
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize)]
pub struct ShaderTocSettings(pub String);

pub trait AssetServerLoadLeague {
    fn load_league<A: Asset>(&self, path: impl Into<HashPath>) -> Handle<A>;

    fn load_league_labeled<'a, A: Asset>(
        &self,
        path: impl Into<HashPath>,
        label: &str,
    ) -> Handle<A>;

    fn load_league_with_settings<'a, A: Asset>(&self, path: &str) -> Handle<A>;
}

impl AssetServerLoadLeague for AssetServer {
    fn load_league<A: Asset>(&self, path: impl Into<HashPath>) -> Handle<A> {
        let path = path.into();
        self.load(format!("data/{:x}.{}", path.hash, path.ext))
    }

    fn load_league_labeled<'a, A: Asset>(
        &self,
        path: impl Into<HashPath>,
        label: &str,
    ) -> Handle<A> {
        let path = path.into();
        self.load(format!("data/{:x}.{}#{label}", path.hash, path.ext))
    }

    fn load_league_with_settings<'a, A: Asset>(&self, path: &str) -> Handle<A> {
        let original_path = path.to_string();
        self.load_with_settings(
            format!("data/{:x}.lol", hash_wad(path)),
            move |settings: &mut ShaderTocSettings| settings.0 = original_path.clone(),
        )
    }
}
