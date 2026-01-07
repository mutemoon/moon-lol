use std::fs::File;
use std::io::{self, BufReader, Cursor, Read};
use std::sync::Arc;

use league_utils::hash_wad;
use zstd::Decoder;

use crate::{
    ArcFileReader, Error, LeagueWad, LeagueWadEntry, LeagueWadLoaderTrait, LeagueWadSubchunk,
    WadDataFormat,
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
    ) -> Result<LeagueWadLoader, Error> {
        let wad_absolute_path = format!("{}/{}", root_dir, wad_relative_path);

        let file = Arc::new(File::open(&wad_absolute_path)?);

        // Read first 272 bytes for header (magic + major + minor + padding + entry_count)
        // Actually, LeagueWad::parse reads magic (2) + major (1) + minor (1) + padding (0x108) + entry_count (4) = 272 bytes.
        // But then it reads entries. So we need to read more.
        // Let's read the whole header first to get entry_count.
        let mut header_bytes = [0u8; 272];
        let mut f = File::open(&wad_absolute_path)?;
        f.read_exact(&mut header_bytes)?;
        let entry_count = u32::from_le_bytes(header_bytes[268..272].try_into().unwrap());

        // Now read the whole thing (header + entries)
        let total_header_size = 272 + (entry_count as usize * 32); // Each entry is 32 bytes
        let mut wad_bytes = vec![0u8; total_header_size];
        let mut f = File::open(&wad_absolute_path)?;
        f.read_exact(&mut wad_bytes)?;

        let (_, wad) = LeagueWad::parse(&wad_bytes).map_err(|_| {
            Error::Io(io::Error::new(
                io::ErrorKind::InvalidData,
                "Failed to parse WAD",
            ))
        })?;

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
    ) -> Result<LeagueWadSubchunk, Error> {
        let entry = wad.get_entry(hash_wad(&subchunk_path))?;

        let mut reader = Self::get_wad_zstd_entry_reader_inner(file.clone(), &entry)?;
        let mut data = Vec::new();
        reader.read_to_end(&mut data)?;

        let (_, sub_chunk) =
            LeagueWadSubchunk::parse(&data, entry.target_size / 16).map_err(|_| {
                Error::Io(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Failed to parse subchunk",
                ))
            })?;

        Ok(sub_chunk)
    }

    pub fn get_wad_zstd_entry_reader(
        &self,
        entry: &LeagueWadEntry,
    ) -> Result<Box<dyn Read>, Error> {
        Self::get_wad_zstd_entry_reader_inner(self.file.clone(), entry)
    }

    fn get_wad_zstd_entry_reader_inner(
        file: Arc<File>,
        entry: &LeagueWadEntry,
    ) -> Result<Box<dyn Read>, Error> {
        let reader = ArcFileReader::new(file, entry.offset as u64).take(entry.size as u64);
        let decoder = Decoder::new(reader)?;
        Ok(Box::new(decoder))
    }

    pub fn get_wad_entry_reader_by_path(
        &self,
        path: &str,
    ) -> Result<BufReader<Cursor<Vec<u8>>>, Error> {
        self.get_wad_entry_buffer_by_path(path)
            .map(|v| BufReader::new(Cursor::new(v)))
    }

    pub fn get_wad_entry_by_hash(&self, hash: u64) -> Result<LeagueWadEntry, Error> {
        self.wad.get_entry(hash)
    }

    pub fn get_wad_entry_by_path(&self, path: &str) -> Result<LeagueWadEntry, Error> {
        self.get_wad_entry_by_hash(hash_wad(&path.to_lowercase()))
    }

    fn read_chunked_entry(
        &self,
        entry: &LeagueWadEntry,
        subchunk_count: u8,
    ) -> Result<Box<dyn Read + '_>, Error> {
        let Some(sub_chunk) = &self.sub_chunk else {
            return Err(Error::Io(io::Error::new(
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
}

impl LeagueWadLoaderTrait for LeagueWadLoader {
    fn get_wad_entry_reader_by_hash(&self, hash: u64) -> Result<Box<dyn Read + '_>, Error> {
        let entry = self.get_wad_entry_by_hash(hash)?;
        match entry.format {
            WadDataFormat::Uncompressed => Ok(Box::new(
                ArcFileReader::new(self.file.clone(), entry.offset as u64).take(entry.size as u64),
            )),
            WadDataFormat::Redirection | WadDataFormat::Gzip => {
                panic!("wad entry format not supported")
            }
            WadDataFormat::Zstd => self.get_wad_zstd_entry_reader(&entry),
            WadDataFormat::Chunked(subchunk_count) => {
                self.read_chunked_entry(&entry, subchunk_count)
            }
        }
    }
}
