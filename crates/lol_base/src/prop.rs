use std::collections::HashMap;

use bevy::asset::{Asset, Assets, UntypedHandle};
use bevy::ecs::resource::Resource;
use bevy::reflect::TypePath;
pub use league_utils::hash_key::{HashKey, LoadHashKeyTrait};
use league_utils::type_name_to_hash;

#[derive(Resource, Asset, TypePath, Default)]
pub struct LeagueProperties(
    pub HashMap<u32, HashMap<u32, UntypedHandle>>,
    pub Vec<String>,
);

impl LeagueProperties {
    pub fn merge(&mut self, other: &LeagueProperties) {
        for (type_hash, other_store) in &other.0 {
            self.0
                .entry(*type_hash)
                .and_modify(|store| store.extend(other_store.clone()))
                .or_insert(other_store.clone());
        }
    }

    pub fn add<'a, T: Asset>(
        &mut self,
        res_assets: &'a mut Assets<T>,
        hash: impl Into<HashKey<T>>,
        asset: T,
    ) {
        let type_name = T::short_type_path();
        let type_hash = type_name_to_hash(type_name);
        self.0
            .entry(type_hash)
            .or_default()
            .insert(hash.into().0.0, res_assets.add(asset).untyped());
    }
}
