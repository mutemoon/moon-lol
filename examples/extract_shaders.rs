//! # extract_shaders
//!
//! 从 ShaderCache.dx11.wad.client 提取 DXBC shader，
//! 通过 HLSLDecompiler 反编译为 HLSL，再用 DXC 编译为 SPIR-V，
//! 输出到 assets/shaders/ 并生成 shader_map.ron 索引文件。
//!
//! ## 依赖工具
//! - `cmd_Decompiler.exe`（HLSLDecompiler，位于 scratch/tools/）：DXBC → HLSL
//! - `dxc.exe`（Vulkan SDK，通常位于 C:\VulkanSDK\<ver>\Bin\）：HLSL → SPIR-V
//!
//! ## 使用方法
//! ```
//! cargo run --example extract_shaders -- \
//!   --game-path "D:\WeGameApps\英雄联盟\Game" \
//!   --toc-paths "shaders/unlit_decal_ps.ps.dx11"
//! ```

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

use bevy::prelude::*;
use clap::Parser;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use league_core::extract::{X3dSharedData, X3dSharedSamplerDef};
use league_file::shader::LeagueShaderToc;
use league_loader::prop_bin::LeagueWadLoaderTrait;
use league_loader::wad::LeagueWadLoader;
use league_utils::hash_wad;
use lol_base_render::shader::LeagueShader;
use lol_base_render::shader_layout::{BindingDescriptor, BindingTypeDesc, ShaderLayoutDescriptor};
use lol_render::shader::{ShaderMap, SharedRenderData, SharedSamplerDef, SharedTextureDef};
use rayon::prelude::*;

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

    // 资源绑定表：SM5.0 条目 32 字节；名字健全性校验失败则退回 40（SM5.1）
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

