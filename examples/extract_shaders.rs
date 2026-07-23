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

use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use clap::Parser;
use league_file::shader::LeagueShaderToc;
use league_loader::prop_bin::LeagueWadLoaderTrait;
use league_loader::wad::LeagueWadLoader;
use league_utils::{LeagueShader, hash_wad};
use serde::{Deserialize, Serialize};

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

    /// DXBC to SPIR-V Compiler (dxbc_compiler.exe) 路径
    #[arg(long, default_value = r"assets\tools\dxbc_compiler.exe")]
    dxbc_compiler: String,

    /// 需要处理的 TOC 路径列表（用逗号分隔，如 "shaders/unlit_decal_ps.ps.dx11"）
    #[arg(long, value_delimiter = ',')]
    toc_paths: Vec<String>,

    /// 是否跳过已存在的输出文件
    #[arg(long, default_value_t = true)]
    skip_existing: bool,
}

/// 用于序列化到 RON 的全局 shader map 结构
#[derive(Debug, Serialize, Deserialize)]
struct ShaderMap {
    /// LeagueShader enum -> (shader hash -> spv path)
    pub entries: HashMap<LeagueShader, HashMap<u64, String>>,
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
            // ParticleMaterialQuadSlice
            "assets/shaders/hlsl/particlesystem/quad_ps_slice.ps.dx11".to_string(),
            "assets/shaders/hlsl/particlesystem/quad_vs.vs.dx11".to_string(),
            // ParticleMaterialQuad（VERT_PATH 与 QuadSlice 相同）
            "assets/shaders/hlsl/particlesystem/quad_ps.ps.dx11".to_string(),
            // ParticleMaterialUnlitDecal
            "assets/shaders/hlsl/environment/unlit_decal_ps.ps.dx11".to_string(),
            "assets/shaders/hlsl/environment/unlit_decal_vs.vs.dx11".to_string(),
            // ParticleMaterialDistortion
            "assets/shaders/hlsl/particlesystem/distortion_ps.ps.dx11".to_string(),
            "assets/shaders/hlsl/particlesystem/distortion_vs.vs.dx11".to_string(),
            // ParticleMaterialMesh
            "assets/shaders/hlsl/particlesystem/mesh_ps.ps.dx11".to_string(),
            "assets/shaders/hlsl/particlesystem/mesh_vs.vs.dx11".to_string(),
            // ParticleMaterialSkinnedMeshParticle
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
        ) {
            Ok(map_entries) => {
                global_entries.insert(shader_type, map_entries);
            }
            Err(e) => {
                eprintln!("[ERROR] 处理 TOC {} 失败: {}", toc_path, e);
            }
        }
    }

    let map_path = out_dir.join("map.ron");
    let shader_map = ShaderMap {
        entries: global_entries,
    };
    let ron_str = ron::ser::to_string_pretty(&shader_map, ron::ser::PrettyConfig::default())?;
    fs::write(&map_path, ron_str)?;

    println!("\n[DONE] 全局 shader 映射已写入 {}", map_path.display());
    Ok(())
}

fn process_toc(
    wad_loader: &LeagueWadLoader,
    toc_path: &str,
    out_dir: &Path,
    dxbc_compiler_path: &Path,
    skip_existing: bool,
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

    // 编译每个 bundled shader
    let mut spv_paths: Vec<Option<String>> = vec![None; shader_toc.bundled_shader_count as usize];

    for idx in 0..shader_toc.bundled_shader_count as usize {
        if idx >= dxbc_blobs.len() {
            eprintln!("[WARN] shader #{} 超出 blobs 范围", idx);
            break;
        }

        let spv_filename = format!("shader_{:04}.spv", idx);
        let spv_path = toc_out_dir.join(&spv_filename);
        let spv_relative = format!("shaders/{}/{}", toc_name, spv_filename);

        if skip_existing && spv_path.exists() {
            println!("  [SKIP] shader_{:04} (已存在)", idx);
            spv_paths[idx] = Some(spv_relative);
            continue;
        }

        print!("  [COMPILE] shader_{:04} ... ", idx);
        std::io::stdout().flush().ok();

        match compile_dxbc_to_spirv(&dxbc_blobs[idx], dxbc_compiler_path, &toc_out_dir, idx) {
            Ok(spv_bytes) => {
                fs::write(&spv_path, &spv_bytes)?;
                println!("OK ({} bytes)", spv_bytes.len());
                spv_paths[idx] = Some(spv_relative);
            }
            Err(e) => {
                println!("FAIL");
                eprintln!("    [ERROR] {}", e);
            }
        }
    }

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
) -> anyhow::Result<Vec<u8>> {
    let prefix = format!("_tmp_{:04}", idx);
    let dxbc_filename = format!("{}.dxbc", prefix);
    let spv_filename = format!("{}.spv", prefix);

    let dxbc_path_tmp = work_dir.join(&dxbc_filename);
    let spv_path_tmp = work_dir.join(&spv_filename);

    // ── Step 1: 写 DXBC 到临时文件 ─────────────────────────────────
    fs::write(&dxbc_path_tmp, dxbc_data)?;

    // ── Step 2: DXBC → SPIR-V via dxbc_compiler ──────────────────────
    // dxbc_compiler.exe --spv <output.spv> <input.dxbc>
    let compiler_out = Command::new(dxbc_compiler_path)
        .arg("--spv")
        .arg(&spv_filename)
        .arg(&dxbc_filename)
        .current_dir(work_dir)
        .output()
        .map_err(|e| anyhow::anyhow!("启动 dxbc_compiler 失败: {}", e))?;

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
