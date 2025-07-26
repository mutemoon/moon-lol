use crate::render::{LeagueMapGeo, LeagueTexture};
use bevy::asset::RenderAssetUsages;
use bevy::ecs::resource::{self, Resource};
use bevy::image::{dds_buffer_to_image, CompressedImageFormats, Image};
use binrw::{args, Endian};
use binrw::{binread, io::NoSeek, BinRead};
use cdragon_prop::PropFile;
use std::hash::Hasher;
use std::os::windows::fs::FileExt;
use std::{
    collections::HashMap,
    fs::File,
    io::{self, Read, Seek, SeekFrom},
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
    pub fn get_entry_reader() {}
}

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
}

pub struct ArcFileReader {
    file: Arc<File>,
    start_offset: u64,
    current_pos: u64,
}

impl ArcFileReader {
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
    pub fn new(root_dir: &str, map_path: &str) -> io::Result<LeagueLoader> {
        let root_dir = Path::new(root_dir);
        let map_relative_path = Path::new(map_path);

        let map_absolute_path = root_dir.join(map_relative_path);

        let file: Arc<File> = Arc::new(File::open(&map_absolute_path)?);

        let wad = LeagueWad::read(&mut ArcFileReader::new(file.clone(), 0)).unwrap();

        let entry = LeagueLoader::get_wad_subchunk_entry(&wad, map_path);

        let reader = LeagueLoader::get_wad_zstd_entry_reader(file.clone(), &entry).unwrap();

        let texture = LeagueWadSubchunk::read_options(
            &mut NoSeek::new(reader),
            Endian::Little,
            args! { count: entry.target_size / 16 },
        )
        .unwrap();

        Ok(LeagueLoader {
            root_dir: root_dir.to_path_buf(),
            map_path: map_relative_path.to_path_buf(),
            wad_file: file,
            sub_chunk: texture,
            wad,
        })
    }

    pub fn get_wad_entry_by_hash(&self, hash: u64) -> LeagueWadEntry {
        LeagueLoader::get_wad_entry(&self.wad, hash)
    }

    pub fn get_wad_entry_by_path(&self, path: &str) -> LeagueWadEntry {
        self.get_wad_entry_by_hash(LeagueLoader::compute_wad_hash(&path.to_lowercase()))
    }

    pub fn get_wad_entry(wad: &LeagueWad, hash: u64) -> LeagueWadEntry {
        wad.entries.get(&hash).unwrap().clone()
    }

    pub fn get_wad_subchunk_entry(wad: &LeagueWad, map_path: &str) -> LeagueWadEntry {
        let subchunk_path = map_path
            .replace(".client", ".subchunktoc")
            .replace("\\", "/")
            .to_lowercase();

        LeagueLoader::get_wad_entry(wad, LeagueLoader::compute_wad_hash(&subchunk_path))
    }

    pub fn get_wad_zstd_entry_reader(
        wad_file: Arc<File>,
        entry: &LeagueWadEntry,
    ) -> io::Result<Box<dyn Read>> {
        Ok(Box::new(
            Decoder::new(
                ArcFileReader::new(wad_file.clone(), entry.offset as u64).take(entry.size as u64),
            )
            .unwrap(),
        ))
    }

    pub fn get_wad_entry_reader(&self, entry: &LeagueWadEntry) -> io::Result<Box<dyn Read + '_>> {
        match entry.format {
            WadDataFormat::Uncompressed => todo!(),
            WadDataFormat::Gzip => todo!(),
            WadDataFormat::Redirection => todo!(),
            WadDataFormat::Zstd => {
                LeagueLoader::get_wad_zstd_entry_reader(self.wad_file.clone(), entry)
            }
            WadDataFormat::Chunked(subchunk_count) => {
                if self.sub_chunk.chunks.is_empty() {
                    Err(io::ErrorKind::AlreadyExists.into())
                } else {
                    let mut readed_size: u64 = 0;
                    let mut result = Vec::with_capacity(entry.target_size as usize);
                    for i in 0..subchunk_count {
                        let subchunk_entry = &self.sub_chunk.chunks
                            [(entry.first_subchunk_index + i as u16) as usize];
                        let mut subchunk_reader = ArcFileReader::new(
                            self.wad_file.clone(),
                            entry.offset as u64 + readed_size,
                        )
                        .take(subchunk_entry.size as u64);
                        readed_size += subchunk_entry.size as u64;
                        if subchunk_entry.size == subchunk_entry.target_size {
                            // Assume no compression
                            subchunk_reader.read_to_end(&mut result)?;
                        } else {
                            zstd::stream::read::Decoder::new(subchunk_reader)?
                                .read_to_end(&mut result)?;
                        }
                    }
                    Ok(Box::new(std::io::Cursor::new(result)))
                }
            }
        }
    }

    pub fn get_wad_entry_reader_by_hash(&self, hash: u64) -> io::Result<Box<dyn Read + '_>> {
        let entry = self.get_wad_entry_by_hash(hash);
        self.get_wad_entry_reader(&entry)
    }

    pub fn get_wad_entry_reader_by_path(&self, path: &str) -> io::Result<Box<dyn Read + '_>> {
        let entry = self.get_wad_entry_by_path(path);
        self.get_wad_entry_reader(&entry)
    }

    pub fn get_texture_by_hash(&self, hash: u64) -> io::Result<LeagueTexture> {
        let reader = self.get_wad_entry_reader_by_hash(hash)?;
        Ok(LeagueTexture::read(&mut NoSeek::new(reader)).unwrap())
    }

    pub fn get_texture_by_path(&self, path: &str) -> io::Result<LeagueTexture> {
        let reader = self.get_wad_entry_reader_by_path(path)?;
        Ok(LeagueTexture::read(&mut NoSeek::new(reader)).unwrap())
    }

    pub fn get_image_by_texture_path(&self, path: &str) -> io::Result<Image> {
        let mut reader = self.get_wad_entry_reader_by_path(path)?;
        match Path::new(path).extension().and_then(|ext| ext.to_str()) {
            Some("tex") => Ok(LeagueTexture::read(&mut NoSeek::new(reader))
                .unwrap()
                .to_bevy_image(RenderAssetUsages::default())
                .unwrap()),
            Some("dds") => {
                let mut buffer = Vec::new();
                reader.read_to_end(&mut buffer)?;
                Ok(dds_buffer_to_image(
                    #[cfg(debug_assertions)]
                    path.to_string(),
                    &buffer,
                    CompressedImageFormats::all(),
                    false,
                )
                .unwrap())
            }
            _ => todo!(),
        }
    }

    pub fn get_map_geo_by_path(&self, path: &str) -> io::Result<LeagueMapGeo> {
        let reader = self.get_wad_entry_reader_by_path(path)?;
        Ok(LeagueMapGeo::read(&mut NoSeek::new(reader)).unwrap())
    }

    pub fn get_prop_bin_by_path(&self, path: &str) -> io::Result<PropFile> {
        let mut reader = self.get_wad_entry_reader_by_path(path)?;
        let data = {
            let mut data: Vec<u8> = Vec::new();
            reader.read_to_end(&mut data)?;
            data
        };
        Ok(PropFile::from_slice(&data).unwrap())
    }

    pub fn compute_wad_hash(s: &str) -> u64 {
        let mut h = XxHash64::with_seed(0);
        h.write(s.as_bytes());
        h.finish()
    }

    pub fn compute_binhash(s: &str) -> u32 {
        s.to_ascii_lowercase().bytes().fold(0x811c9dc5_u32, |h, b| {
            (h ^ b as u32).wrapping_mul(0x01000193)
        })
    }
}
