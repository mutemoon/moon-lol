use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, Write};
use std::path::Path;
use std::sync::Arc;

use binrw::{binrw, BinRead, BinResult, Endian};
use league_file::LeagueShader;
use league_utils::hash_shader;

// --- 1. 定义 binrw 结构体来解析 Shader Bundle ---

/// 代表一个独立的着色器文件（大小 + 数据）
#[binrw]
#[derive(Debug, Clone)]
pub struct ShaderFile {
    pub size: u32,
    #[br(count = size)]
    pub bytes: Vec<u8>,
}

/// 代表一个完整的着色器捆绑包（.glsl_0, .glsl_100, ...）
/// 它包含多个 ShaderFile
#[derive(Debug, Clone)]
pub struct ShaderChunkFile {
    pub files: Vec<ShaderFile>,
}

// 我们为 ShaderChunkFile 手动实现 BinRead，因为它是一个连续的文件流，没有固定的文件计数
impl BinRead for ShaderChunkFile {
    type Args<'a> = ();

    fn read_options<R: std::io::Read + std::io::Seek>(
        reader: &mut R,
        endian: binrw::Endian,
        _args: Self::Args<'_>,
    ) -> BinResult<Self> {
        let mut files = Vec::new();
        // 循环读取，直到文件末尾
        // 当无法再读取到一个 u32 时，说明已经读完所有着色器
        while let Ok(shader_file) = ShaderFile::read_options(reader, endian, ()) {
            files.push(shader_file);
        }
        Ok(ShaderChunkFile { files })
    }
}

// --- 2. 宏定义组合生成函数 ---

/// 根据基础宏定义列表，生成所有可能的组合 (2^n)
/// 例如, 输入 ["A", "B"] 会输出 [[], ["A"], ["B"], ["A", "B"]]
fn generate_define_combinations(base_defines: &[String]) -> Vec<Vec<String>> {
    let n = base_defines.len();
    let mut all_combinations = Vec::new();

    for i in 0..(1 << n) {
        let mut current_combination = Vec::new();
        for j in 0..n {
            // 使用位掩码检查第 j 个宏是否应该包含在当前组合中
            if (i >> j) & 1 == 1 {
                current_combination.push(base_defines[j].clone());
            }
        }
        all_combinations.push(current_combination);
    }

    all_combinations
}

// --- 3. 主函数 ---

