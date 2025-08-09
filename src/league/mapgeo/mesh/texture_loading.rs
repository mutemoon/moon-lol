use super::types::Submesh;
use crate::league::LeagueLoader;
use bevy::image::Image;
use bevy::prelude::*;
use cdragon_prop::{BinEmbed, BinEntry, BinList, BinString, PropFile};

/// 根据 submesh 的材质名，查找材质属性并加载对应的贴图文件。
/// 如果失败，则返回一个默认的白色贴图。
pub fn find_and_load_image_for_submesh(
    submesh: &Submesh,
    map_materials: &PropFile,
    league_loader: &LeagueLoader,
) -> Option<Image> {
    // 1. 根据材质名查找 texturePath
    let binhash = LeagueLoader::hash_bin(&submesh.material_name.text);

    for entry in &map_materials.entries {
        if entry.path.hash == binhash {
            if let Some(texture_path) = find_texture_path_in_material_entry(entry) {
                match league_loader.get_image_by_texture_path(&texture_path) {
                    Ok(image) => return Some(image),
                    Err(e) => warn!("Failed to load texture {}: {}", texture_path, e),
                }
            }
        }
    }

    None
}

/// 在单个材质条目中查找 "texturePath" 的值。
/// 材质属性通常是嵌套的，结构为：samplerValues -> (list) -> (embed) -> texturePath。
/// 这里使用了 `find_map` 来高效地找到第一个匹配项。
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
