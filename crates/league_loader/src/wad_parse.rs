use std::collections::HashMap;
use std::io::{self};

use nom::bytes::complete::{tag, take};
use nom::multi::count;
use nom::number::complete::{le_u16, le_u32, le_u64, le_u8};
use nom::{IResult, Parser};

use crate::Error;

#[derive(Debug)]
pub struct LeagueWad {
    pub major: u8,
    pub minor: u8,
    pub entries: HashMap<u64, LeagueWadEntry>,
}

impl LeagueWad {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (i, _) = tag(&b"RW"[..])(input)?;
        let (i, major) = le_u8(i)?;
        let (i, minor) = le_u8(i)?;
        let (i, _) = take(0x108usize)(i)?;
        let (i, entry_count) = le_u32(i)?;

        let (i, entries_vec) = count(LeagueWadEntry::parse, entry_count as usize).parse(i)?;
        let entries = entries_vec
            .into_iter()
            .map(|entry| (entry.path_hash, entry))
            .collect();

        Ok((
            i,
            LeagueWad {
                major,
                minor,
                entries,
            },
        ))
    }

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

#[derive(Debug, Clone, Copy)]
pub struct LeagueWadEntry {
    pub path_hash: u64,
    pub offset: u32,
    pub size: u32,
    pub target_size: u32,
    pub format: WadDataFormat,
    pub duplicate: bool,
    pub first_subchunk_index: u16,
    pub data_hash: u64,
}

impl LeagueWadEntry {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (i, path_hash) = le_u64(input)?;
        let (i, offset) = le_u32(i)?;
        let (i, size) = le_u32(i)?;
        let (i, target_size) = le_u32(i)?;
        let (i, format_raw) = le_u8(i)?;
        let format = parse_wad_data_format(format_raw);
        let (i, duplicate_raw) = le_u8(i)?;
        let duplicate = duplicate_raw != 0;
        let (i, first_subchunk_index) = le_u16(i)?;
        let (i, data_hash) = le_u64(i)?;

        Ok((
            i,
            LeagueWadEntry {
                path_hash,
                offset,
                size,
                target_size,
                format,
                duplicate,
                first_subchunk_index,
                data_hash,
            },
        ))
    }
}

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug)]
pub struct LeagueWadSubchunk {
    pub chunks: Vec<LeagueWadSubchunkItem>,
}

impl LeagueWadSubchunk {
    pub fn parse(input: &[u8], count_val: u32) -> IResult<&[u8], Self> {
        let (i, chunks) = count(LeagueWadSubchunkItem::parse, count_val as usize).parse(input)?;
        Ok((i, LeagueWadSubchunk { chunks }))
    }
}

#[derive(Debug)]
pub struct LeagueWadSubchunkItem {
    pub size: u32,
    pub target_size: u32,
    pub data_hash: u64,
}

impl LeagueWadSubchunkItem {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (i, size) = le_u32(input)?;
        let (i, target_size) = le_u32(i)?;
        let (i, data_hash) = le_u64(i)?;
        Ok((
            i,
            LeagueWadSubchunkItem {
                size,
                target_size,
                data_hash,
            },
        ))
    }
}
