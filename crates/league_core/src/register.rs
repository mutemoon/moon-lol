use std::{any::type_name, marker::PhantomData};

use bevy::{asset::LoadContext, platform::collections::HashMap, prelude::*};
use serde::de::DeserializeOwned;

use league_property::{from_entry, EntryData};
use league_utils::hash_bin;

#[derive(Resource, Default)]
pub struct AssetLoaderRegistry {
    pub loaders: HashMap<u32, (String, Box<dyn DynamicAssetLoader>)>,
}

impl AssetLoaderRegistry {
    /// 注册类型的辅助函数
    pub fn register<T: Asset + DeserializeOwned + TypePath + Send + Sync + 'static>(&mut self) {
        let type_name = T::short_type_path();

        let hash = if type_name.starts_with("Unk0x") {
            u32::from_str_radix(&type_name[5..], 16).unwrap()
        } else {
            hash_bin(&type_name)
        };

        self.loaders.insert(
            hash,
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
