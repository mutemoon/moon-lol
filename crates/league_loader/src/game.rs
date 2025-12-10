use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use league_property::PropFile;

use crate::{Error, LeagueWadEntry, LeagueWadLoader, LeagueWadLoaderTrait};

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

    pub fn get_prop_bin_by_path(&self, path: &str) -> Result<PropFile, Error> {
        for wad_loader in &self.wads {
            if let Ok(bin) = wad_loader.get_prop_bin_by_path(path) {
                return Ok(bin);
            }
        }

        Err(Error::Custom("Prop file not found"))
    }

    pub fn from_relative_path(root_dir: &str, wads: Vec<&str>) -> Self {
        let mut wad_loaders = Vec::new();
        for wad in wads {
            println!("Loading wad: {}", wad);
            wad_loaders.push(LeagueWadLoader::from_relative_path(root_dir, wad).unwrap());
        }
        Self {
            root_dir: root_dir.to_string(),
            wads: wad_loaders,
        }
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
