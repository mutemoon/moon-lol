use std::io::Cursor;

use bevy::{
    animation::{
        animated_field,
        animation_curves::{AnimatableCurve, AnimatableKeyframeCurve},
        AnimationClip, AnimationTargetId,
    },
    asset::{io::Reader, uuid::Uuid, AssetLoader, LoadContext, RenderAssetUsages},
    image::ImageSampler,
    pbr::StandardMaterial,
    prelude::*,
    render::{
        alpha::AlphaMode,
        mesh::{
            skinning::SkinnedMeshInverseBindposes, Indices, PrimitiveTopology,
            VertexAttributeValues,
        },
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
    },
};
use bincode::deserialize;
use binrw::BinRead;
use league_file::{LeagueTexture, LeagueTextureFormat};
use league_utils::{neg_rotation_z, neg_vec_z};
use thiserror::Error;

use lol_config::{
    ConfigAnimationClip, ConfigSkinnedMeshInverseBindposes, IntermediateMesh, LeagueMaterial,
};

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
pub struct LeagueLoaderSkinnedMeshInverseBindposes;

impl AssetLoader for LeagueLoaderSkinnedMeshInverseBindposes {
    type Asset = SkinnedMeshInverseBindposes;

    type Settings = ();

    type Error = Error;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf).await?;
        let config: ConfigSkinnedMeshInverseBindposes = deserialize(&buf)?;
        Ok(SkinnedMeshInverseBindposes::from(config.inverse_bindposes))
    }
}

#[derive(Default)]
pub struct LeagueLoaderMaterial;

impl AssetLoader for LeagueLoaderMaterial {
    type Asset = StandardMaterial;

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
        let material: LeagueMaterial = bincode::deserialize(&buf)?;
        let image = load_context.load(material.texture_path);
        Ok(StandardMaterial {
            base_color_texture: Some(image),
            unlit: true,

            alpha_mode: AlphaMode::Mask(0.3),
            ..default()
        })
    }
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

        let mut bevy_mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        );

        // 插入必需的位置属性
        bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, mesh.positions.clone());

        // 插入可选属性
        if let Some(ref normals) = mesh.normals {
            bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals.clone());
        }

        if let Some(ref uvs) = mesh.uvs {
            bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs.clone());
        }

        if let Some(ref colors) = mesh.colors {
            bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors.clone());
        }

        if let Some(ref tangents) = mesh.tangents {
            bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_TANGENT, tangents.clone());
        }

        // 插入骨骼动画属性
        if let Some(ref joint_indices) = mesh.joint_indices {
            bevy_mesh.insert_attribute(
                Mesh::ATTRIBUTE_JOINT_INDEX,
                VertexAttributeValues::Uint16x4(joint_indices.clone()),
            );
        }

        if let Some(ref joint_weights) = mesh.joint_weights {
            bevy_mesh.insert_attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT, joint_weights.clone());
        }

        let indices = mesh.indices.clone();

        // 插入索引
        bevy_mesh.insert_indices(Indices::U16(indices));

        Ok(bevy_mesh)
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
        _load_context: &mut LoadContext<'_>,
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

        let image_sampler = ImageSampler::linear();

        let image = Image {
            data: Some(texture.mipmaps[0].clone()),
            texture_descriptor,
            sampler: image_sampler,
            texture_view_descriptor: None,
            asset_usage: RenderAssetUsages::default(),
        };

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
                        AnimatableKeyframeCurve::new(
                            translates.iter().map(|(time, vec)| (*time, neg_vec_z(vec))),
                        )
                        .unwrap(),
                    ),
                );
            }

            if rotations.len() >= 2 {
                clip.add_curve_to_target(
                    AnimationTargetId(Uuid::from_u128(*join_hash as u128)),
                    AnimatableCurve::new(
                        animated_field!(Transform::rotation),
                        AnimatableKeyframeCurve::new(
                            rotations
                                .iter()
                                .map(|(time, quat)| (*time, neg_rotation_z(quat))),
                        )
                        .unwrap(),
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
