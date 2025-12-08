use std::{any::type_name, marker::PhantomData};

use bevy::{platform::collections::HashMap, prelude::*};
use serde::de::DeserializeOwned;

use league_core::SpellObject;

use crate::{from_entry, EntryData};

#[derive(Resource, Default)]
pub struct AssetLoaderRegistry {
    loaders: HashMap<String, Box<dyn DynamicAssetLoader>>,
}

impl AssetLoaderRegistry {
    /// 注册类型的辅助函数
    pub fn register<T: Asset + DeserializeOwned + TypePath + Send + Sync + 'static>(&mut self) {
        self.loaders.insert(
            type_name::<T>().to_string(),
            Box::new(GenericLoader::<T>(PhantomData)),
        );
    }
}

pub trait DynamicAssetLoader: Send + Sync {
    fn load_and_add(&self, world: &mut World, entry: &EntryData);
}

pub struct GenericLoader<T>(PhantomData<T>);

impl<T> DynamicAssetLoader for GenericLoader<T>
where
    T: Asset + DeserializeOwned + TypePath + Send + Sync + 'static,
{
    fn load_and_add(&self, world: &mut World, entry: &EntryData) {
        match from_entry::<T>(entry) {
            Ok(asset) => {
                if let Some(mut assets) = world.get_resource_mut::<Assets<T>>() {
                    let handle = assets.add(asset);
                    info!("已添加资产 [{}]: {:?}", type_name::<T>(), handle);
                } else {
                    error!(
                        "World 中未找到 Assets<{}>，请确保在 App 中 init_asset::<{}>()",
                        type_name::<T>(),
                        type_name::<T>()
                    );
                }
            }
            Err(e) => error!("反序列化 [{}] 失败: {}", type_name::<T>(), e),
        }
    }
}

pub fn init_league_asset(asset_loader_registry: &mut AssetLoaderRegistry) {
    asset_loader_registry.register::<SpellObject>();
}
