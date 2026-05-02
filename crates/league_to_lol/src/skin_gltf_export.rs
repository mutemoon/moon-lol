use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap};

use bevy::prelude::*;
use gltf::accessor::DataType;
use gltf::json::accessor::{Accessor, GenericComponentType, Type};
use gltf::json::animation::{
    Animation, Channel, Interpolation, Property, Sampler, Target as AnimationTarget,
};
use gltf::json::buffer::{Buffer, Target, View};
use gltf::json::image::Image;
use gltf::json::material::{Material, PbrBaseColorFactor, PbrMetallicRoughness, StrengthFactor};
use gltf::json::mesh::{Mesh, Mode, Primitive, Semantic};
use gltf::json::scene::{Scene, UnitQuaternion};
use gltf::json::skin::Skin as JsonSkin;
use gltf::json::texture::{Info, Texture};
use gltf::json::validation::{Checked, USize64};
use gltf::json::{Index, Node, Root};
use image::codecs::png::{CompressionType, FilterType};
use image::{ExtendedColorType, ImageEncoder};
use league_file::mesh_skinned::LeagueSkinnedMesh;
use league_file::skeleton::LeagueSkeleton;
use league_file::texture::{LeagueTexture, LeagueTextureFormat};
use league_utils::{hash_joint, hash_to_type_name};
use lol_base::animation::ConfigAnimationClip;
use texpresso::Format;

use crate::utils::Error;

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

/// 将角色皮肤（网格 + 材质 + 贴图 + 动画）导出为 GLB 文件
/// animations: map of (clip_name, ConfigAnimationClip)
/// material_override: map of submesh_name → texture_png，用于覆盖特定 submesh 的贴图
pub fn export_skin_to_glb(
    skinned_mesh: &LeagueSkinnedMesh,
    texture_png: Option<Vec<u8>>,
    skeleton: Option<&LeagueSkeleton>,
    animations: &[(u32, ConfigAnimationClip)],
    output_path: &str,
    material_override: Option<&std::collections::HashMap<String, Vec<u8>>>,
    hashes: &HashMap<u32, String>,
) -> Result<(), Error> {
    let mut builder = SkinGltfBuilder::new();

    // 添加默认贴图和材质
    let default_material_index = builder.add_material(texture_png.clone(), "skin_material");

    // 为每个 submesh 创建 primitive，根据 submesh 名称使用对应的材质
    let mut primitives = Vec::new();
    for (i, range) in skinned_mesh.ranges.iter().enumerate() {
        // 获取 submesh 名称
        let submesh_name = &range.name;

        println!(
            "{} - {:?}",
            submesh_name,
            material_override.map(|v| v.keys().map(|v| v.to_string()).collect::<Vec<_>>())
        );

        // 检查是否有材质覆盖
        let material_index = if let Some(overrides) = material_override {
            if let Some(override_texture) = overrides.get(submesh_name) {
                // 使用覆盖的贴图创建新材质
                builder.add_material(Some(override_texture.clone()), submesh_name)
            } else {
                default_material_index
            }
        } else {
            default_material_index
        };

        let primitive = builder.create_primitive(skinned_mesh, i, material_index)?;
        primitives.push(primitive);
    }

    // 如果没有 ranges（version 0），整个 mesh 作为一个 primitive
    if skinned_mesh.ranges.is_empty() {
        let primitive = builder.create_full_mesh_primitive(skinned_mesh, default_material_index)?;
        primitives.push(primitive);
    }

    if primitives.is_empty() {
        return Err(Error::Parse("没有可导出的网格数据".to_string()));
    }

    // 创建 mesh
    let mesh = Mesh {
        name: Some("skin_mesh".to_string()),
        primitives,
        extensions: None,
        extras: Default::default(),
        weights: None,
    };
    builder.meshes.push(mesh);

    // 如果有 skeleton，创建关节节点和 Skin
    let (_mesh_node_idx, _skin_idx): (u32, Option<u32>) = if let Some(skel) = skeleton {
        let (mesh_idx, skin_idx) = builder.create_skeleton_nodes(skel)?;
        (mesh_idx, Some(skin_idx))
    } else {
        // 没有 skeleton，只创建单个 mesh 节点
        let node_idx = builder.nodes.len() as u32;
        let node = Node {
            name: Some("skin_node".to_string()),
            camera: None,
            children: None,
            extensions: None,
            extras: Default::default(),
            matrix: None,
            mesh: Some(Index::new(0)),
            rotation: None,
            scale: None,
            translation: None,
            weights: None,
            skin: None,
        };
        builder.nodes.push(node);
        builder.root_node_indices.push(node_idx);
        (node_idx, None)
    };

    // 添加动画
    for (name, clip) in animations {
        builder.add_animation(clip, &hash_to_type_name(name, hashes), skeleton);
    }

    builder.write_to_glb(output_path)
}

struct SkinGltfBuilder {
    buffer_data: Vec<u8>,
    accessors: Vec<Accessor>,
    buffer_views: Vec<View>,
    meshes: Vec<Mesh>,
    nodes: Vec<Node>,
    images: Vec<Image>,
    textures: Vec<Texture>,
    materials: Vec<Material>,
    animations: Vec<Animation>,
    skins: Vec<gltf::json::skin::Skin>,
    root_node_indices: Vec<u32>,
    joint_hash_to_node: HashMap<u32, u32>,
}

