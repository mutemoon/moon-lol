use bevy::asset::RenderAssetUsages;
use bevy::image::{Image, ImageSampler, TextureError};
use bevy::render::render_resource::{
    Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};
use binrw::binread;
use binrw::{BinRead, BinResult, Endian};
use bitflags::bitflags;
use image::{ImageBuffer, Rgba};
use std::io::{Read, Seek};
use texpresso::Format;

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

impl LeagueTexture {
    /// 将 LeagueTexture 转换为 Bevy Image。
    ///
    /// `is_srgb` 参数表示最终 Bevy Image 是否应该是 sRGB 格式。
    /// `render_asset_usages` 定义了图像在渲染管线中的用途。
    pub fn to_bevy_image(
        &self,
        render_asset_usages: RenderAssetUsages,
    ) -> Result<Image, TextureError> {
        // 检查纹理数据是否存在
        if self.mipmaps.is_empty() {
            return Err(TextureError::InvalidData(
                "No mipmap data found.".to_string(),
            ));
        }

        // Create a simple fallback texture to avoid alignment issues
        // Use only the first mipmap level for now
        let base_width = self.width as usize;
        let base_height = self.height as usize;
        let pixel_count = base_width * base_height;

        // 2. 设置 TextureDescriptor
        let texture_descriptor = TextureDescriptor {
            label: None,
            size: Extent3d {
                width: self.width as u32,
                height: self.height as u32,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1, // Use only base mip level to avoid alignment issues
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: match self.format {
                // Use uncompressed formats for better compatibility
                LeagueTextureFormat::Bc1 => TextureFormat::Bc1RgbaUnormSrgb,
                LeagueTextureFormat::Bc3 => TextureFormat::Bc3RgbaUnormSrgb,
                _ => panic!("not bc1 or bc3 is {:?}", self.format),
            }, // 使用上面映射的格式
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        };

        // 3. 设置 ImageSampler（使用默认线性采样）
        let image_sampler = ImageSampler::linear();

        // 4. 创建 Image 实例
        let image = Image {
            data: Some(match self.format {
                LeagueTextureFormat::Bgra8 => {
                    // Convert BGRA to RGBA for the first mipmap only
                    if let Some(first_mip) = self.mipmaps.first() {
                        let expected_size = pixel_count * 4;
                        if first_mip.len() == expected_size {
                            let mut rgba_data = Vec::with_capacity(expected_size);
                            for bgra in first_mip.chunks_exact(4) {
                                rgba_data.extend_from_slice(&[bgra[2], bgra[1], bgra[0], bgra[3]]);
                            }
                            rgba_data
                        } else {
                            // Size mismatch, create fallback
                            vec![128u8; pixel_count * 4]
                        }
                    } else {
                        // No mipmap data, create fallback
                        vec![128u8; pixel_count * 4]
                    }
                }
                _ => self.mipmaps[0].clone(),
            }), // 使用连接后的完整数据
            texture_descriptor,
            sampler: image_sampler,
            texture_view_descriptor: None,
            asset_usage: render_asset_usages,
        };

        Ok(image)
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

fn league_format_to_texpresso_format(format: LeagueTextureFormat) -> Option<Format> {
    match format {
        LeagueTextureFormat::Bc1 => Some(Format::Bc1),

        LeagueTextureFormat::Bc3 => Some(Format::Bc3),
        _ => None,
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

fn save_mipmap_as_png(
    width: u32,
    height: u32,
    format: LeagueTextureFormat,
    data: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
    let width = width as usize;
    let height = height as usize;

    let mut decoded_pixels: Vec<u8> = vec![0; width * height * 4];

    match format {
        LeagueTextureFormat::Bc1 | LeagueTextureFormat::Bc3 => {
            let texpresso_format = league_format_to_texpresso_format(format).ok_or_else(|| {
                format!("Decoding for {:?} is not supported by texpresso.", format)
            })?;

            texpresso_format.decompress(data, width, height, &mut decoded_pixels);
        }
        LeagueTextureFormat::Bgra8 => {
            if data.len() != width * height * 4 {
                return Err(format!(
                    "BGRA8 data size mismatch! Expected {}, got {}.",
                    width * height * 4,
                    data.len()
                )
                .into());
            }

            for (bgra, rgba) in data.chunks_exact(4).zip(decoded_pixels.chunks_exact_mut(4)) {
                rgba[0] = bgra[2];
                rgba[1] = bgra[1];
                rgba[2] = bgra[0];
                rgba[3] = bgra[3];
            }
        }
        LeagueTextureFormat::Etc1 | LeagueTextureFormat::Etc2Eac => {
            return Err(format!("Decoding for {:?} is not supported yet.", format).into());
        }
    }

    Ok(())
}
