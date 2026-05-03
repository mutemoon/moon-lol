use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use bevy::reflect::TypePath;
use league_property::from_entry;
use league_property::prop::PropFile;
use league_utils::type_name_to_hash;
use lol_base::prop::HashKey;
use serde::de::DeserializeOwned;

use crate::Error;
use crate::prop_bin::LeagueWadLoaderTrait;
use crate::wad::LeagueWadLoader;
use crate::wad_parse::LeagueWadEntry;

pub struct LeagueLoader {
    pub root_dir: String,
    pub wads: Vec<LeagueWadLoader>,
}

impl LeagueLoader {
    fn scan_wad_files(root_dir: &Path) -> Vec<PathBuf> {
        let mut wad_files = Vec::new();
        let Ok(entries) = fs::read_dir(root_dir) else {
            return wad_files;
        };

        for entry in entries.flatten() {
            let Ok(file_type) = entry.file_type() else {
                continue;
            };

            if file_type.is_dir() {
                wad_files.extend(Self::scan_wad_files(&entry.path()));
            } else if file_type.is_file() {
                if let Some(file_name) = entry.file_name().to_str() {
                    if file_name.ends_with(".wad.client") {
                        wad_files.push(entry.path());
                    }
                }
            }
        }

        wad_files
    }

    pub fn full(root_dir: &str) -> Result<LeagueLoader, Error> {
        let root_path = Path::new(root_dir);
        let wad_files = Self::scan_wad_files(root_path);

        let wads = wad_files
            .iter()
            .filter_map(|path| {
                path.strip_prefix(root_path)
                    .ok()
                    .and_then(|rel_path| rel_path.to_str())
                    .and_then(|rel_str| LeagueWadLoader::from_relative_path(root_dir, rel_str).ok())
            })
            .collect::<Vec<_>>();

        Ok(LeagueLoader {
            root_dir: root_dir.to_string(),
            wads,
        })
    }

    pub fn get_prop_group_by_paths(&self, paths: Vec<&str>) -> Result<PropGroup, Error> {
        let mut prop_files = Vec::new();
        for path in paths {
            let bin = self.get_prop_bin_by_path(path)?; // 使用?，如果错误则传播
            // 递归获取links的PropGroup，并将其文件合并
            let linked_group = self.get_prop_group_by_paths(
                bin.links
                    .iter()
                    .map(|v| v.text.as_str())
                    .collect::<Vec<_>>(),
            )?;
            prop_files.push(bin);
            prop_files.extend(linked_group.prop_file); // 假设PropGroup有方法into_files或类似
        }
        Ok(PropGroup::new(prop_files))
    }

    pub fn from_relative_path(root_dir: &str, wads: Vec<&str>) -> Self {
        let mut wad_loaders = Vec::new();
        for wad in wads {
            wad_loaders.push(LeagueWadLoader::from_relative_path(root_dir, wad).unwrap());
        }
        Self {
            root_dir: root_dir.to_string(),
            wads: wad_loaders,
        }
    }

    /// 扫描 Champions 目录并将所有英雄 WAD 文件添加到 loader
    pub fn with_all_champions(mut self) -> Self {
        let champions_path = std::path::Path::new(&self.root_dir).join("DATA/FINAL/Champions");
        if let Ok(entries) = std::fs::read_dir(&champions_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                    if name.ends_with(".wad.client") && !name.contains('_') {
                        let rel_path = format!("DATA/FINAL/Champions/{}", name);
                        if let Ok(wad_loader) =
                            LeagueWadLoader::from_relative_path(&self.root_dir, &rel_path)
                        {
                            self.wads.push(wad_loader);
                        }
                    }
                }
            }
        }
        self
    }

    pub fn iter_wad_entries(&self) -> impl Iterator<Item = (&u64, &LeagueWadEntry)> {
        self.wads.iter().flat_map(|wad| wad.wad.entries.iter())
    }
}

impl LeagueWadLoaderTrait for LeagueLoader {
    fn get_wad_entry_reader_by_hash(&self, hash: u64) -> Result<Box<dyn Read + '_>, Error> {
        for wad in &self.wads {
            if let Ok(reader) = wad.get_wad_entry_reader_by_hash(hash) {
                return Ok(reader);
            }
        }

        Err(Error::Custom("Wad reader not found"))
    }
}

pub struct PropGroup {
    prop_file: Vec<PropFile>,
}

impl PropGroup {
    pub fn new(prop_file: Vec<PropFile>) -> Self {
        Self { prop_file }
    }
}

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
}

impl Data for PropGroup {
    fn get_data_option<T: TypePath + DeserializeOwned>(
        &self,
        hash: impl Into<HashKey<T>>,
    ) -> Option<T> {
        let hash = hash.into().0.0;
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
}

impl Data for PropFile {
    fn get_data_option<T: TypePath + DeserializeOwned>(
        &self,
        hash: impl Into<HashKey<T>>,
    ) -> Option<T> {
        self.get_entry(hash.into().0.0)
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
}