/// 从 ConfigAnimationClip 的 mask_weights + 骨架构建 joint_hash → max_weight 映射
/// 返回 None 表示无蒙版（全部关节驱动）
fn build_joint_mask_map(
    clip: &ConfigAnimationClip,
    skeleton: &LeagueSkeleton,
) -> Option<HashMap<u32, f32>> {
    let weights = clip.mask_weights.as_ref()?;
    let influences = &skeleton.modern_data.influences;
    let joints = &skeleton.modern_data.joints;

    let mut joint_to_max_weight: HashMap<i16, f32> = HashMap::new();
    for (skin_bone_idx, &weight) in weights.iter().enumerate() {
        if let Some(&joint_idx) = influences.get(skin_bone_idx) {
            if joint_idx >= 0 && (joint_idx as usize) < joints.len() {
                let max_w = joint_to_max_weight.entry(joint_idx).or_insert(0.0);
                *max_w = max_w.max(weight);
            }
        }
    }

    let mut hash_to_weight = HashMap::new();
    for (&joint_idx, &weight) in &joint_to_max_weight {
        let joint_hash = hash_joint(&joints[joint_idx as usize].name);
        hash_to_weight.insert(joint_hash, weight);
    }
    Some(hash_to_weight)
}

impl SkinGltfBuilder {
    fn new() -> Self {
        Self {
            buffer_data: Vec::new(),
            accessors: Vec::new(),
            buffer_views: Vec::new(),
            meshes: Vec::new(),
            nodes: Vec::new(),
            images: Vec::new(),
            textures: Vec::new(),
            materials: Vec::new(),
            animations: Vec::new(),
            skins: Vec::new(),
            root_node_indices: Vec::new(),
            joint_hash_to_node: HashMap::new(),
        }
    }

    /// 创建骨架节点和 Skin 对象，返回 (mesh_node_index, skin_index)
    fn create_skeleton_nodes(&mut self, skeleton: &LeagueSkeleton) -> Result<(u32, u32), Error> {
        let joints = &skeleton.modern_data.joints;
        let joint_count = joints.len();
        let influences = &skeleton.modern_data.influences;

        let mut joint_vec_index_to_node: Vec<u32> = Vec::with_capacity(joint_count);

        // 创建关节节点，使用 local_transform 作为节点的 TRS
        for joint in joints {
            let node_idx = self.nodes.len() as u32;
            joint_vec_index_to_node.push(node_idx);

            let (scale, rotation, translation) =
                joint.local_transform.to_scale_rotation_translation();

            // 建立 hash → 节点索引映射，用于动画匹配
            let joint_hash = hash_joint(&joint.name);
            self.joint_hash_to_node.insert(joint_hash, node_idx);

            let node = Node {
                name: Some(joint.name.clone()),
                camera: None,
                children: None,
                extensions: None,
                extras: Default::default(),
                matrix: None,
                mesh: None,
                rotation: Some(UnitQuaternion([
                    rotation.x, rotation.y, rotation.z, rotation.w,
                ])),
                scale: Some([scale.x, scale.y, scale.z]),
                translation: Some([translation.x, translation.y, translation.z]),
                weights: None,
                skin: None,
            };
            self.nodes.push(node);
        }

        // 设置父子关系 - 使用 Vec 索引
        for (i, joint) in joints.iter().enumerate() {
            if joint.parent_index >= 0 {
                let parent_idx = joint.parent_index as usize;
                if parent_idx < joint_count {
                    let child_node_idx = joint_vec_index_to_node[i];
                    let parent_node_idx = joint_vec_index_to_node[parent_idx];

                    // 给父节点添加子节点
                    let parent_node = &mut self.nodes[parent_node_idx as usize];
                    let mut children = parent_node.children.take().unwrap_or_default();
                    children.push(Index::new(child_node_idx));
                    parent_node.children = Some(children);
                }
            }
        }

        // 收集原始骨架的根关节（parent_index == -1）
        let skeleton_root_indices: Vec<u32> = joints
            .iter()
            .enumerate()
            .filter(|(_, j)| j.parent_index == -1)
            .map(|(i, _)| joint_vec_index_to_node[i])
            .collect();

        // 原始代码逻辑: skin.joints 和 IBM 都按 influences 顺序
        // skin.joints[i] = index_to_entity[influences[i]]
        // IBM[i] = joints[influences[i]].inverse_bind_transform
        let ordered_joint_indices: Vec<u32> = influences
            .iter()
            .map(|&joint_vec_idx| joint_vec_index_to_node[joint_vec_idx as usize])
            .collect();

        let ibm_accessor_idx = self.add_inverse_bind_matrices_ordered_by_influences(skeleton)?;

        // 创建 Skin（skeleton 稍后设置为统一根节点）
        let skin_idx = self.skins.len() as u32;
        let skin = JsonSkin {
            extensions: None,
            extras: Default::default(),
            inverse_bind_matrices: Some(Index::new(ibm_accessor_idx)),
            joints: ordered_joint_indices
                .iter()
                .map(|&i| Index::new(i))
                .collect(),
            name: Some("armature".to_string()),
            skeleton: None, // 将在创建统一根节点后设置
        };
        self.skins.push(skin);

        // 创建 mesh 节点，关联到 skin
        let mesh_node_idx = self.nodes.len() as u32;
        let node = Node {
            name: Some("skin_node".to_string()),
            camera: None,
            children: None,
            extensions: None,
            extras: Default::default(),
            matrix: None,
            mesh: Some(Index::new(0)),
            rotation: None,
            scale: None,
            translation: None,
            weights: None,
            skin: Some(Index::new(skin_idx)),
        };
        self.nodes.push(node);

        // 创建统一的根节点，父所有骨架根关节 + mesh 节点
        let root_node_idx = self.nodes.len() as u32;
        let mut root_children: Vec<Index<Node>> = skeleton_root_indices
            .iter()
            .map(|&i| Index::new(i))
            .collect();
        root_children.push(Index::new(mesh_node_idx));

        let root_node = Node {
            name: Some("root".to_string()),
            camera: None,
            children: Some(root_children),
            extensions: None,
            extras: Default::default(),
            matrix: None,
            mesh: None,
            rotation: None,
            scale: None,
            translation: None,
            weights: None,
            skin: None,
        };
        self.nodes.push(root_node);

        // 更新 Skin 的 skeleton 指向统一根节点
        self.skins[skin_idx as usize].skeleton = Some(Index::new(root_node_idx));

        // 场景根节点只有一个：统一根节点
        self.root_node_indices.push(root_node_idx);

        Ok((mesh_node_idx, skin_idx))
    }

