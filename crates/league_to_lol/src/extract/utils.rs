use std::path::Path;

use league_loader::game::LeagueLoader;
use league_loader::prop_bin::LeagueWadLoaderTrait;
use serde::Serialize;

use crate::utils::{decode_texture_to_linear_png, decode_texture_to_png};

/// 从 WAD 中提取纹理图片到 assets 目录
/// 获取纹理的目标保存路径（处理后缀转换）
pub fn get_texture_path(texture_name: &str) -> String {
    if texture_name.ends_with(".tex") {
        texture_name.replace(".tex", ".png")
    } else {
        texture_name.to_string()
    }
}

/// 从 WAD 中提取纹理图片到 assets 目录
pub fn extract_texture(loader: &LeagueLoader, texture_name: &str) -> String {
    // 确定目标路径
    let target_path = get_texture_path(texture_name);

    // 检查文件是否已存在
    if Path::new("assets").join(&target_path).exists() {
        return target_path;
    }

    // 从 WAD 加载文件内容
    let Ok(buf) = loader.get_wad_entry_buffer_by_path(texture_name) else {
        println!("[WARN] 无法加载纹理: {}", texture_name);
        return target_path;
    };

    // 如果是 dds 或 png 则直接保存
    let texture_name_lc = texture_name.to_lowercase();
    if texture_name_lc.ends_with(".dds") || texture_name_lc.ends_with(".png") {
        write_to_file(&target_path, buf);
        println!("[EXTRACT] 已提取纹理: {}", target_path);
        return target_path;
    }

    // 解析 .tex 文件
    let Ok((_, texture)) = league_loader::texture::LeagueTexture::parse(&buf) else {
        println!("[WARN] 无法解析纹理: {}", texture_name);
        return target_path;
    };

    // 解码为 PNG
    let Some(png_data) = decode_texture_to_png(&texture) else {
        println!("[WARN] 无法解码纹理: {}", texture_name);
        return target_path;
    };

    // 写入文件
    write_to_file(&target_path, png_data);
    println!("[EXTRACT] 已提取纹理: {}", target_path);

    target_path
}

/// 提取粒子/VFX 贴图：解码为标注线性 gAMA 的 PNG。
/// 因为粒子贴图在游戏里按原值线性采样，且运行时由 ConfigVfxLoader 以 is_srgb=false 载入，
/// 所以无需再为每张贴图生成 .meta 旁车。
pub fn extract_particle_texture(loader: &LeagueLoader, texture_name: &str) -> String {
    let target_path = get_texture_path(texture_name);

    // PNG 已存在则不重复解码写盘
    if Path::new("assets").join(&target_path).exists() {
        return target_path;
    }

    let Ok(buf) = loader.get_wad_entry_buffer_by_path(texture_name) else {
        println!("[WARN] 无法加载粒子纹理: {}", texture_name);
        return target_path;
    };

    // 已是 dds/png 的资源直接落盘（色彩空间由 loader 加载时声明）
    let texture_name_lc = texture_name.to_lowercase();
    if texture_name_lc.ends_with(".dds") || texture_name_lc.ends_with(".png") {
        write_to_file(&target_path, buf);
        println!("[EXTRACT] 已提取粒子纹理: {}", target_path);
        return target_path;
    }

    let Ok((_, texture)) = league_loader::texture::LeagueTexture::parse(&buf) else {
        println!("[WARN] 无法解析粒子纹理: {}", texture_name);
        return target_path;
    };

    let Some(png_data) = decode_texture_to_linear_png(&texture) else {
        println!("[WARN] 无法解码粒子纹理: {}", texture_name);
        return target_path;
    };

    write_to_file(&target_path, png_data);
    println!("[EXTRACT] 已提取粒子纹理(线性): {}", target_path);

    target_path
}

pub fn write_to_file(path: &str, content: impl AsRef<[u8]>) {
    let path = std::path::Path::new("assets").join(path);
    let result = match path.parent() {
        Some(parent) => {
            std::fs::create_dir_all(parent).and_then(|_| std::fs::write(&path, content))
        }
        None => std::fs::write(&path, content),
    };
    // 写入失败（如游戏进程内存映射占用导致 os error 1224）不应中断整个提取流程
    if let Err(err) = result {
        println!("[ERROR] 无法写入文件 {}: {}", path.display(), err);
    }
}

/// 将二进制序列化内容写入文件
pub fn write_bin_to_file<T: Serialize>(path: &str, content: &T) {
    write_to_file(
        path,
        bincode::serialize(content).expect("无法序列化为二进制"),
    );
}
