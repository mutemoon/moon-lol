//! Shader 提取与转译管线：
//! 从 ShaderCache.dx11.wad.client 提取 DXBC shader，
//! 通过 dxbc-compiler.exe 编译为 SPIR-V，生成 RDEF 布局与 ShaderMap，
//! 并在 assets/shaders/ 下输出 .spv 与 map.ron 场景文件。

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

use bevy::prelude::{DynamicWorldBuilder, TaskPoolPlugin, *};
use league_core::extract::{X3dSharedData, X3dSharedSamplerDef};
use league_file::shader::LeagueShaderToc;
use league_loader::prop_bin::LeagueWadLoaderTrait;
use league_loader::wad::LeagueWadLoader;
use league_utils::hash_wad;
use lol_base_render::shader::{
    LeagueShader, ShaderMap, ShaderMapEntry, SharedRenderData, SharedSamplerDef, SharedTextureDef,
};
use lol_base_render::shader_layout::{
    BindingDescriptor, BindingTypeDesc, ShaderLayoutDescriptor, ShaderMemberLayout,
};
use lol_base_render::spirv_strip::{
    STORAGE_INPUT, STORAGE_OUTPUT, interface_vector_widths, remap_bindings, strip_spirv,
    widen_ps_inputs,
};
use rayon::prelude::*;

pub const DEFAULT_TOC_PATHS: &[&str] = &[
    "assets/shaders/hlsl/particlesystem/quad_ps_slice.ps-dx11",
    "assets/shaders/hlsl/particlesystem/quad_vs.vs-dx11",
    "assets/shaders/hlsl/particlesystem/quad_ps.ps-dx11",
    "assets/shaders/hlsl/environment/unlit_decal_ps.ps-dx11",
    "assets/shaders/hlsl/environment/unlit_decal_vs.vs-dx11",
    "assets/shaders/hlsl/particlesystem/distortion_ps.ps-dx11",
    "assets/shaders/hlsl/particlesystem/distortion_vs.vs-dx11",
    "assets/shaders/hlsl/particlesystem/mesh_ps.ps-dx11",
    "assets/shaders/hlsl/particlesystem/mesh_vs.vs-dx11",
    "assets/shaders/hlsl/skinnedmesh/particle_ps.ps-dx11",
    "assets/shaders/hlsl/skinnedmesh/particle_vs.vs-dx11",
];

#[derive(Debug, Clone)]
pub struct ExtractShaderOptions {
    /// 游戏根目录（包含 DATA/FINAL/ 的目录）
    pub game_path: PathBuf,
    /// 输出 shaders 目录（通常为 assets/shaders）
    pub out_dir: PathBuf,
    /// dxbc-compiler 可执行文件路径
    pub dxbc_compiler_path: PathBuf,
    /// 需要处理的 TOC 路径列表，空则使用默认列表
    pub toc_paths: Vec<String>,
    /// 是否跳过已存在的输出文件
    pub skip_existing: bool,
    /// 是否同时提取并保存原始 DXBC 文件
    pub save_dxbc: bool,
}

/// 自动探测 dxbc-compiler 工具路径
pub fn find_dxbc_compiler(assets_dir: &Path) -> Option<PathBuf> {
    let bin_name = if cfg!(target_os = "windows") {
        "dxbc-compiler.exe"
    } else {
        "dxbc-compiler"
    };

    let candidates = [
        assets_dir.join("tools").join(bin_name),
        PathBuf::from("assets").join("tools").join(bin_name),
    ];

    for c in candidates {
        if c.exists() {
            return Some(c);
        }
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let exe_candidates = [
                parent.join("tools").join(bin_name),
                parent.join("assets").join("tools").join(bin_name),
            ];
            for c in exe_candidates {
                if c.exists() {
                    return Some(c);
                }
            }
        }
    }

    None
}

// ---------------------------------------------------------------------------
// RDEF 解析：从 DXBC chunk 提取 cbuffer 名/成员/绑定表和资源绑定表
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct RdefVar {
    name: String,
    offset: u32,
    size: u32,
    #[allow(dead_code)]
    used: bool,
}

