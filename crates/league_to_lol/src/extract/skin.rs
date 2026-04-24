use bevy::asset::{AssetPath, Handle};
use bevy::ecs::archetype;
use bevy::prelude::*;
use league_core::extract::{AnimationGraphData, EnumClipData, SkinCharacterDataProperties};
use league_file::animation::AnimationFile;
use league_file::mesh_skinned::LeagueSkinnedMesh;
use league_loader::game::{Data, LeagueLoader, PropGroup};
use league_loader::prop_bin::LeagueWadLoaderTrait;
use lol_base::animation::{AnimationHandler, ConfigAnimation, ConfigAnimationClip};
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
) {
    let Some(skin_bin_path) = skin_bin_path else {
        return;
    };

    // 加载皮肤 bin 文件获取 SkinCharacterDataProperties 和 links
    let Ok(skin_prop_file) = loader.get_prop_bin_by_path(skin_bin_path) else {
        println!("[WARN] 无法加载皮肤 bin 文件: {}", skin_bin_path);
        return;
    };

    // 从 links 创建 PropGroup 用于查找 SkinCharacterDataProperties 和 AnimationGraphData
    let link_paths: Vec<&str> = skin_prop_file
        .links
        .iter()
        .map(|link| link.text.as_str())
        .filter(|s| !s.is_empty())
        .collect();

    let skin_prop_group = match loader.get_prop_group_by_paths(link_paths) {
        Ok(group) => group,
        Err(_) => {
            println!("[WARN] 无法加载 linked bin 文件");
            return;
        }
    };

    let skin_data = match skin_prop_file.get_by_class::<SkinCharacterDataProperties>() {
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

    let output_glb_path = format!("assets/characters/{}/skin.glb", champ_name.to_lowercase());

    // 加载动画数据并导出到 GLB
    let animations = load_animations_for_skin(loader, &skin_prop_group, anim_graph_hash);

    if let Err(e) = export_skin_to_glb(&skinned_mesh, texture_png, &animations, &output_glb_path) {
        println!("[WARN] 皮肤 GLB 导出失败: {}", e);
        return;
    }

    // 导出动画 Asset（保留独立的 ron 文件用于运行时加载）
    let animation_ron_path =
        export_animation_for_skin(champ_name, skin_bin_path, &skin_prop_group, &skin_data);

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

    app.init_asset::<WorldAsset>();
    app.init_asset::<ConfigAnimation>();

    app.finish();
    app.cleanup();

    let world = app.world_mut();

    let asset_server = world.resource::<AssetServer>();
    let skin_handle: Handle<WorldAsset> = asset_server.load(
        AssetPath::from(format!("characters/{}/skin.glb", champ_name.to_lowercase()))
            .with_label(GltfAssetLabel::Scene(0).to_string()),
    );

    // 如果有动画，创建 AnimationHandler
    let animation_handler = animation_ron_path.map(|anim_path| {
        let anim_handle: Handle<ConfigAnimation> = asset_server.load(&anim_path);
        AnimationHandler(anim_handle)
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

    let output_skin_path = format!("assets/characters/{}/skin.ron", champ_name.to_lowercase());
    super::utils::write_to_file(&output_skin_path, serialized_scene);
}

/// 加载动画数据并导出到 GLB
/// 从 skin bin 的 links 组成的 PropGroup 中获取 AnimationGraphData
fn load_animations_for_skin(
    loader: &LeagueLoader,
    anim_prop_group: &PropGroup,
    anim_graph_hash: u32,
) -> Vec<(String, ConfigAnimationClip)> {
    // 从 PropGroup 中获取 AnimationGraphData
    let Some(anim_graph_data) =
        anim_prop_group.get_data_option::<AnimationGraphData>(anim_graph_hash)
    else {
        println!("[WARN] 无法获取 AnimationGraphData，从 links 中未找到");
        return Vec::new();
    };

    let mut animations = Vec::new();

    // 遍历所有 AtomicClipData，加载对应的 .anm 文件
    if let Some(clip_data_map) = &anim_graph_data.m_clip_data_map {
        for (hash, clip) in clip_data_map {
            if let EnumClipData::AtomicClipData(atomic_clip) = clip {
                let anm_path = &atomic_clip.m_animation_resource_data.m_animation_file_path;

                // 跳过空路径
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
                let clip_data = load_animation_file(anm_file);

                // 使用 hash 作为动画名称
                let name = format!("anim_{}", hash);
                animations.push((name, clip_data));
            }
        }
    }

    animations
}

/// 导出动画 Asset 并返回 asset 路径
fn export_animation_for_skin(
    champ_name: &str,
    skin_bin_path: &str,
    skin_prop_group: &PropGroup,
    skin_data: &SkinCharacterDataProperties,
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

    // Build node_index_map for AtomicClipData nodes
    let mut node_index_map = std::collections::HashMap::new();
    if let Some(ref clip_data_map) = anim_graph_data.m_clip_data_map {
        for (hash, clip) in clip_data_map {
            if matches!(clip, league_core::extract::EnumClipData::AtomicClipData(_)) {
                let next_index = node_index_map.len();
                node_index_map.insert(*hash, next_index);
            }
        }
    }

    // Convert to ConfigAnimation
    let config_animation = animation_graph_to_config(&anim_graph_data, &node_index_map);

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
