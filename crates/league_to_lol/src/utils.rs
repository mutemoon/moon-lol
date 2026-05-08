use image::codecs::png::{CompressionType, FilterType};
use image::{ExtendedColorType, ImageEncoder};
use league_file::texture::{LeagueTexture, LeagueTextureFormat};
use serde::de::DeserializeOwned;
use texpresso::Format;
use thiserror::Error;

use std::fs::File;
use std::io::Read;

/// 将 LeagueTexture 解码为 PNG 格式
pub fn decode_texture_to_png(texture: &LeagueTexture) -> Option<Vec<u8>> {
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
