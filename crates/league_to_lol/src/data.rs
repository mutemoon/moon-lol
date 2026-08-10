//! 从 Prop 文件按类型泛型读取数据的 trait（原 `league_loader::game::Data`）。
//!
//! 依赖 bevy 的 `TypePath`（用类型短名算 class hash）与 `lol_base::HashKey`，
//! 仅供 bevy-bound 的提取栈使用；bevy-free 的 `league_loader` 不再提供。

use bevy::reflect::TypePath;
use league_loader::game::PropGroup;
use league_property::from_entry;
use league_property::prop::PropFile;
use league_utils::type_name_to_hash;
use lol_base::hash_key::HashKey;
use serde::de::DeserializeOwned;

pub trait Data {
    fn get_data<T: TypePath + DeserializeOwned>(&self, hash: impl Into<HashKey<T>>) -> T {
        self.get_data_option(hash).unwrap()
    }

    fn get_data_option<T: TypePath + DeserializeOwned>(
        &self,
        hash: impl Into<HashKey<T>>,
    ) -> Option<T>;

    fn get_by_class<T: TypePath + DeserializeOwned>(&self) -> Option<T>;

    fn get_all_by_class<T: TypePath + DeserializeOwned>(&self) -> Vec<T>;

    fn get_all_by_class_with_hash<T: TypePath + DeserializeOwned>(&self) -> Vec<(u32, T)>;
}

impl Data for PropGroup {
    fn get_data_option<T: TypePath + DeserializeOwned>(
        &self,
        hash: impl Into<HashKey<T>>,
    ) -> Option<T> {
        let hash = hash.into().0;
        self.prop_file
            .iter()
            .find_map(|v| v.get_data_option::<T>(hash))
    }

    /// 通过 class hash 获取数据
    fn get_by_class<T: TypePath + DeserializeOwned>(&self) -> Option<T> {
        self.prop_file.iter().find_map(|v| v.get_by_class::<T>())
    }

    /// 获取所有某类型的数据
    fn get_all_by_class<T: TypePath + DeserializeOwned>(&self) -> Vec<T> {
        self.prop_file
            .iter()
            .flat_map(|v| v.get_all_by_class::<T>())
            .collect()
    }

    fn get_all_by_class_with_hash<T: TypePath + DeserializeOwned>(&self) -> Vec<(u32, T)> {
        self.prop_file
            .iter()
            .flat_map(|v| v.get_all_by_class_with_hash::<T>())
            .collect()
    }
}

impl Data for PropFile {
    fn get_data_option<T: TypePath + DeserializeOwned>(
        &self,
        hash: impl Into<HashKey<T>>,
    ) -> Option<T> {
        self.get_entry(hash.into().0)
            .and_then(|v| from_entry::<T>(v).ok())
    }

    fn get_by_class<T: TypePath + DeserializeOwned>(&self) -> Option<T> {
        let type_name = T::short_type_path();
        let class_hash = type_name_to_hash(type_name);
        for (bin_class_hash, entry) in self.iter_class_hash_and_entry() {
            if bin_class_hash == class_hash {
                return from_entry::<T>(entry).ok();
            }
        }
        None
    }

    fn get_all_by_class<T: TypePath + DeserializeOwned>(&self) -> Vec<T> {
        let type_name = T::short_type_path();
        let class_hash = type_name_to_hash(type_name);
        self.iter_class_hash_and_entry()
            .filter(|(bin_class_hash, _)| *bin_class_hash == class_hash)
            .filter_map(|(_, entry)| from_entry::<T>(entry).ok())
            .collect()
    }

    fn get_all_by_class_with_hash<T: TypePath + DeserializeOwned>(&self) -> Vec<(u32, T)> {
        let type_name = T::short_type_path();
        let class_hash = type_name_to_hash(type_name);
        self.iter_class_hash_and_entry()
            .filter(|(bin_class_hash, _)| *bin_class_hash == class_hash)
            .filter_map(|(_, entry)| from_entry::<T>(entry).ok().map(|v| (entry.hash, v)))
            .collect()
    }
}
