//! Riot WPK 容器解析。
//!
//! WPK 是 Riot 自有的 Wwise 音频打包格式，结构极简：仅是一批 `.wem` 的拼接容器，
//! 无 HIRC/元数据。布局如下（全部小端）：
//! - magic: `r3d2`（4 字节）
//! - version: u32（已知为 1）
//! - count: u32（文件数量）
//! - offsets: [u32; count]，每个是对应 entry 头的绝对偏移
//! - entry@offset: dataOffset(u32), dataSize(u32), nameLen(u32, UTF-16 字符数), name([u16; nameLen] UTF-16LE)
//!
//! 文件名形如 `123456789.wem`，其数字部分即 Wwise 的 source id（wem id）。

use nom::IResult;
use nom::number::complete::le_u32;

pub const MAGIC: &[u8; 4] = b"r3d2";

#[derive(Debug, Clone)]
pub struct WpkEntry {
    /// wem id（文件名的数字部分）。
    pub id: u32,
    /// 原始 `.wem` 字节。
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Default)]
pub struct WpkFile {
    pub entries: Vec<WpkEntry>,
}

impl WpkFile {
    /// 解析整个 WPK；`input` 必须是完整文件内容（entry 使用绝对偏移）。
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (i, magic) = nom::bytes::complete::take(4usize)(input)?;
        if magic != MAGIC {
            return Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Tag,
            )));
        }
        let (i, _version) = le_u32(i)?;
        let (mut i, count) = le_u32(i)?;

        let mut offsets = Vec::with_capacity(count as usize);
        for _ in 0..count {
            let (rest, off) = le_u32(i)?;
            offsets.push(off as usize);
            i = rest;
        }

        let mut entries = Vec::with_capacity(count as usize);
        for off in offsets {
            if off + 12 > input.len() {
                continue;
            }
            let entry_buf = &input[off..];
            let (rest, data_offset) = le_u32(entry_buf)?;
            let (rest, data_size) = le_u32(rest)?;
            let (rest, name_len) = le_u32(rest)?;

            let name = read_utf16_name(rest, name_len as usize);
            let id = parse_wem_id(&name);

            let start = data_offset as usize;
            let end = start + data_size as usize;
            if start > input.len() || end > input.len() {
                continue;
            }
            let data = input[start..end].to_vec();
            if let Some(id) = id {
                entries.push(WpkEntry { id, data });
            }
        }

        Ok((&input[input.len()..], WpkFile { entries }))
    }
}

/// 从 UTF-16LE 字节读取 `char_count` 个字符。
fn read_utf16_name(buf: &[u8], char_count: usize) -> String {
    let mut units = Vec::with_capacity(char_count);
    for k in 0..char_count {
        let base = k * 2;
        if base + 2 > buf.len() {
            break;
        }
        units.push(u16::from_le_bytes([buf[base], buf[base + 1]]));
    }
    String::from_utf16_lossy(&units)
}

/// 从形如 `123456789.wem` 的文件名提取数字 id。
fn parse_wem_id(name: &str) -> Option<u32> {
    let stem = name.split(['.', '/', '\\']).next().unwrap_or(name);
    let digits: String = stem.chars().filter(|c| c.is_ascii_digit()).collect();
    digits.parse::<u32>().ok()
}
