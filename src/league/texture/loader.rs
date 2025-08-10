use crate::league::{LeagueLoaderError, LeagueTexture, LeagueTextureFormat};
use bevy::{
    asset::{AssetLoader, LoadContext, RenderAssetUsages},
    image::{Image, ImageSampler, TextureError},
    render::render_resource::{
        Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    },
};
use binrw::BinRead;
use std::io::Cursor;

#[derive(Default)]
pub struct LeagueLoaderImage;

impl AssetLoader for LeagueLoaderImage {
    type Asset = Image;

    type Settings = ();

    type Error = LeagueLoaderError;

    async fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf).await?;
        let mut reader = Cursor::new(buf);
        let texture = LeagueTexture::read(&mut reader)?;

        if texture.mipmaps.is_empty() {
            return Err(LeagueLoaderError::Texture(TextureError::InvalidData(
                "No mipmap data found.".to_string(),
            )));
        }

        let base_width = texture.width as usize;
        let base_height = texture.height as usize;
        let pixel_count = base_width * base_height;

        let texture_descriptor = TextureDescriptor {
            label: None,
            size: Extent3d {
                width: texture.width as u32,
                height: texture.height as u32,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: match texture.format {
                LeagueTextureFormat::Bc1 => TextureFormat::Bc1RgbaUnormSrgb,
                LeagueTextureFormat::Bc3 => TextureFormat::Bc3RgbaUnormSrgb,
                _ => panic!("not bc1 or bc3 is {:?}", texture.format),
            },
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        };

        let image_sampler = ImageSampler::linear();

        let image = Image {
            data: Some(match texture.format {
                LeagueTextureFormat::Bgra8 => {
                    if let Some(first_mip) = texture.mipmaps.first() {
                        let expected_size = pixel_count * 4;
                        if first_mip.len() == expected_size {
                            let mut rgba_data = Vec::with_capacity(expected_size);
                            for bgra in first_mip.chunks_exact(4) {
                                rgba_data.extend_from_slice(&[bgra[2], bgra[1], bgra[0], bgra[3]]);
                            }
                            rgba_data
                        } else {
                            vec![128u8; pixel_count * 4]
                        }
                    } else {
                        vec![128u8; pixel_count * 4]
                    }
                }
                _ => texture.mipmaps[0].clone(),
            }),
            texture_descriptor,
            sampler: image_sampler,
            texture_view_descriptor: None,
            asset_usage: RenderAssetUsages::default(),
        };

        Ok(image)
    }
}