#[derive(Debug, Clone)]
struct RdefCbuffer {
    name: String,
    size: u32,
    vars: Vec<RdefVar>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum ResKind {
    CBuffer,
    Sampler,
    Texture,
    Other(u32),
}

#[derive(Debug, Clone)]
struct RdefResource {
    name: String,
    kind: ResKind,
    bind_point: u32,
}

#[derive(Debug, Clone)]
struct RdefInfo {
    cbuffers: Vec<RdefCbuffer>,
    resources: Vec<RdefResource>,
    is_pixel: bool,
}

fn u32_at(b: &[u8], off: usize) -> Option<u32> {
    Some(u32::from_le_bytes(b.get(off..off + 4)?.try_into().ok()?))
}

fn cstr_at(b: &[u8], off: usize) -> Option<String> {
    let end = b[off..].iter().position(|&c| c == 0)? + off;
    let s = std::str::from_utf8(&b[off..end]).ok()?;
    Some(s.to_string())
}

fn parse_rdef(dxbc: &[u8]) -> Result<RdefInfo, String> {
    if dxbc.get(..4) != Some(b"DXBC") {
        return Err("非 DXBC 文件（magic 不符）".into());
    }
    let chunk_count = u32_at(dxbc, 0x1C).ok_or("读 chunkCount 越界")?;
    let mut base = None;
    for c in 0..chunk_count as usize {
        let off = u32_at(dxbc, 0x20 + 4 * c).ok_or("chunk 偏移表越界")? as usize;
        if dxbc.get(off..off + 4) == Some(b"RDEF") {
            base = Some(off + 8);
            break;
        }
    }
    let base = base.ok_or("无 RDEF chunk")?;
    let rd = |o: usize| u32_at(dxbc, base + o).ok_or(format!("RDEF+{o} 越界"));
    let cb_count = rd(0)? as usize;
    let cb_off = rd(4)? as usize;
    let res_count = rd(8)? as usize;
    let res_off = rd(12)? as usize;
    let major = *dxbc.get(base + 17).ok_or("RDEF 头越界")?;
    let program_type = u16::from_le_bytes(
        dxbc.get(base + 18..base + 20)
            .ok_or("RDEF 头越界")?
            .try_into()
            .unwrap(),
    );
    let var_stride = if major >= 5 { 40 } else { 24 };

    let mut cbuffers = Vec::new();
    for i in 0..cb_count {
        let e = base + cb_off + i * 24;
        let name = cstr_at(
            dxbc,
            base + u32_at(dxbc, e).ok_or("cbuffer 条目越界")? as usize,
        )
        .ok_or("cbuffer 名越界")?;
        let var_count = u32_at(dxbc, e + 4).ok_or("varCount 越界")? as usize;
        let var_off = u32_at(dxbc, e + 8).ok_or("varOffset 越界")? as usize;
        let size = u32_at(dxbc, e + 12).ok_or("cbuffer size 越界")?;
        let mut vars = Vec::new();
        for v in 0..var_count {
            let ve = base + var_off + v * var_stride;
            let vname = cstr_at(
                dxbc,
                base + u32_at(dxbc, ve).ok_or("var 条目越界")? as usize,
            )
            .ok_or("var 名越界")?;
            let offset = u32_at(dxbc, ve + 4).ok_or("var offset 越界")?;
            let vsize = u32_at(dxbc, ve + 8).ok_or("var size 越界")?;
            let flags = u32_at(dxbc, ve + 12).ok_or("var flags 越界")?;
            vars.push(RdefVar {
                name: vname,
                offset,
                size: vsize,
                used: flags & 2 != 0,
            });
        }
        cbuffers.push(RdefCbuffer { name, size, vars });
    }

    let mut resources = Vec::new();
    'stride: for stride in [32usize, 40] {
        let mut parsed = Vec::new();
        for i in 0..res_count {
            let e = base + res_off + i * stride;
            let Some(name_off) = u32_at(dxbc, e) else {
                continue 'stride;
            };
            let Some(name) = cstr_at(dxbc, base + name_off as usize) else {
                continue 'stride;
            };
            if name.is_empty() || name.len() > 200 || !name.chars().all(|c| c.is_ascii_graphic()) {
                continue 'stride;
            }
            let input_type = u32_at(dxbc, e + 4).ok_or("res 类型越界")?;
            let bind_point = u32_at(dxbc, e + 20).ok_or("res bindPoint 越界")?;
            let kind = match input_type {
                0 => ResKind::CBuffer,
                2 => ResKind::Texture,
                3 => ResKind::Sampler,
                t => ResKind::Other(t),
            };
            parsed.push(RdefResource {
                name,
                kind,
                bind_point,
            });
        }
        resources = parsed;
        break;
    }

    Ok(RdefInfo {
        cbuffers,
        resources,
        is_pixel: program_type == 0xFFFF,
    })
}

fn build_rdef_layout(dxbc: &[u8]) -> Result<ShaderLayoutDescriptor, String> {
    let rdef = parse_rdef(dxbc)?;
    let shift: u32 = if rdef.is_pixel { 100 } else { 0 };

    let cb_map: HashMap<&str, (u32, &[RdefVar])> = rdef
        .cbuffers
        .iter()
        .map(|cb| (cb.name.as_str(), (cb.size, cb.vars.as_slice())))
        .collect();

    let mut bindings: BTreeMap<String, BindingDescriptor> = BTreeMap::new();

    for res in &rdef.resources {
        let type_base: u32 = match res.kind {
            ResKind::CBuffer => 0,
            ResKind::Texture => 16,
            ResKind::Sampler => 32,
            ResKind::Other(t) => {
                eprintln!(
                    "[RDEF] 未知资源类型 {} (kind={t})，bind_point={} — 跳过",
                    res.name, res.bind_point
                );
                continue;
            }
        };
        let binding_index = shift + type_base + res.bind_point;

        let type_desc = match res.kind {
            ResKind::CBuffer => {
                let (total_size, vars) = cb_map.get(res.name.as_str()).copied().unwrap_or((0, &[]));
                let members: BTreeMap<String, ShaderMemberLayout> = vars
                    .iter()
                    .map(|v| {
                        (
                            v.name.clone(),
                            ShaderMemberLayout {
                                name: v.name.clone(),
                                offset: v.offset as usize,
                                size: v.size as usize,
                            },
                        )
                    })
                    .collect();
                BindingTypeDesc::UniformBuffer {
                    total_size: total_size as usize,
                    members,
                }
            }
            ResKind::Texture => BindingTypeDesc::Texture2d,
            ResKind::Sampler => BindingTypeDesc::Sampler,
            ResKind::Other(_) => continue,
        };

        bindings.insert(
            res.name.clone(),
            BindingDescriptor {
                binding_index,
                name: res.name.clone(),
                type_desc,
            },
        );
    }

    Ok(ShaderLayoutDescriptor { bindings })
}