/// 从 DXBC 的 RDEF chunk 构建完整 ShaderLayoutDescriptor。
/// 与 spirv-reflect 等价——包含所有 cbuffer（含成员名/offset/size）和纹理/采样器绑定。
/// binding_index 使用与 dxbc-compiler 一致的公式：shift + typeBase + regIndex
fn build_rdef_layout(dxbc: &[u8]) -> Result<ShaderLayoutDescriptor, String> {
    use lol_base_render::shader_layout::{BindingDescriptor, BindingTypeDesc, ShaderMemberLayout};

    let rdef = parse_rdef(dxbc)?;
    let shift: u32 = if rdef.is_pixel { 100 } else { 0 };

    // ── cbuffer 索引：name → (size, vars) ──
    let cb_map: std::collections::HashMap<&str, (u32, &[RdefVar])> = rdef
        .cbuffers
        .iter()
        .map(|cb| (cb.name.as_str(), (cb.size, cb.vars.as_slice())))
        .collect();

    let mut bindings: std::collections::BTreeMap<String, BindingDescriptor> =
        std::collections::BTreeMap::new();

    for res in &rdef.resources {
        // 公式 binding = shift + typeBase + regIndex
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
                let members: std::collections::BTreeMap<String, ShaderMemberLayout> = vars
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

/// 从 ShaderCache.dx11.wad.client 提取 Shader 并转换为 SPIR-V
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// 游戏根目录（包含 DATA/FINAL/ 的目录）
    #[arg(long, default_value = r"D:\WeGameApps\英雄联盟\Game")]
    game_path: String,

    /// 输出目录
    #[arg(long, default_value = "assets/shaders")]
    out_dir: String,

    /// DXBC to SPIR-V Compiler (dxbc-compiler.exe) 路径
    #[arg(long, default_value = r"assets\tools\dxbc-compiler.exe")]
    dxbc_compiler: String,

    /// 需要处理的 TOC 路径列表（用逗号分隔，如 "shaders/unlit_decal_ps.ps.dx11"）
    #[arg(long, value_delimiter = ',')]
    toc_paths: Vec<String>,

    /// 是否跳过已存在的输出文件
    #[arg(long, default_value_t = false)]
    skip_existing: bool,

    /// 是否同时提取并保存原始 DXBC 文件
    #[arg(long, default_value_t = false)]
    save_dxbc: bool,
}

fn get_league_shader_type(toc_path: &str) -> Option<LeagueShader> {
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

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // 检查工具是否存在
    let dxbc_compiler_path = PathBuf::from(&args.dxbc_compiler);
    if !dxbc_compiler_path.exists() {
        eprintln!(
            "[ERROR] 找不到 dxbc_compiler: {}",
            dxbc_compiler_path.display()
        );
        eprintln!("        请确保 dxbc_compiler.exe 存在，或通过 --dxbc-compiler 指定路径");
        std::process::exit(1);
    }

    // 加载 ShaderCache WAD
    let wad_relative = "DATA/FINAL/ShaderCache.dx11.wad.client";
    println!("[INFO] 加载 WAD: {}/{}", args.game_path, wad_relative);
    let wad_loader = LeagueWadLoader::from_relative_path(&args.game_path, wad_relative)
        .unwrap_or_else(|e| {
            eprintln!("[ERROR] 无法加载 WAD 文件: {}", e);
            eprintln!("        路径: {}/{}", args.game_path, wad_relative);
            std::process::exit(1);
        });

    println!(
        "[INFO] WAD 加载成功，包含 {} 个条目",
        wad_loader.wad.entries.len()
    );

    let out_dir = Path::new(&args.out_dir);
    fs::create_dir_all(out_dir)?;

    let toc_paths: Vec<String> = if args.toc_paths.is_empty() {
        vec![
            "assets/shaders/hlsl/particlesystem/quad_ps_slice.ps-dx11".to_string(),
            "assets/shaders/hlsl/particlesystem/quad_vs.vs-dx11".to_string(),
            "assets/shaders/hlsl/particlesystem/quad_ps.ps-dx11".to_string(),
            "assets/shaders/hlsl/environment/unlit_decal_ps.ps-dx11".to_string(),
            "assets/shaders/hlsl/environment/unlit_decal_vs.vs-dx11".to_string(),
            "assets/shaders/hlsl/particlesystem/distortion_ps.ps-dx11".to_string(),
            "assets/shaders/hlsl/particlesystem/distortion_vs.vs-dx11".to_string(),
            "assets/shaders/hlsl/particlesystem/mesh_ps.ps-dx11".to_string(),
            "assets/shaders/hlsl/particlesystem/mesh_vs.vs-dx11".to_string(),
            "assets/shaders/hlsl/skinnedmesh/particle_ps.ps-dx11".to_string(),
            "assets/shaders/hlsl/skinnedmesh/particle_vs.vs-dx11".to_string(),
        ]
    } else {
        args.toc_paths.clone()
    };

    let mut global_entries = HashMap::new();
    // 每个家族的变体 hash → def 名字集合，供接口对齐 pass 做 VS/PS 变体配对
    let mut global_defs: HashMap<LeagueShader, HashMap<u64, Vec<String>>> = HashMap::new();
    // 全局 RDEF 布局：spv 绝对路径 → ShaderLayoutDescriptor
    let mut global_rdef_layouts: HashMap<PathBuf, ShaderLayoutDescriptor> = HashMap::new();

    for toc_path in &toc_paths {
        println!("\n[TOC] 处理: {}", toc_path);
        let Some(shader_type) = get_league_shader_type(toc_path) else {
            eprintln!(
                "[WARNING] 无法将 TOC 路径 {} 映射到任何已知的 LeagueShader 枚举，跳过",
                toc_path
            );
            continue;
        };

        match process_toc(
            &wad_loader,
            toc_path,
            out_dir,
            &dxbc_compiler_path,
            args.skip_existing,
            args.save_dxbc,
        ) {
            Ok((map_entries, hash_to_defs, rdef_layouts)) => {
                global_entries.insert(shader_type, map_entries);
                global_defs.insert(shader_type, hash_to_defs);
                global_rdef_layouts.extend(rdef_layouts);
            }
            Err(e) => {
                eprintln!("[ERROR] 处理 TOC {} 失败: {}", toc_path, e);
            }
        }
    }

    println!(
        "\n[RDEF] 共 {} 个唯一 SPIR-V 文件已从 RDEF 反射布局",
        global_rdef_layouts.len()
    );

    let mut app = App::new();
    app.add_plugins((
        bevy::asset::AssetPlugin::default(),
        bevy::scene::ScenePlugin,
        TaskPoolPlugin::default(),
    ));

    app.init_asset::<Shader>();

    app.register_type::<ShaderMap>();
    app.register_type::<lol_render::shader::ShaderMapEntry>();
    app.register_type::<lol_base_render::shader::LeagueShader>();
    app.register_type::<lol_base_render::shader_layout::ShaderMemberLayout>();
    app.register_type::<lol_base_render::shader_layout::BindingTypeDesc>();
    app.register_type::<lol_base_render::shader_layout::BindingDescriptor>();
    app.register_type::<lol_base_render::shader_layout::ShaderLayoutDescriptor>();

    let asset_server = app.world().resource::<AssetServer>().clone();

    // 展开为 (家族, 变体 hash, spv 绝对路径, 资产相对路径) 任务列表
    // stage 不再外传，由 binding index 推导（< 100 = VERTEX，>= 100 = FRAGMENT）
    let mut reflect_jobs: Vec<(LeagueShader, u64, PathBuf, String)> = Vec::new();
    for (shader_type, inner_map) in global_entries {
        for (u64_hash, spv_relative_path) in inner_map {
            let normalized_path = spv_relative_path.replace('\\', "/");
            let rel_sub_path = if normalized_path.starts_with("shaders/") {
                &normalized_path[8..]
            } else {
                &normalized_path
            };
            let spv_abs_path = out_dir.join(rel_sub_path);
            reflect_jobs.push((shader_type, u64_hash, spv_abs_path, normalized_path));
        }
    }

    // ── 使用 RDEF 布局（无需 spirv-reflect）──
    let mut metas: HashMap<LeagueShader, HashMap<u64, Handle<Shader>>> = HashMap::new();
    let mut layouts_map: HashMap<LeagueShader, HashMap<u64, ShaderLayoutDescriptor>> =
        HashMap::new();
    // 记录每个 (家族, 变体 hash) 对应的 .spv 绝对路径，供离线统一 pass 改写 binding
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

    // ── 离线统一 pass：家族槽位并集布局 + 改写 .spv binding 装饰 ───────────
    let unified = unify_family_layouts(&mut layouts_map, &spv_path_map);

    // ── 离线接口对齐 pass：PS 输入向量加宽到配对 VS 变体输出的精确宽度 ────
    align_stage_interfaces(&spv_path_map, &global_defs);

    // ── 布局去重建池：同家族大量变体共享同一套 cbuffer 布局，只存一份 ────
    // 按家族名 + hash 排序遍历，保证池索引跨次运行稳定（map.ron diff 可读）
    let mut layout_pool: Vec<ShaderLayoutDescriptor> = Vec::new();
    let mut entries: HashMap<LeagueShader, HashMap<u64, lol_render::shader::ShaderMapEntry>> =
        HashMap::new();
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
                lol_render::shader::ShaderMapEntry {
                    shader_handle,
                    layout_index: layout_index as u32,
                },
            );
        }
    }
    let total_variants: usize = entries.values().map(|m| m.len()).sum();
    println!(
        "[POOL] {} 个变体布局去重为 {} 套",
        total_variants,
        layout_pool.len()
    );

    app.insert_resource(ShaderMap {
        entries,
        layouts: layout_pool,
        unified,
    });

    // 提取共享渲染数据（采样器 / 共享贴图定义），来自 Shaders.wad.client 内
    // assets/shaders/shareddata.bin；作为 Reflect 资源随 map.ron 一并序列化。
    let shared_render_data = extract_shared_render_data(&args.game_path);
    println!(
        "[SHARED] 采样器 {} 个，共享贴图 {} 个",
        shared_render_data.samplers.len(),
        shared_render_data.textures.len()
    );
    app.insert_resource(shared_render_data);

    let world = app.world_mut();
    let type_registry = world.resource::<AppTypeRegistry>().clone();
    let type_registry_read = type_registry.read();

    let scene = DynamicWorldBuilder::from_world(world, &type_registry_read)
        .extract_resources()
        .build();
    let serialized_scene = scene.serialize(&type_registry_read).unwrap();

    let map_path = out_dir.join("map.ron");
    fs::write(&map_path, serialized_scene)?;

    println!(
        "\n[DONE] 全局 shader 映射与内存布局已按 Bevy 场景格式写入 {}",
        map_path.display()
    );

    Ok(())
}

