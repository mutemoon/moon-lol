use std::process::Command;

use league_to_lol::extract::extract_all;

fn main() {
    let game_path = r"D:\WeGameApps\英雄联盟\Game";
    extract_all(game_path);
    post_process_mapgeo();
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
