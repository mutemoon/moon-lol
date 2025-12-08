use std::io::{Cursor, Read};

use binrw::{io::NoSeek, BinRead};
use league_file::LeagueMapGeo;
use league_property::PropFile;
use league_utils::hash_wad;

use crate::{Error, LeagueWadLoader};

pub struct LeagueWadMapLoader {
    pub wad_loader: LeagueWadLoader,
    pub map_geo: LeagueMapGeo,
    pub materials_bin: PropFile,
}

impl LeagueWadMapLoader {
    pub fn from_loader(root_dir: &str, map: &str) -> Result<LeagueWadMapLoader, Error> {
        let wad_loader = LeagueWadLoader::from_relative_path(
            root_dir,
            "DATA/FINAL/Maps/Shipping/Map11.wad.client",
        )?;

        let map_geo_path = format!("data/maps/mapgeometry/map11/{}.mapgeo", map);

        let entry = wad_loader.wad.get_entry(hash_wad(&map_geo_path))?;

        let reader = wad_loader.get_wad_zstd_entry_reader(&entry)?;

        let map_geo = LeagueMapGeo::read(&mut NoSeek::new(reader))?;

        let map_materials_bin_path = format!("data/maps/mapgeometry/map11/{}.materials.bin", map);

        let entry = wad_loader
            .wad
            .get_entry(hash_wad(&map_materials_bin_path))?;

        let mut reader = wad_loader.get_wad_zstd_entry_reader(&entry)?;

        let mut data = Vec::with_capacity(entry.target_size as usize);

        reader.read_to_end(&mut data)?;

        let materials_bin = PropFile::read(&mut Cursor::new(data))?;

        Ok(LeagueWadMapLoader {
            wad_loader,
            map_geo,
            materials_bin,
        })
    }
}