/// 将 X3DSharedSamplerDef 的地址模式归一化：原字段缺省为 Clamp，原值 0 为 Wrap，原值 2 为 Mirror。
/// 输出规范值：0 = ClampToEdge，1 = Repeat，2 = MirrorRepeat。
fn resolve_address_mode(v: Option<u8>) -> u8 {
    match v {
        None => 0,
        Some(0) => 1,
        Some(2) => 2,
        Some(_) => 0,
    }
}

/// 从 Shaders.wad.client 内 assets/shaders/shareddata.bin 提取共享采样器与共享贴图定义。
/// 采样器为顶层 X3DSharedSamplerDef entry；共享贴图为 X3DSharedData.textures 内嵌列表。
fn extract_shared_render_data(game_path: &str) -> SharedRenderData {
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

    // 采样器：顶层 X3DSharedSamplerDef entry
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

    // 共享贴图：X3DSharedData.textures 内嵌列表
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

/// 枚举 base_defines 的全部子集，反解出每个变体 hash 对应的 def 名字集合。
/// League 的 TOC 对每个 shader 家族枚举了 base_defines 的全部 2^N 组合，
/// 每个变体 hash = hash_shader_spec(该组合的 def 名字)，与运行时查表同一函数，
/// 据此可反向还原。base_define 过多（> 24）时放弃反解，返回空表。
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

/// 把一组 def 名字转为 .spv 文件名（不含扩展名）：
/// 按字典序排序后用 `__` 连接（与 hash_shader_spec 的排序约定一致），
/// 空集记为 `BASE`；过长（> 200 字符）时截断并追加短 hash 防止超过 Windows 路径上限。
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

fn process_toc(
    wad_loader: &LeagueWadLoader,
    toc_path: &str,
    out_dir: &Path,
    dxbc_compiler_path: &Path,
    skip_existing: bool,
    save_dxbc: bool,
) -> anyhow::Result<(
    HashMap<u64, String>,
    HashMap<u64, Vec<String>>,
    HashMap<PathBuf, ShaderLayoutDescriptor>,
)> {
    let toc_hash = hash_wad(toc_path);
    let mut toc_reader = wad_loader
        .get_wad_entry_reader_by_hash(toc_hash)
        .map_err(|e| {
            anyhow::anyhow!("找不到 TOC 文件 {} (hash={:x}): {}", toc_path, toc_hash, e)
        })?;

    let mut toc_bytes = Vec::new();
    toc_reader.read_to_end(&mut toc_bytes)?;

    let (_, shader_toc) =
        LeagueShaderToc::parse(&toc_bytes).map_err(|e| anyhow::anyhow!("解析 TOC 失败: {}", e))?;

    let shader_type_str = if shader_toc.shader_type == 0 {
        "vertex"
    } else {
        "pixel"
    };
    println!(
        "  [TOC] shader_count={}, bundled={}, type={}",
        shader_toc.shader_count, shader_toc.bundled_shader_count, shader_type_str
    );

    let chunk_count = ((shader_toc.bundled_shader_count as f32 / 100.0).ceil() as usize).max(1);
    println!("  [TOC] 需要读取 {} 个 chunk 文件", chunk_count);

    let mut dxbc_blobs: Vec<Vec<u8>> = Vec::new();

    for i in 0..chunk_count {
        let chunk_path = format!("{}_{}", toc_path, i * 100);
        let chunk_hash = hash_wad(&chunk_path);

        let mut chunk_reader = wad_loader
            .get_wad_entry_reader_by_hash(chunk_hash)
            .map_err(|e| {
                anyhow::anyhow!("找不到 chunk {} (hash={:x}): {}", chunk_path, chunk_hash, e)
            })?;

        let mut chunk_bytes = Vec::new();
        chunk_reader.read_to_end(&mut chunk_bytes)?;

        let blobs = parse_dxbc_chunk(&chunk_bytes)?;
        println!("  [CHUNK] {} → {} 个 shader blobs", chunk_path, blobs.len());
        dxbc_blobs.extend(blobs);
    }

    println!(
        "  [INFO] 共读取 {} 个 bundled shader blobs",
        dxbc_blobs.len()
    );

    let toc_name = sanitize_name(toc_path);
    let toc_out_dir = out_dir.join(&toc_name);
    fs::create_dir_all(&toc_out_dir)?;

    // 反解每个变体 hash 的 def 集合，并为每个 bundled shader 选代表 def：
    // 多个 def 组合可能经 shader_ids 共享同一 bundled 文件，取 def 最少者（字典序 tie-break）
    // 作为文件名，既最简洁可读又保证不同 bundled 文件名不重复。
    let hash_to_defs = build_hash_to_defs(&shader_toc.base_defines);
    let matched = shader_toc
        .shader_hashes
        .iter()
        .filter(|h| hash_to_defs.contains_key(h))
        .count();
    println!(
        "  [DEFS] base_defines={}, def 反解命中 {}/{} 变体",
        shader_toc.base_defines.len(),
        matched,
        shader_toc.shader_hashes.len()
    );
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

    // 编译每个 bundled shader：每个 idx 独立调用一次 dxbc_compiler.exe，临时文件按
    // idx 命名互不冲突，用 rayon 并行加速
    let pb_compile = ProgressBar::new(shader_toc.bundled_shader_count as u64);
    pb_compile.set_style(
        ProgressStyle::default_bar()
            .template(&format!("{{spinner:.green}} [{{elapsed_precise}}] [{{bar:40.magenta/blue}}] {{pos}}/{{len}} 编译着色器 [{}]: {{msg}}", toc_name))
            .unwrap()
            .progress_chars("#>-"),
    );

    let is_pixel = shader_toc.shader_type != 0;
    let spv_paths: Vec<Option<String>> = (0..shader_toc.bundled_shader_count as usize)
        .into_par_iter()
        .map(|idx| {
            pb_compile.inc(1);
            if idx >= dxbc_blobs.len() {
                pb_compile.println(format!(
                    "  {} shader #{} 超出 blobs 范围",
                    style("[WARN]").yellow().bold(),
                    idx
                ));
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

            // 一次性迁移：旧命名文件存在且新命名不存在 → 直接改名
            //（内容不变，含已 remap 的 binding），避免重新编译
            if old_path != spv_path && old_path.exists() && !spv_path.exists() {
                if let Err(e) = fs::rename(&old_path, &spv_path) {
                    pb_compile.println(format!(
                        "  {} shader_{:04} 迁移旧文件失败: {}",
                        style("[ERROR]").red().bold(),
                        idx,
                        e
                    ));
                    return None;
                }
            }

            if save_dxbc {
                let dxbc_path = toc_out_dir.join(format!("{}.dxbc", stem));
                let _ = fs::write(&dxbc_path, &dxbc_blobs[idx]);
            }

            if skip_existing && spv_path.exists() {
                return Some(spv_relative);
            }

            pb_compile.set_message(stem.clone());

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
                        Err(e) => {
                            pb_compile.println(format!(
                                "  {} shader_{:04} 写入失败: {}",
                                style("[ERROR]").red().bold(),
                                idx,
                                e
                            ));
                            None
                        }
                    }
                }
                Err(e) => {
                    pb_compile.println(format!(
                        "  {} shader_{:04} 编译失败: {}",
                        style("[ERROR]").red().bold(),
                        idx,
                        e
                    ));
                    None
                }
            }
        })
        .collect();
    pb_compile.finish_with_message("编译完成");

    // ── RDEF 布局：为每个成功编译的 bundled shader 解析 RDEF ──
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
        match build_rdef_layout(&dxbc_blobs[idx]) {
            Ok(layout) => {
                rdef_layouts.insert(spv_abs, layout);
            }
            Err(e) => {
                eprintln!(
                    "  {} bundled #{} ({}) RDEF 解析失败: {}",
                    style("[WARN]").yellow().bold(),
                    idx,
                    stem,
                    e
                );
            }
        }
    }
    println!(
        "  [RDEF] 解析 {} 个 bundled shader RDEF 布局",
        rdef_layouts.len()
    );

    // 构建 shader_map：shader_hash → spv 路径
    // shader_ids[shader_index] 是到 bundled shaders 的间接索引
    let mut map_entries: HashMap<u64, String> = HashMap::new();
    for (shader_index, &shader_hash) in shader_toc.shader_hashes.iter().enumerate() {
        let shader_id = shader_toc.shader_ids[shader_index] as usize;
        if let Some(Some(spv_path)) = spv_paths.get(shader_id) {
            map_entries.insert(shader_hash, spv_path.clone());
        }
    }

    println!("  [DONE] {} 个 shader 编译处理完成", map_entries.len());

    Ok((map_entries, hash_to_defs, rdef_layouts))
}