    /// 添加 inverse bind matrices accessor - 按 influences 顺序排列
    fn add_inverse_bind_matrices_ordered_by_influences(
        &mut self,
        skeleton: &LeagueSkeleton,
    ) -> Result<u32, Error> {
        let joints = &skeleton.modern_data.joints;
        let influences = &skeleton.modern_data.influences;

        self.align_to_4();
        let offset = self.buffer_data.len();

        for &joint_vec_idx in influences {
            let ibm = joints[joint_vec_idx as usize].inverse_bind_transform;
            for i in 0..16 {
                let val = ibm.to_cols_array()[i];
                self.buffer_data.extend_from_slice(&val.to_le_bytes());
            }
        }

        let count = influences.len();
        let byte_length = count * 16 * 4;
        let view_idx = self.buffer_views.len() as u32;
        self.buffer_views.push(View {
            name: Some("inverseBindMatrices".to_string()),
            buffer: Index::new(0),
            byte_offset: Some(USize64(offset as u64)),
            byte_length: USize64(byte_length as u64),
            byte_stride: None,
            target: None,
            extensions: None,
            extras: Default::default(),
        });

        let accessor_idx = self.accessors.len() as u32;
        self.accessors.push(Accessor {
            buffer_view: Some(Index::new(view_idx)),
            byte_offset: Some(USize64(0)),
            component_type: Checked::Valid(GenericComponentType(DataType::F32)),
            count: USize64(count as u64),
            type_: Checked::Valid(Type::Mat4),
            extensions: None,
            extras: Default::default(),
            min: None,
            max: None,
            name: Some("inverseBindMatrices".to_string()),
            normalized: false,
            sparse: None,
        });

        Ok(accessor_idx)
    }

    fn add_animation(
        &mut self,
        clip: &ConfigAnimationClip,
        name: &str,
        skeleton: Option<&LeagueSkeleton>,
    ) {
        let mut samplers = Vec::new();
        let mut channels = Vec::new();

        let mask_map = skeleton.and_then(|skel| build_joint_mask_map(clip, skel));

        for joint_idx in 0..clip.joint_hashes.len() {
            // 蒙版过滤：weight == 0 的关节不写 animation channel，留在 rest pose
            if let Some(ref mask) = mask_map {
                match mask.get(&clip.joint_hashes[joint_idx]) {
                    Some(&w) if w == 0.0 => continue,
                    _ => {}
                }
            }
            self.process_joint_channels(clip, joint_idx, &mut samplers, &mut channels);
        }

        if samplers.is_empty() {
            println!(
                "[gltf_export] Animation '{}': ALL channels filtered out, using idle fallback",
                name
            );
            // Use 2 keyframes so Bevy creates an UnevenSampleAutoCurve with finite domain,
            // which extends AnimationClip::duration > 0 and prevents seek_time NaN.
            let idle_times = [0.0_f32, 0.0001_f32];
            let idle_values = [[0.0, 0.0, 0.0, 1.0], [0.0, 0.0, 0.0, 1.0]];
            let input_idx = self.add_float_accessor(&idle_times);
            let output_idx = self.add_vec4_accessor(&idle_values, None);
            samplers.push(Sampler {
                input: Index::new(input_idx),
                interpolation: Checked::Valid(Interpolation::Linear),
                output: Index::new(output_idx),
                extensions: None,
                extras: Default::default(),
            });
            // 给根节点添加一个 rotation channel，使用单位四元数不影响姿态
            if let Some(&root_node) = self.root_node_indices.first() {
                channels.push(Channel {
                    sampler: Index::new(0),
                    target: AnimationTarget {
                        node: Index::new(root_node),
                        path: Checked::Valid(Property::Rotation),
                        extensions: None,
                        extras: Default::default(),
                    },
                    extensions: None,
                    extras: Default::default(),
                });
            }
        }

        self.animations.push(Animation {
            name: Some(name.to_string()),
            samplers,
            channels,
            extensions: None,
            extras: Default::default(),
        });
    }

