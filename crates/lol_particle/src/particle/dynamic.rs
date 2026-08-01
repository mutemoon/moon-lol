//! 纯动态材质：运行时从 ShaderMap 拿布局，按需拼装组合 BindGroupLayout。
//!
//! 参考 `examples` 里的 `pure_ecs_dynamic_layout.rs`，但有三点不同：
//!   1. 布局来源改为 [`ShaderMap`]（不再手动维护 VS/FS 布局库）；
//!   2. layout 缓存的 key 由 vert / frag 各自的 (LeagueShader + defs 数组) 共同组成，
//!      取代原例子里的 (vert_hash, frag_hash) 一对 u64；
//!   3. 材质里额外存一个 `Arc<ConfigVfxEmitterDefinition>`，供后续数据填充使用。
//!
//! 骨架说明：本模块搭好"从 key → 布局 → 组合 BindGroupLayout → bind group"的完整链路，
//! uniform / texture 的实际数据来自 fallback / 全零占位；真正从 emitter_def 填充参数
//! 的逻辑留待后续。

use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

use bevy::ecs::system::SystemParamItem;
use bevy::ecs::system::lifetimeless::{SRes, SResMut};
use bevy::mesh::MeshVertexBufferLayoutRef;
use bevy::pbr::{MaterialPipeline, MaterialPipelineKey};
use bevy::prelude::*;
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_resource::{
    AddressMode, AsBindGroup, AsBindGroupError, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingResources,
    BindingType, BlendComponent, BlendFactor, BlendOperation, BlendState, Buffer,
    BufferBindingType, BufferInitDescriptor, BufferUsages, FilterMode, MipmapFilterMode,
    PipelineCache, PreparedBindGroup, RenderPipelineDescriptor, Sampler, SamplerBindingType,
    SamplerDescriptor, ShaderStages, SpecializedMeshPipelineError, TextureSampleType,
    TextureViewDimension, UnpreparedBindGroup,
};
use bevy::render::renderer::RenderDevice;
use bevy::render::texture::{FallbackImage, GpuImage};
use bevy::render::{Extract, ExtractSchedule, Render, RenderApp, RenderSystems};
use league_utils::LeagueShader;
use lol_base_render::particle::ConfigVfxEmitterDefinition;
use lol_base_render::shader::{ShaderMap, SharedRenderData, SharedSamplerDef};
use lol_base_render::shader_layout::{BindingTypeDesc, ShaderLayoutDescriptor};

use crate::{
    ATTRIBUTE_LIFETIME, ATTRIBUTE_UV_FRAME, ATTRIBUTE_WORLD_POSITION, ATTRIBUTE_WORLD_POSITION_VEC4,
};

// ---------------------------------------------------------------------------
// 渲染几何族
// ---------------------------------------------------------------------------

/// 粒子渲染几何族，对齐逆向 `ShaderEffect_BuildShaderPathAndDefines`(sub_1412DD450)
/// 的族选择矩阵：0=quad / 1=mesh / 2=skinned；renderType==7 → UnlitDecal 特例；
/// distortion pass(2,3) → Distortion 变体。
/// 贯穿「材质 → key → specialize → 逐帧更新」，决定 shader 家族、纹理绑定名与顶点布局。
#[derive(Copy, Clone, Hash, PartialEq, Eq, Debug)]
pub enum ParticleRenderKind {
    Quad,
    Mesh,
    Distortion,
    SkinnedMesh,
    UnlitDecal,
}

// ---------------------------------------------------------------------------
// 组合布局 key：vert / frag 各自 (LeagueShader + defs) 共同决定一份组合布局
// ---------------------------------------------------------------------------

/// 布局缓存键：由 vert 家族 + vert defs、frag 家族 + frag defs 四者共同决定
/// 一个具体的组合 GPU BindGroupLayout。
///
/// defs 在构造时排序归一化，保证顺序不同但内容相同的 defs 命中同一份布局。
#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct PipelineLayoutKey {
    pub vert_shader: LeagueShader,
    pub vert_defs: Vec<String>,
    pub frag_shader: LeagueShader,
    pub frag_defs: Vec<String>,
}

impl PipelineLayoutKey {
    pub fn new(
        vert_shader: LeagueShader,
        vert_defs: Vec<String>,
        frag_shader: LeagueShader,
        frag_defs: Vec<String>,
    ) -> Self {
        let mut vert_defs = vert_defs;
        let mut frag_defs = frag_defs;
        vert_defs.sort();
        frag_defs.sort();
        Self {
            vert_shader,
            vert_defs,
            frag_shader,
            frag_defs,
        }
    }
}

// ---------------------------------------------------------------------------
// Render World 资源
// ---------------------------------------------------------------------------

/// 缓存按需拼装出的组合 BindGroupLayout，键为 [`PipelineLayoutKey`]
#[derive(Resource, Default)]
pub struct RenderDynamicLayoutCache {
    pub layouts: HashMap<PipelineLayoutKey, BindGroupLayout>,
}

/// Extract 阶段收集到的、场景中实际用到的所有 key 及其两侧布局描述，
/// 供 Prepare 阶段现场拼 BindGroupLayout（无需在 Render World 再访问 ShaderMap）
#[derive(Resource, Default)]
pub struct ExtractedMaterialKeys {
    pub keys: Vec<ExtractedKeyEntry>,
}

