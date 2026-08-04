//! 粒子播放页的资产加载。
//!
//! 因为粒子渲染 server（lol_particle）只负责播放、唯一输入是一段
//! `ConfigVfxSystemDefinition` 的 RON 字符串，所以英雄列表与英雄粒子的读取/提取
//! 放在桌面端后端：用真实类型解析 skin0_vfx.ron（ConfigVfx），再把其中每个
//! system 序列化为 RON 字符串交给前端，由前端按需发给 server 播放。

use std::path::PathBuf;

use lol_share::{ConfigVfx, ConfigVfxSystemDefinition};
use serde::{Deserialize, Serialize};

use crate::error::AppError;

/// 单个粒子系统：hash、粒子名称，可直接发给 server 播放的 RON 字符串，
/// 以及可供前端调整的结构化定义。
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ParticleSystemDef {
    pub hash: u32,
    pub name: String,
    pub def_ron: String,
    pub def: ConfigVfxSystemDefinition,
}

/// 定位 assets/characters 目录（复用 lol_client 的 workspace 根探测）。
fn characters_dir() -> Result<PathBuf, AppError> {
    let root = crate::process::workspace_root()
        .ok_or_else(|| AppError::Generic("找不到 workspace 根目录".into()))?;
    Ok(root.join("assets").join("characters"))
}

/// 列出所有带 skin0_vfx.ron 的英雄（名称升序）。
#[tauri::command]
pub fn list_particle_heroes() -> Result<Vec<String>, AppError> {
    let base = characters_dir()?;
    let read_dir = std::fs::read_dir(&base)
        .map_err(|e| AppError::Generic(format!("读取 {} 失败: {e}", base.display())))?;

    let mut out = Vec::new();
    for entry in read_dir.flatten() {
        if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if base
            .join(&name)
            .join("skins")
            .join("skin0_vfx.ron")
            .is_file()
        {
            out.push(name);
        }
    }
    out.sort();
    Ok(out)
}

/// 加载某英雄的 skin0_vfx.ron，解析为 ConfigVfx，并把每个 system
/// 序列化为 RON 字符串返回（按粒子名称升序）。
#[tauri::command]
pub fn load_hero_particles(hero: String) -> Result<Vec<ParticleSystemDef>, AppError> {
    let vfx_path = characters_dir()?
        .join(&hero)
        .join("skins")
        .join("skin0_vfx.ron");
    if !vfx_path.is_file() {
        return Err(AppError::Generic(format!(
            "英雄 {hero} 不存在 skin0_vfx.ron"
        )));
    }

    let content = std::fs::read_to_string(&vfx_path)
        .map_err(|e| AppError::Generic(format!("读取 {} 失败: {e}", vfx_path.display())))?;
    // 因为磁盘上的 VfxTexture 只序列化 path，所以这里解析出的定义仍是纯 serde 结构，
    // 重新序列化后可被 server 端同一类型无损反序列化。
    let config: ConfigVfx = ron::from_str(&content)
        .map_err(|e| AppError::Generic(format!("解析 {hero} 的 ConfigVfx 失败: {e}")))?;

    let mut systems = Vec::with_capacity(config.systems.len());
    for (&hash, def) in &config.systems {
        let def_ron = ron::ser::to_string(def)
            .map_err(|e| AppError::Generic(format!("序列化 system {hash:08x} 失败: {e}")))?;
        systems.push(ParticleSystemDef {
            hash,
            name: def.particle_name.clone(),
            def_ron,
            def: def.clone(),
        });
    }
    systems.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(systems)
}

/// 将前端调整后的 ConfigVfxSystemDefinition 重新序列化为 RON 字符串。
#[tauri::command]
pub fn serialize_vfx_system(def: ConfigVfxSystemDefinition) -> Result<String, AppError> {
    ron::ser::to_string(&def)
        .map_err(|e| AppError::Generic(format!("序列化 system 失败: {e}")))
}