pub fn get_league_shader_type(toc_path: &str) -> Option<LeagueShader> {
    let lower = toc_path.to_lowercase();
    if lower.contains("quad_ps_slice") {
        Some(LeagueShader::QuadPsSlice)
    } else if lower.contains("quad_vs") {
        Some(LeagueShader::QuadVs)
    } else if lower.contains("quad_ps") {
        Some(LeagueShader::QuadPs)
    } else if lower.contains("unlit_decal_ps") {
        Some(LeagueShader::UnlitDecalPs)
    } else if lower.contains("unlit_decal_vs") {
        Some(LeagueShader::UnlitDecalVs)
    } else if lower.contains("distortion_ps") {
        Some(LeagueShader::DistortionPs)
    } else if lower.contains("distortion_vs") {
        Some(LeagueShader::DistortionVs)
    } else if lower.contains("mesh_ps") {
        Some(LeagueShader::MeshPs)
    } else if lower.contains("mesh_vs") {
        Some(LeagueShader::MeshVs)
    } else if lower.contains("particle_ps") {
        Some(LeagueShader::SkinnedMeshParticlePs)
    } else if lower.contains("particle_vs") {
        Some(LeagueShader::SkinnedMeshParticleVs)
    } else {
        None
    }
}

fn resolve_address_mode(v: Option<u8>) -> u8 {
    match v {
        None => 0,
        Some(0) => 1,
        Some(2) => 2,
        Some(_) => 0,
    }
}

pub fn extract_shared_render_data(game_path: &str) -> SharedRenderData {
    let mut data = SharedRenderData::default();

    let wad_rel = "DATA/FINAL/Shaders/Shaders.wad.client";
    let loader = match LeagueWadLoader::from_relative_path(game_path, wad_rel) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("[WARN] 无法加载 Shaders WAD ({wad_rel}): {e}");
            return data;
        }
    };
    let prop = match loader.get_prop_bin_by_path("assets/shaders/shareddata.bin") {
        Ok(p) => p,
        Err(e) => {
            eprintln!("[WARN] 无法读取 assets/shaders/shareddata.bin: {e}");
            return data;
        }
    };

    let sampler_hash = league_utils::type_name_to_hash("X3DSharedSamplerDef");
    for entry in prop.iter_entry_by_class(sampler_hash) {
        if let Ok(def) = league_property::from_entry::<X3dSharedSamplerDef>(entry) {
            data.samplers.insert(
                def.name.clone(),
                SharedSamplerDef {
                    address_mode_u: resolve_address_mode(def.address_mode_u),
                    address_mode_v: resolve_address_mode(def.address_mode_v),
                    address_mode_w: resolve_address_mode(def.address_mode_w),
                    max_anisotropy: def.max_anisotropy.unwrap_or(0),
                    mip_filter: def.mip_filter.unwrap_or(1),
                    mip_lod_bias: def.mip_lod_bias.unwrap_or(0),
                    register: def.register.unwrap_or(-1),
                },
            );
        }
    }

    let data_hash = league_utils::type_name_to_hash("X3DSharedData");
    for entry in prop.iter_entry_by_class(data_hash) {
        if let Ok(shared) = league_property::from_entry::<X3dSharedData>(entry) {
            for tex in &shared.textures {
                data.textures.insert(
                    tex.name.clone(),
                    SharedTextureDef {
                        kind: tex.r#type.unwrap_or(0),
                        sampler: tex.sampler.unwrap_or(0),
                        default_value: tex.default_value.unwrap_or(Vec4::ZERO),
                    },
                );
            }
        }
    }

    data
}

fn build_hash_to_defs(
    base_defines: &[league_file::shader::ShaderMacroDefinition],
) -> HashMap<u64, Vec<String>> {
    let names: Vec<String> = base_defines.iter().map(|d| d.name.text.clone()).collect();
    let n = names.len();
    let mut map = HashMap::new();
    if n > 24 {
        eprintln!("[WARN] base_define 数量 {n} 过多，跳过 def 反解");
        return map;
    }
    for mask in 0u32..(1u32 << n) {
        let defs: Vec<String> = (0..n)
            .filter(|&j| mask & (1 << j) != 0)
            .map(|j| names[j].clone())
            .collect();
        let hash = league_utils::hash_shader_spec(&defs);
        map.insert(hash, defs);
    }
    map
}

fn defs_to_stem(defs: &[String]) -> String {
    if defs.is_empty() {
        return "BASE".to_string();
    }
    let mut sorted = defs.to_vec();
    sorted.sort();
    let stem = sorted.join("__");
    if stem.len() > 200 {
        let h = league_utils::hash_shader(&stem);
        format!("{}__{:016x}", &stem[..180], h)
    } else {
        stem
    }
}

fn sanitize_name(path: &str) -> String {
    let normalized = path.replace('\\', "/");
    let relative = normalized
        .strip_prefix("assets/shaders/")
        .unwrap_or(&normalized);

    let shader_type = if relative.contains(".ps.") || relative.contains(".ps-") {
        "ps"
    } else if relative.contains(".vs.") || relative.contains(".vs-") {
        "vs"
    } else {
        ""
    };

    let path_obj = Path::new(relative);
    let parent = path_obj.parent().unwrap_or(Path::new(""));
    let file_name = path_obj.file_name().and_then(|f| f.to_str()).unwrap_or("");
    let base_name = file_name.split('.').next().unwrap_or(file_name);

    let mut clean_base = base_name.replace("_ps_", "_").replace("_vs_", "_");
    if clean_base.ends_with("_ps") {
        clean_base.truncate(clean_base.len() - 3);
    } else if clean_base.ends_with("_vs") {
        clean_base.truncate(clean_base.len() - 3);
    }

    let mut clean_path = parent.join(clean_base);
    if !shader_type.is_empty() {
        clean_path = clean_path.join(shader_type);
    }
    clean_path.to_string_lossy().replace('\\', "/")
}