pub struct ExtractedKeyEntry {
    pub key: PipelineLayoutKey,
    pub vert_layout: Arc<ShaderLayoutDescriptor>,
    pub frag_layout: Arc<ShaderLayoutDescriptor>,
}

/// as_bind_group 创建的 uniform Buffer 句柄缓存（key → uniform 名 → Buffer）。
/// 后续每帧数据可走 RenderQueue::write_buffer 原地更新这些 Buffer，
/// 免去改材质资产触发整套 bind group 重建。（骨架阶段仅存放，暂无写入系统）
#[derive(Resource, Default)]
pub struct DynamicUniformBufferCache {
    pub buffers: HashMap<PipelineLayoutKey, HashMap<String, Buffer>>,
}

/// Render World 侧的共享采样器缓存：把 [`SharedRenderData`] 里的采样器定义
/// 一次性创建为 wgpu [`Sampler`]（键为采样器名，如 "Clamp_No_Mip"），
/// 供 as_bind_group 解析 `_SharedSampler` 绑定时按名取用。
#[derive(Resource, Default)]
pub struct SharedSamplerCache {
    pub samplers: HashMap<String, Sampler>,
    /// 游戏默认采样器（Linear 过滤 + Clamp 寻址），as_bind_group 未命中具名/配对采样器时的回退
    pub default_sampler: Option<Sampler>,
    /// 已从 SharedRenderData 构建完成，避免每帧重建
    pub built: bool,
}

/// Render World 侧的共享渲染数据副本（由主 World 的 [`SharedRenderData`] extract 而来）
#[derive(Resource, Default)]
pub struct RenderSharedRenderData(pub SharedRenderData);

/// 依据共享采样器定义创建 wgpu Sampler。
/// League 的共享采样器统一使用线性过滤；mip_filter==0 表示关闭 mipmap 采样。
fn create_shared_sampler(render_device: &RenderDevice, def: &SharedSamplerDef) -> Sampler {
    let to_address = |m: u8| match m {
        1 => AddressMode::Repeat,
        2 => AddressMode::MirrorRepeat,
        _ => AddressMode::ClampToEdge,
    };
    let no_mip = def.mip_filter == 0;
    let anisotropy = (def.max_anisotropy as u16).max(1);
    // 各向异性要求 min/mag/mipmap 全为 Linear，无 mip 时禁用
    let anisotropy_clamp = if !no_mip && anisotropy > 1 {
        anisotropy
    } else {
        1
    };
    render_device.create_sampler(&SamplerDescriptor {
        label: Some("SharedSampler"),
        address_mode_u: to_address(def.address_mode_u),
        address_mode_v: to_address(def.address_mode_v),
        address_mode_w: to_address(def.address_mode_w),
        mag_filter: FilterMode::Linear,
        min_filter: FilterMode::Linear,
        mipmap_filter: if no_mip {
            MipmapFilterMode::Nearest
        } else {
            MipmapFilterMode::Linear
        },
        lod_min_clamp: 0.0,
        lod_max_clamp: if no_mip { 0.0 } else { 32.0 },
        anisotropy_clamp,
        ..Default::default()
    })
}

/// Extract：把主 World 的 SharedRenderData（若已随 map.ron 加载）复制到 Render World
fn extract_shared_render_data(
    shared: Extract<Option<Res<SharedRenderData>>>,
    mut render_shared: ResMut<RenderSharedRenderData>,
) {
    if let Some(shared) = shared.as_ref() {
        if shared.is_changed() {
            render_shared.0 = (*shared).clone();
        }
    }
}

/// Prepare：首帧数据就绪后，把共享采样器定义构建为 wgpu Sampler 缓存（仅构建一次）
fn prepare_shared_samplers(
    render_device: Res<RenderDevice>,
    render_shared: Res<RenderSharedRenderData>,
    mut cache: ResMut<SharedSamplerCache>,
) {
    if cache.built {
        return;
    }
    // 游戏默认采样器状态为 Linear 过滤 + Clamp 寻址（未显式设置寻址即 Clamp）。
    // 先无条件建好该默认采样器，作为未命中具名/配对采样器时的回退，
    // 取代 Bevy fallback_image 的 Nearest 采样器，以对齐游戏行为。
    if cache.default_sampler.is_none() {
        cache.default_sampler = Some(render_device.create_sampler(&SamplerDescriptor {
            label: Some("SharedSamplerDefault"),
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: MipmapFilterMode::Linear,
            ..Default::default()
        }));
    }
    // 共享采样器数据尚未随 map.ron 就绪时，仅先建默认采样器，待数据到达再补建具名采样器。
    if render_shared.0.samplers.is_empty() {
        return;
    }
    for (name, def) in &render_shared.0.samplers {
        cache
            .samplers
            .insert(name.clone(), create_shared_sampler(&render_device, def));
    }
    cache.built = true;
}

// ---------------------------------------------------------------------------
// 布局 → BindGroupLayoutEntry
// ---------------------------------------------------------------------------

