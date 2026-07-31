use std::collections::BTreeMap;
use std::sync::Arc;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// 成员布局（uniform block 内的每个字段）
// ---------------------------------------------------------------------------

/// 单个 uniform 成员的物理内存布局
#[derive(Reflect, Debug, Clone, Serialize, Deserialize, PartialEq)]
#[reflect(Default)]
pub struct ShaderMemberLayout {
    pub name: String,
    pub offset: usize,
    pub size: usize,
}

impl Default for ShaderMemberLayout {
    fn default() -> Self {
        Self {
            name: String::new(),
            offset: 0,
            size: 0,
        }
    }
}

// ---------------------------------------------------------------------------
// Binding 类型描述
// ---------------------------------------------------------------------------

/// 每个 binding 槽位的类型，members 只在 UniformBuffer 变体中存在
#[derive(Reflect, Debug, Clone, Serialize, Deserialize, PartialEq)]
#[reflect(Default)]
pub enum BindingTypeDesc {
    /// Uniform Buffer：包含成员字段列表和字节总大小
    UniformBuffer {
        total_size: usize,
        members: BTreeMap<String, ShaderMemberLayout>,
    },
    /// 2D 纹理（SampledImage / CombinedImageSampler）
    Texture2d,
    /// 采样器
    Sampler,
}

impl Default for BindingTypeDesc {
    fn default() -> Self {
        Self::UniformBuffer {
            total_size: 0,
            members: BTreeMap::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// Binding 描述符
// ---------------------------------------------------------------------------

/// 单个 binding 槽位的完整描述
#[derive(Reflect, Debug, Clone, Serialize, Deserialize, PartialEq)]
#[reflect(Default)]
pub struct BindingDescriptor {
    pub binding_index: u32,
    /// SPIRV-Reflect 中的 binding 名称，如 "$Globals"、"DIFFUSE_MAP__SMP"、"DIFFUSE_MAP__TX"
    #[serde(default)]
    pub name: String,
    pub type_desc: BindingTypeDesc,
}

impl Default for BindingDescriptor {
    fn default() -> Self {
        Self {
            binding_index: 0,
            name: String::new(),
            type_desc: BindingTypeDesc::default(),
        }
    }
}

// ---------------------------------------------------------------------------
// ShaderLayoutDescriptor：一套 shader 变体的完整 binding 布局
// ---------------------------------------------------------------------------

/// 一套 shader 变体（vs 或 ps）的全量 binding 布局描述。
/// key 为 binding name，BTreeMap 保证按 name 排序。
#[derive(Reflect, Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[reflect(Default)]
pub struct ShaderLayoutDescriptor {
    pub bindings: BTreeMap<String, BindingDescriptor>,
}

impl ShaderLayoutDescriptor {
    /// 查找指定 name 的 binding 下成员变量的物理 offset
    pub fn get_member_offset(&self, binding_name: &str, member_name: &str) -> Option<usize> {
        match self.bindings.get(binding_name)?.type_desc {
            BindingTypeDesc::UniformBuffer { ref members, .. } => {
                members.get(member_name).map(|m| m.offset)
            }
            _ => None,
        }
    }

    /// 查找指定 name 的 binding 的 uniform buffer 总大小
    pub fn get_uniform_total_size(&self, binding_name: &str) -> Option<usize> {
        match self.bindings.get(binding_name)?.type_desc {
            BindingTypeDesc::UniformBuffer { total_size, .. } => Some(total_size),
            _ => None,
        }
    }

    /// 将两个 descriptor（VS + PS）合并
    /// 由于 VS/PS 的 binding 名称不重叠，直接合并即可
    pub fn merge(vs: &Self, ps: &Self) -> Self {
        let mut bindings = vs.bindings.clone();
        for (name, ps_desc) in &ps.bindings {
            bindings
                .entry(name.clone())
                .and_modify(|existing| {
                    // 正常不会走到这里（VS/PS binding 不重叠）
                    if existing.name.is_empty() && !ps_desc.name.is_empty() {
                        existing.name = ps_desc.name.clone();
                    }
                })
                .or_insert_with(|| ps_desc.clone());
        }
        Self { bindings }
    }
}

// ---------------------------------------------------------------------------
// Arc 包装：材质中共享 layout 描述符
// ---------------------------------------------------------------------------

/// 线程安全共享的 layout 描述符
pub type SharedLayoutDescriptor = Arc<ShaderLayoutDescriptor>;