    fn process_joint_channels(
        &mut self,
        clip: &ConfigAnimationClip,
        joint_idx: usize,
        samplers: &mut Vec<Sampler>,
        channels: &mut Vec<Channel>,
    ) {
        let joint_hash = clip.joint_hashes[joint_idx];
        let Some(&node_idx) = self.joint_hash_to_node.get(&joint_hash) else {
            return;
        };

        // 处理平移
        if let Some(data) = clip.translates.get(joint_idx).filter(|v| v.len() >= 2) {
            let times: Vec<f32> = data.iter().map(|(t, _)| *t).collect();
            let values: Vec<[f32; 3]> = data.iter().map(|(_, v)| [v.x, v.y, v.z]).collect();
            self.push_channel(
                node_idx,
                Property::Translation,
                &times,
                &values,
                samplers,
                channels,
            );
        }

        // 处理旋转
        if let Some(data) = clip.rotations.get(joint_idx).filter(|v| v.len() >= 2) {
            let times: Vec<f32> = data.iter().map(|(t, _)| *t).collect();
            let values: Vec<[f32; 4]> = data.iter().map(|(_, q)| [q.x, q.y, q.z, q.w]).collect();
            self.push_channel_quat(
                node_idx,
                Property::Rotation,
                &times,
                &values,
                samplers,
                channels,
            );
        }

        // 处理缩放
        if let Some(data) = clip.scales.get(joint_idx).filter(|v| v.len() >= 2) {
            let times: Vec<f32> = data.iter().map(|(t, _)| *t).collect();
            let values: Vec<[f32; 3]> = data.iter().map(|(_, v)| [v.x, v.y, v.z]).collect();
            self.push_channel(
                node_idx,
                Property::Scale,
                &times,
                &values,
                samplers,
                channels,
            );
        }
    }

    /// Sanitize keyframe data: filter NaN/Inf, sort by time, deduplicate, and ensure
    /// strictly increasing timestamps. Returns None if fewer than 2 valid keyframes remain.
    fn sanitize_keyframes<T: Copy>(times: &[f32], values: &[T]) -> Option<(Vec<f32>, Vec<T>)> {
        // Pair times and values, filtering out non-finite timestamps
        let mut pairs: Vec<(f32, T)> = times
            .iter()
            .copied()
            .zip(values.iter().copied())
            .filter(|(t, _)| t.is_finite())
            .collect();

        if pairs.len() < 2 {
            return None;
        }

        pairs.sort_by(|(a, _), (b, _)| a.total_cmp(b));

        let mut clean_times = Vec::with_capacity(pairs.len());
        let mut clean_values = Vec::with_capacity(pairs.len());

        for (t, v) in pairs {
            if let Some(&last_t) = clean_times.last() {
                if t <= last_t {
                    clean_times.push(last_t + 0.0001);
                } else {
                    clean_times.push(t);
                }
            } else {
                clean_times.push(t);
            }
            clean_values.push(v);
        }

        Some((clean_times, clean_values))
    }

    fn push_channel(
        &mut self,
        node_idx: u32,
        path: Property,
        times: &[f32],
        values: &[[f32; 3]],
        samplers: &mut Vec<Sampler>,
        channels: &mut Vec<Channel>,
    ) {
        let Some((clean_times, clean_values)) = Self::sanitize_keyframes(times, values) else {
            return;
        };
        let input_idx = self.add_float_accessor(&clean_times);
        let output_idx = self.add_vec3_accessor(&clean_values, false, None);
        let sampler_idx = samplers.len() as u32;

        samplers.push(Sampler {
            input: Index::new(input_idx),
            interpolation: Checked::Valid(Interpolation::Linear),
            output: Index::new(output_idx),
            extensions: None,
            extras: Default::default(),
        });

        channels.push(Channel {
            sampler: Index::new(sampler_idx),
            target: AnimationTarget {
                node: Index::new(node_idx),
                path: Checked::Valid(path),
                extensions: None,
                extras: Default::default(),
            },
            extensions: None,
            extras: Default::default(),
        });
    }

    fn push_channel_quat(
        &mut self,
        node_idx: u32,
        path: Property,
        times: &[f32],
        values: &[[f32; 4]],
        samplers: &mut Vec<Sampler>,
        channels: &mut Vec<Channel>,
    ) {
        let Some((clean_times, clean_values)) = Self::sanitize_keyframes(times, values) else {
            return;
        };
        let input_idx = self.add_float_accessor(&clean_times);
        let output_idx = self.add_vec4_accessor(&clean_values, None);
        let sampler_idx = samplers.len() as u32;

        samplers.push(Sampler {
            input: Index::new(input_idx),
            interpolation: Checked::Valid(Interpolation::Linear),
            output: Index::new(output_idx),
            extensions: None,
            extras: Default::default(),
        });

        channels.push(Channel {
            sampler: Index::new(sampler_idx),
            target: AnimationTarget {
                node: Index::new(node_idx),
                path: Checked::Valid(path),
                extensions: None,
                extras: Default::default(),
            },
            extensions: None,
            extras: Default::default(),
        });
    }