/// 从一份 [`ShaderLayoutDescriptor`] 构建 WGPU 的 BindGroupLayoutEntry 列表。
///
/// 项目里的 `ShaderLayoutDescriptor` 不带 stage 字段，因此由调用方按布局来源
/// 显式指定可见阶段（vert 布局 → VERTEX，frag 布局 → FRAGMENT）。
pub fn build_layout_entries(
    desc: &ShaderLayoutDescriptor,
    stage: ShaderStages,
) -> Vec<BindGroupLayoutEntry> {
    let mut entries = Vec::with_capacity(desc.bindings.len());
    for binding in desc.bindings.values() {
        let ty = match &binding.type_desc {
            BindingTypeDesc::UniformBuffer { .. } => BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            BindingTypeDesc::Texture2d => BindingType::Texture {
                sample_type: TextureSampleType::Float { filterable: true },
                view_dimension: TextureViewDimension::D2,
                multisampled: false,
            },
            BindingTypeDesc::Sampler => BindingType::Sampler(SamplerBindingType::Filtering),
        };
        entries.push(BindGroupLayoutEntry {
            binding: binding.binding_index,
            visibility: stage,
            ty,
            count: None,
        });
    }
    entries
}

/// 现场拼积木：VS 布局条目（VERTEX）+ FS 布局条目（FRAGMENT）合并为一份组合 entries。
/// prepare（创建 GPU BindGroupLayout）与 specialize（填 BindGroupLayoutDescriptor）
/// 共用此单一事实来源，保证两条路径 entries 一致，wgpu 依此去重共享底层对象。
pub fn build_combined_entries(
    vert_desc: &ShaderLayoutDescriptor,
    frag_desc: &ShaderLayoutDescriptor,
) -> Vec<BindGroupLayoutEntry> {
    let mut entries = build_layout_entries(vert_desc, ShaderStages::VERTEX);
    entries.extend(build_layout_entries(frag_desc, ShaderStages::FRAGMENT));
    entries
}

// ---------------------------------------------------------------------------
// 材质定义
// ---------------------------------------------------------------------------

/// 按语义命名的可选贴图输入；[`ParticleMaterialDynamic::create`] 按 kind 把它们
/// 映射到各族 unified 布局里的实际绑定名（`NAME__TX` / `*_SharedTexture`）。
#[derive(Default, Clone)]
pub struct ParticleTextureInputs {
    pub texture: Option<Handle<Image>>,
    pub particle_color_texture: Option<Handle<Image>>,
    pub texture_mult: Option<Handle<Image>>,
    pub color_remap_ramp: Option<Handle<Image>>,
    pub normal_map: Option<Handle<Image>>,
    pub back_buffer: Option<Handle<Image>>,
}

#[derive(Asset, TypePath, Clone, Debug)]
pub struct ParticleMaterialDynamic {
    /// 渲染几何族：决定 shader 家族 / 纹理绑定名 / 顶点布局 / 逐帧更新分支
    pub kind: ParticleRenderKind,
    /// vert 家族 + defs（决定 VS 侧布局）
    pub vert_shader: LeagueShader,
    pub vert_defs: Vec<String>,
    /// frag 家族 + defs（决定 FS 侧布局）
    pub frag_shader: LeagueShader,
    pub frag_defs: Vec<String>,
    /// VS 侧布局描述（Arc 共享，从 ShaderMap 取出）
    pub vert_layout: Arc<ShaderLayoutDescriptor>,
    /// FS 侧布局描述（Arc 共享，从 ShaderMap 取出）
    pub frag_layout: Arc<ShaderLayoutDescriptor>,
    /// VS/FS 变体自身反射布局：$Globals 由松散全局变量拼成，每个变体会剔除
    /// 未使用成员并重新紧排 offset（成员 offset 随变体漂移），set_param 必须
    /// 按变体实际 offset 写入；vert_layout/frag_layout 并集布局仅用于 bind
    /// group 槽位覆盖与缓冲区尺寸
    pub vert_variant_layout: Arc<ShaderLayoutDescriptor>,
    pub frag_variant_layout: Arc<ShaderLayoutDescriptor>,
    /// 预编译 SPIR-V 顶点 / 片元着色器
    pub shader_vert: Handle<Shader>,
    pub shader_frag: Handle<Shader>,
    /// 发射器定义：供后续从中读取参数填充 uniform / texture
    pub emitter_def: Arc<ConfigVfxEmitterDefinition>,
    pub blend_mode: u8,
    /// CPU 端 uniform 字节 blob（binding_index → 字节），由 set_param 按成员名写入，
    /// as_bind_group 时作为对应 uniform buffer 的初始内容上传
    pub uniforms: BTreeMap<u32, Vec<u8>>,
    /// 贴图句柄（binding 名 → Handle），as_bind_group 时按 binding 名匹配 Texture2d 槽位；
    /// 未提供或尚未加载完成的槽位回退到 fallback
    pub textures: HashMap<String, Handle<Image>>,
}

