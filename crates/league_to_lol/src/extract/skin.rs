use std::collections::HashMap;

use bevy::asset::{AssetPath, Handle};
use bevy::ecs::archetype;
use bevy::prelude::*;
use league_core::extract::{
    AnimationGraphData, EnumClipData, SkinCharacterDataProperties, StaticMaterialDef,
};
use league_file::animation::AnimationFile;
use league_file::mesh_skinned::LeagueSkinnedMesh;
use league_file::skeleton::LeagueSkeleton;
use league_loader::game::{Data, LeagueLoader, PropGroup};
use league_loader::prop_bin::LeagueWadLoaderTrait;
use lol_base::animation::{ConfigAnimationClip, LOLAnimationGraph, LOLAnimationGraphHandle};
use lol_base::character::{HealthBar, Skin};
use ron::ser::{PrettyConfig, to_string_pretty};

use crate::animation::load_animation_file;
use crate::extract::animation::animation_graph_to_config;
use crate::skin_gltf_export::{decode_texture_to_png, export_skin_to_glb};

/// 导出角色的皮肤 GLB 和皮肤场景文件
pub fn extract_skin_for_champion(
    loader: &LeagueLoader,
    champ_name: &str,
    skin_bin_path: Option<&str>,
    hashes: &HashMap<u32, String>,
) {
    let Some(skin_bin_path) = skin_bin_path else {
        return;
    };

    // Get skin_id from skin_bin_path (e.g., "skin0" from ".../skins/skin0.bin")
    let skin_id = skin_bin_path
        .split('/')
        .last()
        .unwrap_or("skin0")
        .trim_end_matches(".bin");

    println!("[DEBUG] 正在加载 {} skin {}", champ_name, skin_id);

    let skin_prop_group = match loader.get_prop_group_by_paths(vec![skin_bin_path]) {
        Ok(group) => group,
        Err(_) => {
            println!("[WARN] 无法加载 linked bin 文件");
            return;
        }
    };

    let skin_data = match skin_prop_group.get_by_class::<SkinCharacterDataProperties>() {
        Some(data) => data,
        None => {
            println!("[WARN] 无法获取 SkinCharacterDataProperties");
            return;
        }
    };

    let anim_graph_hash = skin_data.skin_animation_properties.animation_graph_data;

    let skin_mesh_properties = match &skin_data.skin_mesh_properties {
        Some(props) => props,
        None => return,
    };

    let simple_skin_path = match &skin_mesh_properties.simple_skin {
        Some(path) => path,
        None => return,
    };

    let texture_path = match &skin_mesh_properties.texture {
        Some(path) => path.clone(),
        None => return,
    };

    // 加载 .skn 文件
    let skn_buf = match loader.get_wad_entry_buffer_by_path(simple_skin_path) {
        Ok(buf) => buf,
        Err(_) => {
            println!("[WARN] 无法加载 SKN 文件: {}", simple_skin_path);
            return;
        }
    };

    let (_, skinned_mesh) = match LeagueSkinnedMesh::parse(&skn_buf) {
        Ok(mesh) => mesh,
        Err(_) => {
            println!("[WARN] 无法解析 SKN 文件: {}", simple_skin_path);
            return;
        }
    };

    // 加载 .tex 文件并解码为 PNG
    let texture_png = loader
        .get_wad_entry_buffer_by_path(&texture_path)
        .ok()
        .and_then(|buf| {
            let (_, texture) = league_file::texture::LeagueTexture::parse(&buf).ok()?;
            decode_texture_to_png(&texture)
        });

    // 加载 .skl 文件（骨架数据）
    let skeleton = skin_mesh_properties.skeleton.as_ref().and_then(|skl_path| {
        loader
            .get_wad_entry_buffer_by_path(skl_path)
            .ok()
            .and_then(|buf| LeagueSkeleton::parse(&buf).ok().map(|(_, s)| s))
    });

    let output_glb_path = format!(
        "assets/characters/{}/skins/{}.glb",
        champ_name.to_lowercase(),
        skin_id
    );

    // 加载动画数据并导出到 GLB
    let (animations, hash_to_glb_index) =
        load_animations_for_skin(loader, &skin_prop_group, anim_graph_hash, hashes);

    // 加载 material override 的贴图
    // materialOverride 的 material 字段是 link 哈希，需要加载对应的 bin 文件获取贴图
    // 注意：材质可能存储在不同的 wad 文件中，需要使用 skin_prop_group 来获取
    let material_override = skin_mesh_properties.material_override.as_ref().map(|overrides| {
        let mut override_map = std::collections::HashMap::new();
        for override_item in overrides {
            let submesh_name = &override_item.submesh;
            if let Some(material_hash) = override_item.material {
                // 通过 skin_prop_group 获取 StaticMaterialDef
                if let Some(static_material) = skin_prop_group.get_data_option::<StaticMaterialDef>(material_hash  ) {
                    // 遍历 sampler_values 找到 Diffuse_Texture
                    if let Some(samplers) = &static_material.sampler_values {
                        for sampler in samplers {
                            if &sampler.texture_name == "Diffuse_Texture" {
                                if let Some(texture_path) = &sampler.texture_path {
                                    println!("[DEBUG] Found Diffuse_Texture for submesh '{}': path={}", submesh_name, texture_path);
                                    if let Ok(buf) = loader.get_wad_entry_buffer_by_path(texture_path) {
                                        if let Ok((_, texture)) = league_file::texture::LeagueTexture::parse(&buf) {
                                            if let Some(png_data) = decode_texture_to_png(&texture) {
                                                override_map.insert(override_item.submesh.clone(), png_data);
                                            }
                                            else {
                                                println!("[DEBUG] no png_data");
                                            }
                                        }
                                        else {
                                            println!("[DEBUG] no LeagueTexture::parse");
                                        }
                                    }
                                    else {
                                        println!("[DEBUG] no get_wad_entry_buffer_by_path(texture_path");
                                    }
                                }
                            }
                        }
                    } else {
                        println!("[DEBUG] No sampler_values for submesh '{}'", submesh_name);
                    }
                } else {
                    println!("[DEBUG] StaticMaterialDef not found in skin_prop_group for submesh '{}': hash={}", submesh_name, material_hash);
                }
            } else if let Some(texture_path) = &override_item.texture {
                if let Ok(buf) = loader.get_wad_entry_buffer_by_path(texture_path) {
                    if let Ok((_, texture)) = league_file::texture::LeagueTexture::parse(&buf) {
                        if let Some(png_data) = decode_texture_to_png(&texture) {
                            override_map.insert(override_item.submesh.clone(), png_data);
                        }
                    }
                }
            }
        }
        override_map
    });

    if let Err(e) = export_skin_to_glb(
        &skinned_mesh,
        texture_png,
        skeleton.as_ref(),
        &animations,
        &output_glb_path,
        material_override.as_ref(),
        hashes,
    ) {
        println!("[WARN] 皮肤 GLB 导出失败: {}", e);
        return;
    } else {
        println!("{:?}", skin_mesh_properties.material_override);
    }

    // 获取 scale 和 bar_type
    let scale = skin_mesh_properties.skin_scale.unwrap_or(1.0);
    let bar_type = skin_data
        .health_bar_data
        .as_ref()
        .and_then(|h| h.unit_health_bar_style)
        .unwrap_or(0);

    // 构建皮肤场景 skin.ron
    let mut app = App::new();

    app.add_plugins(AssetPlugin::default());
    app.add_plugins(TaskPoolPlugin::default());

    app.init_asset::<AnimationGraph>();
    app.init_asset::<AnimationClip>();
    app.init_asset::<WorldAsset>();
    app.init_asset::<LOLAnimationGraph>();

    app.finish();
    app.cleanup();

    let world = app.world_mut();

    let asset_server = world.resource::<AssetServer>();
    let skin_handle: Handle<WorldAsset> = asset_server.load(
        AssetPath::from(format!(
            "characters/{}/skins/{}.glb",
            champ_name.to_lowercase(),
            skin_id
        ))
        .with_label(GltfAssetLabel::Scene(0).to_string()),
    );

    // 导出动画 Asset（保留独立的 ron 文件用于运行时加载）
    let gltf_path = format!(
        "characters/{}/skins/{}.glb",
        champ_name.to_lowercase(),
        skin_id
    );
    let animation_ron_path = export_animation_for_skin(
        asset_server,
        champ_name,
        skin_bin_path,
        &skin_prop_group,
        &skin_data,
        hashes,
        &gltf_path,
        &hash_to_glb_index,
    );

    // 如果有动画，创建 AnimationHandler
    let animation_handler = animation_ron_path.map(|anim_path| {
        let anim_handle: Handle<LOLAnimationGraph> = asset_server.load(&anim_path);
        let anim_graph_handle =
            asset_server.load::<AnimationGraph>(&format!("{}#animation_graph", anim_path));
        (
            LOLAnimationGraphHandle(anim_handle),
            AnimationGraphHandle(anim_graph_handle),
        )
    });

    let mut entity_builder = world.spawn((
        Skin { scale },
        HealthBar { bar_type },
        Visibility::default(),
        WorldAssetRoot(skin_handle),
    ));
    if let Some(handler) = animation_handler {
        entity_builder.insert(handler);
    }
    let _entity = entity_builder.id();

    let type_registry = world.resource::<AppTypeRegistry>();
    let type_registry = type_registry.read();
    let scene = DynamicWorldBuilder::from_world(&world, &type_registry)
        .deny_component::<InheritedVisibility>()
        .deny_component::<ViewVisibility>()
        .deny_component::<GlobalTransform>()
        .deny_component::<Transform>()
        .deny_component::<TransformTreeChanged>()
        .extract_entities(
            // we do this instead of a query, in order to completely sidestep default query filters.
            // while we could use `Allow<_>`, this wouldn't account for custom disabled components
            world
                .archetypes()
                .iter()
                .flat_map(archetype::Archetype::entities)
                .map(archetype::ArchetypeEntity::id),
        )
        .extract_resources()
        .build();
    let serialized_scene = scene.serialize(&type_registry).unwrap();

    let output_skin_path = format!(
        "assets/characters/{}/skins/{}.ron",
        champ_name.to_lowercase(),
        skin_id
    );
    super::utils::write_to_file(&output_skin_path, serialized_scene);
}

