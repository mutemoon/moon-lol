use std::io::Cursor;

use bevy::{
    animation::{
        animated_field,
        animation_curves::{AnimatableCurve, AnimatableKeyframeCurve},
        AnimationClip, AnimationTargetId,
    },
    asset::{io::Reader, uuid::Uuid, AssetLoader, LoadContext},
    image::ImageSampler,
    mesh::skinning::SkinnedMeshInverseBindposes,
    pbr::StandardMaterial,
    prelude::*,
    render::render_resource::{
        Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    },
};
use binrw::BinRead;
use thiserror::Error;

use league_file::{LeagueMeshStatic, LeagueTexture, LeagueTextureFormat};
use league_to_lol::mesh_static_to_bevy_mesh;
use lol_config::{ConfigAnimationClip, IntermediateMesh};

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Custom(String),

    #[error("{0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Bincode(#[from] bincode::Error),

    #[error("{0}")]
    BinRead(#[from] binrw::Error),
}

#[derive(Default)]
pub struct LeagueLoaderMesh;

impl AssetLoader for LeagueLoaderMesh {
    type Asset = Mesh;

    type Settings = ();

    type Error = Error;

    async fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf).await?;
        let mesh: IntermediateMesh = bincode::deserialize(&buf)?;
        Ok(mesh.into())
    }

    fn extensions(&self) -> &[&str] {
        &["mesh"]
    }
}

#[derive(Default)]
pub struct LeagueLoaderMeshStatic;

impl AssetLoader for LeagueLoaderMeshStatic {
    type Asset = Mesh;

    type Settings = ();

    type Error = Error;

    async fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf).await?;
        let mut reader = Cursor::new(buf);
        let mesh = LeagueMeshStatic::read(&mut reader)?;
        Ok(mesh_static_to_bevy_mesh(mesh))
    }

    fn extensions(&self) -> &[&str] {
        &["scb"]
    }
}

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
        let mut reader = Cursor::new(buf);
        let texture = LeagueTexture::read(&mut reader)?;

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
            ..default()
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
            ..default()
        };

        load_context.add_labeled_asset("srgb".to_string(), srgb_image);

        Ok(image)
    }

    fn extensions(&self) -> &[&str] {
        &["tex"]
    }
}

#[derive(Default)]
pub struct LeagueLoaderAnimationClip;

impl AssetLoader for LeagueLoaderAnimationClip {
    type Asset = AnimationClip;

    type Settings = ();

    type Error = Error;

    async fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf).await?;
        let animation: ConfigAnimationClip = bincode::deserialize(&buf)?;

        let mut clip = AnimationClip::default();
        for (i, join_hash) in animation.joint_hashes.iter().enumerate() {
            let translates = animation.translates.get(i).unwrap();
            let rotations = animation.rotations.get(i).unwrap();
            let scales = animation.scales.get(i).unwrap();

            if translates.len() >= 2 {
                clip.add_curve_to_target(
                    AnimationTargetId(Uuid::from_u128(*join_hash as u128)),
                    AnimatableCurve::new(
                        animated_field!(Transform::translation),
                        AnimatableKeyframeCurve::new(translates.clone()).unwrap(),
                    ),
                );
            }

            if rotations.len() >= 2 {
                clip.add_curve_to_target(
                    AnimationTargetId(Uuid::from_u128(*join_hash as u128)),
                    AnimatableCurve::new(
                        animated_field!(Transform::rotation),
                        AnimatableKeyframeCurve::new(rotations.clone()).unwrap(),
                    ),
                );
            }

            if scales.len() >= 2 {
                clip.add_curve_to_target(
                    AnimationTargetId(Uuid::from_u128(*join_hash as u128)),
                    AnimatableCurve::new(
                        animated_field!(Transform::scale),
                        AnimatableKeyframeCurve::new(scales.clone().into_iter()).unwrap(),
                    ),
                );
            }
        }
        Ok(clip)
    }
}
