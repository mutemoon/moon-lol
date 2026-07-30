use std::fs::File;
use std::io::Read;

use image::codecs::png::{CompressionType, FilterType};
use image::{ExtendedColorType, ImageEncoder};
use league_file::texture::{LeagueTexture, LeagueTextureFormat};
use serde::de::DeserializeOwned;
use texpresso::Format;
use thiserror::Error;

/// 将 LeagueTexture 解压为裸 RGBA8 像素（不含任何色彩空间信息）
fn decode_texture_to_rgba(texture: &LeagueTexture) -> Option<Vec<u8>> {
    let format = match texture.format {
        LeagueTextureFormat::Bc1 => Some(Format::Bc1),
        LeagueTextureFormat::Bc3 => Some(Format::Bc3),
        LeagueTextureFormat::Bgra8 => None,
        _ => return None,
    };

    let rgba_data = if let Some(f) = format {
        let mut rgba = vec![0u8; texture.width as usize * texture.height as usize * 4];
        f.decompress(
            &texture.mipmaps[0],
            texture.width as usize,
            texture.height as usize,
            &mut rgba,
        );
        rgba
    } else if texture.format == LeagueTextureFormat::Bgra8 {
        let mut data = texture.mipmaps[0].clone();
        for chunk in data.chunks_exact_mut(4) {
            chunk.swap(0, 2);
        }
        data
    } else {
        return None;
    };

    Some(rgba_data)
}

/// 将 LeagueTexture 解码为 PNG 格式（不写色彩空间块，供 UI/mesh 等按默认 sRGB 语义使用）
pub fn decode_texture_to_png(texture: &LeagueTexture) -> Option<Vec<u8>> {
    let rgba_data = decode_texture_to_rgba(texture)?;

    let mut png_data = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new_with_quality(
        &mut png_data,
        CompressionType::Fast,
        FilterType::NoFilter,
    );
    encoder
        .write_image(
            &rgba_data,
            texture.width as u32,
            texture.height as u32,
            ExtendedColorType::Rgba8,
        )
        .ok()?;
    Some(png_data)
}

/// 将 LeagueTexture 解码为「线性」PNG：写入 gAMA=1.0 块显式声明像素为线性数据。
/// 因为游戏是 gamma-space 直采、粒子贴图应按原值线性使用，
/// 所以给粒子贴图导出时标注线性，让图片查看器按线性解释、避免被当作 sRGB 显示。
pub fn decode_texture_to_linear_png(texture: &LeagueTexture) -> Option<Vec<u8>> {
    let rgba_data = decode_texture_to_rgba(texture)?;

    let mut png_data = Vec::new();
    {
        let mut encoder =
            png::Encoder::new(&mut png_data, texture.width as u32, texture.height as u32);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        // gAMA=1.0 → 线性伽马，声明存储像素即线性数据
        encoder.set_source_gamma(png::ScaledFloat::new(1.0));
        let mut writer = encoder.write_header().ok()?;
        writer.write_image_data(&rgba_data).ok()?;
        writer.finish().ok()?;
    }
    Some(png_data)
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Custom error: {0}")]
    Custom(String),

    #[error("{0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("{0}")]
    Bincode(#[from] bincode::Error),

    #[error("{0}")]
    LeagueLoader(#[from] league_loader::Error),
}

pub fn get_bin_path(path: &str) -> String {
    format!("{path}.bin")
}

pub fn get_struct_from_file<T: DeserializeOwned>(path: &str) -> Result<T, Error> {
    let mut file = File::open(format!("assets/{}", &get_bin_path(path)))?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;
    let data = bincode::deserialize(&data)?;
    Ok(data)
}
