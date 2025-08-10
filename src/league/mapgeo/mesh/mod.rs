mod enums;
mod skinned;
mod static_mesh;
mod types;
mod vertex_parsing;

pub use enums::*;
pub use skinned::*;
pub use static_mesh::*;
pub use types::*;

use crate::league::{LeagueLoader, LeagueMapGeo};
use bevy::prelude::*;
use cdragon_prop::{BinEmbed, BinEntry, BinList, BinString, PropFile};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct LeagueMaterial {
    pub texture_path: String,
}

pub fn find_and_load_image_for_submesh(
    material_name: &str,
    map_materials: &PropFile,
) -> Option<LeagueMaterial> {
    // 1. 根据材质名查找 texturePath
    let binhash = LeagueLoader::hash_bin(material_name);

    for entry in &map_materials.entries {
        if entry.path.hash == binhash {
            if let Some(texture_path) = find_texture_path_in_material_entry(entry) {
                return Some(LeagueMaterial {
                    texture_path: texture_path,
                });
            }
        }
    }

    None
}

fn find_texture_path_in_material_entry(
    material_entry: &BinEntry, // 请替换为你的材质条目具体类型
) -> Option<String> {
    // 1. 获取 "samplerValues" 列表
    let sampler_values =
        material_entry.getv::<BinList>(LeagueLoader::hash_bin("samplerValues").into())?;

    // 2. 将列表转换为可迭代的 BinEmbed
    let embedded_samplers = sampler_values.downcast::<BinEmbed>()?;

    // 3. 遍历所有 sampler，查找第一个包含 "texturePath" 的
    // `find_map` 会在找到第一个 Some(T) 后立即停止，比 filter_map + collect + first 更高效
    embedded_samplers.into_iter().find_map(|sampler_item| {
        let texture_name = &sampler_item
            .getv::<BinString>(LeagueLoader::hash_bin("textureName").into())?
            .0;
        if !(texture_name == "DiffuseTexture" || texture_name == "Diffuse_Texture") {
            return None;
        }
        sampler_item
            .getv::<BinString>(LeagueLoader::hash_bin("texturePath").into())
            .map(|v| v.0.clone())
    })
}
