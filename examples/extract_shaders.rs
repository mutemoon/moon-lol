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

use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

use bevy::prelude::*;
use clap::Parser;
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use league_file::shader::LeagueShaderToc;
use league_loader::prop_bin::LeagueWadLoaderTrait;
use league_loader::wad::LeagueWadLoader;
use league_utils::{LeagueShader, hash_wad};
use league_core::extract::{X3dSharedData, X3dSharedSamplerDef};
use lol_render::shader::{SharedRenderData, SharedSamplerDef, SharedTextureDef, ShaderMap};
use rayon::prelude::*;

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

    // 当未指定 --toc-paths 时，默认提取 startup_load_shaders 中使用的全部 shader TOC
    // 对应 lol_render/src/shader.rs 中 startup_load_shaders 里的路径，
    // 后缀从旧版 .glsl 改为新版 .dx11
    let toc_paths: Vec<String> = if args.toc_paths.is_empty() {
        vec![
            // QuadPsSlice 家族
            "assets/shaders/hlsl/particlesystem/quad_ps_slice.ps.dx11".to_string(),
            "assets/shaders/hlsl/particlesystem/quad_vs.vs.dx11".to_string(),
            // QuadPs 家族（VS 与 QuadPsSlice 相同）
            "assets/shaders/hlsl/particlesystem/quad_ps.ps.dx11".to_string(),
            // UnlitDecal 家族
            "assets/shaders/hlsl/environment/unlit_decal_ps.ps.dx11".to_string(),
            "assets/shaders/hlsl/environment/unlit_decal_vs.vs.dx11".to_string(),
            // Distortion 家族
            "assets/shaders/hlsl/particlesystem/distortion_ps.ps.dx11".to_string(),
            "assets/shaders/hlsl/particlesystem/distortion_vs.vs.dx11".to_string(),
            // Mesh 家族
            "assets/shaders/hlsl/particlesystem/mesh_ps.ps.dx11".to_string(),
            "assets/shaders/hlsl/particlesystem/mesh_vs.vs.dx11".to_string(),
            // SkinnedMeshParticle 家族
            "assets/shaders/hlsl/skinnedmesh/particle_ps.ps.dx11".to_string(),
            "assets/shaders/hlsl/skinnedmesh/particle_vs.vs.dx11".to_string(),
        ]
    } else {
        args.toc_paths.clone()
    };

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

    let mut global_entries = HashMap::new();

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
            Ok(map_entries) => {
                global_entries.insert(shader_type, map_entries);
            }
            Err(e) => {
                eprintln!("[ERROR] 处理 TOC {} 失败: {}", toc_path, e);
            }
        }
    }

    let mut app = App::new();
    app.add_plugins((
        bevy::asset::AssetPlugin::default(),
        bevy::scene::ScenePlugin,
        TaskPoolPlugin::default(),
    ));

    app.init_asset::<Shader>();

    app.register_type::<ShaderMap>();
    app.register_type::<lol_render::shader::ShaderMapEntry>();
    app.register_type::<league_utils::LeagueShader>();
    app.register_type::<lol_render::particle::shader_layout::ShaderMemberLayout>();
    app.register_type::<lol_render::particle::shader_layout::BindingTypeDesc>();
    app.register_type::<lol_render::particle::shader_layout::BindingDescriptor>();
    app.register_type::<lol_render::particle::shader_layout::ShaderLayoutDescriptor>();

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

    // 多个变体 hash 经 shader_ids 间接索引共享同一 .spv，按唯一路径去重后用
    // rayon 并行反射（临时 yml 以 spv 路径命名，去重后并发无冲突）
    let unique_spv_paths: Vec<PathBuf> = reflect_jobs
        .iter()
        .map(|(_, _, p, _)| p.clone())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    let pb_reflect = ProgressBar::new(unique_spv_paths.len() as u64);
    pb_reflect.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} 反射 SPIR-V 布局 ({eta}) {msg}")
            .unwrap()
            .progress_chars("#>-"),
    );
    let layout_cache: HashMap<PathBuf, ShaderLayoutDescriptor> = unique_spv_paths
        .par_iter()
        .map(|spv_abs_path| {
            let layout = match reflect_spirv_via_cli(spv_abs_path, Path::new("spirv-reflect.exe")) {
                Ok(layout) => layout,
                Err(e) => {
                    pb_reflect.println(format!(
                        "  {} {:?} 反射失败: {}",
                        style("[ERROR]").red().bold(),
                        spv_abs_path.display(),
                        e
                    ));
                    ShaderLayoutDescriptor::default()
                }
            };
            pb_reflect.inc(1);
            (spv_abs_path.clone(), layout)
        })
        .collect();
    pb_reflect.finish_with_message("反射提取完成");

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
            .insert(u64_hash, layout_cache[&spv_abs_path].clone());
    }

    // ── 离线统一 pass：家族槽位并集布局 + 改写 .spv binding 装饰 ───────────
    let unified = unify_family_layouts(&mut layouts_map, &spv_path_map);

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
) -> anyhow::Result<HashMap<u64, String>> {
    // 读取 TOC 文件
    let toc_hash = hash_wad(toc_path);
    let mut toc_reader = wad_loader
        .get_wad_entry_reader_by_hash(toc_hash)
        .map_err(|e| anyhow::anyhow!("找不到 TOC 文件 (hash={:x}): {}", toc_hash, e))?;

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

    // chunk 路径格式: "{toc_path}_{i*100}"
    // 例如: "shaders/unlit_decal_ps.ps.dx11_0", "shaders/unlit_decal_ps.ps.dx11_100", ...
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

        // 解析 chunk: 每个 shader 是 4字节 LE 长度 + DXBC 数据(length-1字节) + null
        let blobs = parse_dxbc_chunk(&chunk_bytes)?;
        println!("  [CHUNK] {} → {} 个 shader blobs", chunk_path, blobs.len());
        dxbc_blobs.extend(blobs);
    }

    println!(
        "  [INFO] 共读取 {} 个 bundled shader blobs",
        dxbc_blobs.len()
    );

    // 创建输出子目录
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
                Ok(spv_bytes) => match fs::write(&spv_path, &spv_bytes) {
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
                },
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

    Ok(map_entries)
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
        let _ = fs::copy(
            &dxbc_path_tmp,
            work_dir.join(format!("shader_{:04}.dxbc", idx)),
        );
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

    let shader_type = if relative.contains(".ps.") {
        "ps"
    } else if relative.contains(".vs.") {
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

use std::collections::BTreeMap;

use lol_render::particle::shader_layout::{
    BindingDescriptor, BindingTypeDesc, ShaderLayoutDescriptor, ShaderMemberLayout,
};

/// 使用外部 spirv-reflect.exe 工具进行反射解析并直接转换为 ShaderLayoutDescriptor
///
/// 采用 `-o tmp_file` 写临时文件方式，避免 stdout 中特殊字符（如 $Globals）导致管道解析失败。
fn reflect_spirv_via_cli(
    spv_path: &Path,
    spirv_reflect_path: &Path,
) -> Result<ShaderLayoutDescriptor, String> {
    // 临时 yml 放系统 temp 目录，文件名用 spv 路径 hash（短且唯一，并发无冲突）。
    // 不能直接在 spv 旁边拼长后缀：def 组合文件名本已接近 Windows MAX_PATH（260），
    // 再加后缀会超限，导致 spirv-reflect 以 0xc0000409 崩溃。
    let path_hash = league_utils::hash_shader(&spv_path.to_string_lossy());
    let tmp_yml = std::env::temp_dir().join(format!("spv_reflect_{:016x}.yml", path_hash));

    let status = Command::new(spirv_reflect_path)
        .arg(spv_path)
        .arg("-y")
        .arg("-o")
        .arg(&tmp_yml)
        .status()
        .map_err(|e| format!("启动 spirv-reflect 失败: {}", e))?;

    if !status.success() {
        return Err(format!("spirv-reflect 执行失败，exit={}", status));
    }

    let yaml_str = fs::read_to_string(&tmp_yml)
        .map_err(|e| format!("读取临时 yml 失败 {:?}: {}", tmp_yml, e))?;
    let _ = fs::remove_file(&tmp_yml);

    parse_spirv_reflect_yaml(&yaml_str)
}

// ── Block variable（锚点变量）信息 ────────────────────────────────────────────

/// 从 `all_block_variables` 段解析所有 `&bvN` 锚点变量，
/// 返回 anchor_id → (total_size, members) 的映射。
///
/// spirv-reflect YAML 中 `block: *bvN` 引用需要通过这张表来展开。
fn parse_block_variables(
    yaml_str: &str,
) -> BTreeMap<String, (usize, BTreeMap<String, ShaderMemberLayout>)> {
    // 两阶段：
    // 1. 扫描所有顶层 bv 条目（`- &bvN` 开头），记录 name/offset/size
    // 2. 找到 member_count > 0 的顶层 bv（即 struct root），重新关联成员
    //
    // 结构特点：顶层 bv 按顺序排列，struct root 的 members 段紧跟其后，
    // 内容是 `- *bvM` 形式的别名引用。

    // 第一遍：记录每个 anchor 的 (name, offset, size) 和 member_count
    struct BvEntry {
        anchor: String, // e.g. "bv15"
        name: String,
        offset: usize,
        size: usize,
        member_count: usize,
        // 直接的子成员 anchor id 列表（从 `- *bvM` 解析）
        child_anchors: Vec<String>,
    }

    let mut entries: Vec<BvEntry> = Vec::new();
    let mut in_block_variables = false;
    let mut in_members_of_current = false;

    for line in yaml_str.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("all_block_variables:") {
            in_block_variables = true;
            continue;
        }
        // 遇到下一个顶层 section 则结束
        if in_block_variables
            && !trimmed.is_empty()
            && !trimmed.starts_with('-')
            && !trimmed.starts_with('#')
            && trimmed.ends_with(':')
            && !line.starts_with(' ')
            && !line.starts_with('\t')
        {
            // 顶层 key（无缩进），说明 all_block_variables 段结束
            in_block_variables = false;
            in_members_of_current = false;
        }
        if !in_block_variables {
            continue;
        }

        // 新顶层 bv 条目：`  - &bvN`
        if trimmed.starts_with("- &bv") {
            in_members_of_current = false;
            let anchor = trimmed
                .trim_start_matches('-')
                .trim()
                .trim_start_matches('&')
                .to_string();
            entries.push(BvEntry {
                anchor,
                name: String::new(),
                offset: 0,
                size: 0,
                member_count: 0,
                child_anchors: Vec::new(),
            });
            continue;
        }

        let Some(last) = entries.last_mut() else {
            continue;
        };

        if trimmed.starts_with("members:") {
            in_members_of_current = last.member_count > 0;
            continue;
        }

        if in_members_of_current {
            // `      - *bvM`
            if trimmed.starts_with("- *bv") {
                let child = trimmed
                    .trim_start_matches('-')
                    .trim()
                    .trim_start_matches('*')
                    .to_string();
                last.child_anchors.push(child);
            }
            continue;
        }

        if let Some(val) = parse_yaml_key_val(trimmed, "name:") {
            last.name = val.trim_matches('"').to_string();
        } else if let Some(val) = parse_yaml_key_val(trimmed, "offset:") {
            last.offset = val
                .split_whitespace()
                .next()
                .unwrap_or("")
                .parse()
                .unwrap_or(0);
        } else if let Some(val) = parse_yaml_key_val(trimmed, "size:") {
            last.size = val
                .split_whitespace()
                .next()
                .unwrap_or("")
                .parse()
                .unwrap_or(0);
        } else if let Some(val) = parse_yaml_key_val(trimmed, "member_count:") {
            last.member_count = val
                .split_whitespace()
                .next()
                .unwrap_or("")
                .parse()
                .unwrap_or(0);
        }
    }

    // 建立 anchor → BvEntry 的快速查找表（按 index）
    // 用 anchor 字符串做 key
    let anchor_map: BTreeMap<String, usize> = entries
        .iter()
        .enumerate()
        .map(|(i, e)| (e.anchor.clone(), i))
        .collect();

    // 对每个顶层 struct root（member_count > 0），展开它的子成员
    let mut result: BTreeMap<String, (usize, BTreeMap<String, ShaderMemberLayout>)> =
        BTreeMap::new();

    for entry in &entries {
        if entry.member_count == 0 {
            continue;
        }
        let mut members: BTreeMap<String, ShaderMemberLayout> = BTreeMap::new();
        for child_anchor in &entry.child_anchors {
            let Some(&child_idx) = anchor_map.get(child_anchor) else {
                continue;
            };
            let child = &entries[child_idx];
            if child.name.is_empty() {
                // 匿名 padding 成员，跳过
                continue;
            }
            members.insert(
                child.name.clone(),
                ShaderMemberLayout {
                    name: child.name.clone(),
                    offset: child.offset,
                    size: child.size,
                },
            );
        }
        // total_size 直接取 struct root 自身的 size 字段
        result.insert(entry.anchor.clone(), (entry.size, members));
    }

    result
}