/// 解析 dx11_0 chunk 文件，返回 DXBC blob 列表
/// 格式: [4字节 LE 长度][数据(length-1字节)][1字节 null] 重复
fn parse_dxbc_chunk(data: &[u8]) -> anyhow::Result<Vec<Vec<u8>>> {
    let mut blobs = Vec::new();
    let mut offset = 0usize;

    while offset < data.len() {
        if offset + 4 > data.len() {
            break;
        }

        let length = u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;

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

        // length 包含 null terminator，实际 DXBC 数据是 length-1 字节
        let dxbc_size = length.saturating_sub(1);
        let dxbc_data = data[offset + 4..offset + 4 + dxbc_size].to_vec();
        blobs.push(dxbc_data);

        offset += 4 + length;
    }

    Ok(blobs)
}

/// 将单个 DXBC blob 转换为 SPIR-V bytes
/// 流程: DXBC → dxbc_compiler.exe → SPIR-V
fn compile_dxbc_to_spirv(
    dxbc_data: &[u8],
    dxbc_compiler_path: &Path,
    work_dir: &Path,
    idx: usize,
    is_pixel: bool,
    save_dxbc: bool,
) -> anyhow::Result<Vec<u8>> {
    let prefix = format!("_tmp_{:04}", idx);
    let dxbc_filename = format!("{}.dxbc", prefix);
    let spv_filename = format!("{}.spv", prefix);

    let dxbc_path_tmp = work_dir.join(&dxbc_filename);
    let spv_path_tmp = work_dir.join(&spv_filename);

    // ── Step 1: 写 DXBC 到临时文件 ─────────────────────────────────
    fs::write(&dxbc_path_tmp, dxbc_data)?;

    // ── Step 2: DXBC → SPIR-V via dxbc_compiler ──────────────────────
    // dxbc_compiler.exe --spv <output.spv> --set 3 <input.dxbc>
    let mut cmd = Command::new(dxbc_compiler_path);
    cmd.arg("--spv").arg(&spv_filename).arg("--set").arg("3");
    if is_pixel {
        cmd.arg("--binding-shift").arg("100");
    }
    cmd.arg(&dxbc_filename).current_dir(work_dir);
    let compiler_out = cmd
        .output()
        .map_err(|e| anyhow::anyhow!("启动 dxbc_compiler 失败: {}", e))?;

    // 如果指定了 save_dxbc 参数，则保存原始 dxbc 文件
    if save_dxbc {
        let _ = fs::copy(&dxbc_path_tmp, work_dir.join(format!("{}.dxbc", prefix)));
    }
    let _ = fs::remove_file(&dxbc_path_tmp);

    if !compiler_out.status.success() {
        let stderr = String::from_utf8_lossy(&compiler_out.stderr);
        let stdout = String::from_utf8_lossy(&compiler_out.stdout);
        let _ = fs::remove_file(&spv_path_tmp);
        return Err(anyhow::anyhow!(
            "dxbc_compiler 编译失败:\nstdout: {}\nstderr: {}",
            stdout.trim(),
            stderr.trim()
        ));
    }

    // ── Step 3: 读取 SPIR-V 文件 ───────────────────────────────────
    let spv_bytes = fs::read(&spv_path_tmp)?;
    let _ = fs::remove_file(&spv_path_tmp);

    // 剥离 wgpu 默认设备不支持的能力（Vulkan 1.2 memory model 等），避免 alpha_blend_mesh_pipeline is invalid
    let spv_bytes = lol_render::loaders::spirv_strip::strip_spirv(&spv_bytes);

    Ok(spv_bytes)
}

