use std::io::Read;

use league_file::LeagueTexture;
use league_property::PropFile;
use league_utils::hash_wad;

use crate::Error;

pub trait LeagueWadLoaderTrait {
    fn get_wad_entry_reader_by_hash(&self, hash: u64) -> Result<Box<dyn Read + '_>, Error>;

    fn get_wad_entry_buffer_by_hash(&self, hash: u64) -> Result<Vec<u8>, Error> {
        let mut reader = self.get_wad_entry_reader_by_hash(hash)?;
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer)?;
        Ok(buffer)
    }

    fn get_wad_entry_buffer_by_path(&self, path: &str) -> Result<Vec<u8>, Error> {
        self.get_wad_entry_buffer_by_hash(hash_wad(path))
    }

    fn get_prop_bin_by_hash(&self, hash: u64) -> Result<PropFile, Error> {
        let buffer = self.get_wad_entry_buffer_by_hash(hash)?;
        let (_, prop) = PropFile::parse(&buffer).map_err(|_| {
            Error::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Failed to parse PROP",
            ))
        })?;
        Ok(prop)
    }

    fn get_prop_bin_by_path(&self, path: &str) -> Result<PropFile, Error> {
        self.get_prop_bin_by_hash(hash_wad(path))
    }

    fn get_skin_bin_by_path(&self, skin: &str) -> Result<PropFile, Error> {
        let path = format!("data/{}.bin", skin);
        self.get_prop_bin_by_path(&path)
    }

    fn get_texture_by_hash(&self, hash: u64) -> Result<LeagueTexture, Error> {
        let buffer = self.get_wad_entry_buffer_by_hash(hash)?;
        let (_, texture) = LeagueTexture::parse(&buffer).map_err(|_| {
            Error::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Failed to parse TEX",
            ))
        })?;
        Ok(texture)
    }

    fn get_texture_by_path(&self, path: &str) -> Result<LeagueTexture, Error> {
        self.get_texture_by_hash(hash_wad(path))
    }
}
