use std::process::Command;

use league_to_lol::extract::extract_all;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

fn main() {
    let game_path = r"D:\WeGameApps\英雄联盟\Game";
    let hashes_dir = "assets/hashes";
    extract_all(game_path, hashes_dir);
    post_process_mapgeo();
    // post_process_all_skin_glb();
}

fn post_process_mapgeo() {
    println!("[POST] 开始后处理地图几何...");

    let map_name = "sr_seasonal_map";
    let input_glb = format!("assets/maps/{}/mapgeo.glb", map_name);
    let output_gltf = "assets/maps/output.gltf";

    // Step 1: gltf-transform optimize
    println!("[POST] 1. 优化 GLTF (draco + ktx2)...");
    let cmd = format!(
        "npx gltf-transform optimize {} {} --compress draco --texture-compress ktx2",
        input_glb, output_gltf
    );
    #[cfg(target_os = "windows")]
    let status = Command::new("cmd")
        .args(["/C", &cmd])
        .status()
        .expect("gltf-transform 命令执行失败");
    #[cfg(not(target_os = "windows"))]
    let status = Command::new("sh")
        .args(["-c", &cmd])
        .status()
        .expect("gltf-transform 命令执行失败");
    if !status.success() {
        eprintln!("[ERROR] gltf-transform 失败");
        return;
    }

    // Step 2: 读取并修复 texture source
    println!("[POST] 2. 修复纹理扩展...");
    let data = std::fs::read_to_string(output_gltf).expect("读取 output.gltf 失败");
    let mut json: serde_json::Value = serde_json::from_str(&data).expect("解析 GLTF JSON 失败");

    if let Some(textures) = json.get_mut("textures").and_then(|t| t.as_array_mut()) {
        for texture in textures {
            if let Some(extensions) = texture
                .get_mut("extensions")
                .and_then(|e| e.as_object_mut())
            {
                if let Some(khr_basisu) = extensions.remove("KHR_texture_basisu") {
                    if let Some(source) = khr_basisu.get("source").cloned() {
                        texture["source"] = source;
                    }
                } else if let Some(ext_webp) = extensions.remove("EXT_texture_webp") {
                    if let Some(source) = ext_webp.get("source").cloned() {
                        texture["source"] = source;
                    }
                }
            }
        }
    }

    std::fs::write(output_gltf, serde_json::to_string_pretty(&json).unwrap())
        .expect("写入 output.gltf 失败");

    // Step 3: gltf-pipeline 转换回 glb
    println!("[POST] 3. 转换回 GLB...");
    let final_glb = format!("assets/maps/{}/mapgeo.glb", map_name);
    let cmd = format!("npx gltf-pipeline -i {} -o {}", output_gltf, final_glb);
    #[cfg(target_os = "windows")]
    let status = Command::new("cmd")
        .args(["/C", &cmd])
        .status()
        .expect("gltf-pipeline 命令执行失败");
    #[cfg(not(target_os = "windows"))]
    let status = Command::new("sh")
        .args(["-c", &cmd])
        .status()
        .expect("gltf-pipeline 命令执行失败");
    if !status.success() {
        eprintln!("[ERROR] gltf-pipeline 失败");
        return;
    }

    // Step 4: 清理临时文件
    println!("[POST] 4. 清理临时文件...");
    let _ = std::fs::remove_file(output_gltf);
    let _ = std::fs::remove_file("assets/maps/output.bin");

    if let Some(images) = json.get("images").and_then(|i| i.as_array()) {
        for image in images {
            if let Some(uri) = image.get("uri").and_then(|u| u.as_str()) {
                let path = format!("assets/maps/{}", uri);
                let _ = std::fs::remove_file(path);
            }
        }
    }

    println!("[POST] 后处理完成!");
}

fn post_process_all_skin_glb() {
    println!("[POST] 开始压缩所有皮肤 GLB 文件...");

    let glb_files: Vec<_> = walkdir::WalkDir::new("assets/characters")
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "glb"))
        .map(|e| e.path().to_string_lossy().to_string())
        .collect();

    if glb_files.is_empty() {
        println!("[POST] 未找到皮肤 GLB 文件");
        return;
    }

    println!("[POST] 找到 {} 个皮肤 GLB 文件", glb_files.len());

    glb_files.par_iter().for_each(|glb_path| {
        let input_glb = glb_path;
        let output_gltf = format!("{}.gltf", glb_path);

        // Step 1: gltf-transform optimize
        print!("[POST] 优化 {}...", input_glb);
        let cmd = format!(
            "npx gltf-transform optimize {} {} --compress draco --texture-compress ktx2",
            input_glb, output_gltf
        );
        let status = Command::new("cmd").args(["/C", &cmd]).status();
        if !status.map(|s| s.success()).unwrap_or(false) {
            println!(" 失败");
            return;
        }

        // Step 2: 读取并修复 texture source
        let data = std::fs::read_to_string(&output_gltf);
        if data.is_err() {
            println!(" 失败");
            let _ = std::fs::remove_file(&output_gltf);
            return;
        }
        let mut json: serde_json::Value = serde_json::from_str(&data.unwrap()).unwrap();

        if let Some(textures) = json.get_mut("textures").and_then(|t| t.as_array_mut()) {
            for texture in textures {
                if let Some(extensions) = texture
                    .get_mut("extensions")
                    .and_then(|e| e.as_object_mut())
                {
                    if let Some(khr_basisu) = extensions.remove("KHR_texture_basisu") {
                        if let Some(source) = khr_basisu.get("source").cloned() {
                            texture["source"] = source;
                        }
                    } else if let Some(ext_webp) = extensions.remove("EXT_texture_webp") {
                        if let Some(source) = ext_webp.get("source").cloned() {
                            texture["source"] = source;
                        }
                    }
                }
            }
        }

        if std::fs::write(&output_gltf, serde_json::to_string(&json).unwrap()).is_err() {
            println!(" 失败");
            let _ = std::fs::remove_file(&output_gltf);
            return;
        }

        // Step 3: gltf-pipeline 转换回 glb
        let cmd = format!("npx gltf-pipeline -i {} -o {}", output_gltf, input_glb);
        let status = Command::new("cmd").args(["/C", &cmd]).status();
        if !status.map(|s| s.success()).unwrap_or(false) {
            println!(" 失败");
            let _ = std::fs::remove_file(&output_gltf);
            return;
        }

        // Step 4: 清理临时文件
        let _ = std::fs::remove_file(&output_gltf);
        if let Some(images) = json.get("images").and_then(|i| i.as_array()) {
            for image in images {
                if let Some(uri) = image.get("uri").and_then(|u| u.as_str()) {
                    let path = format!("assets/{}", uri);
                    let _ = std::fs::remove_file(path);
                }
            }
        }

        println!(" 完成");
    });

    println!("[POST] 皮肤 GLB 压缩完成!");
}