impl ParticleMaterialDynamic {
    /// 从 `kind` + `emitter_def` + `ShaderMap` 推导出材质：
    ///   - shader 家族按 kind 选择（对齐逆向的族 × pass 装配矩阵）；
    ///   - Quad 的 frag 家族按 emitter_def 是否含 slice_technique_range 在
    ///     QuadPsSlice / QuadPs 间切换（slice 是独立 shader 家族，仅 PS 追加，
    ///     与逆向 `*_PS_Slice.ps` 一致）；
    ///   - defs 本轮默认全关（与逆向默认描述符一致，见 assembly::derive_defs）；
    ///   - blend_mode 从 emitter_def.blend_mode 派生。
    ///
    /// defs → hash → 从 ShaderMap 解析 shader handle 与该变体的 cbuffer 反射布局。
    pub fn create(
        kind: ParticleRenderKind,
        emitter_def: Arc<ConfigVfxEmitterDefinition>,
        textures_in: ParticleTextureInputs,
        shader_map: &ShaderMap,
    ) -> Self {
        // PS 基路径对齐逆向：quad=ParticleSystem/QUAD_PS｜mesh=MESH_PS｜
        // skin=SkinnedMesh/PARTICLE_PS｜decal=Environment/UNLIT_DECAL_PS｜
        // distortion=DISTORTION_* 变体（VS 对应）
        let (vert_shader, frag_shader) = match kind {
            ParticleRenderKind::Quad => (
                LeagueShader::QuadVs,
                if emitter_def.slice_technique_range.is_some() {
                    LeagueShader::QuadPsSlice
                } else {
                    LeagueShader::QuadPs
                },
            ),
            ParticleRenderKind::Mesh => (LeagueShader::MeshVs, LeagueShader::MeshPs),
            ParticleRenderKind::Distortion => {
                (LeagueShader::DistortionVs, LeagueShader::DistortionPs)
            }
            ParticleRenderKind::SkinnedMesh => (
                LeagueShader::SkinnedMeshParticleVs,
                LeagueShader::SkinnedMeshParticlePs,
            ),
            ParticleRenderKind::UnlitDecal => {
                (LeagueShader::UnlitDecalVs, LeagueShader::UnlitDecalPs)
            }
        };

        let blend_mode = emitter_def.blend_mode.unwrap_or(4);

        // 本轮 defs 默认全关（对应已抽取 SPIR-V 的 BASE 变体）；
        // 后续由 assembly::derive_defs 按逆向宏表派生具体组合
        let vert_defs: Vec<String> = vec![];
        let frag_defs: Vec<String> = vec![];

        let vert_hash = league_utils::hash_shader_spec(&vert_defs);
        let frag_hash = league_utils::hash_shader_spec(&frag_defs);

        let shader_vert = shader_map
            .get_shader_handle(vert_shader, vert_hash)
            .unwrap_or_default();
        let shader_frag = shader_map
            .get_shader_handle(frag_shader, frag_hash)
            .unwrap_or_default();

        // 布局取家族“并集”统一布局（get_unified）而非单变体反射布局：
        // 单变体布局只含该变体反射出的活跃槽位（BASE 变体甚至为空），
        // 但编译好的 SPIR-V 模块仍会访问全部槽位（如 PIXEL_COLOR_REMAP_RAMP 贴图），
        // 只有并集布局才能覆盖 shader 可能访问的全部 binding，避免 descriptor invalid。
        let vert_layout = shader_map
            .get_unified(vert_shader)
            .map(|layout| Arc::new(layout.clone()))
            .unwrap_or_default();
        let frag_layout = shader_map
            .get_unified(frag_shader)
            .map(|layout| Arc::new(layout.clone()))
            .unwrap_or_default();

        // set_param 的 offset 查表必须用变体自身反射布局：并集布局合并成员时
        // 保留的是首个变体的 offset（如 MeshVs 并集里 kColorFactor@128），而
        // BASE 变体 $Globals 剔除 vReflection 后 kColorFactor 实际在 112，按并集
        // offset 写入会整体错位、shader 读到全零。变体布局缺失时回退并集布局
        let vert_variant_layout = shader_map
            .get_layout(vert_shader, vert_hash)
            .unwrap_or_else(|| vert_layout.clone());
        let frag_variant_layout = shader_map
            .get_layout(frag_shader, frag_hash)
            .unwrap_or_else(|| frag_layout.clone());

        // 为两侧布局里的每个 uniform buffer 预分配零填充 CPU blob（按 binding_index 归档）
        let mut uniforms: BTreeMap<u32, Vec<u8>> = BTreeMap::new();
        for desc in [vert_layout.as_ref(), frag_layout.as_ref()] {
            for binding in desc.bindings.values() {
                if let BindingTypeDesc::UniformBuffer { total_size, .. } = &binding.type_desc {
                    uniforms.insert(binding.binding_index, vec![0u8; (*total_size).max(16)]);
                }
            }
        }

        // 各贴图按语义映射到片元侧绑定名（各族 unified 布局已核实）：
        // 主贴图 decal 族叫 DIFFUSE_MAP__TX，其余族叫 TEXTURE__TX；
        // 多余的条目无害——as_bind_group 只遍历布局里实际存在的槽位。
        let mut textures: HashMap<String, Handle<Image>> = HashMap::new();
        let diffuse_binding = match kind {
            ParticleRenderKind::UnlitDecal => "DIFFUSE_MAP__TX",
            _ => "TEXTURE__TX",
        };
        if let Some(texture) = textures_in.texture {
            textures.insert(diffuse_binding.to_string(), texture);
        }
        if let Some(texture) = textures_in.particle_color_texture {
            textures.insert("PARTICLE_COLOR_TEXTURE__TX".to_string(), texture);
        }
        if let Some(texture) = textures_in.texture_mult {
            textures.insert("TEXTUREMULT__TX".to_string(), texture);
        }
        // 颜色重映射斜坡：QuadPs/MeshPs/SkinnedMeshParticlePs 无条件采样它，
        // 并在 remap_color.w > 0 时用其 rgb 替换最终色。
        // 默认 fallback 是不透明白（w=1），会把粒子整片替换成白 → 失色变黑白；
        // 因此绑定一张 alpha=0 的透明贴图，使 remap_color.w==0，shader 跳过替换、保留原色。
        if let Some(texture) = textures_in.color_remap_ramp {
            textures.insert("PIXEL_COLOR_REMAP_RAMP_SharedTexture".to_string(), texture);
        }
        // Distortion 族专属：法线扰动贴图 + back-buffer 拷贝
        if let Some(texture) = textures_in.normal_map {
            textures.insert("NORMAL_MAP__TX".to_string(), texture);
        }
        if let Some(texture) = textures_in.back_buffer {
            textures.insert(
                "SAMPLER_BACK_BUFFER_COPY_SharedTexture".to_string(),
                texture,
            );
        }

        let mut material = Self {
            kind,
            vert_shader,
            vert_defs,
            frag_shader,
            frag_defs,
            vert_layout,
            frag_layout,
            vert_variant_layout,
            frag_variant_layout,
            shader_vert,
            shader_frag,
            emitter_def,
            blend_mode,
            uniforms,
            textures,
        };

        // 一次性常量上传对齐逆向 SetupParticleShader：sliceTechniqueRange > 0 时
        // 上传 SLICE_RANGE = (r, 1/r², 0, 0)；不写则 uniform 全零，切片遮罩会裁掉
        // 全部像素导致粒子不可见。无该成员的家族 set_param 安全 no-op
        if let Some(range) = material.emitter_def.slice_technique_range {
            if range > 0.0 {
                material.set_param(
                    "SLICE_RANGE",
                    Vec4::new(range, 1.0 / (range * range), 0.0, 0.0),
                );
            }
        }

        // 战争迷雾默认参数：MeshPs 等家族的 RGB 输出会乘上 FOW 可见性因子
        // fma(fow_uv.z, fow_sample.w - FOW_EDGE_CONTROL.w, FOW_EDGE_CONTROL.w)，
        // uniform 全零时因子为 0，RGB 被整体抹黑（additive 混合下完全不可见）；
        // 写 w=1 表示“无迷雾全可见”，无该成员的家族安全 no-op
        material.set_param("FOW_EDGE_CONTROL", Vec4::new(0.0, 0.0, 0.0, 1.0));

        material
    }

