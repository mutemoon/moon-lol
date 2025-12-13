use std::any::type_name;
use std::marker::PhantomData;
use std::sync::LazyLock;

use bevy::asset::LoadContext;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use league_core::{
    AnimationGraphData, BarracksConfig, BuffData, CharacterRecord, MapContainer,
    MapPlaceableContainer, ResourceResolver, SkinCharacterDataProperties, SpellDataResource,
    SpellObject, StaticMaterialDef, UiElementEffectAnimationData, UiElementGroupButtonData,
    UiElementIconData, UiElementRegionData, Unk0xad65d8c4, VfxSystemDefinitionData,
};
use league_property::{from_entry, EntryData};
use league_utils::type_name_to_hash;
use serde::de::DeserializeOwned;

use crate::HashKey;

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

    fn load(&self, world: &mut World, hash: u32, handle: &mut UntypedHandle) -> UntypedHandle;
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

    fn load(&self, world: &mut World, hash: u32, handle: &mut UntypedHandle) -> UntypedHandle {
        let mut res_assets = world.resource_mut::<Assets<T>>();
        let asset = res_assets.remove(&handle.clone().typed()).unwrap();
        res_assets.insert(HashKey::from(hash), asset).unwrap();
        HashKey::<T>::from(hash).into()
    }
}

pub fn init_league_asset(app: &mut App) {
    app.init_asset::<SpellDataResource>();
    app.init_asset::<MapContainer>();
    app.init_asset::<StaticMaterialDef>();
    app.init_asset::<SkinCharacterDataProperties>();
    app.init_asset::<SpellObject>();
    app.init_asset::<MapPlaceableContainer>();
    app.init_asset::<Unk0xad65d8c4>();
    app.init_asset::<VfxSystemDefinitionData>();
    app.init_asset::<CharacterRecord>();
    app.init_asset::<ResourceResolver>();
    app.init_asset::<UiElementIconData>();
    app.init_asset::<UiElementRegionData>();
    app.init_asset::<BuffData>();
    app.init_asset::<BarracksConfig>();
    app.init_asset::<AnimationGraphData>();
    app.init_asset::<UiElementEffectAnimationData>();
    app.init_asset::<UiElementGroupButtonData>();
}

pub static ASSET_LOADER_REGISTRY: LazyLock<AssetLoaderRegistry> = LazyLock::new(|| {
    let mut registry = AssetLoaderRegistry::default();
    registry.register::<SpellDataResource>();
    registry.register::<MapContainer>();
    registry.register::<StaticMaterialDef>();
    registry.register::<SkinCharacterDataProperties>();
    registry.register::<SpellObject>();
    registry.register::<MapPlaceableContainer>();
    registry.register::<Unk0xad65d8c4>();
    registry.register::<VfxSystemDefinitionData>();
    registry.register::<CharacterRecord>();
    registry.register::<ResourceResolver>();
    registry.register::<UiElementIconData>();
    registry.register::<UiElementRegionData>();
    registry.register::<BuffData>();
    registry.register::<BarracksConfig>();
    registry.register::<AnimationGraphData>();
    registry.register::<UiElementEffectAnimationData>();
    registry.register::<UiElementGroupButtonData>();
    registry
});
