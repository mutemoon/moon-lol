use crate::render::{LeagueBinCharacterRecord, LeagueMapGeo, LeagueTexture};
use bevy::asset::RenderAssetUsages;
use bevy::ecs::resource::Resource;
use bevy::image::{dds_buffer_to_image, CompressedImageFormats, Image};
use binrw::{args, binread, io::NoSeek, BinRead, Endian};
use cdragon_hashes::wad::compute_wad_hash;
use cdragon_prop::{BinString, PropFile};
use std::io::BufReader;
#[cfg(unix)]
use std::os::unix::fs::FileExt;
#[cfg(windows)]
use std::os::windows::fs::FileExt;
use std::{
    collections::HashMap,
    fs::File,
    hash::Hasher,
    io::{self, Cursor, Read, Seek, SeekFrom},
    path::{Path, PathBuf},
    sync::Arc,
};
use twox_hash::XxHash64;
use zstd::Decoder;

#[binread]
#[derive(Debug)]
#[br(magic = b"RW")]
#[br(little)]
pub struct LeagueWad {
    pub major: u8,
    pub minor: u8,

    #[br(pad_before = 0x108)]
    pub entry_count: u32,

    #[br(count = entry_count)]
    #[br(map = |entries: Vec<LeagueWadEntry>| {
        entries
            .into_iter()
            .map(|entry| (entry.path_hash, entry))
            .collect()
    })]
    pub entries: HashMap<u64, LeagueWadEntry>,
}

impl LeagueWad {
    #[inline]
    pub fn get_entry(&self, hash: u64) -> Option<&LeagueWadEntry> {
        self.entries.get(&hash)
    }
}

pub struct LeagueProp(pub PropFile);

unsafe impl Sync for LeagueProp {}
unsafe impl Send for LeagueProp {}

#[binread]
#[derive(Debug, Clone, Copy)]
#[br(little)]
pub struct LeagueWadEntry {
    pub path_hash: u64,
    pub offset: u32,
    pub size: u32,
    pub target_size: u32,

    #[br(map = |v: u8| parse_wad_data_format(v))]
    pub format: WadDataFormat,

    #[br(map = |v: u8| v != 0)]
    pub duplicate: bool,

    pub first_subchunk_index: u16,
    pub data_hash: u64,
}

#[binread]
#[derive(Debug, Clone, Copy)]
#[br(little)]
pub enum WadDataFormat {
    Uncompressed,
    Gzip,
    Redirection,
    Zstd,
    Chunked(u8),
}

#[inline]
fn parse_wad_data_format(format: u8) -> WadDataFormat {
    match format {
        0 => WadDataFormat::Uncompressed,
        1 => WadDataFormat::Gzip,
        2 => WadDataFormat::Redirection,
        3 => WadDataFormat::Zstd,
        b if b & 0xf == 4 => WadDataFormat::Chunked(b >> 4),
        _ => panic!("Invalid WadDataFormat: {}", format),
    }
}

#[binread]
#[derive(Debug)]
#[br(little)]
#[br(import { count: u32 })]
pub struct LeagueWadSubchunk {
    #[br(count = count)]
    pub chunks: Vec<LeagueWadSubchunkItem>,
}

#[binread]
#[derive(Debug)]
#[br(little)]
pub struct LeagueWadSubchunkItem {
    pub size: u32,
    pub target_size: u32,
    pub data_hash: u64,
}

#[derive(Resource)]
pub struct LeagueLoader {
    pub root_dir: PathBuf,
    pub map_path: PathBuf,

    pub wad_file: Arc<File>,
    pub sub_chunk: LeagueWadSubchunk,

    pub wad: LeagueWad,
    pub map_geo: LeagueMapGeo,
    pub map_materials: LeagueProp,
}

pub struct ArcFileReader {
    file: Arc<File>,
    start_offset: u64,
    current_pos: u64,
}

impl ArcFileReader {
    #[inline]
    pub fn new(file: Arc<File>, start_offset: u64) -> Self {
        Self {
            file,
            start_offset,
            current_pos: 0,
        }
    }
}