fn parse_dxbc_chunk(data: &[u8]) -> Result<Vec<Vec<u8>>, String> {
    let mut blobs = Vec::new();
    let mut offset = 0usize;

    while offset < data.len() {
        if offset + 4 > data.len() {
            break;
        }

        let length = u32::from_le_bytes(
            data[offset..offset + 4]
                .try_into()
                .map_err(|e| format!("解析 length 失败: {e}"))?,
        ) as usize;

        if length == 0 {
            break;
        }

        if offset + 4 + length > data.len() {
            eprintln!(
                "[WARN] blob @ offset {} 数据截断 (声明长度 {}，剩余 {})",
                offset,
                length,
                data.len() - offset - 4
            );
            break;
        }

        let dxbc_size = length.saturating_sub(1);
        let dxbc_data = data[offset + 4..offset + 4 + dxbc_size].to_vec();
        blobs.push(dxbc_data);

        offset += 4 + length;
    }

    Ok(blobs)
}

fn compile_dxbc_to_spirv(
    dxbc_data: &[u8],
    dxbc_compiler_path: &Path,
    work_dir: &Path,
    idx: usize,
    is_pixel: bool,
    save_dxbc: bool,
) -> Result<Vec<u8>, String> {
    let prefix = format!("_tmp_{:04}", idx);
    let dxbc_filename = format!("{}.dxbc", prefix);
    let spv_filename = format!("{}.spv", prefix);

    let dxbc_path_tmp = work_dir.join(&dxbc_filename);
    let spv_path_tmp = work_dir.join(&spv_filename);

    fs::write(&dxbc_path_tmp, dxbc_data).map_err(|e| format!("写入临时 DXBC 失败: {e}"))?;

    let mut cmd = Command::new(dxbc_compiler_path);
    cmd.arg("--spv").arg(&spv_filename).arg("--set").arg("3");
    if is_pixel {
        cmd.arg("--binding-shift").arg("100");
    }
    cmd.arg(&dxbc_filename).current_dir(work_dir);

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    let compiler_out = cmd
        .output()
        .map_err(|e| format!("启动 dxbc_compiler 失败: {e}"))?;

    if save_dxbc {
        let _ = fs::copy(&dxbc_path_tmp, work_dir.join(format!("{}.dxbc", prefix)));
    }
    let _ = fs::remove_file(&dxbc_path_tmp);

    if !compiler_out.status.success() {
        let stderr = String::from_utf8_lossy(&compiler_out.stderr);
        let stdout = String::from_utf8_lossy(&compiler_out.stdout);
        let _ = fs::remove_file(&spv_path_tmp);
        return Err(format!(
            "dxbc_compiler 编译失败:\nstdout: {}\nstderr: {}",
            stdout.trim(),
            stderr.trim()
        ));
    }

    let spv_bytes = fs::read(&spv_path_tmp).map_err(|e| format!("读取临时 SPV 失败: {e}"))?;
    let _ = fs::remove_file(&spv_path_tmp);

    let spv_bytes = strip_spirv(&spv_bytes);

    Ok(spv_bytes)
}

fn paired_vs_family(family: LeagueShader) -> Option<LeagueShader> {
    match family {
        LeagueShader::QuadPs | LeagueShader::QuadPsSlice => Some(LeagueShader::QuadVs),
        LeagueShader::MeshPs => Some(LeagueShader::MeshVs),
        LeagueShader::DistortionPs => Some(LeagueShader::DistortionVs),
        LeagueShader::UnlitDecalPs => Some(LeagueShader::UnlitDecalVs),
        LeagueShader::SkinnedMeshParticlePs => Some(LeagueShader::SkinnedMeshParticleVs),
        LeagueShader::QuadVs
        | LeagueShader::MeshVs
        | LeagueShader::DistortionVs
        | LeagueShader::UnlitDecalVs
        | LeagueShader::SkinnedMeshParticleVs => None,
    }
}