    /// 按成员名写入 uniform：在 vert/frag 布局里查到该成员所属 binding 与 offset，
    /// 写入对应 binding 的 CPU 字节 blob。返回是否至少命中一个成员。
    pub fn set_param<T: Copy>(&mut self, member_name: &str, value: T) -> bool {
        let bytes = unsafe {
            std::slice::from_raw_parts(&value as *const T as *const u8, std::mem::size_of::<T>())
        };
        self.write_after_member(member_name, 0, bytes)
    }

    /// 在成员 offset + extra_offset 处写入原始字节（用于成员后紧跟的匿名字段，
    /// 如 TEXTURE_INFO 之后的 uv_scale）。同名成员在 VS/PS 都存在时分别写入各自 blob。
    /// offset 查表用变体自身布局：变体没反射出的成员即 shader 不读，跳过是安全 no-op。
    pub fn write_after_member(
        &mut self,
        member_name: &str,
        extra_offset: usize,
        bytes: &[u8],
    ) -> bool {
        let mut hit = false;
        for desc in [
            self.vert_variant_layout.clone(),
            self.frag_variant_layout.clone(),
        ] {
            for binding in desc.bindings.values() {
                if let BindingTypeDesc::UniformBuffer { members, .. } = &binding.type_desc {
                    if let Some(member) = members.get(member_name) {
                        if let Some(blob) = self.uniforms.get_mut(&binding.binding_index) {
                            let start = member.offset + extra_offset;
                            if start < blob.len() {
                                let n = bytes.len().min(blob.len() - start);
                                blob[start..start + n].copy_from_slice(&bytes[..n]);
                                hit = true;
                            }
                        }
                    }
                }
            }
        }
        hit
    }

    /// 组合本材质的布局缓存键
    pub fn layout_key(&self) -> PipelineLayoutKey {
        PipelineLayoutKey::new(
            self.vert_shader,
            self.vert_defs.clone(),
            self.frag_shader,
            self.frag_defs.clone(),
        )
    }
}

// ---------------------------------------------------------------------------
// bind_group_data / pipeline key
// ---------------------------------------------------------------------------

/// pipeline 专化键：携带几何族、布局 key、两侧布局描述与 shader handle。
/// Hash/Eq 按 (kind + layout key + shader handle + blend_mode) 判定——布局 key 已唯一
/// 标识两份布局描述，无需遍历 Arc<ShaderLayoutDescriptor> 里的整棵 BTreeMap；
/// kind 纳入键因为 specialize 是静态函数，只能从 key.bind_group_data 读它选顶点布局。
#[derive(Clone, Debug)]
pub struct ParticleMaterialDynamicKey {
    pub kind: ParticleRenderKind,
    pub layout_key: PipelineLayoutKey,
    pub vert_layout: Arc<ShaderLayoutDescriptor>,
    pub frag_layout: Arc<ShaderLayoutDescriptor>,
    pub shader_vert: Handle<Shader>,
    pub shader_frag: Handle<Shader>,
    pub blend_mode: u8,
}

