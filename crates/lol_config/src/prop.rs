use std::collections::HashMap;
use std::marker::PhantomData;

use bevy::asset::{Asset, Assets, UntypedHandle};
use bevy::ecs::resource::Resource;
use bevy::reflect::TypePath;
use league_utils::{hash_bin, type_name_to_hash};

#[derive(Debug)]
pub struct HashKey<T>((u32, PhantomData<T>));

impl<T> From<&u32> for HashKey<T> {
    fn from(value: &u32) -> Self {
        Self((*value, PhantomData))
    }
}

impl<T> From<u32> for HashKey<T> {
    fn from(value: u32) -> Self {
        Self((value, PhantomData))
    }
}

impl<T> From<&str> for HashKey<T> {
    fn from(value: &str) -> Self {
        Self((hash_bin(value), PhantomData))
    }
}

impl<T> From<&String> for HashKey<T> {
    fn from(value: &String) -> Self {
        Self((hash_bin(value), PhantomData))
    }
}

impl<T> From<String> for HashKey<T> {
    fn from(value: String) -> Self {
        Self((hash_bin(&value), PhantomData))
    }
}

impl<T> From<&HashKey<T>> for HashKey<T> {
    fn from(value: &HashKey<T>) -> Self {
        Self((value.0 .0, PhantomData))
    }
}

impl<T> Clone for HashKey<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for HashKey<T> {}

impl<T> PartialEq for HashKey<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 .0 == other.0 .0
    }
}

#[derive(Resource, Asset, TypePath, Default)]
pub struct LeagueProperties(pub HashMap<u32, HashMap<u32, UntypedHandle>>);

impl LeagueProperties {
    pub fn get<'a, T: Asset>(
        &self,
        res_assets: &'a Assets<T>,
        hash: impl Into<HashKey<T>>,
    ) -> Option<&'a T> {
        let type_name = T::short_type_path();
        let type_hash = type_name_to_hash(type_name);
        let store = self.0.get(&type_hash)?;
        let untyped_handle = store.get(&hash.into().0 .0)?;
        let handle = untyped_handle.clone().typed::<T>();
        res_assets.get(&handle)
    }

    pub fn merge(&mut self, other: &LeagueProperties) {
        for (type_hash, other_store) in &other.0 {
            self.0
                .entry(*type_hash)
                .and_modify(|store| store.extend(other_store.clone()))
                .or_insert(other_store.clone());
        }
    }

    pub fn iter<'a, T: Asset>(
        &'a self,
        res_assets: &'a Assets<T>,
    ) -> impl Iterator<Item = (HashKey<T>, &'a T)> {
        let type_name = T::short_type_path();
        let type_hash = type_name_to_hash(type_name);
        let store = self.0.get(&type_hash);

        store.into_iter().flat_map(|store| {
            store.iter().map(|(hash, handle)| {
                let handle = handle.clone().typed::<T>();
                let asset = res_assets.get(&handle).unwrap();
                (HashKey::from(hash), asset)
            })
        })
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
            .insert(hash.into().0 .0, res_assets.add(asset).untyped());
    }
}