fn align_stage_interfaces(
    spv_path_map: &HashMap<LeagueShader, HashMap<u64, PathBuf>>,
    global_defs: &HashMap<LeagueShader, HashMap<u64, Vec<String>>>,
    log: &impl Fn(&str),
) {
    use std::collections::{BTreeMap, BTreeSet};

    for (family, variants) in spv_path_map {
        let Some(vs_family) = paired_vs_family(*family) else {
            continue;
        };
        let Some(vs_variants) = spv_path_map.get(&vs_family) else {
            continue;
        };
        let (Some(ps_defs_map), Some(vs_defs_map)) =
            (global_defs.get(family), global_defs.get(&vs_family))
        else {
            log(&format!(
                "[WARN] {family:?}/{vs_family:?} 缺少 def 反解表，跳过接口对齐"
            ));
            continue;
        };

        let vs_macros: HashSet<&str> = vs_defs_map.values().flatten().map(|s| s.as_str()).collect();

        let mut ps_vs_files: BTreeMap<&PathBuf, BTreeSet<&PathBuf>> = BTreeMap::new();
        for (ps_hash, ps_path) in variants {
            let Some(ps_defs) = ps_defs_map.get(ps_hash) else {
                continue;
            };
            let paired_defs: Vec<String> = ps_defs
                .iter()
                .filter(|d| vs_macros.contains(d.as_str()))
                .cloned()
                .collect();
            let vs_hash = league_utils::hash_shader_spec(&paired_defs);
            let Some(vs_path) = vs_variants.get(&vs_hash) else {
                log(&format!(
                    "[WARN] {family:?} 变体 {} 的配对 VS 变体 {} 缺失，跳过",
                    defs_to_stem(ps_defs),
                    defs_to_stem(&paired_defs)
                ));
                continue;
            };
            ps_vs_files.entry(ps_path).or_default().insert(vs_path);
        }

        let mut vs_cover: HashMap<&PathBuf, Option<BTreeMap<(u32, u32), u32>>> = HashMap::new();
        let mut rewritten = 0usize;
        for (ps_path, vs_paths) in &ps_vs_files {
            let Ok(bytes) = fs::read(ps_path) else {
                continue;
            };
            let Some(in_widths) = interface_vector_widths(&bytes, STORAGE_INPUT) else {
                continue;
            };

            let mut desired: Option<BTreeMap<(u32, u32), u32>> = None;
            for vs_path in vs_paths {
                let vs_out = vs_cover.entry(vs_path).or_insert_with(|| {
                    fs::read(vs_path)
                        .ok()
                        .and_then(|b| interface_vector_widths(&b, STORAGE_OUTPUT))
                });
                let Some(vs_out) = vs_out.as_ref() else {
                    continue;
                };
                let mut m: BTreeMap<(u32, u32), u32> = BTreeMap::new();
                for (&(loc, comp), _) in &in_widths {
                    let cover = vs_out
                        .range((loc, 0)..=(loc, comp))
                        .next_back()
                        .filter(|&(&(_, c0), &w)| c0 + w > comp);
                    let Some((&(_, c0), &w)) = cover else {
                        log(&format!(
                            "[WARN] {} 输入 loc{loc} comp{comp} 在配对 VS {} 无输出覆盖",
                            ps_path.display(),
                            vs_path.display()
                        ));
                        continue;
                    };
                    let cap = in_widths
                        .range((loc, comp + 1)..(loc + 1, 0))
                        .map(|(&(_, c), _)| c)
                        .next()
                        .unwrap_or(4)
                        .min(4);
                    m.insert((loc, comp), (c0 + w).min(cap) - comp);
                }
                desired = Some(match desired {
                    None => m,
                    Some(prev) => {
                        let mut merged = BTreeMap::new();
                        for (k, w) in &prev {
                            if let Some(w2) = m.get(k) {
                                if w2 != w {
                                    log(&format!(
                                        "[WARN] {} 多个配对 VS 在 {k:?} 目标宽度冲突（{w} vs {w2}）",
                                        ps_path.display()
                                    ));
                                }
                                merged.insert(*k, (*w).min(*w2));
                            }
                        }
                        merged
                    }
                });
            }
            let Some(desired) = desired else {
                continue;
            };

            let mut targets: BTreeMap<(u32, u32), u32> = BTreeMap::new();
            for (&(loc, comp), &cur) in &in_widths {
                let Some(&want) = desired.get(&(loc, comp)) else {
                    continue;
                };
                if cur < want {
                    targets.insert((loc, comp), want);
                }
            }
            if targets.is_empty() {
                continue;
            }
            match widen_ps_inputs(&bytes, &targets) {
                Some(new_bytes) => match fs::write(ps_path, &new_bytes) {
                    Ok(()) => rewritten += 1,
                    Err(e) => log(&format!("[ERROR] 写回 {} 失败: {e}", ps_path.display())),
                },
                None => log(&format!(
                    "[WARN] {} 存在无法安全加宽的接口变量用法，保留原样",
                    ps_path.display()
                )),
            }
        }
        if rewritten > 0 {
            log(&format!(
                "[ALIGN] {family:?}: 加宽 {rewritten}/{} 个 .spv 输入接口",
                ps_vs_files.len()
            ));
        }
    }
}

fn build_family_union(
    variants: &HashMap<u64, ShaderLayoutDescriptor>,
) -> BTreeMap<String, BindingDescriptor> {
    let mut union: BTreeMap<String, BindingDescriptor> = BTreeMap::new();
    for layout in variants.values() {
        for (name, desc) in &layout.bindings {
            let Some(existing) = union.get_mut(name) else {
                union.insert(name.clone(), desc.clone());
                continue;
            };
            match (&mut existing.type_desc, &desc.type_desc) {
                (
                    BindingTypeDesc::UniformBuffer {
                        total_size: a,
                        members: am,
                    },
                    BindingTypeDesc::UniformBuffer {
                        total_size: b,
                        members: bm,
                    },
                ) => {
                    *a = (*a).max(*b);
                    for (member_name, member) in bm {
                        am.entry(member_name.clone())
                            .or_insert_with(|| member.clone());
                    }
                }
                (BindingTypeDesc::Texture2d, BindingTypeDesc::Texture2d) => {}
                (BindingTypeDesc::Sampler, BindingTypeDesc::Sampler) => {}
                _ => {}
            }
        }
    }
    union
}

