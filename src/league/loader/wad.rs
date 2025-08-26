use std::io::{self, BufReader};
use std::{
    collections::HashMap,
    fs::File,
    io::{Cursor, Read},
    sync::Arc,
};

use binrw::{args, io::NoSeek, BinRead, Endian};
use tokio::io::AsyncWriteExt;
use zstd::Decoder;

use crate::league::{
    from_entry, get_asset_writer, ArcFileReader, CharacterRecord, EntryData, LeagueLoader,
    LeagueLoaderError, LeagueTexture, LeagueWad, LeagueWadEntry, LeagueWadSubchunk, PropFile,
    SkinCharacterDataProperties, WadDataFormat,
};

pub struct LeagueWadLoader {
    pub wad: LeagueWad,
    pub file: Arc<File>,
    pub sub_chunk: Option<LeagueWadSubchunk>,
}

impl LeagueWadLoader {
    pub fn from_relative_path(
        root_dir: &str,
        wad_relative_path: &str,
    ) -> Result<LeagueWadLoader, LeagueLoaderError> {
        let wad_absolute_path = format!("{}/{}", root_dir, wad_relative_path);

        let file = Arc::new(File::open(&wad_absolute_path)?);

        let wad = LeagueWad::read(&mut ArcFileReader::new(file.clone(), 0))?;

        let subchunk_path: String = wad_relative_path.replace(".client", ".subchunktoc");

        let sub_chunk = Self::get_sub_chunk(&wad, &file, &subchunk_path).ok();

        Ok(LeagueWadLoader {
            wad,
            file,
            sub_chunk,
        })
    }

    pub fn get_sub_chunk(
        wad: &LeagueWad,
        file: &Arc<File>,
        subchunk_path: &str,
    ) -> Result<LeagueWadSubchunk, LeagueLoaderError> {
        let entry = wad.get_entry(LeagueLoader::hash_wad(&subchunk_path))?;

        let reader = Self::get_wad_zstd_entry_reader_inner(file.clone(), &entry)?;

        Ok(LeagueWadSubchunk::read_options(
            &mut NoSeek::new(reader),
            Endian::Little,
            args! { count: entry.target_size / 16 },
        )?)
    }

    pub fn get_wad_zstd_entry_reader(
        &self,
        entry: &LeagueWadEntry,
    ) -> Result<Box<dyn Read>, LeagueLoaderError> {
        Self::get_wad_zstd_entry_reader_inner(self.file.clone(), entry)
    }

    fn get_wad_zstd_entry_reader_inner(
        file: Arc<File>,
        entry: &LeagueWadEntry,
    ) -> Result<Box<dyn Read>, LeagueLoaderError> {
        let reader = ArcFileReader::new(file, entry.offset as u64).take(entry.size as u64);
        let decoder = Decoder::new(reader)?;
        Ok(Box::new(decoder))
    }