impl Read for ArcFileReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let absolute_offset = self.start_offset + self.current_pos;

        #[cfg(unix)]
        let bytes_read = self.file.read_at(buf, absolute_offset)?;
        #[cfg(windows)]
        let bytes_read = self.file.seek_read(buf, absolute_offset)?;

        self.current_pos += bytes_read as u64;

        Ok(bytes_read)
    }
}

impl Seek for ArcFileReader {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let new_pos = match pos {
            SeekFrom::Start(p) => p as i64,
            SeekFrom::End(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::Unsupported,
                    "SeekFrom::End is not supported",
                ));
            }
            SeekFrom::Current(p) => self.current_pos as i64 + p,
        };

        if new_pos < 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Invalid seek to a negative position",
            ));
        }

        self.current_pos = new_pos as u64;
        Ok(self.current_pos)
    }
}

impl LeagueLoader {
    pub fn new(root_dir: &str, map_path: &str, map_geo_path: &str) -> io::Result<LeagueLoader> {
        let root_dir = Path::new(root_dir);
        let map_relative_path = Path::new(map_path);
        let map_absolute_path = root_dir.join(map_relative_path);

        let file = Arc::new(File::open(&map_absolute_path)?);
        let wad = LeagueWad::read(&mut ArcFileReader::new(file.clone(), 0))
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        // Load subchunk data
        let sub_chunk = Self::load_subchunk(&wad, &file, map_path)?;

        // Load map geometry
        let map_geo = Self::load_map_geo(&wad, &file, map_geo_path)?;

        // Load map materials
        let map_materials = Self::load_map_materials(&wad, &file, map_geo_path)?;

        // Load character map
        // let character_map = Self::load_character_map(&wad, &file)?;

        Ok(LeagueLoader {
            root_dir: root_dir.to_path_buf(),
            map_path: map_relative_path.to_path_buf(),
            wad_file: file,
            sub_chunk,
            wad,
            map_geo,
            map_materials: LeagueProp(map_materials),
        })
    }