fn unify_family_layouts(
    layouts_map: &mut HashMap<LeagueShader, HashMap<u64, ShaderLayoutDescriptor>>,
    spv_path_map: &HashMap<LeagueShader, HashMap<u64, PathBuf>>,
    log: &impl Fn(&str),
) -> HashMap<LeagueShader, ShaderLayoutDescriptor> {
    let mut unions: HashMap<LeagueShader, BTreeMap<String, BindingDescriptor>> = HashMap::new();
    for (family, variants) in layouts_map.iter() {
        unions.insert(*family, build_family_union(variants));
    }

    let vk_bases: HashMap<LeagueShader, u32> = unions
        .keys()
        .map(|family| {
            let base = match paired_vs_family(*family) {
                Some(vs) => unions.get(&vs).map(|u| u.len() as u32).unwrap_or(0),
                None => 0,
            };
            (*family, base)
        })
        .collect();

    for (family, union) in unions.iter_mut() {
        let base = vk_bases[family];
        for (rank, desc) in union.values_mut().enumerate() {
            desc.binding_index = base + rank as u32;
        }
        log(&format!(
            "[UNIFY] {family:?}: {} 个变体 → 并集 {} 个 binding",
            layouts_map[family].len(),
            union.len(),
        ));
    }

    for (family, variants) in layouts_map.iter() {
        let Some(union) = unions.get(family) else {
            continue;
        };
        let vk_index: BTreeMap<&String, u32> =
            union.iter().map(|(n, d)| (n, d.binding_index)).collect();

        let mut done_paths: HashSet<PathBuf> = HashSet::new();
        let mut rewritten = 0usize;
        for (hash, layout) in variants {
            let Some(spv_path) = spv_path_map.get(family).and_then(|m| m.get(hash)) else {
                continue;
            };
            if !done_paths.insert(spv_path.clone()) {
                continue;
            }
            let mut remap: BTreeMap<u32, u32> = BTreeMap::new();
            for (name, desc) in &layout.bindings {
                if let Some(&new) = vk_index.get(name) {
                    remap.insert(desc.binding_index, new);
                }
            }
            match fs::read(spv_path) {
                Ok(bytes) => {
                    if let Some(new_bytes) = remap_bindings(&bytes, &remap) {
                        if let Err(e) = fs::write(spv_path, &new_bytes) {
                            log(&format!("[ERROR] 写回 {} 失败: {e}", spv_path.display()));
                        } else {
                            rewritten += 1;
                        }
                    }
                }
                Err(e) => log(&format!("[ERROR] 读取 {} 失败: {e}", spv_path.display())),
            }
        }
        log(&format!(
            "[UNIFY] {family:?}: 改写 {}/{} 个 .spv 文件",
            rewritten,
            done_paths.len()
        ));
    }

    for (family, variants) in layouts_map.iter_mut() {
        let Some(union) = unions.get(family) else {
            continue;
        };
        for layout in variants.values_mut() {
            layout
                .bindings
                .retain(|_, desc| matches!(desc.type_desc, BindingTypeDesc::UniformBuffer { .. }));
            for (name, desc) in layout.bindings.iter_mut() {
                if let Some(union_desc) = union.get(name) {
                    desc.binding_index = union_desc.binding_index;
                }
            }
        }
    }

    unions
        .into_iter()
        .map(|(family, bindings)| (family, ShaderLayoutDescriptor { bindings }))
        .collect()
}

fn process_toc(
    wad_loader: &LeagueWadLoader,
    toc_path: &str,
    out_dir: &Path,
    dxbc_compiler_path: &Path,
    skip_existing: bool,
    save_dxbc: bool,
    log: &impl Fn(&str),
) -> Result<
    (
        HashMap<u64, String>,
        HashMap<u64, Vec<String>>,
        HashMap<PathBuf, ShaderLayoutDescriptor>,
    ),
    String,
> {
    let toc_hash = hash_wad(toc_path);
    let mut toc_reader = wad_loader
        .get_wad_entry_reader_by_hash(toc_hash)
        .map_err(|e| format!("找不到 TOC 文件 {} (hash={:x}): {}", toc_path, toc_hash, e))?;

    let mut toc_bytes = Vec::new();
    toc_reader
        .read_to_end(&mut toc_bytes)
        .map_err(|e| format!("读取 TOC 数据失败: {e}"))?;

    let (_, shader_toc) =
        LeagueShaderToc::parse(&toc_bytes).map_err(|e| format!("解析 TOC 失败: {e}"))?;

    let shader_type_str = if shader_toc.shader_type == 0 {
        "vertex"
    } else {
        "pixel"
    };
    log(&format!(
        "  [TOC] shader_count={}, bundled={}, type={}",
        shader_toc.shader_count, shader_toc.bundled_shader_count, shader_type_str
    ));

    let chunk_count = ((shader_toc.bundled_shader_count as f32 / 100.0).ceil() as usize).max(1);
    let mut dxbc_blobs: Vec<Vec<u8>> = Vec::new();

    for i in 0..chunk_count {
        let chunk_path = format!("{}_{}", toc_path, i * 100);
        let chunk_hash = hash_wad(&chunk_path);

        let mut chunk_reader = wad_loader
            .get_wad_entry_reader_by_hash(chunk_hash)
            .map_err(|e| format!("找不到 chunk {} (hash={:x}): {}", chunk_path, chunk_hash, e))?;

        let mut chunk_bytes = Vec::new();
        chunk_reader
            .read_to_end(&mut chunk_bytes)
            .map_err(|e| format!("读取 chunk 失败: {e}"))?;

        let blobs = parse_dxbc_chunk(&chunk_bytes)?;
        dxbc_blobs.extend(blobs);
    }

    let toc_name = sanitize_name(toc_path);
    let toc_out_dir = out_dir.join(&toc_name);
    fs::create_dir_all(&toc_out_dir).map_err(|e| format!("创建 TOC 输出目录失败: {e}"))?;

    let hash_to_defs = build_hash_to_defs(&shader_toc.base_defines);
    let mut bundled_defs: HashMap<usize, Vec<String>> = HashMap::new();
    for (shader_index, &shader_hash) in shader_toc.shader_hashes.iter().enumerate() {
        let idx = shader_toc.shader_ids[shader_index] as usize;
        let Some(defs) = hash_to_defs.get(&shader_hash) else {
            continue;
        };
        let better = match bundled_defs.get(&idx) {
            None => true,
            Some(cur) => {
                defs.len() < cur.len()
                    || (defs.len() == cur.len() && defs_to_stem(defs) < defs_to_stem(cur))
            }
        };
        if better {
            bundled_defs.insert(idx, defs.clone());
        }
    }

    let is_pixel = shader_toc.shader_type != 0;
    let spv_paths: Vec<Option<String>> = (0..shader_toc.bundled_shader_count as usize)
        .into_par_iter()
        .map(|idx| {
            if idx >= dxbc_blobs.len() {
                return None;
            }

            let stem = bundled_defs
                .get(&idx)
                .map(|d| defs_to_stem(d))
                .unwrap_or_else(|| format!("shader_{:04}", idx));
            let spv_filename = format!("{}.spv", stem);
            let spv_path = toc_out_dir.join(&spv_filename);
            let old_path = toc_out_dir.join(format!("shader_{:04}.spv", idx));
            let spv_relative = format!("shaders/{}/{}", toc_name, spv_filename);

            if old_path != spv_path && old_path.exists() && !spv_path.exists() {
                let _ = fs::rename(&old_path, &spv_path);
            }

            if save_dxbc {
                let dxbc_path = toc_out_dir.join(format!("{}.dxbc", stem));
                let _ = fs::write(&dxbc_path, &dxbc_blobs[idx]);
            }

            if skip_existing && spv_path.exists() {
                return Some(spv_relative);
            }

            match compile_dxbc_to_spirv(
                &dxbc_blobs[idx],
                dxbc_compiler_path,
                &toc_out_dir,
                idx,
                is_pixel,
                save_dxbc,
            ) {
                Ok(spv_bytes) => {
                    if save_dxbc {
                        let dxbc_path = toc_out_dir.join(format!("{}.dxbc", stem));
                        let _ = fs::write(&dxbc_path, &dxbc_blobs[idx]);
                    }
                    match fs::write(&spv_path, &spv_bytes) {
                        Ok(()) => Some(spv_relative),
                        Err(_) => None,
                    }
                }
                Err(_) => None,
            }
        })
        .collect();

    let mut rdef_layouts: HashMap<PathBuf, ShaderLayoutDescriptor> = HashMap::new();
    for idx in 0..shader_toc.bundled_shader_count as usize {
        if spv_paths[idx].is_none() || idx >= dxbc_blobs.len() {
            continue;
        }
        let stem = bundled_defs
            .get(&idx)
            .map(|d| defs_to_stem(d))
            .unwrap_or_else(|| format!("shader_{:04}", idx));
        let spv_filename = format!("{}.spv", stem);
        let spv_abs = toc_out_dir.join(&spv_filename);
        if let Ok(layout) = build_rdef_layout(&dxbc_blobs[idx]) {
            rdef_layouts.insert(spv_abs, layout);
        }
    }

    let mut map_entries: HashMap<u64, String> = HashMap::new();
    for (shader_index, &shader_hash) in shader_toc.shader_hashes.iter().enumerate() {
        let shader_id = shader_toc.shader_ids[shader_index] as usize;
        if let Some(Some(spv_path)) = spv_paths.get(shader_id) {
            map_entries.insert(shader_hash, spv_path.clone());
        }
    }

    log(&format!(
        "  [DONE] {} 个 shader 变体处理完成",
        map_entries.len()
    ));

    Ok((map_entries, hash_to_defs, rdef_layouts))
}