    fn add_float_accessor(&mut self, data: &[f32]) -> u32 {
        self.align_to_4();
        let offset = self.buffer_data.len();

        for &v in data {
            self.buffer_data.extend_from_slice(&v.to_le_bytes());
        }

        let byte_length = data.len() * 4;
        let view_idx = self.buffer_views.len() as u32;
        self.buffer_views.push(View {
            name: None,
            buffer: Index::new(0),
            byte_offset: Some(USize64(offset as u64)),
            byte_length: USize64(byte_length as u64),
            byte_stride: None,
            target: None, // Animation sampler data uses no specific target
            extensions: None,
            extras: Default::default(),
        });

        // Compute min/max for animation input accessors (required by spec)
        // For SCALAR type, min/max must be arrays with one element
        let min_val = data.iter().cloned().fold(f32::MAX, f32::min);
        let max_val = data.iter().cloned().fold(f32::MIN, f32::max);

        let accessor_idx = self.accessors.len() as u32;
        self.accessors.push(Accessor {
            buffer_view: Some(Index::new(view_idx)),
            byte_offset: Some(USize64(0)),
            component_type: Checked::Valid(GenericComponentType(DataType::F32)),
            count: USize64(data.len() as u64),
            type_: Checked::Valid(Type::Scalar),
            extensions: None,
            extras: Default::default(),
            min: Some(serde_json::to_value(vec![min_val]).unwrap()),
            max: Some(serde_json::to_value(vec![max_val]).unwrap()),
            name: None,
            normalized: false,
            sparse: None,
        });

        accessor_idx
    }

    fn add_vec4_accessor(&mut self, data: &[[f32; 4]], target: Option<Target>) -> u32 {
        self.align_to_4();
        let offset = self.buffer_data.len();

        for v in data {
            self.buffer_data.extend_from_slice(&v[0].to_le_bytes());
            self.buffer_data.extend_from_slice(&v[1].to_le_bytes());
            self.buffer_data.extend_from_slice(&v[2].to_le_bytes());
            self.buffer_data.extend_from_slice(&v[3].to_le_bytes());
        }

        let byte_length = data.len() * 16;
        let view_idx = self.buffer_views.len() as u32;
        self.buffer_views.push(View {
            name: None,
            buffer: Index::new(0),
            byte_offset: Some(USize64(offset as u64)),
            byte_length: USize64(byte_length as u64),
            byte_stride: None,
            target: target.map(|t| Checked::Valid(t)),
            extensions: None,
            extras: Default::default(),
        });

        let accessor_idx = self.accessors.len() as u32;
        self.accessors.push(Accessor {
            buffer_view: Some(Index::new(view_idx)),
            byte_offset: Some(USize64(0)),
            component_type: Checked::Valid(GenericComponentType(DataType::F32)),
            count: USize64(data.len() as u64),
            type_: Checked::Valid(Type::Vec4),
            extensions: None,
            extras: Default::default(),
            min: None,
            max: None,
            name: None,
            normalized: false,
            sparse: None,
        });

        accessor_idx
    }

    fn align_to_4(&mut self) {
        let padding = (4 - (self.buffer_data.len() % 4)) % 4;
        if padding > 0 {
            self.buffer_data.resize(self.buffer_data.len() + padding, 0);
        }
    }

    /// 添加材质（可选贴图），返回材质索引
    /// name: 材质名称，用于区分不同 submesh 的材质
    fn add_material(&mut self, texture_png: Option<Vec<u8>>, name: &str) -> u32 {
        let mut pbr = PbrMetallicRoughness {
            base_color_factor: PbrBaseColorFactor([1.0, 1.0, 1.0, 1.0]),
            base_color_texture: None,
            metallic_factor: StrengthFactor(0.0),
            roughness_factor: StrengthFactor(1.0),
            metallic_roughness_texture: None,
            extensions: None,
            extras: Default::default(),
        };

        if let Some(png_data) = texture_png {
            if !png_data.is_empty() {
                self.align_to_4();
                let offset = self.buffer_data.len();
                self.buffer_data.extend_from_slice(&png_data);

                let view_idx = self.buffer_views.len() as u32;
                self.buffer_views.push(View {
                    name: Some("texture_view".to_string()),
                    buffer: Index::new(0),
                    byte_offset: Some(USize64(offset as u64)),
                    byte_length: USize64(png_data.len() as u64),
                    byte_stride: None,
                    target: None,
                    extensions: None,
                    extras: Default::default(),
                });

                let image_idx = self.images.len() as u32;
                self.images.push(Image {
                    name: Some(format!("{}_texture", name)),
                    uri: None,
                    buffer_view: Some(Index::new(view_idx)),
                    mime_type: Some(gltf::json::image::MimeType("image/png".to_string())),
                    extensions: None,
                    extras: Default::default(),
                });

                let tex_idx = self.textures.len() as u32;
                self.textures.push(Texture {
                    name: Some(format!("{}_tex", name)),
                    sampler: None,
                    source: Index::new(image_idx),
                    extensions: None,
                    extras: Default::default(),
                });

                pbr.base_color_texture = Some(Info {
                    index: Index::new(tex_idx),
                    tex_coord: 0,
                    extensions: None,
                    extras: Default::default(),
                });
            }
        }

        let material_idx = self.materials.len() as u32;
        self.materials.push(Material {
            name: Some(name.to_string()),
            pbr_metallic_roughness: pbr,
            normal_texture: None,
            occlusion_texture: None,
            emissive_texture: None,
            emissive_factor: Default::default(),
            alpha_mode: Checked::Valid(gltf::json::material::AlphaMode::Mask),
            alpha_cutoff: Some(gltf::json::material::AlphaCutoff(0.3)),
            double_sided: false,
            extensions: None,
            extras: Default::default(),
        });

        material_idx
    }