/// 加载动画数据并导出到 GLB
/// 从 skin bin 的 links 组成的 PropGroup 中获取 AnimationGraphData
/// 返回 (animations, hash_to_glb_index) - animations 按 hash 排序，hash_to_glb_index 记录 hash 对应的 GLB 动画索引
fn load_animations_for_skin(
    loader: &LeagueLoader,
    anim_prop_group: &PropGroup,
    anim_graph_hash: u32,
    _hashes: &HashMap<u32, String>,
) -> (Vec<(u32, ConfigAnimationClip)>, HashMap<u32, usize>) {
    // 从 PropGroup 中获取 AnimationGraphData
    let Some(anim_graph_data) =
        anim_prop_group.get_data_option::<AnimationGraphData>(anim_graph_hash)
    else {
        println!("[WARN] 无法获取 AnimationGraphData，从 links 中未找到");
        return (Vec::new(), HashMap::new());
    };

    let mut animations: Vec<(u32, ConfigAnimationClip)> = Vec::new();
    let mut hash_to_glb_index: HashMap<u32, usize> = HashMap::new();

    // 遍历所有 AtomicClipData，加载对应的 .anm 文件
    let Some(clip_data_map) = &anim_graph_data.m_clip_data_map else {
        return (animations, hash_to_glb_index);
    };

    for (hash, clip) in clip_data_map {
        let EnumClipData::AtomicClipData(atomic_clip) = clip else {
            continue;
        };

        let anm_path = &atomic_clip.m_animation_resource_data.m_animation_file_path;
        if anm_path.is_empty() {
            continue;
        }

        // 加载 .anm 文件
        let Ok(anm_buf) = loader.get_wad_entry_buffer_by_path(anm_path) else {
            println!("[WARN] 无法加载 .anm 文件: {}", anm_path);
            continue;
        };

        // 解析 .anm 文件
        let Ok((_, anm_file)) = AnimationFile::parse(&anm_buf) else {
            println!("[WARN] 无法解析 .anm 文件: {}", anm_path);
            continue;
        };

        // 转换为 ConfigAnimationClip
        let mut clip_data = load_animation_file(anm_file);

        // 附加蒙版数据（m_mask_data_map）
        if let Some(mask_name) = atomic_clip.m_mask_data_name {
            if let Some(mask_map) = &anim_graph_data.m_mask_data_map {
                if let Some(mask_data) = mask_map.get(&mask_name) {
                    clip_data.mask_weights = Some(mask_data.m_weight_list.clone());
                }
            }
        }

        animations.push((*hash, clip_data));
    }

    // 记录每个 hash 对应的 GLB 动画索引
    for (idx, (hash, _)) in animations.iter().enumerate() {
        hash_to_glb_index.insert(*hash, idx);
    }

    println!("{:?}", hash_to_glb_index);

    (animations, hash_to_glb_index)
}