/// 执行完整 Shader 提取、转译、对齐与 map.ron 生成管线
pub fn extract_shaders_pipeline(
    options: &ExtractShaderOptions,
    on_log: Option<&dyn Fn(&str)>,
) -> Result<(), String> {
    let log = |msg: &str| {
        if let Some(cb) = on_log {
            cb(msg);
        } else {
            println!("{msg}");
        }
    };

    if !options.dxbc_compiler_path.exists() {
        return Err(format!(
            "找不到 dxbc-compiler: {}",
            options.dxbc_compiler_path.display()
        ));
    }

    let game_path_str = options.game_path.to_string_lossy().to_string();
    let wad_relative = "DATA/FINAL/ShaderCache.dx11.wad.client";
    log(&format!(
        "[SHADER] 加载 ShaderCache WAD: {}/{}",
        game_path_str, wad_relative
    ));

    let wad_loader = LeagueWadLoader::from_relative_path(&game_path_str, wad_relative)
        .map_err(|e| format!("无法加载 ShaderCache.dx11.wad.client: {e}"))?;

    log(&format!(
        "[SHADER] WAD 加载成功，包含 {} 个条目",
        wad_loader.wad.entries.len()
    ));

    fs::create_dir_all(&options.out_dir)
        .map_err(|e| format!("创建输出目录 {} 失败: {e}", options.out_dir.display()))?;

    let toc_paths: Vec<String> = if options.toc_paths.is_empty() {
        DEFAULT_TOC_PATHS.iter().map(|s| s.to_string()).collect()
    } else {
        options.toc_paths.clone()
    };

    let mut global_entries = HashMap::new();
    let mut global_defs: HashMap<LeagueShader, HashMap<u64, Vec<String>>> = HashMap::new();
    let mut global_rdef_layouts: HashMap<PathBuf, ShaderLayoutDescriptor> = HashMap::new();

    for (idx, toc_path) in toc_paths.iter().enumerate() {
        log(&format!(
            "[SHADER] [{}/{}] 处理 TOC: {}",
            idx + 1,
            toc_paths.len(),
            toc_path
        ));
        let Some(shader_type) = get_league_shader_type(toc_path) else {
            log(&format!("[WARN] 无法映射 TOC 路径: {toc_path}，跳过"));
            continue;
        };

        match process_toc(
            &wad_loader,
            toc_path,
            &options.out_dir,
            &options.dxbc_compiler_path,
            options.skip_existing,
            options.save_dxbc,
            &log,
        ) {
            Ok((map_entries, hash_to_defs, rdef_layouts)) => {
                global_entries.insert(shader_type, map_entries);
                global_defs.insert(shader_type, hash_to_defs);
                global_rdef_layouts.extend(rdef_layouts);
            }
            Err(e) => {
                log(&format!("[ERROR] 处理 TOC {} 失败: {}", toc_path, e));
            }
        }
    }

    log(&format!(
        "[SHADER] 共 {} 个唯一 SPIR-V 变体解析出 RDEF 布局",
        global_rdef_layouts.len()
    ));

    let mut app = App::new();
    app.add_plugins((
        bevy::asset::AssetPlugin::default(),
        bevy::scene::ScenePlugin,
        TaskPoolPlugin::default(),
    ));

    app.init_asset::<Shader>();

    app.register_type::<ShaderMap>();
    app.register_type::<ShaderMapEntry>();
    app.register_type::<LeagueShader>();
    app.register_type::<ShaderMemberLayout>();
    app.register_type::<BindingTypeDesc>();
    app.register_type::<BindingDescriptor>();
    app.register_type::<ShaderLayoutDescriptor>();
    app.register_type::<SharedRenderData>();
    app.register_type::<SharedSamplerDef>();
    app.register_type::<SharedTextureDef>();

    let asset_server = app.world().resource::<AssetServer>().clone();

    let mut reflect_jobs: Vec<(LeagueShader, u64, PathBuf, String)> = Vec::new();
    for (shader_type, inner_map) in global_entries {
        for (u64_hash, spv_relative_path) in inner_map {
            let normalized_path = spv_relative_path.replace('\\', "/");
            let rel_sub_path = if normalized_path.starts_with("shaders/") {
                &normalized_path[8..]
            } else {
                &normalized_path
            };
            let spv_abs_path = options.out_dir.join(rel_sub_path);
            reflect_jobs.push((shader_type, u64_hash, spv_abs_path, normalized_path));
        }
    }

    let mut metas: HashMap<LeagueShader, HashMap<u64, Handle<Shader>>> = HashMap::new();
    let mut layouts_map: HashMap<LeagueShader, HashMap<u64, ShaderLayoutDescriptor>> =
        HashMap::new();
    let mut spv_path_map: HashMap<LeagueShader, HashMap<u64, PathBuf>> = HashMap::new();

    for (shader_type, u64_hash, spv_abs_path, normalized_path) in reflect_jobs {
        let shader_handle = asset_server.load(normalized_path);

        spv_path_map
            .entry(shader_type)
            .or_insert_with(HashMap::new)
            .insert(u64_hash, spv_abs_path.clone());

        metas
            .entry(shader_type)
            .or_insert_with(HashMap::new)
            .insert(u64_hash, shader_handle);
        layouts_map
            .entry(shader_type)
            .or_insert_with(HashMap::new)
            .insert(
                u64_hash,
                global_rdef_layouts
                    .get(&spv_abs_path)
                    .cloned()
                    .unwrap_or_default(),
            );
    }

    log("[SHADER] 执行离线统一 pass（家族槽位并集与 binding 重写）...");
    let unified = unify_family_layouts(&mut layouts_map, &spv_path_map, &log);

    log("[SHADER] 执行离线接口对齐 pass（PS 输入分量对齐 VS 输出）...");
    align_stage_interfaces(&spv_path_map, &global_defs, &log);

    let mut layout_pool: Vec<ShaderLayoutDescriptor> = Vec::new();
    let mut entries: HashMap<LeagueShader, HashMap<u64, ShaderMapEntry>> = HashMap::new();
    let mut families: Vec<LeagueShader> = layouts_map.keys().copied().collect();
    families.sort_by_key(|f| format!("{f:?}"));

    for family in families {
        let variant_layouts = &layouts_map[&family];
        let family_metas = &metas[&family];
        let mut hashes: Vec<u64> = variant_layouts.keys().copied().collect();
        hashes.sort_unstable();
        for hash in hashes {
            let layout = &variant_layouts[&hash];
            let layout_index = match layout_pool.iter().position(|l| l == layout) {
                Some(idx) => idx,
                None => {
                    layout_pool.push(layout.clone());
                    layout_pool.len() - 1
                }
            };
            let shader_handle = family_metas[&hash].clone();
            entries.entry(family).or_insert_with(HashMap::new).insert(
                hash,
                ShaderMapEntry {
                    shader_handle,
                    layout_index: layout_index as u32,
                },
            );
        }
    }

    let total_variants: usize = entries.values().map(|m| m.len()).sum();
    log(&format!(
        "[SHADER] {} 个变体布局去重为 {} 套 cbuffer 内存布局",
        total_variants,
        layout_pool.len()
    ));

    app.insert_resource(ShaderMap {
        entries,
        layouts: layout_pool,
        unified,
    });

    log("[SHADER] 提取共享渲染数据（采样器与全局共享贴图）...");
    let shared_render_data = extract_shared_render_data(&game_path_str);
    log(&format!(
        "[SHADER] 共享采样器 {} 个，共享贴图 {} 个",
        shared_render_data.samplers.len(),
        shared_render_data.textures.len()
    ));
    app.insert_resource(shared_render_data);

    let world = app.world_mut();
    let type_registry = world.resource::<AppTypeRegistry>().clone();
    let type_registry_read = type_registry.read();

    let scene = DynamicWorldBuilder::from_world(world, &type_registry_read)
        .extract_resources()
        .build();
    let serialized_scene = scene
        .serialize(&type_registry_read)
        .map_err(|e| format!("序列化 ShaderMap 场景失败: {e}"))?;

    let map_path = options.out_dir.join("map.ron");
    fs::write(&map_path, serialized_scene)
        .map_err(|e| format!("写入 {} 失败: {e}", map_path.display()))?;

    log(&format!(
        "[SHADER] 全局 Shader 映射与内存布局已写入 {}",
        map_path.display()
    ));

    Ok(())
}