    fn load_subchunk(
        wad: &LeagueWad,
        file: &Arc<File>,
        map_path: &str,
    ) -> io::Result<LeagueWadSubchunk> {
        let entry = Self::get_wad_subchunk_entry(wad, map_path);
        let reader = Self::get_wad_zstd_entry_reader(file, &entry)?;

        LeagueWadSubchunk::read_options(
            &mut NoSeek::new(reader),
            Endian::Little,
            args! { count: entry.target_size / 16 },
        )
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    fn load_map_geo(
        wad: &LeagueWad,
        file: &Arc<File>,
        map_geo_path: &str,
    ) -> io::Result<LeagueMapGeo> {
        let entry = Self::get_wad_entry(wad, Self::compute_wad_hash(map_geo_path));
        let reader = Self::get_wad_zstd_entry_reader(file, &entry)?;

        LeagueMapGeo::read(&mut NoSeek::new(reader))
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    fn load_map_materials(
        wad: &LeagueWad,
        file: &Arc<File>,
        map_geo_path: &str,
    ) -> io::Result<PropFile> {
        let map_materials_path = map_geo_path
            .replace(".mapgeo", ".materials.bin")
            .replace('\\', "/")
            .to_lowercase();

        let entry = Self::get_wad_entry(wad, Self::compute_wad_hash(&map_materials_path));
        let mut reader = Self::get_wad_zstd_entry_reader(file, &entry)?;

        let mut data = Vec::with_capacity(entry.target_size as usize);
        reader.read_to_end(&mut data)?;

        PropFile::from_slice(&data).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    fn load_character_map(
        wad: &LeagueWad,
        file: &Arc<File>,
    ) -> io::Result<HashMap<u32, LeagueBinCharacterRecord>> {
        const MAP11_PATH: &str = "data/maps/shipping/map11/map11.bin";
        let character_hash = LeagueLoader::hash_bin("Character");
        let character_record_hash = LeagueLoader::hash_bin("CharacterRecord");
        let name_hash = LeagueLoader::hash_bin("name");

        let entry = Self::get_wad_entry(wad, Self::compute_wad_hash(MAP11_PATH));
        let mut reader = Self::get_wad_zstd_entry_reader(file, &entry)?;

        let mut data = Vec::with_capacity(entry.target_size as usize);
        reader.read_to_end(&mut data)?;

        let map_shipping = PropFile::from_slice(&data)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        let character_map: HashMap<u32, LeagueBinCharacterRecord> = map_shipping
            .entries
            .iter()
            .filter(|v| v.ctype.hash == character_hash)
            .filter_map(|v| v.getv::<BinString>(name_hash.into()))
            .filter_map(|v| {
                let char_path = format!("data/characters/{0}/{0}.bin", v.0.to_lowercase());
                let entry = Self::get_wad_entry(wad, compute_wad_hash(&char_path));
                Self::get_wad_zstd_entry_reader(file, &entry).ok()
            })
            .filter_map(|mut reader| {
                let mut data = Vec::new();
                reader.read_to_end(&mut data).ok()?;
                PropFile::from_slice(&data).ok()
            })
            .flat_map(|prop_file| {
                prop_file
                    .entries
                    .into_iter()
                    .filter(|v| v.ctype.hash == character_record_hash)
            })
            .filter_map(|entry| Some((entry.path.hash, (&entry).into())))
            .collect();

        Ok(character_map)
    }

    #[inline]
    pub fn get_wad_entry_by_hash(&self, hash: u64) -> Option<LeagueWadEntry> {
        self.wad.get_entry(hash).copied()
    }

    #[inline]
    pub fn get_wad_entry_by_path(&self, path: &str) -> Option<LeagueWadEntry> {
        self.get_wad_entry_by_hash(Self::compute_wad_hash(&path.to_lowercase()))
    }

    pub fn get_wad_entry_reader(&self, entry: &LeagueWadEntry) -> io::Result<Box<dyn Read + '_>> {
        match entry.format {
            WadDataFormat::Uncompressed => Ok(Box::new(
                ArcFileReader::new(self.wad_file.clone(), entry.offset as u64)
                    .take(entry.size as u64),
            )),
            WadDataFormat::Redirection | WadDataFormat::Gzip => Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "wad entry format not supported",
            )),
            WadDataFormat::Zstd => Self::get_wad_zstd_entry_reader(&self.wad_file, entry),
            WadDataFormat::Chunked(subchunk_count) => {
                self.read_chunked_entry(entry, subchunk_count)
            }
        }
    }

