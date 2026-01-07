use bitflags::bitflags;
use nom::bytes::complete::{tag, take};
use nom::number::complete::{le_u16, le_u8};
use nom::{IResult, Parser};

#[derive(Debug)]
pub struct LeagueTexture {
    pub width: u16,
    pub height: u16,
    pub _is_extended_format_maybe: u8,
    pub format: LeagueTextureFormat,
    pub resource_type: LeagueTextureType,
    pub flags: LeagueTextureFlags,
    pub mipmaps: Vec<Vec<u8>>,
}

impl LeagueTexture {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (i, _) = tag(&b"TEX\0"[..])(input)?;
        let (i, width) = le_u16(i)?;
        let (i, height) = le_u16(i)?;
        let (i, _is_extended_format_maybe) = le_u8(i)?;
        let (i, format_raw) = le_u8(i)?;
        let format = match format_raw {
            1 => LeagueTextureFormat::Etc1,
            2 | 3 => LeagueTextureFormat::Etc2Eac,
            10 | 11 => LeagueTextureFormat::Bc1,
            12 => LeagueTextureFormat::Bc3,
            20 => LeagueTextureFormat::Bgra8,
            _ => panic!("Invalid LeagueTextureFormat: {}", format_raw),
        };

        let (i, resource_type_raw) = le_u8(i)?;
        let resource_type = match resource_type_raw {
            0 => LeagueTextureType::Texture,
            1 => LeagueTextureType::Cube,
            2 => LeagueTextureType::Surface,
            3 => LeagueTextureType::Volume,
            _ => panic!("Invalid LeagueTextureType: {}", resource_type_raw),
        };

        let (i, flags_raw) = le_u8(i)?;
        let flags = LeagueTextureFlags::from_bits_truncate(flags_raw);

        let (i, mipmaps) = parse_mipmaps(i, width, height, format, flags)?;

        Ok((
            i,
            LeagueTexture {
                width,
                height,
                _is_extended_format_maybe,
                format,
                resource_type,
                flags,
                mipmaps,
            },
        ))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LeagueTextureFormat {
    Etc1,
    Etc2Eac,
    Bc1,
    Bc3,
    Bgra8,
}

#[derive(Debug, Clone, Copy, PartialEq)]
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

fn parse_mipmaps(
    input: &[u8],
    width: u16,
    height: u16,
    format: LeagueTextureFormat,
    flags: LeagueTextureFlags,
) -> IResult<&[u8], Vec<Vec<u8>>> {
    let mip_count = if flags.contains(LeagueTextureFlags::HasMipMaps) {
        ((width.max(height) as f32).log2().floor() as usize) + 1
    } else {
        1
    };

    let mut mipmaps = vec![Vec::new(); mip_count];
    let mut current_input = input;

    for i in (0..mip_count).rev() {
        let current_width = (width as u32 >> i).max(1) as usize;
        let current_height = (height as u32 >> i).max(1) as usize;

        let block_size = get_block_size(format);
        let (width_in_blocks, height_in_blocks) =
            calculate_block_count(format, current_width, current_height);
        let mip_size = width_in_blocks * height_in_blocks * block_size;

        let (i_next, mip_data) = take(mip_size).parse(current_input)?;
        mipmaps[i] = mip_data.to_vec();
        current_input = i_next;
    }

    Ok((current_input, mipmaps))
}