    /// 为指定 submesh 创建 primitive
    fn create_primitive(
        &mut self,
        skinned_mesh: &LeagueSkinnedMesh,
        submesh_index: usize,
        material_index: u32,
    ) -> Result<Primitive, Error> {
        let range = &skinned_mesh.ranges[submesh_index];
        let vertex_size = skinned_mesh.vertex_declaration.get_vertex_size() as usize;

        // 解析顶点数据
        let vertex_start = range.start_vertex as usize * vertex_size;
        let vertex_end = vertex_start + (range.vertex_count as usize * vertex_size);
        let vertex_slice = &skinned_mesh.vertex_buffer[vertex_start..vertex_end];

        let (positions, normals, uvs, bone_indices, bone_weights) =
            Self::parse_vertices(vertex_slice, vertex_size);

        // 解析索引数据
        let index_start = range.start_index as usize * 2;
        let index_end = index_start + (range.index_count as usize * 2);
        let index_slice = &skinned_mesh.index_buffer[index_start..index_end];

        let indices: Vec<u16> = index_slice
            .chunks_exact(2)
            .map(|b| u16::from_le_bytes(b.try_into().unwrap()))
            .map(|idx| idx - range.start_vertex as u16)
            .collect();

        self.build_primitive(
            &positions,
            &normals,
            &uvs,
            &bone_indices,
            &bone_weights,
            &indices,
            material_index,
        )
    }

    /// 为整个 mesh 创建 primitive（version 0 没有 ranges）
    fn create_full_mesh_primitive(
        &mut self,
        skinned_mesh: &LeagueSkinnedMesh,
        material_index: u32,
    ) -> Result<Primitive, Error> {
        let vertex_size = skinned_mesh.vertex_declaration.get_vertex_size() as usize;
        let (positions, normals, uvs, bone_indices, bone_weights) =
            Self::parse_vertices(&skinned_mesh.vertex_buffer, vertex_size);

        let indices: Vec<u16> = skinned_mesh
            .index_buffer
            .chunks_exact(2)
            .map(|b| u16::from_le_bytes(b.try_into().unwrap()))
            .collect();

        self.build_primitive(
            &positions,
            &normals,
            &uvs,
            &bone_indices,
            &bone_weights,
            &indices,
            material_index,
        )
    }

    /// 从顶点 buffer 解析 position/normal/uv/bone_indices/bone_weights
    fn parse_vertices(
        vertex_data: &[u8],
        vertex_size: usize,
    ) -> (
        Vec<[f32; 3]>,
        Vec<[f32; 3]>,
        Vec<[f32; 2]>,
        Vec<[u16; 4]>, // Changed from u8 to u16 to match original code
        Vec<[f32; 4]>,
    ) {
        let vertex_count = vertex_data.len() / vertex_size;
        let mut positions = Vec::with_capacity(vertex_count);
        let mut normals = Vec::with_capacity(vertex_count);
        let mut uvs = Vec::with_capacity(vertex_count);
        let mut bone_indices = Vec::with_capacity(vertex_count);
        let mut bone_weights = Vec::with_capacity(vertex_count);

        for chunk in vertex_data.chunks_exact(vertex_size) {
            // Position: offset 0, 12 bytes
            let px = f32::from_le_bytes(chunk[0..4].try_into().unwrap());
            let py = f32::from_le_bytes(chunk[4..8].try_into().unwrap());
            let pz = f32::from_le_bytes(chunk[8..12].try_into().unwrap());
            positions.push([px, py, pz]);

            // Bone indices: offset 12, 4 bytes (4 u8) - convert to u16 like original code
            bone_indices.push([
                chunk[12] as u16,
                chunk[13] as u16,
                chunk[14] as u16,
                chunk[15] as u16,
            ]);

            // Bone weights: offset 16, 16 bytes (4 f32)
            let bw0 = f32::from_le_bytes(chunk[16..20].try_into().unwrap());
            let bw1 = f32::from_le_bytes(chunk[20..24].try_into().unwrap());
            let bw2 = f32::from_le_bytes(chunk[24..28].try_into().unwrap());
            let bw3 = f32::from_le_bytes(chunk[28..32].try_into().unwrap());
            bone_weights.push([bw0, bw1, bw2, bw3]);

            let normal_offset = 32;
            let nx =
                f32::from_le_bytes(chunk[normal_offset..normal_offset + 4].try_into().unwrap());
            let ny = f32::from_le_bytes(
                chunk[normal_offset + 4..normal_offset + 8]
                    .try_into()
                    .unwrap(),
            );
            let nz = f32::from_le_bytes(
                chunk[normal_offset + 8..normal_offset + 12]
                    .try_into()
                    .unwrap(),
            );
            normals.push([nx, ny, nz]);

            // UV: offset 44, 8 bytes
            let uv_offset = 44;
            let u = f32::from_le_bytes(chunk[uv_offset..uv_offset + 4].try_into().unwrap());
            let v = f32::from_le_bytes(chunk[uv_offset + 4..uv_offset + 8].try_into().unwrap());
            uvs.push([u, v]);
        }

        (positions, normals, uvs, bone_indices, bone_weights)
    }

