use binrw::binread;
use binrw::{BinRead, BinResult, Endian};
use bitflags::bitflags;
use std::io::{Read, Seek};

#[binread]
#[derive(Debug)]
#[br(little)]
pub struct LeagueTexture {
    #[br(magic = b"TEX\0")]
    _magic: (),

    pub width: u16,
    pub height: u16,

    _is_extended_format_maybe: u8,

    #[br(map = |v:u8| match v {
        1 => LeagueTextureFormat::Etc1,
        2 | 3 => LeagueTextureFormat::Etc2Eac,
        10| 11 => LeagueTextureFormat::Bc1,
        12 => LeagueTextureFormat::Bc3,
        20 => LeagueTextureFormat::Bgra8,
        _ => panic!("Invalid LeagueTextureFormat: {}", v),
    })]
    pub format: LeagueTextureFormat,

    pub resource_type: LeagueTextureType,

    pub flags: LeagueTextureFlags,

    #[br(parse_with = parse_mipmaps, args(width, height, format, flags))]
    pub mipmaps: Vec<Vec<u8>>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LeagueTextureFormat {
    Etc1,
    Etc2Eac,
    Bc1,
    Bc3,
    Bgra8,
}

#[derive(Debug, BinRead)]
#[br(repr = u8)]
pub enum LeagueTextureType {
    Texture = 0,
    Cube = 1,
    Surface = 2,
    Volume = 3,
}

bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct LeagueTextureFlags: u8 {
        const HasMipMaps = 1 << 0;
    }
}

impl BinRead for LeagueTextureFlags {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        _args: Self::Args<'_>,
    ) -> BinResult<Self> {
        let val = u8::read_options(reader, endian, ())?;
        Ok(Self::from_bits_truncate(val))
    }
}

const fn get_block_size(format: LeagueTextureFormat) -> usize {
    match format {
        LeagueTextureFormat::Bc1 => 8,
        LeagueTextureFormat::Bc3 => 16,
        LeagueTextureFormat::Bgra8 => 4,
        LeagueTextureFormat::Etc1 | LeagueTextureFormat::Etc2Eac => 8,
    }
}

fn calculate_block_count(
    format: LeagueTextureFormat,
    width: usize,
    height: usize,
) -> (usize, usize) {
    match format {
        LeagueTextureFormat::Bc1 | LeagueTextureFormat::Bc3 => {
            let width_in_blocks = (width + 3) / 4;
            let height_in_blocks = (height + 3) / 4;
            (width_in_blocks, height_in_blocks)
        }
        LeagueTextureFormat::Bgra8 => (width, height),
        _ => {
            let width_in_blocks = (width + 3) / 4;
            let height_in_blocks = (height + 3) / 4;
            (width_in_blocks, height_in_blocks)
        }
    }
}

fn parse_mipmaps<R: Read>(
    reader: &mut R,
    _endian: Endian,
    args: (u16, u16, LeagueTextureFormat, LeagueTextureFlags),
) -> BinResult<Vec<Vec<u8>>> {
    let (width, height, format, flags) = args;

    let mip_count = if flags.contains(LeagueTextureFlags::HasMipMaps) {
        ((width.max(height) as f32).log2().floor() as usize) + 1
    } else {
        1
    };

    let mut mipmaps = vec![Vec::new(); mip_count];

    for i in (0..mip_count).rev() {
        let current_width = (width as u32 >> i).max(1) as usize;
        let current_height = (height as u32 >> i).max(1) as usize;

        let block_size = get_block_size(format);
        let (width_in_blocks, height_in_blocks) =
            calculate_block_count(format, current_width, current_height);
        let mip_size = width_in_blocks * height_in_blocks * block_size;

        let mut mip_data = vec![0u8; mip_size];
        reader.read_exact(&mut mip_data)?;

        mipmaps[i] = mip_data;
    }

    Ok(mipmaps)
}
