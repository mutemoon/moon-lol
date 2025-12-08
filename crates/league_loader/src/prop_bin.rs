use std::io::Read;

use binrw::{io::NoSeek, BinRead};

use league_file::LeagueTexture;
use league_property::PropFile;
use league_utils::hash_wad;

use crate::Error;

pub trait PropBinLoader {
    fn get_wad_entry_reader_by_hash(&self, hash: u64) -> Result<Box<dyn Read + '_>, Error>;

    fn get_wad_entry_no_seek_reader_by_hash(
        &self,
        hash: u64,
    ) -> Result<NoSeek<Box<dyn Read + '_>>, Error> {
        self.get_wad_entry_reader_by_hash(hash)
            .map(|v| NoSeek::new(v))
    }

    fn get_wad_entry_no_seek_reader_by_path(
        &self,
        path: &str,
    ) -> Result<NoSeek<Box<dyn Read + '_>>, Error> {
        self.get_wad_entry_no_seek_reader_by_hash(hash_wad(path))
    }

    fn get_wad_entry_buffer_by_path(&self, path: &str) -> Result<Vec<u8>, Error> {
        let mut reader = self.get_wad_entry_no_seek_reader_by_path(path)?;
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer)?;
        Ok(buffer)
    }

    fn get_prop_bin_by_hash(&self, hash: u64) -> Result<PropFile, Error> {
        let mut reader = self.get_wad_entry_no_seek_reader_by_hash(hash)?;
        Ok(PropFile::read(&mut reader)?)
    }
    fn get_prop_bin_by_path(&self, path: &str) -> Result<PropFile, Error> {
        let mut reader = self.get_wad_entry_no_seek_reader_by_path(path)?;
        Ok(PropFile::read(&mut reader)?)
    }

    fn get_skin_bin_by_path(&self, skin: &str) -> Result<PropFile, Error> {
        let path = format!("data/{}.bin", skin);
        let skin_bin = self.get_prop_bin_by_path(&path).unwrap();
        Ok(skin_bin)
    }

    fn get_texture_by_hash(&self, hash: u64) -> Result<LeagueTexture, Error> {
        let mut reader = self.get_wad_entry_no_seek_reader_by_hash(hash)?;
        Ok(LeagueTexture::read(&mut reader)?)
    }
    fn get_texture_by_path(&self, path: &str) -> Result<LeagueTexture, Error> {
        let mut reader = self.get_wad_entry_no_seek_reader_by_path(path)?;
        Ok(LeagueTexture::read(&mut reader)?)
    }
}