    pub fn get_wad_entry_no_seek_reader_by_hash(
        &self,
        hash: u64,
    ) -> Result<NoSeek<Box<dyn Read + '_>>, LeagueLoaderError> {
        let entry = self.get_wad_entry_by_hash(hash)?;
        self.get_wad_entry_reader(&entry).map(|v| NoSeek::new(v))
    }

    pub fn get_wad_entry_no_seek_reader_by_path(
        &self,
        path: &str,
    ) -> Result<NoSeek<Box<dyn Read + '_>>, LeagueLoaderError> {
        self.get_wad_entry_no_seek_reader_by_hash(LeagueLoader::hash_wad(path))
    }

    pub fn get_wad_entry_reader_by_path(
        &self,
        path: &str,
    ) -> Result<BufReader<Cursor<Vec<u8>>>, LeagueLoaderError> {
        self.get_wad_entry_buffer_by_path(path)
            .map(|v| BufReader::new(Cursor::new(v)))
    }

    pub fn get_wad_entry_buffer_by_path(&self, path: &str) -> Result<Vec<u8>, LeagueLoaderError> {
        let entry = self.get_wad_entry_by_path(path)?;
        let mut buf = Vec::new();
        self.get_wad_entry_reader(&entry).map(|mut v| {
            v.read_to_end(&mut buf).unwrap();
            buf
        })
    }

    pub fn get_wad_entry_by_hash(&self, hash: u64) -> Result<LeagueWadEntry, LeagueLoaderError> {
        self.wad.get_entry(hash)
    }

    pub fn get_wad_entry_by_path(&self, path: &str) -> Result<LeagueWadEntry, LeagueLoaderError> {
        self.get_wad_entry_by_hash(LeagueLoader::hash_wad(&path.to_lowercase()))
    }

    pub fn get_texture_by_hash(&self, hash: u64) -> Result<LeagueTexture, LeagueLoaderError> {
        let mut reader = self.get_wad_entry_no_seek_reader_by_hash(hash)?;
        Ok(LeagueTexture::read(&mut reader)?)
    }

    pub fn get_texture_by_path(&self, path: &str) -> Result<LeagueTexture, LeagueLoaderError> {
        let mut reader = self.get_wad_entry_no_seek_reader_by_path(path)?;
        Ok(LeagueTexture::read(&mut reader)?)
    }

    pub fn get_prop_bin_by_path(&self, path: &str) -> Result<PropFile, LeagueLoaderError> {
        let mut reader = self.get_wad_entry_no_seek_reader_by_path(path)?;
        Ok(PropFile::read(&mut reader)?)
    }

    pub fn get_character_bin_by_path(
        &self,
        character_record: &str,
    ) -> Result<PropFile, LeagueLoaderError> {
        let name = character_record.split("/").nth(1).unwrap();

        let path = format!("data/characters/{0}/{0}.bin", name);

        let character_bin = self.get_prop_bin_by_path(&path).unwrap();

        Ok(character_bin)
    }

    pub fn load_character_record(&self, character_record: &str) -> CharacterRecord {
        let character_bin = self.get_character_bin_by_path(character_record).unwrap();

        let entry = character_bin
            .entries
            .iter()
            .find(|v| v.hash == LeagueLoader::hash_bin(&character_record))
            .unwrap();

        from_entry(entry)
    }

    pub fn get_skin_bin_by_path(&self, skin: &str) -> Result<PropFile, LeagueLoaderError> {
        let path = format!("data/{}.bin", skin);
        let skin_bin = self.get_prop_bin_by_path(&path).unwrap();
        Ok(skin_bin)
    }

    pub fn load_character_skin(
        &self,
        skin: &str,
    ) -> (SkinCharacterDataProperties, HashMap<u32, EntryData>) {
        let skin_bin = self.get_skin_bin_by_path(skin).unwrap();

        let awi = skin_bin
            .iter_entry_by_class(LeagueLoader::hash_bin("SkinCharacterDataProperties"))
            .collect::<Vec<_>>();

        let skin_character_data_properties = awi.iter().next().unwrap();

        let flat_map: HashMap<_, _> = skin_bin
            .links
            .iter()
            .map(|v| self.get_prop_bin_by_path(&v.text).unwrap())
            .flat_map(|v| v.entries)
            .map(|v| (v.hash, v))
            .collect();

        (from_entry(skin_character_data_properties), flat_map)
    }

    pub fn get_wad_entry_reader(
        &self,
        entry: &LeagueWadEntry,
    ) -> Result<Box<dyn Read + '_>, LeagueLoaderError> {
        match entry.format {
            WadDataFormat::Uncompressed => Ok(Box::new(
                ArcFileReader::new(self.file.clone(), entry.offset as u64).take(entry.size as u64),
            )),
            WadDataFormat::Redirection | WadDataFormat::Gzip => {
                panic!("wad entry format not supported")
            }
            WadDataFormat::Zstd => self.get_wad_zstd_entry_reader(entry),
            WadDataFormat::Chunked(subchunk_count) => {
                self.read_chunked_entry(entry, subchunk_count)
            }
        }
    }

    fn read_chunked_entry(
        &self,
        entry: &LeagueWadEntry,
        subchunk_count: u8,
    ) -> Result<Box<dyn Read + '_>, LeagueLoaderError> {
        let Some(sub_chunk) = &self.sub_chunk else {
            return Err(LeagueLoaderError::Io(io::Error::new(
                io::ErrorKind::NotFound,
                "Subchunk not found",
            )));
        };

        let mut offset = 0u64;
        let mut result = Vec::with_capacity(entry.target_size as usize);

        for i in 0..subchunk_count {
            let chunk_index = (entry.first_subchunk_index as usize) + (i as usize);
            if chunk_index >= sub_chunk.chunks.len() {
                panic!("Subchunk index out of bounds");
            }

            let subchunk_entry = &sub_chunk.chunks[chunk_index];
            let mut subchunk_reader =
                ArcFileReader::new(self.file.clone(), entry.offset as u64 + offset)
                    .take(subchunk_entry.size as u64);

            offset += subchunk_entry.size as u64;

            if subchunk_entry.size == subchunk_entry.target_size {
                subchunk_reader.read_to_end(&mut result)?;
            } else {
                zstd::stream::read::Decoder::new(subchunk_reader)?.read_to_end(&mut result)?;
            }
        }

        Ok(Box::new(Cursor::new(result)))
    }

    pub async fn save_wad_entry_to_file(&self, path: &str) -> Result<(), LeagueLoaderError> {
        let buffer = self.get_wad_entry_buffer_by_path(path)?;
        let mut file = get_asset_writer(&path).await?;
        file.write_all(&buffer).await?;
        file.flush().await?;
        Ok(())
    }
}