fn main() -> std::io::Result<()> {
    let paths = vec![
        "assets/ASSETS/shaders/hlsl/skinnedmesh/particle_vs.vs.glsl",
        "assets/ASSETS/shaders/hlsl/skinnedmesh/particle_ps.ps.glsl",
        "assets/ASSETS/shaders/hlsl/particlesystem/distortion_mesh_ps.ps.glsl",
        "assets/ASSETS/shaders/hlsl/particlesystem/distortion_mesh_vs.vs.glsl",
        "assets/ASSETS/shaders/hlsl/particlesystem/distortion_ps.ps.glsl",
        "assets/ASSETS/shaders/hlsl/particlesystem/distortion_vs.vs.glsl",
        "assets/ASSETS/shaders/hlsl/particlesystem/mesh_ps_slice.ps.glsl",
        "assets/ASSETS/shaders/hlsl/particlesystem/mesh_ps.ps.glsl",
        "assets/ASSETS/shaders/hlsl/particlesystem/mesh_vs.vs.glsl",
        "assets/ASSETS/shaders/hlsl/particlesystem/quad_ps_fixedalphauv.ps.glsl",
        "assets/ASSETS/shaders/hlsl/particlesystem/quad_ps_slice.ps.glsl",
        "assets/ASSETS/shaders/hlsl/particlesystem/quad_ps.ps.glsl",
        "assets/ASSETS/shaders/hlsl/particlesystem/quad_screenspaceuv.ps.glsl",
        "assets/ASSETS/shaders/hlsl/particlesystem/quad_screenspaceuv.vs.glsl",
        "assets/ASSETS/shaders/hlsl/particlesystem/quad_vs_fixedalphauv.vs.glsl",
        "assets/ASSETS/shaders/hlsl/particlesystem/quad_vs.vs.glsl",
        "assets/ASSETS/shaders/hlsl/particlesystem/shadow_mesh_ps.ps.glsl",
        "assets/ASSETS/shaders/hlsl/particlesystem/shadow_mesh_vs.vs.glsl",
        "assets/ASSETS/shaders/hlsl/particlesystem/shadow_quad_ps.ps.glsl",
        "assets/ASSETS/shaders/hlsl/particlesystem/shadow_quad_vs.vs.glsl",
        "assets/ASSETS/shaders/hlsl/particlesystem/simple_projected_ps.ps.glsl",
        "assets/ASSETS/shaders/hlsl/particlesystem/simple_projected_vs.vs.glsl",
        "assets/ASSETS/shaders/hlsl/environment/unlit_decal_vs.vs.glsl",
        "assets/ASSETS/shaders/hlsl/environment/unlit_decal_ps.ps.glsl",
    ];

    let out_put_dir = "assets/shaders_extract";

    // 确保 shaders 输出目录存在
    fs::create_dir_all(out_put_dir)?;

    for path_str in paths {
        println!("> 处理文件: {}", path_str);

        // --- 准备路径和输出目录 ---
        let path = Path::new(path_str);
        let file_name = path.file_name().unwrap().to_str().unwrap();
        let shader_category = path
            .parent()
            .unwrap()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap();

        let stem = file_name.strip_suffix(".glsl").unwrap_or(file_name);

        let (shader_type, original_filename_base) = if stem.ends_with(".ps") {
            ("ps", stem.strip_suffix(".ps").unwrap())
        } else if stem.ends_with(".vs") {
            ("vs", stem.strip_suffix(".vs").unwrap())
        } else {
            println!("! 无法确定着色器类型: {}", file_name);
            continue;
        };

        let shader_name = original_filename_base.replace("_ps", "").replace("_vs", "");

        let output_dir_str = format!("{out_put_dir}/{shader_category}/{shader_name}/{shader_type}");
        fs::create_dir_all(&output_dir_str)?;
        println!("  - 输出目录: {}", output_dir_str);

        // --- 读取着色器目录文件 (TOC) ---
        let toc_file = File::open(path_str)?;
        let shader_toc = LeagueShader::read(&mut BufReader::new(toc_file)).unwrap();

        // println!(
        //     "shader_toc: {:#?}",
        //     shader_toc
        //         .base_defines
        //         .iter()
        //         .map(|v| (v.name.text.clone(), v.value.text.clone()))
        //         .collect::<Vec<(String, String)>>()
        // );

        let base_defines: Vec<String> = shader_toc
            .base_defines
            .iter()
            .map(|v| v.name.text.clone())
            .collect();

        println!(
            "  - 找到 {} 个基础宏定义: {:?}",
            base_defines.len(),
            base_defines
        );
        // --- 生成所有组合并开始提取 ---
        let combinations = generate_define_combinations(&base_defines);
        println!("  - 生成了 {} 种组合", combinations.len());

        // 用于缓存已读取的捆绑包文件，避免重复IO
        let mut bundle_cache: HashMap<String, Arc<ShaderChunkFile>> = HashMap::new();

        for mut combo in combinations {
            // --- 计算当前组合的哈希 ---
            // 必须排序以确保哈希值稳定
            combo.sort();
            let defines_string: String = combo
                .iter()
                .map(|v| {
                    shader_toc
                        .base_defines
                        .iter()
                        .find(|v2| v2.name.text == *v)
                        .unwrap()
                })
                .map(|v| format!("{}={}", v.name.text, v.value.text))
                .collect::<Vec<String>>()
                .join("");
            let defines_hash = hash_shader(&defines_string);

            // --- 查找哈希对应的索引和ID ---
            let Some(shader_index) = shader_toc
                .shader_hashes
                .iter()
                .position(|&h| h == defines_hash)
            else {
                continue;
            };

            let shader_id = shader_toc.shader_ids[shader_index];

            // --- 定位并读取捆绑包 ---
            let shader_bundle_id = 100 * (shader_id / 100);
            let shader_index_in_bundle = (shader_id % 100) as usize;
            let shader_bundle_path = format!("{}_{}", path_str, shader_bundle_id);

            // 检查缓存
            if !bundle_cache.contains_key(&shader_bundle_path) {
                // 缓存未命中，从文件读取
                if let Ok(bundle_file) = File::open(&shader_bundle_path) {
                    let mut reader = BufReader::new(bundle_file);
                    if let Ok(chunk_file) =
                        ShaderChunkFile::read_options(&mut reader, Endian::Little, ())
                    {
                        // 存入缓存
                        bundle_cache.insert(shader_bundle_path.clone(), Arc::new(chunk_file));
                    } else {
                        println!("! 无法解析捆绑包: {}", shader_bundle_path);
                        // 插入一个空的标记，避免重复尝试读取坏文件
                        bundle_cache.insert(
                            shader_bundle_path.clone(),
                            Arc::new(ShaderChunkFile { files: vec![] }),
                        );
                    }
                } else {
                    println!("! 无法打开捆绑包: {}", shader_bundle_path);
                    continue;
                }
            }

            // 从缓存中获取捆绑包数据
            if let Some(chunk_file_arc) = bundle_cache.get(&shader_bundle_path) {
                if shader_index_in_bundle < chunk_file_arc.files.len() {
                    // --- 提取并保存字节码 ---
                    let bytecode = &chunk_file_arc.files[shader_index_in_bundle].bytes;

                    let content = String::from_utf8_lossy(bytecode);

                    // let converted = if shader_type == "vs" {
                    //     convert(&content)
                    // } else {
                    //     convert_frag(&content)
                    // };
                    let converted = content;

                    let name = if combo.is_empty() {
                        "BASE"
                    } else {
                        &combo.join("__").to_uppercase()
                    };

                    let output_filename = format!(
                        "{}.{}",
                        name,
                        if shader_type == "ps" { "frag" } else { "vert" }
                    );

                    let output_path = Path::new(&output_dir_str).join(&output_filename);

                    let mut output_file = File::create(&output_path)?;

                    output_file.write_all(converted.as_bytes())?;
                } else {
                    println!(
                        "! 索引越界: ID {} 在 {} 中索引为 {}, 但文件只有 {} 个着色器",
                        shader_id,
                        shader_bundle_path,
                        shader_index_in_bundle,
                        chunk_file_arc.files.len()
                    );
                }
            }
        }
    }

    Ok(())
}
