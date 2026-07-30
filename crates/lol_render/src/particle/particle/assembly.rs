//! 装配层：对齐逆向 `ShaderEffect_BuildShaderPathAndDefines`(sub_1412DD450) 的
//! 「几何族 / pass / 宏」派生规则，作为集中、可扩展的单一装配入口，
//! 替代散落在各 emitter 的临时逻辑。
//!
//! 逆向装配矩阵（IDA 已核实）：
//!   - 几何族 v14：0=quad / 1=mesh / 2=skinned（由 renderType 与 passIndex 表推导）；
//!     renderType==7 → UnlitDecal 特例；renderType==12 跳过。
//!   - pass a3：1=shadow / 2,3=distortion / 其它=普通。
//!   - UV 模式（desc.gap200[0]）：1=SCREEN_SPACE_UV、2=SEPARATE_ALPHA_UV、
//!     3..5=LOCAL_SPACE_UV；quad 的 1/2 走片段追加，mesh/skin 走同名 define。
//!   - 追加宏：REFLECTIVE、MULT_PASS、Slice(`*_PS_Slice`，仅 PS 追加)、
//!     USE_VERTEX_COLORS、SOFT_PARTICLES、MASKED、ALPHA_EROSION、DISABLE_FOW、
//!     PALETTIZE_TEXTURES。UnlitDecal 仅支持 MASKED/MULT_PASS/ALPHA_EROSION/
//!     PALETTIZE_TEXTURES。

use lol_base::particle::ConfigVfxEmitterDefinition;

use crate::particle::emitters::utils::{EmitterType, get_emitter_type};
use crate::particle::particle::dynamic::ParticleRenderKind;

/// 渲染 pass，对齐逆向 pass 参数 a3：1=shadow / 2,3=distortion / 其它=普通。
/// 本轮仅普通 pass 有已抽取的 SPIR-V 变体；shadow / distortion-pass 列为延后项。
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub enum ParticleRenderPass {
    #[default]
    Normal,
    Shadow,
    Distortion,
}

/// 从发射器定义推导渲染几何族（复用 [`get_emitter_type`] 的图元分类）。
/// 未知图元回退 Quad（与逆向默认 renderType 走 quad 族一致）。
pub fn derive_kind(def: &ConfigVfxEmitterDefinition) -> ParticleRenderKind {
    match get_emitter_type(def) {
        EmitterType::Quad => ParticleRenderKind::Quad,
        EmitterType::Mesh => ParticleRenderKind::Mesh,
        EmitterType::SkinnedMesh => ParticleRenderKind::SkinnedMesh,
        EmitterType::Decal => ParticleRenderKind::UnlitDecal,
        EmitterType::Distortion => ParticleRenderKind::Distortion,
        EmitterType::Unknown => ParticleRenderKind::Quad,
    }
}

/// 按逆向宏表派生 (vs_defs, ps_defs)。
///
/// 本轮仅产出对应已抽取 SPIR-V 变体的子集：驱动各宏的配置字段
/// （palette/erosion/reflection/soft_particle/team_color/uv_mode 等）在
/// `ConfigVfxEmitterDefinition` 中尚未补全，全部默认关闭——与逆向默认
/// 描述符（全零）装配出的 BASE 变体一致。字段补全后在此按矩阵逐项接入。
pub fn derive_defs(
    _def: &ConfigVfxEmitterDefinition,
    _kind: ParticleRenderKind,
    _pass: ParticleRenderPass,
) -> (Vec<String>, Vec<String>) {
    (vec![], vec![])
}
