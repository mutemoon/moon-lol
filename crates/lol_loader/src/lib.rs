use std::collections::HashMap;
use std::io::Cursor;

use bevy::animation::animation_curves::{AnimatableCurve, AnimatableKeyframeCurve};
use bevy::animation::{animated_field, AnimationClip, AnimationTargetId};
use bevy::asset::uuid::Uuid;
use bevy::asset::{AssetLoader, LoadContext};
use bevy::image::ImageSampler;
use bevy::math::bounding::Aabb3d;
use bevy::prelude::*;
use bevy::render::render_resource::{
    Extent3d, ShaderStage, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};
use bevy::shader::{ShaderImport, Source, ValidateShader};
use binrw::BinRead;
use league_core::EnvironmentVisibility;
use league_file::{
    AnimationFile, LeagueMapGeo, LeagueMeshStatic, LeagueShaderChunk, LeagueShaderToc,
    LeagueSkeleton, LeagueSkinnedMesh, LeagueTexture, LeagueTextureFormat,
};
use league_property::PropFile;
use league_to_lol::{
    convert_frag, convert_vert, load_animation_file, mesh_static_to_bevy_mesh, parse_vertex_data,
    skinned_mesh_to_intermediate, submesh_to_intermediate,
};
use league_utils::{get_shader_uuid_by_hash, hash_wad};
use lol_config::{ConfigMapGeo, LeagueProperties, ResourceShaderPackage, ASSET_LOADER_REGISTRY};
use lol_core::LeagueSkinMesh;
use regex::Regex;
use serde::{Deserialize, Serialize};
use thiserror::Error;

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
pub struct LeagueLoaderProperty;

impl AssetLoader for LeagueLoaderProperty {
    type Asset = LeagueProperties;

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
        let mut cursor = Cursor::new(buf);
        let prop_bin = PropFile::read(&mut cursor)?;

        let mut handles = HashMap::new();
        for (entry_hash, entry) in prop_bin.iter_class_hash_and_entry() {
            let Some((_, loader)) = ASSET_LOADER_REGISTRY.loaders.get(&entry_hash) else {
                continue;
            };

            let handle = loader.load_and_add(load_context, entry);

            if !handles.contains_key(&entry_hash) {
                handles.insert(entry_hash, HashMap::new());
            };

            let store = handles.get_mut(&entry_hash).unwrap();

            store.insert(entry.hash, handle);
        }

        let paths = prop_bin.links.into_iter().map(|v| v.text).collect();

        Ok(LeagueProperties(handles, paths))
    }

    fn extensions(&self) -> &[&str] {
        &["bin"]
    }
}

#[derive(Default)]
pub struct LeagueLoaderMesh;

impl AssetLoader for LeagueLoaderMesh {
    type Asset = LeagueSkinMesh;

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
        let mut cursor = Cursor::new(buf);
        let league_skinned_mesh = LeagueSkinnedMesh::read(&mut cursor)?;

        let mut submeshes = Vec::new();
        for (i, _) in league_skinned_mesh.ranges.iter().enumerate() {
            let mesh = skinned_mesh_to_intermediate(&league_skinned_mesh, i);
            submeshes.push(_load_context.add_labeled_asset(i.to_string(), mesh.into()));
        }

        Ok(LeagueSkinMesh { submeshes })
    }

    fn extensions(&self) -> &[&str] {
        &["skn"]
    }
}

#[derive(Default)]
pub struct LeagueLoaderMapgeo;

impl AssetLoader for LeagueLoaderMapgeo {
    type Asset = ConfigMapGeo;

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
        let mut cursor = Cursor::new(buf);
        let league_mapgeo = LeagueMapGeo::read(&mut cursor)?;

        let mut submeshes = Vec::new();

        for (i, map_mesh) in league_mapgeo.meshes.iter().enumerate() {
            // if map_mesh.layer_transition_behavior != LayerTransitionBehavior::Unaffected {
            //     continue;
            // }

            if !map_mesh
                .environment_visibility
                .contains(EnvironmentVisibility::Layer1)
            {
                continue;
            }

            let (all_positions, all_normals, all_uvs) = parse_vertex_data(&league_mapgeo, map_mesh);

            for (j, submesh) in map_mesh.submeshes.iter().enumerate() {
                let intermediate_meshes = submesh_to_intermediate(
                    &submesh,
                    &league_mapgeo,
                    map_mesh,
                    &all_positions,
                    &all_normals,
                    &all_uvs,
                );

                submeshes.push((
                    load_context.add_labeled_asset(format!("{i}-{j}"), intermediate_meshes.into()),
                    submesh.material_name.text.clone(),
                    Aabb3d {
                        min: map_mesh.bounding_box.min.into(),
                        max: map_mesh.bounding_box.max.into(),
                    },
                ));
            }
        }

        Ok(ConfigMapGeo { submeshes })
    }

    fn extensions(&self) -> &[&str] {
        &["mapgeo"]
    }
}