    /// 构建 GLTF Primitive
    fn build_primitive(
        &mut self,
        positions: &[[f32; 3]],
        normals: &[[f32; 3]],
        uvs: &[[f32; 2]],
        bone_indices: &[[u16; 4]], // Changed from u8 to u16
        bone_weights: &[[f32; 4]],
        indices: &[u16],
        material_index: u32,
    ) -> Result<Primitive, Error> {
        let mut attributes = BTreeMap::new();

        // Position accessor
        let pos_accessor_idx = self.add_vec3_accessor(positions, true, Some(Target::ArrayBuffer));
        attributes.insert(
            Checked::Valid(Semantic::Positions),
            Index::new(pos_accessor_idx),
        );

        // Normal accessor
        let norm_accessor_idx = self.add_vec3_accessor(normals, false, Some(Target::ArrayBuffer));
        attributes.insert(
            Checked::Valid(Semantic::Normals),
            Index::new(norm_accessor_idx),
        );

        // UV accessor
        let uv_accessor_idx = self.add_vec2_accessor(uvs);
        attributes.insert(
            Checked::Valid(Semantic::TexCoords(0)),
            Index::new(uv_accessor_idx),
        );

        // Bone indices (JOINTS_0) - U16 x4
        let joint_accessor_idx = self.add_vec4_u16_accessor(bone_indices);
        attributes.insert(
            Checked::Valid(Semantic::Joints(0)),
            Index::new(joint_accessor_idx),
        );

        // Bone weights (WEIGHTS_0)
        let weight_accessor_idx = self.add_vec4_accessor(bone_weights, Some(Target::ArrayBuffer));
        attributes.insert(
            Checked::Valid(Semantic::Weights(0)),
            Index::new(weight_accessor_idx),
        );

        // Index accessor
        let idx_accessor_idx = self.add_index_accessor(indices);

        Ok(Primitive {
            attributes,
            indices: Some(Index::new(idx_accessor_idx)),
            material: Some(Index::new(material_index)),
            mode: Checked::Valid(Mode::Triangles),
            targets: None,
            extensions: None,
            extras: Default::default(),
        })
    }

    fn add_vec4_u16_accessor(&mut self, data: &[[u16; 4]]) -> u32 {
        self.align_to_4();
        let offset = self.buffer_data.len();

        for v in data {
            self.buffer_data.extend_from_slice(&v[0].to_le_bytes());
            self.buffer_data.extend_from_slice(&v[1].to_le_bytes());
            self.buffer_data.extend_from_slice(&v[2].to_le_bytes());
            self.buffer_data.extend_from_slice(&v[3].to_le_bytes());
        }

        let byte_length = data.len() * 8; // 4 * u16 = 8 bytes
        let view_idx = self.buffer_views.len() as u32;
        self.buffer_views.push(View {
            name: None,
            buffer: Index::new(0),
            byte_offset: Some(USize64(offset as u64)),
            byte_length: USize64(byte_length as u64),
            byte_stride: None,
            target: Some(Checked::Valid(Target::ArrayBuffer)),
            extensions: None,
            extras: Default::default(),
        });

        let accessor_idx = self.accessors.len() as u32;
        self.accessors.push(Accessor {
            buffer_view: Some(Index::new(view_idx)),
            byte_offset: Some(USize64(0)),
            component_type: Checked::Valid(GenericComponentType(DataType::U16)),
            count: USize64(data.len() as u64),
            type_: Checked::Valid(Type::Vec4),
            extensions: None,
            extras: Default::default(),
            min: None,
            max: None,
            name: None,
            normalized: false,
            sparse: None,
        });

        accessor_idx
    }

    fn add_vec3_accessor(
        &mut self,
        data: &[[f32; 3]],
        compute_bounds: bool,
        target: Option<Target>,
    ) -> u32 {
        self.align_to_4();
        let offset = self.buffer_data.len();

        for v in data {
            self.buffer_data.extend_from_slice(&v[0].to_le_bytes());
            self.buffer_data.extend_from_slice(&v[1].to_le_bytes());
            self.buffer_data.extend_from_slice(&v[2].to_le_bytes());
        }

        let byte_length = data.len() * 12;
        let view_idx = self.buffer_views.len() as u32;
        self.buffer_views.push(View {
            name: None,
            buffer: Index::new(0),
            byte_offset: Some(USize64(offset as u64)),
            byte_length: USize64(byte_length as u64),
            byte_stride: None,
            target: target.map(|t| Checked::Valid(t)),
            extensions: None,
            extras: Default::default(),
        });

        let (min, max) = if compute_bounds {
            let mut min = [f32::MAX; 3];
            let mut max = [f32::MIN; 3];
            for v in data {
                for i in 0..3 {
                    min[i] = min[i].min(v[i]);
                    max[i] = max[i].max(v[i]);
                }
            }
            (
                Some(serde_json::to_value(min.to_vec()).unwrap()),
                Some(serde_json::to_value(max.to_vec()).unwrap()),
            )
        } else {
            (None, None)
        };

        let accessor_idx = self.accessors.len() as u32;
        self.accessors.push(Accessor {
            buffer_view: Some(Index::new(view_idx)),
            byte_offset: Some(USize64(0)),
            component_type: Checked::Valid(GenericComponentType(DataType::F32)),
            count: USize64(data.len() as u64),
            type_: Checked::Valid(Type::Vec3),
            extensions: None,
            extras: Default::default(),
            min,
            max,
            name: None,
            normalized: false,
            sparse: None,
        });

        accessor_idx
    }