    fn read_chunked_entry(
        &self,
        entry: &LeagueWadEntry,
        subchunk_count: u8,
    ) -> io::Result<Box<dyn Read + '_>> {
        if self.sub_chunk.chunks.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "No subchunk data available",
            ));
        }

        let mut offset = 0u64;
        let mut result = Vec::with_capacity(entry.target_size as usize);

        for i in 0..subchunk_count {
            let chunk_index = (entry.first_subchunk_index as usize) + (i as usize);
            if chunk_index >= self.sub_chunk.chunks.len() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Subchunk index out of bounds",
                ));
            }

            let subchunk_entry = &self.sub_chunk.chunks[chunk_index];
            let mut subchunk_reader =
                ArcFileReader::new(self.wad_file.clone(), entry.offset as u64 + offset)
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

    pub fn get_wad_entry_no_seek_reader_by_hash(
        &self,
        hash: u64,
    ) -> io::Result<NoSeek<Box<dyn Read + '_>>> {
        let entry = self
            .get_wad_entry_by_hash(hash)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "WAD entry not found"))?;
        self.get_wad_entry_reader(&entry).map(|v| NoSeek::new(v))
    }

    pub fn get_wad_entry_no_seek_reader_by_path(
        &self,
        path: &str,
    ) -> io::Result<NoSeek<Box<dyn Read + '_>>> {
        self.get_wad_entry_no_seek_reader_by_hash(Self::compute_wad_hash(path))
    }

    pub fn get_wad_entry_reader_by_path(
        &self,
        path: &str,
    ) -> io::Result<BufReader<Cursor<Vec<u8>>>> {
        self.get_wad_entry_buffer_by_path(path)
            .map(|v| BufReader::new(Cursor::new(v)))
    }

    pub fn get_wad_entry_buffer_by_path(&self, path: &str) -> io::Result<Vec<u8>> {
        let entry = self
            .get_wad_entry_by_path(path)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "WAD entry not found"))?;
        let mut buf = Vec::new();
        self.get_wad_entry_reader(&entry).map(|mut v| {
            v.read_to_end(&mut buf).unwrap();
            buf
        })
    }

    pub fn get_texture_by_hash(&self, hash: u64) -> io::Result<LeagueTexture> {
        let mut reader = self.get_wad_entry_no_seek_reader_by_hash(hash)?;
        LeagueTexture::read(&mut reader).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    pub fn get_texture_by_path(&self, path: &str) -> io::Result<LeagueTexture> {
        let mut reader = self.get_wad_entry_no_seek_reader_by_path(path)?;
        LeagueTexture::read(&mut reader).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    pub fn get_image_by_texture_path(&self, path: &str) -> io::Result<Image> {
        match Path::new(path).extension().and_then(|ext| ext.to_str()) {
            Some("tex") => {
                let mut reader = self.get_wad_entry_no_seek_reader_by_path(path)?;
                let texture = LeagueTexture::read(&mut reader)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                texture
                    .to_bevy_image(RenderAssetUsages::default())
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
            }
            Some("dds") => {
                let buffer = self.get_wad_entry_buffer_by_path(path)?;
                dds_buffer_to_image(
                    #[cfg(debug_assertions)]
                    path.to_string(),
                    &buffer,
                    CompressedImageFormats::all(),
                    false,
                )
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
            }
            _ => Err(io::Error::new(
                io::ErrorKind::Unsupported,
                format!("Unsupported texture format: {}", path),
            )),
        }
    }

    pub fn get_prop_bin_by_path(&self, path: &str) -> io::Result<PropFile> {
        let mut reader = self.get_wad_entry_no_seek_reader_by_path(path)?;
        let mut data = Vec::new();
        reader.read_to_end(&mut data)?;
        PropFile::from_slice(&data).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    #[inline]
    pub fn compute_wad_hash(s: &str) -> u64 {
        let mut h = XxHash64::with_seed(0);
        h.write(s.to_ascii_lowercase().as_bytes());
        h.finish()
    }

    #[inline]
    pub fn hash_bin(s: &str) -> u32 {
        s.to_ascii_lowercase().bytes().fold(0x811c9dc5_u32, |h, b| {
            (h ^ b as u32).wrapping_mul(0x01000193)
        })
    }

    pub fn hash_joint(s: &str) -> u32 {
        let mut hash = 0u32;
        for b in s.to_ascii_lowercase().bytes() {
            hash = (hash << 4) + (b as u32);

            let high = hash & 0xf0000000;

            if high != 0 {
                hash ^= high >> 24;
            }

            hash &= !high;
        }
        hash
    }

    #[inline]
    fn get_wad_entry(wad: &LeagueWad, hash: u64) -> LeagueWadEntry {
        *wad.entries.get(&hash).expect("WAD entry not found")
    }

    fn get_wad_subchunk_entry(wad: &LeagueWad, map_path: &str) -> LeagueWadEntry {
        let subchunk_path = map_path
            .replace(".client", ".subchunktoc")
            .replace('\\', "/")
            .to_lowercase();

        Self::get_wad_entry(wad, Self::compute_wad_hash(&subchunk_path))
    }

    fn get_wad_zstd_entry_reader(
        wad_file: &Arc<File>,
        entry: &LeagueWadEntry,
    ) -> io::Result<Box<dyn Read>> {
        let reader =
            ArcFileReader::new(wad_file.clone(), entry.offset as u64).take(entry.size as u64);
        let decoder =
            Decoder::new(reader).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        Ok(Box::new(decoder))
    }
}
