use std::collections::HashMap;
use std::sync::Arc;

use bevy::prelude::*;
use bevy::reflect::Reflect;
use serde::{Deserialize, Serialize};

use crate::shader_layout::ShaderLayoutDescriptor;

// ---------------------------------------------------------------------------
// LeagueShader：shader 家族标识（离线提取归一化后的变体类型）
// ---------------------------------------------------------------------------

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, Reflect, Default)]
pub enum LeagueShader {
    QuadPsSlice,
    QuadVs,
    #[default]
    QuadPs,
    UnlitDecalPs,
    UnlitDecalVs,
    DistortionPs,
    DistortionVs,
    MeshPs,
    MeshVs,
    SkinnedMeshParticlePs,
    SkinnedMeshParticleVs,
}

// ---------------------------------------------------------------------------
// ShaderMapEntry：单个 (LeagueShader, hash) 对应的 shader + layout
// ---------------------------------------------------------------------------

#[derive(Reflect, Debug, Clone, Default)]
#[reflect(Default)]
pub struct ShaderMapEntry {
    pub shader_handle: Handle<Shader>,
    /// 该 shader 变体 cbuffer 布局在 `ShaderMap::layouts` 去重池中的索引
    /// （VS 和 PS 各自独立存储，合并由材质层完成）
    pub layout_index: u32,
}

// ---------------------------------------------------------------------------
// ShaderMap：主 World 资源，存储全量 shader 变体的 handle + layout
// ---------------------------------------------------------------------------

#[derive(Resource, Reflect, Default, Debug, Clone)]
#[reflect(Resource, Default)]
pub struct ShaderMap {
    /// 每个 (LeagueShader, 变体 hash) 的 shader handle + 布局池索引
    pub entries: HashMap<LeagueShader, HashMap<u64, ShaderMapEntry>>,
    /// 变体 cbuffer 布局去重池：大量变体共享同一套布局，只存一份
    /// （成员 offset/size 随变体漂移，供 CPU 侧 set_param 写入使用）
    pub layouts: Vec<ShaderLayoutDescriptor>,
    /// 每个家族所有变体的槽位并集布局（由 extract_shaders 离线统一 pass 生成，
    /// binding_index 已归一化为 .spv 实际的 Vulkan 压缩编号：
    /// VS = 并集排名，PS = |配对 VS 并集| + 并集排名）
    pub unified: HashMap<LeagueShader, ShaderLayoutDescriptor>,
}

impl ShaderMap {
    /// 获取指定 shader 变体的 handle
    pub fn get_shader_handle(
        &self,
        shader_type: LeagueShader,
        hash: u64,
    ) -> Option<Handle<Shader>> {
        self.entries
            .get(&shader_type)?
            .get(&hash)
            .map(|entry| entry.shader_handle.clone())
    }

    /// 获取指定 shader 变体的 layout 描述符（从去重池取出并克隆）
    pub fn get_layout(
        &self,
        shader_type: LeagueShader,
        hash: u64,
    ) -> Option<Arc<ShaderLayoutDescriptor>> {
        let entry = self.entries.get(&shader_type)?.get(&hash)?;
        self.layouts
            .get(entry.layout_index as usize)
            .map(|layout| Arc::new(layout.clone()))
    }

    /// 获取指定家族的槽位并集统一布局
    pub fn get_unified(&self, shader_type: LeagueShader) -> Option<&ShaderLayoutDescriptor> {
        self.unified.get(&shader_type)
    }

    /// 查询指定 name 下某成员的物理 offset
    pub fn get_member_offset(
        &self,
        shader_type: LeagueShader,
        hash: u64,
        binding_name: &str,
        member_name: &str,
    ) -> Option<usize> {
        let entry = self.entries.get(&shader_type)?.get(&hash)?;
        self.layouts
            .get(entry.layout_index as usize)
            .and_then(|layout| layout.get_member_offset(binding_name, member_name))
    }
}

// ---------------------------------------------------------------------------
// SharedRenderData：共享渲染数据（来自 Shaders.wad.client 内 assets/shaders/shareddata.bin）
// ---------------------------------------------------------------------------

/// 共享采样器定义。原始 X3DSharedSamplerDef 的语义在离线提取期已归一化：
/// - address_mode_*：0 = ClampToEdge（原字段缺省），1 = Repeat（原值 0），2 = MirrorRepeat（原值 2）
/// - mip_filter：0 = 关闭 mipmap 采样，1 = 线性 mip
#[derive(Reflect, Debug, Clone, Default)]
#[reflect(Default)]
pub struct SharedSamplerDef {
    pub address_mode_u: u8,
    pub address_mode_v: u8,
    pub address_mode_w: u8,
    pub max_anisotropy: u8,
    pub mip_filter: u8,
    pub mip_lod_bias: i32,
    pub register: i32,
}

/// 共享贴图定义（帧缓冲拷贝 / 系统贴图等，运行期暂以占位贴图绑定）
#[derive(Reflect, Debug, Clone, Default)]
#[reflect(Default)]
pub struct SharedTextureDef {
    /// 原 X3DSharedTextureDef.type（贴图维度类别）
    pub kind: u8,
    /// 关联的共享采样器 link 哈希
    pub sampler: u32,
    /// 缺省值（占位绑定时可作为常量回退）
    pub default_value: Vec4,
}

/// 主 World 资源：随 shaders/map.ron 一并加载，供动态材质在装配 bind group 时
/// 解析 `_SharedSampler` / `_SharedTexture` 绑定
#[derive(Resource, Reflect, Default, Debug, Clone)]
#[reflect(Resource, Default)]
pub struct SharedRenderData {
    /// 采样器名（如 "Clamp_No_Mip"）→ 采样器定义
    pub samplers: HashMap<String, SharedSamplerDef>,
    /// 共享贴图名（如 "FOW_MAP"）→ 贴图定义
    pub textures: HashMap<String, SharedTextureDef>,
}

// ---------------------------------------------------------------------------
// 调试辅助资源
// ---------------------------------------------------------------------------

/// 记录已插入的 Shader handle，用于 debug 检测
#[derive(Resource, Default)]
pub struct DebugShaderHandles(pub Vec<Handle<Shader>>);

// ---------------------------------------------------------------------------
// Startup 系统：加载 map.ron
// ---------------------------------------------------------------------------

pub fn startup_load_shaders(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(DynamicWorldRoot(asset_server.load("shaders/map.ron")));
}

// ---------------------------------------------------------------------------
// 迁移自 lol_base::shader
// ---------------------------------------------------------------------------

#[derive(Asset, TypePath)]
pub struct ResourceShaderPackage {
    pub handles: HashMap<u64, Handle<Shader>>,
}

#[derive(Asset, TypePath)]
pub struct ResourceShaderChunk {}