    fn add_vec2_accessor(&mut self, data: &[[f32; 2]]) -> u32 {
        self.align_to_4();
        let offset = self.buffer_data.len();

        for v in data {
            self.buffer_data.extend_from_slice(&v[0].to_le_bytes());
            self.buffer_data.extend_from_slice(&v[1].to_le_bytes());
        }

        let byte_length = data.len() * 8;
        let view_idx = self.buffer_views.len() as u32;
        self.buffer_views.push(View {
            name: None,
            buffer: Index::new(0),
            byte_offset: Some(USize64(offset as u64)),
            byte_length: USize64(byte_length as u64),
            byte_stride: None,
            target: Some(Checked::Valid(Target::ArrayBuffer)),
            extensions: None,
            extras: Default::default(),
        });

        let accessor_idx = self.accessors.len() as u32;
        self.accessors.push(Accessor {
            buffer_view: Some(Index::new(view_idx)),
            byte_offset: Some(USize64(0)),
            component_type: Checked::Valid(GenericComponentType(DataType::F32)),
            count: USize64(data.len() as u64),
            type_: Checked::Valid(Type::Vec2),
            extensions: None,
            extras: Default::default(),
            min: None,
            max: None,
            name: None,
            normalized: false,
            sparse: None,
        });

        accessor_idx
    }

    fn add_index_accessor(&mut self, data: &[u16]) -> u32 {
        self.align_to_4();
        let offset = self.buffer_data.len();

        for &idx in data {
            self.buffer_data.extend_from_slice(&idx.to_le_bytes());
        }

        let byte_length = data.len() * 2;
        let view_idx = self.buffer_views.len() as u32;
        self.buffer_views.push(View {
            name: None,
            buffer: Index::new(0),
            byte_offset: Some(USize64(offset as u64)),
            byte_length: USize64(byte_length as u64),
            byte_stride: None,
            target: Some(Checked::Valid(Target::ElementArrayBuffer)),
            extensions: None,
            extras: Default::default(),
        });

        let accessor_idx = self.accessors.len() as u32;
        self.accessors.push(Accessor {
            buffer_view: Some(Index::new(view_idx)),
            byte_offset: Some(USize64(0)),
            component_type: Checked::Valid(GenericComponentType(DataType::U16)),
            count: USize64(data.len() as u64),
            type_: Checked::Valid(Type::Scalar),
            extensions: None,
            extras: Default::default(),
            min: None,
            max: None,
            name: None,
            normalized: false,
            sparse: None,
        });

        accessor_idx
    }

    fn write_to_glb(self, output_path: &str) -> Result<(), Error> {
        let buffer = Buffer {
            name: Some("skin_buffer".to_string()),
            byte_length: USize64(self.buffer_data.len() as u64),
            uri: None,
            extensions: None,
            extras: Default::default(),
        };

        let node_indices: Vec<Index<Node>> = self
            .root_node_indices
            .iter()
            .map(|&i| Index::new(i))
            .collect();

        let scene = Scene {
            name: Some("skin_scene".to_string()),
            nodes: node_indices,
            extensions: None,
            extras: Default::default(),
        };

        let gltf_json = Root {
            asset: gltf::json::asset::Asset {
                version: "2.0".into(),
                ..Default::default()
            },
            buffers: vec![buffer],
            buffer_views: self.buffer_views,
            accessors: self.accessors,
            meshes: self.meshes,
            nodes: self.nodes,
            materials: self.materials,
            textures: self.textures,
            images: self.images,
            scene: Some(Index::new(0)),
            scenes: vec![scene],
            animations: self.animations,
            skins: self.skins,
            ..Default::default()
        };

        let json_string = serde_json::to_string(&gltf_json)
            .map_err(|e| Error::Parse(format!("序列化 GLTF JSON 失败: {}", e)))?;

        // Prepare binary data with padding
        let mut binary_data = self.buffer_data;
        let padding = (4 - (binary_data.len() % 4)) % 4;
        for _ in 0..padding {
            binary_data.push(0);
        }

        // Use gltf crate's binary serialization
        let glb = gltf::binary::Glb {
            header: gltf::binary::Header {
                magic: *b"glTF",
                version: 2,
                length: 0, // Let to_vec() calculate
            },
            json: Cow::Owned(json_string.into_bytes()),
            bin: Some(Cow::Owned(binary_data)),
        };

        let glb_data = glb
            .to_vec()
            .map_err(|e| Error::Parse(format!("生成 GLB 失败: {}", e)))?;

        let path = std::path::Path::new(output_path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| Error::Parse(format!("创建目录失败: {}", e)))?;
        }

        std::fs::write(output_path, glb_data)
            .map_err(|e| Error::Parse(format!("写入 GLB 失败: {}", e)))?;

        Ok(())
    }
}