/// 解析 spirv-reflect.exe -y 输出的文本并直接填充 ShaderLayoutDescriptor
///
/// 策略：
/// 1. 先调用 `parse_block_variables` 建立锚点表（解决 `*bvN` 别名引用）
/// 2. 扫描 `all_descriptor_bindings` 段，对每个 binding 通过 `block: *bvN` 查表获取 members
/// 3. 修正 descriptor_type 映射：UBO = 6（`VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER`），
///    Sampler = 0，CombinedImageSampler = 1，SampledImage = 2，StorageImage = 3
fn parse_spirv_reflect_yaml(yaml_str: &str) -> Result<ShaderLayoutDescriptor, String> {
    // ── 第一步：解析 all_block_variables 锚点表 ────────────────────────────────
    let block_var_table = parse_block_variables(yaml_str);

    // ── 第二步：解析 all_descriptor_bindings ──────────────────────────────────
    let mut bindings: BTreeMap<String, BindingDescriptor> = BTreeMap::new();

    let mut in_descriptor_bindings = false;
    let mut current_spirv_id: Option<u32> = None;
    let mut current_binding = 0u32;
    let mut current_binding_name = String::new(); // binding 的名称，如 "$Globals"、"DIFFUSE_MAP__SMP"
    // descriptor_type 用 i32 存储，-1 表示未知（VK_DESCRIPTOR_TYPE_???），跳过
    let mut current_descriptor_type: i32 = -1;
    // block_anchor 记录 `block: *bvN` 中的锚点 id
    let mut current_block_anchor = String::new();
    // 对于 Texture/Sampler 这种没有 block 的 binding，也保留 members
    let mut current_members: BTreeMap<String, ShaderMemberLayout> = BTreeMap::new();
    let mut current_struct_size = 0usize;

    // 仅在 all_descriptor_bindings 段内且处于顶层 binding 条目中解析成员
    let mut in_members_inline = false;
    let mut current_member_name = String::new();
    let mut current_member_offset = 0usize;
    let mut current_member_size = 0usize;

    let flush = |bindings: &mut BTreeMap<String, BindingDescriptor>,
                 spirv_id: Option<u32>,
                 binding: u32,
                 binding_name: &str,
                 descriptor_type: i32,
                 struct_size: usize,
                 members: &BTreeMap<String, ShaderMemberLayout>| {
        if spirv_id.is_none() {
            return;
        }
        // VkDescriptorType 枚举值：
        //   -1 = VK_DESCRIPTOR_TYPE_??? (UNDEFINED) — 跳过
        //    0 = VK_DESCRIPTOR_TYPE_SAMPLER
        //    1 = VK_DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER
        //    2 = VK_DESCRIPTOR_TYPE_SAMPLED_IMAGE
        //    3 = VK_DESCRIPTOR_TYPE_STORAGE_IMAGE
        //    6 = VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER
        //    7 = VK_DESCRIPTOR_TYPE_STORAGE_BUFFER
        match descriptor_type {
            6 | 7 => {
                bindings.insert(
                    binding_name.to_string(),
                    BindingDescriptor {
                        binding_index: binding,
                        name: binding_name.to_string(),
                        type_desc: BindingTypeDesc::UniformBuffer {
                            total_size: struct_size,
                            members: members.clone(),
                        },
                    },
                );
            }
            1 | 2 => {
                bindings.insert(
                    binding_name.to_string(),
                    BindingDescriptor {
                        binding_index: binding,
                        name: binding_name.to_string(),
                        type_desc: BindingTypeDesc::Texture2d,
                    },
                );
            }
            0 => {
                bindings.insert(
                    binding_name.to_string(),
                    BindingDescriptor {
                        binding_index: binding,
                        name: binding_name.to_string(),
                        type_desc: BindingTypeDesc::Sampler,
                    },
                );
            }
            _ => {} // 负数(-1)或其他未知类型，跳过
        }
    };

    for line in yaml_str.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("all_descriptor_bindings:") {
            in_descriptor_bindings = true;
            continue;
        }
        if in_descriptor_bindings
            && (trimmed.starts_with("all_interface_variables:") || trimmed.starts_with("module:"))
        {
            // flush 最后一条 binding
            // 若 block_anchor 非空，从锚点表中取成员
            let (sz, mems) = if !current_block_anchor.is_empty() {
                if let Some((s, m)) = block_var_table.get(&current_block_anchor) {
                    (*s, m.clone())
                } else {
                    (current_struct_size, current_members.clone())
                }
            } else {
                (current_struct_size, current_members.clone())
            };
            flush(
                &mut bindings,
                current_spirv_id,
                current_binding,
                &current_binding_name,
                current_descriptor_type,
                sz,
                &mems,
            );
            break;
        }

        if !in_descriptor_bindings {
            continue;
        }

        // 新 binding 条目
        if trimmed.starts_with("- &db") || trimmed.starts_with("- spirv_id:") {
            // flush 上一条
            let (sz, mems) = if !current_block_anchor.is_empty() {
                if let Some((s, m)) = block_var_table.get(&current_block_anchor) {
                    (*s, m.clone())
                } else {
                    (current_struct_size, current_members.clone())
                }
            } else {
                (current_struct_size, current_members.clone())
            };
            flush(
                &mut bindings,
                current_spirv_id,
                current_binding,
                &current_binding_name,
                current_descriptor_type,
                sz,
                &mems,
            );
            // 重置
            current_spirv_id = Some(0);
            current_binding = 0;
            current_binding_name.clear();
            current_descriptor_type = -1;
            current_struct_size = 0;
            current_block_anchor.clear();
            current_members.clear();
            in_members_inline = false;

            // 如果是 `- spirv_id: N` 内联形式，直接解析
            if let Some(val) = parse_yaml_key_val(trimmed, "spirv_id:") {
                current_spirv_id = val.parse::<u32>().ok();
            }
            continue;
        }

        // `block: *bvN  # "BlockName"` — 记录锚点
        if trimmed.starts_with("block:") {
            in_members_inline = false;
            if let Some(rest) = trimmed.strip_prefix("block:") {
                let rest = rest.trim();
                // rest 形如 `*bv15 # "$Globals"` 或 `*bv15`
                if let Some(alias) = rest.strip_prefix('*') {
                    let anchor_id = alias
                        .split_whitespace()
                        .next()
                        .unwrap_or("")
                        .split('#')
                        .next()
                        .unwrap_or("")
                        .trim();
                    current_block_anchor = anchor_id.to_string();
                }
            }
            continue;
        }

        if let Some(val) = parse_yaml_key_val(trimmed, "spirv_id:") {
            current_spirv_id = val.parse::<u32>().ok();
        } else if !in_members_inline {
            // binding 层级的字段（在 members 内部时不应电解析 binding 字段）
            if trimmed.starts_with("name:") {
                // binding 的名称，如 "$Globals"、"DIFFUSE_MAP__SMP"
                if let Some(val) = parse_yaml_key_val(trimmed, "name:") {
                    current_binding_name = val.trim_matches('"').to_string();
                }
            } else if trimmed.starts_with("binding:") {
                if let Some(val) = parse_yaml_key_val(trimmed, "binding:") {
                    if let Ok(b) = val.parse::<u32>() {
                        current_binding = b;
                    }
                }
            } else if let Some(val) = parse_yaml_key_val(trimmed, "descriptor_type:") {
                // 用 i32 解析，支持 -1 （VK_DESCRIPTOR_TYPE_???)
                let type_str = val.split('#').next().unwrap_or("").trim();
                if let Ok(t) = type_str.parse::<i32>() {
                    current_descriptor_type = t;
                }
            } else if trimmed.starts_with("members:") && current_block_anchor.is_empty() {
                // 只在没有 block 引用时解析内联 members
                in_members_inline = true;
            }
        } else {
            // in_members_inline: 解析内联 member 字段
            if trimmed.starts_with("- name:") {
                if !current_member_name.is_empty() {
                    current_members.insert(
                        current_member_name.clone(),
                        ShaderMemberLayout {
                            name: current_member_name.clone(),
                            offset: current_member_offset,
                            size: current_member_size,
                        },
                    );
                }
                current_member_name.clear();
                current_member_offset = 0;
                current_member_size = 0;
                if let Some(val) = parse_yaml_key_val(trimmed, "- name:") {
                    current_member_name = val.trim_matches('"').to_string();
                }
            } else if let Some(val) = parse_yaml_key_val(trimmed, "name:") {
                current_member_name = val.trim_matches('"').to_string();
            } else if let Some(val) = parse_yaml_key_val(trimmed, "offset:") {
                if let Ok(o) = val.parse::<usize>() {
                    current_member_offset = o;
                }
            } else if let Some(val) = parse_yaml_key_val(trimmed, "size:") {
                if let Ok(s) = val.parse::<usize>() {
                    current_member_size = s;
                    current_struct_size =
                        current_struct_size.max(current_member_offset + current_member_size);
                }
            }
        }
    }

    Ok(ShaderLayoutDescriptor { bindings })
}

fn parse_yaml_key_val<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    if let Some(idx) = line.find(key) {
        let val_part = &line[idx + key.len()..];
        return Some(val_part.trim());
    }
    None
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