/// 将路径字符串转为安全的目录结构路径，形如 "hlsl/environment/unlit_decal/ps"
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

// ---------------------------------------------------------------------------
// 离线统一 pass：家族槽位并集布局 + 改写 .spv binding 装饰
// ---------------------------------------------------------------------------

/// VS/PS 家族配对表：PS 家族 → 同 pipeline 的 VS 家族（VS 家族返回 None）
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

/// 离线接口对齐 pass：把每个 PS 变体的输入接口向量加宽到配对 VS 变体
/// 输出的精确宽度。
///
/// 根因：D3D11 允许 VS 输出比 PS 输入宽（如 MeshVs 输出 TEXCOORD1 为 vec3
/// 而 MeshPs 只读 vec2），而 Vulkan 未启用 maintenance4 时要求输出分量数 ≤
/// 输入分量数（VUID-RuntimeSpirv-maintenance4-06817）；wgpu-hal 仅在请求
/// EXPERIMENTAL_MESH_SHADER 时才置位 maintenance4，本机 Pascal 显卡无法请求，
/// 所以离线加宽 PS 输入。但 VUID-RuntimeSpirv-OpEntryPoint-08743 又要求 PS
/// 每个输入分量都有 VS 输出覆盖，两约束合并即宽度必须与配对 VS 输出精确
/// 相等，不能按家族全变体取最大宽度，必须按变体配对：运行时
/// assembly::derive_defs 从同一 emitter 配置派生 (vs_defs, ps_defs)，两家族
/// 共享的宏必然同开同关，故 PS 变体 defs 投影到 VS 宏名全集即配对 VS 变体
/// 的 defs，hash_shader_spec 后查表定位其 .spv（与运行时查表同一函数）。
///
/// 目标宽度 = 覆盖本输入起始 component 的 VS 输出变量的覆盖终点，受 PS 同
/// location 下一变量起始与 4 分量边界封顶；已达标文件 widen_ps_inputs 返回
/// None，二次提取天然幂等。输入已比目标宽的文件无法缩窄（加宽不可逆），
/// 打 [OVERWIDE] 标记提示删除重建。
fn align_stage_interfaces(
    spv_path_map: &HashMap<LeagueShader, HashMap<u64, PathBuf>>,
    global_defs: &HashMap<LeagueShader, HashMap<u64, Vec<String>>>,
) {
    use std::collections::{BTreeMap, BTreeSet};

    use lol_render::loaders::spirv_strip::{
        STORAGE_INPUT, STORAGE_OUTPUT, interface_vector_widths, widen_ps_inputs,
    };

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
            eprintln!("[WARN] {family:?}/{vs_family:?} 缺少 def 反解表，跳过接口对齐");
            continue;
        };
        // VS 家族宏名全集：PS defs 投影到该集合即运行时配对 VS 变体的 defs
        let vs_macros: HashSet<&str> = vs_defs_map.values().flatten().map(|s| s.as_str()).collect();

        // 1. 逐 PS 变体 hash 定位配对 VS 变体，按文件聚合：多个 hash 经
        //    shader_ids 共享同一 .spv，不同 hash 可能配对不同 VS 文件
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
                eprintln!(
                    "[WARN] {family:?} 变体 {} 的配对 VS 变体 {} 缺失，跳过",
                    defs_to_stem(ps_defs),
                    defs_to_stem(&paired_defs)
                );
                continue;
            };
            ps_vs_files.entry(ps_path).or_default().insert(vs_path);
        }

        // 2. VS 文件输出宽度缓存（(loc, comp) → 分量数）
        let mut vs_cover: HashMap<&PathBuf, Option<BTreeMap<(u32, u32), u32>>> = HashMap::new();

        let mut rewritten = 0usize;
        for (ps_path, vs_paths) in &ps_vs_files {
            let Ok(bytes) = fs::read(ps_path) else {
                continue;
            };
            let Some(in_widths) = interface_vector_widths(&bytes, STORAGE_INPUT) else {
                continue;
            };

            // 每个配对 VS 分别算精确目标宽度，多 VS 时取交集校验一致
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
                    // 覆盖本输入起始 component 的 VS 输出：同 loc 起始 ≤ comp 的
                    // 最大者，其覆盖终点决定精确目标（VS 与 PS 按语义打包一致，
                    // 但 VS 可能把 PS 拆成两段读的区间合并成一个宽输出）
                    let cover = vs_out
                        .range((loc, 0)..=(loc, comp))
                        .next_back()
                        .filter(|&(&(_, c0), &w)| c0 + w > comp);
                    let Some((&(_, c0), &w)) = cover else {
                        eprintln!(
                            "[WARN] {} 输入 loc{loc} comp{comp} 在配对 VS {} 无输出覆盖（08743 不可修复）",
                            ps_path.display(),
                            vs_path.display()
                        );
                        continue;
                    };
                    // 封顶：PS 同 location 下一变量起始位，否则 4 分量边界
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
                                    eprintln!(
                                        "[WARN] {} 多个配对 VS 在 {k:?} 目标宽度冲突（{w} vs {w2}），取最小保 08743",
                                        ps_path.display()
                                    );
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

            // 3. 与当前宽度比对：窄则加宽，宽则标记重建（加宽不可逆）
            let mut targets: BTreeMap<(u32, u32), u32> = BTreeMap::new();
            for (&(loc, comp), &cur) in &in_widths {
                let Some(&want) = desired.get(&(loc, comp)) else {
                    continue;
                };
                if cur > want {
                    println!(
                        "[OVERWIDE] {} loc{loc} comp{comp} 当前 {cur} > 目标 {want}，需删除重建",
                        ps_path.display()
                    );
                } else if cur < want {
                    targets.insert((loc, comp), want);
                }
            }
            if targets.is_empty() {
                continue;
            }
            match widen_ps_inputs(&bytes, &targets) {
                Some(new_bytes) => match fs::write(ps_path, &new_bytes) {
                    Ok(()) => rewritten += 1,
                    Err(e) => eprintln!("[ERROR] 写回 {} 失败: {e}", ps_path.display()),
                },
                None => eprintln!(
                    "[WARN] {} 存在无法安全加宽的接口变量用法，保留原样（targets={targets:?}）",
                    ps_path.display()
                ),
            }
        }
        if rewritten > 0 {
            println!(
                "[ALIGN] {family:?}: 加宽 {rewritten}/{} 个 .spv 输入接口",
                ps_vs_files.len()
            );
        }
    }
}