/// 导出动画 Asset 并返回 asset 路径
/// hash_to_glb_index: 记录每个 hash 对应的 GLB 动画索引
fn export_animation_for_skin(
    _asset_server: &AssetServer,
    champ_name: &str,
    skin_bin_path: &str,
    skin_prop_group: &PropGroup,
    skin_data: &SkinCharacterDataProperties,
    hashes: &HashMap<u32, String>,
    gltf_path: &str,
    hash_to_glb_index: &HashMap<u32, usize>,
) -> Option<String> {
    let anim_graph_hash = skin_data.skin_animation_properties.animation_graph_data;

    // 从 PropGroup 中获取 AnimationGraphData
    let anim_graph_data =
        match skin_prop_group.get_data_option::<AnimationGraphData>(anim_graph_hash) {
            Some(data) => data,
            None => {
                println!("[WARN] 无法获取 AnimationGraphData，从 links 中未找到");
                return None;
            }
        };

    // Get skin_id from skin_bin_path (e.g., "skin0" from ".../skins/skin0.bin")
    let skin_id = skin_bin_path
        .split('/')
        .last()
        .unwrap_or("skin0")
        .trim_end_matches(".bin");

    // Build node_index_map using the GLB indices from hash_to_glb_index
    let mut node_index_map = std::collections::HashMap::new();
    if let Some(ref clip_data_map) = anim_graph_data.m_clip_data_map {
        for (hash, clip) in clip_data_map {
            if let EnumClipData::AtomicClipData(_) = clip {
                if let Some(&glb_index) = hash_to_glb_index.get(hash) {
                    node_index_map.insert(*hash, AnimationNodeIndex::new(glb_index));
                }
            }
        }
    }

    // Convert to ConfigAnimation
    let config_animation = animation_graph_to_config(
        &anim_graph_data,
        &node_index_map,
        hashes,
        gltf_path.to_string(),
    );

    // Export to .ron file
    let anim_path = format!(
        "characters/{}/animations/{}.ron",
        champ_name.to_lowercase(),
        skin_id.to_lowercase()
    );
    let output_path = format!("assets/{}", anim_path);
    let serialized = to_string_pretty(&config_animation, PrettyConfig::default()).unwrap();
    super::utils::write_to_file(&output_path, &serialized);

    Some(anim_path)
}
