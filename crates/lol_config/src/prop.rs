use std::collections::HashMap;
use std::marker::PhantomData;

use bevy::asset::uuid::Uuid;
use bevy::asset::{Asset, AssetId, Assets, Handle, UntypedHandle};
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

impl<T: Asset> From<HashKey<T>> for AssetId<T> {
    fn from(value: HashKey<T>) -> Self {
        AssetId::Uuid {
            uuid: Uuid::from_u128(value.0 .0 as u128),
        }
    }
}

impl<T: Asset> From<AssetId<T>> for HashKey<T> {
    fn from(value: AssetId<T>) -> Self {
        match value {
            AssetId::Uuid { uuid } => HashKey((uuid.as_u128() as u32, PhantomData)),
            _ => panic!("AssetId is not Uuid"),
        }
    }
}

impl<T: Asset> From<HashKey<T>> for Handle<T> {
    fn from(value: HashKey<T>) -> Self {
        Handle::Uuid(Uuid::from_u128(value.0 .0 as u128), PhantomData)
    }
}

impl<T: Asset> From<HashKey<T>> for UntypedHandle {
    fn from(value: HashKey<T>) -> Self {
        Handle::from(value).untyped()
    }
}

pub trait LoadHashKeyTrait<T: Asset> {
    fn load_hash(&self, hash: impl Into<HashKey<T>>) -> Option<&T>;
}

impl<T: Asset> LoadHashKeyTrait<T> for Assets<T> {
    fn load_hash(&self, hash: impl Into<HashKey<T>>) -> Option<&T> {
        self.get(AssetId::from(hash.into()))
    }
}

#[derive(Resource, Asset, TypePath, Default)]
pub struct LeagueProperties(
    pub HashMap<u32, HashMap<u32, UntypedHandle>>,
    pub Vec<String>,
);

impl LeagueProperties {
    pub fn get<'a, T: Asset>(
        &self,
        res_assets: &'a Assets<T>,
        hash: impl Into<HashKey<T>>,
    ) -> Option<&'a T> {
        // let type_name = T::short_type_path();
        // let type_hash = type_name_to_hash(type_name);
        // let store = self.0.get(&type_hash)?;
        // let untyped_handle = store.get(&hash.into().0 .0)?;
        // let handle = untyped_handle.clone().typed::<T>();
        // res_assets.get(&handle)

        res_assets.get(AssetId::from(hash.into()))
    }

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
            .insert(hash.into().0 .0, res_assets.add(asset).untyped());
    }
}