#[derive(Default)]
pub struct LeagueLoaderSkeleton;

impl AssetLoader for LeagueLoaderSkeleton {
    type Asset = LeagueSkeleton;

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
        let mut cursor = Cursor::new(buf);
        let league_skeleton = LeagueSkeleton::read(&mut cursor)?;

        Ok(league_skeleton)
    }

    fn extensions(&self) -> &[&str] {
        &["skl"]
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
        let mut reader = Cursor::new(buf);
        let animation_file = AnimationFile::read(&mut reader)?;

        let animation = load_animation_file(animation_file);

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

    fn extensions(&self) -> &[&str] {
        &["anm"]
    }
}

#[derive(Default, Serialize, Deserialize)]
pub struct ShaderTocSettings(pub String);

#[derive(Default)]
pub struct LeagueLoaderShaderToc;

impl AssetLoader for LeagueLoaderShaderToc {
    type Asset = ResourceShaderPackage;

    type Settings = ShaderTocSettings;

    type Error = Error;

    async fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        settings: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf).await?;
        let mut reader = Cursor::new(buf);

        let shader_toc = LeagueShaderToc::read(&mut reader)?;

        let path = &settings.0;

        let mut handles = HashMap::new();

        let mut chunks = Vec::new();

        let mut max_struct = "".to_string();

        let mut max_uniform_sampler = "".to_string();

        for i in 0..((shader_toc.bundled_shader_count as f32 / 100.0).ceil() as usize) {
            let chunk_hash = hash_wad(&format!("{}_{}", path, i * 100));

            let chunk = load_context
                .read_asset_bytes(&format!("data/{:x}.lol", chunk_hash))
                .await
                .unwrap();

            let shader_chunk = LeagueShaderChunk::read_le(&mut Cursor::new(chunk))?;

            for shader_file in shader_chunk.files.iter() {
                let content = shader_file.text.clone();

                // content.matches("struct")

                let re = Regex::new(r"struct[\w\W]*?\}").unwrap(); // 匹配 YYYY-MM-DD 日期格式
                let matches = re.find_iter(&content);

                for mat in matches {
                    if mat.as_str().len() > max_struct.len() {
                        max_struct = mat.as_str().to_string();
                    }
                }

                let re = Regex::new(r"uniform sampler2D[\w\W]*?\n\n").unwrap();
                let matches = re.find_iter(&content);

                for mat in matches {
                    if mat.as_str().len() > max_uniform_sampler.len() {
                        max_uniform_sampler = mat.as_str().to_string();
                    }
                }

                chunks.push(content);
            }
        }

        let mut shader_handles = Vec::new();
        for i in 0..shader_toc.bundled_shader_count {
            let mut content = chunks[i as usize].clone();

            let re = Regex::new(r"struct[\w\W]*?\}").unwrap(); // 匹配 YYYY-MM-DD 日期格式
            let matches = re.find_iter(&content);

            let ranges = matches.map(|mat| mat.range()).collect::<Vec<_>>();

            for range in ranges {
                content.replace_range(range, &max_struct);
            }

            let re = Regex::new(r"uniform sampler2D[\w\W]*?\n\n").unwrap();
            let matches = re.find_iter(&content);

            let ranges = matches.map(|mat| mat.range()).collect::<Vec<_>>();

            for range in ranges {
                content.replace_range(range, &max_uniform_sampler);
            }

            let converted = if shader_toc.shader_type == 0 {
                convert_vert(&content)
            } else {
                convert_frag(&content)
            };

            let source = if shader_toc.shader_type == 0 {
                Source::Glsl(converted.clone().into(), ShaderStage::Vertex)
            } else {
                Source::Glsl(converted.clone().into(), ShaderStage::Fragment)
            };

            let shader = Shader {
                path: path.clone(),
                imports: Default::default(),
                import_path: ShaderImport::Custom(path.clone()),
                source,
                additional_imports: Default::default(),
                shader_defs: Default::default(),
                file_dependencies: Default::default(),
                validate_shader: ValidateShader::Disabled,
            };

            shader_handles.push((
                converted.clone(),
                load_context.add_labeled_asset(i.to_string(), shader),
            ));
        }

        for (shader_index, shader_hash) in shader_toc.shader_hashes.into_iter().enumerate() {
            let shader_id = shader_toc.shader_ids[shader_index];

            let (converted, handle) = &shader_handles[shader_id as usize];

            if get_shader_uuid_by_hash(&path, shader_hash) == Uuid::from_u128(0xdee3e40ffaa02909) {
                debug!("shader_id: {}", shader_id);
                debug!("converted: {}", converted);
            }

            handles.insert(shader_hash, handle.clone());
        }

        Ok(ResourceShaderPackage { handles })
    }

    fn extensions(&self) -> &[&str] {
        &["glsl"]
    }
}