impl PartialEq for ParticleMaterialDynamicKey {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
            && self.layout_key == other.layout_key
            && self.shader_vert == other.shader_vert
            && self.shader_frag == other.shader_frag
            && self.blend_mode == other.blend_mode
    }
}

impl Eq for ParticleMaterialDynamicKey {}

impl std::hash::Hash for ParticleMaterialDynamicKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.kind.hash(state);
        self.layout_key.hash(state);
        self.shader_vert.hash(state);
        self.shader_frag.hash(state);
        self.blend_mode.hash(state);
    }
}

// ---------------------------------------------------------------------------
// AsBindGroup（手动实现）
// ---------------------------------------------------------------------------

impl AsBindGroup for ParticleMaterialDynamic {
    type Data = ParticleMaterialDynamicKey;
    type Param = (
        SRes<RenderAssets<GpuImage>>,
        SRes<FallbackImage>,
        SRes<RenderDynamicLayoutCache>,
        SResMut<DynamicUniformBufferCache>,
        SRes<SharedSamplerCache>,
    );

    fn label() -> &'static str {
        "ParticleMaterialDynamicBindGroup"
    }

    fn bind_group_data(&self) -> Self::Data {
        ParticleMaterialDynamicKey {
            kind: self.kind,
            layout_key: self.layout_key(),
            vert_layout: self.vert_layout.clone(),
            frag_layout: self.frag_layout.clone(),
            shader_vert: self.shader_vert.clone(),
            shader_frag: self.shader_frag.clone(),
            blend_mode: self.blend_mode,
        }
    }

    fn as_bind_group(
        &self,
        _layout_descriptor: &BindGroupLayoutDescriptor,
        render_device: &RenderDevice,
        _pipeline_cache: &PipelineCache,
        (image_assets, fallback_image, layout_cache, buffer_cache, shared_sampler_cache): &mut SystemParamItem<
            '_,
            '_,
            Self::Param,
        >,
    ) -> Result<PreparedBindGroup, AsBindGroupError> {
        let key = self.layout_key();
        // 组合 Layout 由 prepare 系统按需创建；首帧尚未就绪时重试
        let layout = match layout_cache.layouts.get(&key) {
            Some(layout) => layout,
            None => {
                return Err(AsBindGroupError::RetryNextUpdate);
            }
        };

        let fallback_view = &fallback_image.d2.texture_view;
        let fallback_sampler = &fallback_image.d2.sampler;

        let mut created_buffers: Vec<(u32, String, Buffer)> = Vec::new();
        let mut texture_entries: Vec<(u32, BindingResource)> = Vec::new();

        // 遍历 vert + frag 两侧布局的每个 binding，填资源：
        // uniform 用 CPU blob 数据（set_param 写入，如 mProj/vCamera/TEXTURE_INFO）；
        // Texture2d 按 binding 名从 textures 查已加载贴图，缺失/未就绪回退 fallback；采样器用 fallback
        for desc in [self.vert_layout.as_ref(), self.frag_layout.as_ref()] {
            for (name, binding) in &desc.bindings {
                let binding_index = binding.binding_index;
                match &binding.type_desc {
                    BindingTypeDesc::UniformBuffer { total_size, .. } => {
                        let size = (*total_size).max(16);
                        // 用 CPU blob 里的实际数据作为初始内容（未写入部分保持零）
                        let mut contents = vec![0u8; size];
                        if let Some(blob) = self.uniforms.get(&binding_index) {
                            let n = blob.len().min(size);
                            contents[..n].copy_from_slice(&blob[..n]);
                        }
                        let buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
                            label: Some(name),
                            contents: &contents,
                            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                        });
                        created_buffers.push((binding_index, name.clone(), buffer));
                    }
                    BindingTypeDesc::Texture2d => {
                        // 按 binding 名查贴图句柄 → GpuImage；未提供或尚未加载完成则回退 fallback。
                        // 共享贴图 "<Name>_SharedTexture"（FOW_MAP 等帧缓冲/系统贴图）当前不在 self.textures
                        // 内，同样落入 fallback 作为合理占位，待后续接入真实数据。
                        let view = self
                            .textures
                            .get(name)
                            .and_then(|handle| image_assets.get(handle))
                            .map(|gpu_image| &gpu_image.texture_view)
                            .unwrap_or(fallback_view);
                        texture_entries.push((binding_index, BindingResource::TextureView(view)));
                    }
                    BindingTypeDesc::Sampler => {
                        // 共享采样器 "<Name>_SharedSampler"：按名从共享采样器缓存取用
                        // （Linear + shareddata 定义的地址模式/mip）；
                        // 形如 "XXX__SMP" 的采样器：取同前缀 "XXX__TX" 贴图自带 sampler；
                        // 均未命中时回退到游戏默认采样器（Linear + Clamp），与游戏行为一致，
                        // 仅在默认采样器尚未建好（首帧）时才退到 Bevy fallback（Nearest）
                        let sampler = name
                            .strip_suffix("_SharedSampler")
                            .and_then(|shared_name| shared_sampler_cache.samplers.get(shared_name))
                            .or_else(|| {
                                name.strip_suffix("__SMP")
                                    .map(|prefix| format!("{prefix}__TX"))
                                    .and_then(|tex_key| self.textures.get(&tex_key))
                                    .and_then(|handle| image_assets.get(handle))
                                    .map(|gpu_image| &gpu_image.sampler)
                            })
                            .or(shared_sampler_cache.default_sampler.as_ref())
                            .unwrap_or(fallback_sampler);
                        texture_entries.push((binding_index, BindingResource::Sampler(sampler)));
                    }
                }
            }
        }

        let mut bind_group_entries = Vec::new();
        for (binding, _name, buffer) in &created_buffers {
            bind_group_entries.push(BindGroupEntry {
                binding: *binding,
                resource: BindingResource::Buffer(buffer.as_entire_buffer_binding()),
            });
        }
        for (binding, resource) in texture_entries {
            bind_group_entries.push(BindGroupEntry { binding, resource });
        }

        // 缓存 uniform Buffer 句柄（Buffer 内部是 Arc，clone 只增引用计数），
        // 供后续每帧 write_buffer 原地更新
        let cached = buffer_cache.buffers.entry(key).or_default();
        for (_binding, name, buffer) in &created_buffers {
            cached.insert(name.clone(), buffer.clone());
        }

        let bind_group =
            render_device.create_bind_group(Some(Self::label()), layout, &bind_group_entries);

        Ok(PreparedBindGroup {
            bindings: BindingResources(vec![]),
            bind_group,
        })
    }

    fn unprepared_bind_group(
        &self,
        _layout: &BindGroupLayout,
        _render_device: &RenderDevice,
        _param: &mut SystemParamItem<'_, '_, Self::Param>,
        _force_no_bindless: bool,
    ) -> Result<UnpreparedBindGroup, AsBindGroupError> {
        Err(AsBindGroupError::CreateBindGroupDirectly)
    }

    fn bind_group_layout_entries(
        _render_device: &RenderDevice,
        _force_no_bindless: bool,
    ) -> Vec<BindGroupLayoutEntry> {
        vec![]
    }
}

