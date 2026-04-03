use bevy::asset::{AssetLoader, LoadContext};
use bevy::image::ImageSampler;
use bevy::prelude::Image;
use bevy::render::render_resource::{
    Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};
use league_file::texture::{LeagueTexture, LeagueTextureFormat};

use super::error::Error;

#[derive(Default)]
pub struct LeagueLoaderImage;

impl AssetLoader for LeagueLoaderImage {
    type Asset = Image;

    type Settings = ();

    type Error = Error;

    async fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        _settings: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf).await?;
        let (_, texture) = LeagueTexture::parse(&buf).map_err(|e| Error::Parse(e.to_string()))?;

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
                LeagueTextureFormat::Bc1 => TextureFormat::Bc1RgbaUnorm,
                LeagueTextureFormat::Bc3 => TextureFormat::Bc3RgbaUnorm,
                _ => panic!("not bc1 or bc3 is {:?}", texture.format),
            },
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        };

        let image = Image {
            data: Some(texture.mipmaps[0].clone()),
            texture_descriptor,
            sampler: ImageSampler::linear(),
            ..Default::default()
        };

        let srgb_image = Image {
            data: Some(texture.mipmaps[0].clone()),
            texture_descriptor: TextureDescriptor {
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
            },
            sampler: ImageSampler::linear(),
            ..Default::default()
        };

        load_context.add_labeled_asset("srgb".to_string(), srgb_image);

        Ok(image)
    }

    fn extensions(&self) -> &[&str] {
        &["tex"]
    }
}
