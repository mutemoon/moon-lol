use std::any::type_name;
use std::marker::PhantomData;

use bevy::asset::LoadContext;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use league_property::{from_entry, EntryData};
use league_utils::type_name_to_hash;
use serde::de::DeserializeOwned;

#[derive(Resource, Default)]
pub struct AssetLoaderRegistry {
    pub loaders: HashMap<u32, (String, Box<dyn DynamicAssetLoader>)>,
}

impl AssetLoaderRegistry {
    /// 注册类型的辅助函数
    pub fn register<T: Asset + DeserializeOwned + TypePath + Send + Sync + 'static>(&mut self) {
        let type_name = T::short_type_path();

        self.loaders.insert(
            type_name_to_hash(type_name),
            (
                type_name.to_string(),
                Box::new(GenericLoader::<T>(PhantomData)),
            ),
        );
    }
}

pub trait DynamicAssetLoader: Send + Sync {
    fn load_and_add(&self, load_context: &mut LoadContext<'_>, entry: &EntryData) -> UntypedHandle;
}

pub struct GenericLoader<T>(PhantomData<T>);

impl<T> DynamicAssetLoader for GenericLoader<T>
where
    T: Asset + DeserializeOwned + TypePath + Send + Sync + 'static,
{
    fn load_and_add(&self, load_context: &mut LoadContext<'_>, entry: &EntryData) -> UntypedHandle {
        match from_entry::<T>(entry) {
            Ok(asset) => load_context
                .add_labeled_asset(entry.hash.to_string(), asset)
                .untyped(),
            Err(e) => panic!("反序列化 [{}] 失败: {}", type_name::<T>(), e),
        }
    }
}