// ---------------------------------------------------------------------------
// Material 实现
// ---------------------------------------------------------------------------

impl Material for ParticleMaterialDynamic {
    // 粒子是半透明叠加绘制，不写深度 prepass、不投阴影；否则 prepass
    // 管线无 fragment 阶段，specialize 会拿到 fragment=None
    fn enable_prepass() -> bool {
        false
    }

    fn enable_shadows() -> bool {
        false
    }

    fn alpha_mode(&self) -> AlphaMode {
        match self.blend_mode {
            1 | 4 => AlphaMode::Blend,
            _ => AlphaMode::Opaque,
        }
    }

    fn specialize(
        _pipeline: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let data = &key.bind_group_data;

        // 覆写为预编译 SPIR-V 着色器对：入口点 main，不经 naga_oil，shader defs 无意义
        descriptor.vertex.entry_point = Some("main".into());
        descriptor.vertex.shader = data.shader_vert.clone();
        descriptor.vertex.shader_defs.clear();

        // prepass/shadow 管线没有 fragment 阶段（或无 color target），粒子材质
        // 不参与这些 pass 的着色，只在存在时覆写，避免 unwrap 炸渲染线程
        if let Some(fragment) = descriptor.fragment.as_mut() {
            fragment.entry_point = Some("main".into());
            fragment.shader = data.shader_frag.clone();
            fragment.shader_defs.clear();

            // blend mode
            if let Some(Some(target)) = fragment.targets.get_mut(0).map(|t| t.as_mut()) {
                if data.blend_mode == 4 {
                    target.blend = Some(BlendState {
                        color: BlendComponent {
                            src_factor: BlendFactor::SrcAlpha,
                            dst_factor: BlendFactor::One,
                            operation: BlendOperation::Add,
                        },
                        alpha: BlendComponent::OVER,
                    });
                }
            }
        }

        // 顶点布局按几何族选择（shader location 与各族 SPIR-V 输入变量一致）；
        // UnlitDecal 不覆盖（沿用默认布局，网格来自地图几何体）
        match data.kind {
            ParticleRenderKind::Quad => {
                let vertex_layout = layout.0.get_layout(&[
                    ATTRIBUTE_WORLD_POSITION.at_shader_location(0),
                    Mesh::ATTRIBUTE_COLOR.at_shader_location(1),
                    ATTRIBUTE_UV_FRAME.at_shader_location(2),
                    ATTRIBUTE_LIFETIME.at_shader_location(3),
                ])?;
                descriptor.vertex.buffers = vec![vertex_layout];
            }
            ParticleRenderKind::Mesh => {
                // MeshVs BASE 反汇编实际输入：POSITION@0(vec3)、NORMAL@1(vec3)、TEXCOORD@2(vec2)
                let vertex_layout = layout.0.get_layout(&[
                    Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
                    Mesh::ATTRIBUTE_NORMAL.at_shader_location(1),
                    Mesh::ATTRIBUTE_UV_0.at_shader_location(2),
                ])?;
                descriptor.vertex.buffers = vec![vertex_layout];
            }
            ParticleRenderKind::Distortion => {
                let vertex_layout = layout.0.get_layout(&[
                    ATTRIBUTE_WORLD_POSITION_VEC4.at_shader_location(0),
                    Mesh::ATTRIBUTE_COLOR.at_shader_location(3),
                    ATTRIBUTE_LIFETIME.at_shader_location(8),
                    Mesh::ATTRIBUTE_UV_0.at_shader_location(9),
                ])?;
                descriptor.vertex.buffers = vec![vertex_layout];
            }
            ParticleRenderKind::SkinnedMesh => {
                let vertex_layout = layout.0.get_layout(&[
                    Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
                    Mesh::ATTRIBUTE_JOINT_WEIGHT.at_shader_location(1),
                    Mesh::ATTRIBUTE_NORMAL.at_shader_location(2),
                    Mesh::ATTRIBUTE_JOINT_INDEX.at_shader_location(7),
                    Mesh::ATTRIBUTE_UV_0.at_shader_location(8),
                ])?;
                descriptor.vertex.buffers = vec![vertex_layout];
            }
            ParticleRenderKind::UnlitDecal => {}
        }
        descriptor.primitive.cull_mode = None;

        // 合并 vert + frag 布局条目，塞进 bind group 3 号槽
        let combined_entries = build_combined_entries(&data.vert_layout, &data.frag_layout);
        let layout_desc = BindGroupLayoutDescriptor {
            label: "ParticleMaterialDynamicPipelineLayout".into(),
            entries: combined_entries,
        };
        if descriptor.layout.len() > 3 {
            descriptor.layout[3] = layout_desc;
        } else {
            while descriptor.layout.len() < 3 {
                descriptor.layout.push(BindGroupLayoutDescriptor {
                    label: "EmptyPlaceholder".into(),
                    entries: vec![],
                });
            }
            descriptor.layout.push(layout_desc);
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Extract / Prepare 系统
// ---------------------------------------------------------------------------

/// Extract：收集场景中实际用到的所有 [`PipelineLayoutKey`] 及其两侧布局描述。
/// 材质组合极少变化，无新增/变更时直接跳过，避免每帧遍历与堆分配。
fn extract_dynamic_material_keys(
    material_query: Extract<Query<&MeshMaterial3d<ParticleMaterialDynamic>>>,
    changed_materials: Extract<Query<(), Changed<MeshMaterial3d<ParticleMaterialDynamic>>>>,
    material_assets: Extract<Res<Assets<ParticleMaterialDynamic>>>,
    mut extracted: ResMut<ExtractedMaterialKeys>,
) {
    if !material_assets.is_changed() && changed_materials.is_empty() {
        return;
    }

    let mut seen = std::collections::HashSet::new();
    extracted.keys.clear();
    for mat_handle in material_query.iter() {
        if let Some(mat) = material_assets.get(&mat_handle.0) {
            let key = mat.layout_key();
            if seen.insert(key.clone()) {
                extracted.keys.push(ExtractedKeyEntry {
                    key,
                    vert_layout: mat.vert_layout.clone(),
                    frag_layout: mat.frag_layout.clone(),
                });
            }
        }
    }
}

/// Prepare："运行时拼积木"——只为场景中真正存在的材质组合创建 GPU Layout
fn prepare_dynamic_bind_group_layouts(
    render_device: Res<RenderDevice>,
    extracted_keys: Res<ExtractedMaterialKeys>,
    mut cache: ResMut<RenderDynamicLayoutCache>,
) {
    if !extracted_keys.is_changed() {
        return;
    }

    for entry in &extracted_keys.keys {
        if cache.layouts.contains_key(&entry.key) {
            continue;
        }
        let combined_entries = build_combined_entries(&entry.vert_layout, &entry.frag_layout);
        let layout = render_device
            .create_bind_group_layout("ParticleMaterialDynamicLayout", &combined_entries);
        cache.layouts.insert(entry.key.clone(), layout);
    }
}

// ---------------------------------------------------------------------------
// 插件
// ---------------------------------------------------------------------------

#[derive(Default)]
pub struct PluginDynamicMaterial;

impl Plugin for PluginDynamicMaterial {
    fn build(&self, app: &mut App) {
        // 粒子不参与 prepass/shadow（enable_prepass/enable_shadows 已关）
        app.add_plugins(MaterialPlugin::<ParticleMaterialDynamic>::default());
        app.init_asset::<ParticleMaterialDynamic>();

        if let Some(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app.init_resource::<RenderDynamicLayoutCache>();
            render_app.init_resource::<ExtractedMaterialKeys>();
            render_app.init_resource::<DynamicUniformBufferCache>();
            render_app.init_resource::<SharedSamplerCache>();
            render_app.init_resource::<RenderSharedRenderData>();
            render_app.add_systems(
                ExtractSchedule,
                (extract_dynamic_material_keys, extract_shared_render_data),
            );
            render_app.add_systems(
                Render,
                (prepare_dynamic_bind_group_layouts, prepare_shared_samplers)
                    .in_set(RenderSystems::Prepare),
            );
        }
    }
}
