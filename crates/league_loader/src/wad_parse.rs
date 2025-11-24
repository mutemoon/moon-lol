use std::{
    collections::HashMap,
    io::{self},
};

use crate::Error;

use binrw::binread;

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
    pub fn get_entry(&self, hash: u64) -> Result<LeagueWadEntry, Error> {
        self.entries
            .get(&hash)
            .ok_or_else(|| {
                Error::Io(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("WAD entry not found: {:x}", hash),
                ))
            })
            .cloned()
    }
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