/// 汇总一个家族所有变体的 binding 名并集。
/// 同名 binding 类型冲突时告警并保留首个（脚本已验证当前资产零类型冲突）；
/// UniformBuffer 取最大 total_size 并合并成员名（仅供展示，成员 offset 以
/// 各变体自己的反射布局为准）。
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
                (a, b) => {
                    eprintln!("[WARN] binding {name} 类型在变体间冲突（{a:?} vs {b:?}），保留首个");
                }
            }
        }
    }
    union
}

/// 家族槽位并集统一：
/// 1. 每个家族取所有变体 binding 名并集，binding_index 直接赋 .spv 实际的
///    Vulkan 压缩编号（VS = 并集排名，PS = |配对 VS 并集| + 并集排名，与
///    wgpu-hal 合并 BindGroupLayout 的压缩重编号结果对齐）；
/// 2. 按配对表把每个 .spv 的 binding 装饰改写为上述 Vulkan 压缩编号；按名字
///    建旧→新映射，二次提取（--skip-existing）时旧编号已是目标值，改写自然幂等；
/// 3. 把每个变体布局裁剪为只剩 cbuffer（纹理/采样器槽位信息已收敛到 unified
///    并集表，变体级是死数据），并把 binding_index 归一化为同一 Vulkan 编号。
fn unify_family_layouts(
    layouts_map: &mut HashMap<LeagueShader, HashMap<u64, ShaderLayoutDescriptor>>,
    spv_path_map: &HashMap<LeagueShader, HashMap<u64, PathBuf>>,
) -> HashMap<LeagueShader, ShaderLayoutDescriptor> {
    // ── 1. 构建每个家族并集，binding_index 直接赋 .spv 实际的 Vulkan 压缩编号 ──
    //    VS = 并集排名；PS = |配对 VS 并集| + 并集排名，与 wgpu-hal 合并
    //    BindGroupLayout 后的压缩编号、以及第 2 步改写进 .spv 的编号完全一致
    let mut unions: HashMap<LeagueShader, BTreeMap<String, BindingDescriptor>> = HashMap::new();
    for (family, variants) in layouts_map.iter() {
        unions.insert(*family, build_family_union(variants));
    }
    // PS 家族的 vk_base = 配对 VS 并集大小（需先建好全部并集才能查）
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
        println!(
            "[UNIFY] {family:?}: {} 个变体 → 并集 {} 个 binding [{}]",
            layouts_map[family].len(),
            union.len(),
            union
                .iter()
                .map(|(n, d)| format!("{n}={}", d.binding_index))
                .collect::<Vec<_>>()
                .join(", "),
        );
    }

    // ── 2. 按唯一 .spv 文件改写 binding 装饰为 Vulkan 压缩编号 ────────────
    for (family, variants) in layouts_map.iter() {
        let Some(union) = unions.get(family) else {
            continue;
        };
        // binding 名 → Vulkan 压缩编号（即并集里已赋好的 binding_index）
        let vk_index: BTreeMap<&String, u32> =
            union.iter().map(|(n, d)| (n, d.binding_index)).collect();

        let mut done_paths: HashSet<PathBuf> = HashSet::new();
        let mut rewritten = 0usize;
        for (hash, layout) in variants {
            let Some(spv_path) = spv_path_map.get(family).and_then(|m| m.get(hash)) else {
                continue;
            };
            // 多个 shader hash 经 shader_ids 间接索引共享同一 .spv，按文件去重
            if !done_paths.insert(spv_path.clone()) {
                continue;
            }
            // 旧 binding 值（该变体反射结果）→ 新 Vulkan 编号，按名字查表
            let mut remap: BTreeMap<u32, u32> = BTreeMap::new();
            for (name, desc) in &layout.bindings {
                if let Some(&new) = vk_index.get(name) {
                    remap.insert(desc.binding_index, new);
                }
            }
            match fs::read(spv_path) {
                Ok(bytes) => {
                    if let Some(new_bytes) =
                        lol_render::loaders::spirv_strip::remap_bindings(&bytes, &remap)
                    {
                        if let Err(e) = fs::write(spv_path, &new_bytes) {
                            eprintln!("[ERROR] 写回 {} 失败: {e}", spv_path.display());
                        } else {
                            rewritten += 1;
                        }
                    }
                }
                Err(e) => eprintln!("[ERROR] 读取 {} 失败: {e}", spv_path.display()),
            }
        }
        println!(
            "[UNIFY] {family:?}: 改写 {rewritten}/{} 个 .spv 文件",
            done_paths.len()
        );
    }

    // ── 3. 裁剪变体布局为只剩 cbuffer，并归一化 binding_index 为 Vulkan 压缩编号（与 .spv 一致） ──
    //（必须在第 2 步之后：.spv 改写需要变体的纹理/采样器旧编号建 remap 表）
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
