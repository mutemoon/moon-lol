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
use lol_base_render::animation::{ConfigAnimationClip, LOLAnimationGraph, LOLAnimationGraphHandle};
use lol_base::audio::{AudioBank, ConfigAudio};
use lol_base::character::{HealthBar, Skin};
use lol_base_render::particle::{ConfigVfx, ConfigVfxHandle};
use ron::ser::{PrettyConfig, to_string_pretty};

use crate::animation::load_animation_file;
use crate::extract::animation::animation_graph_to_config;
use crate::extract::audio::export_audio_for_skin;
use crate::extract::utils::{extract_particle_texture, extract_texture, get_texture_path, write_to_file};
use crate::skin_gltf_export::export_skin_to_glb;
use crate::utils::decode_texture_to_png;

/// 导出角色的皮肤 GLB 和皮肤场景文件
pub fn extract_skin_for_champion(
    loader: &LeagueLoader,
    champ_name: &str,
    skin_bin_path: Option<&str>,
    hashes: &HashMap<u32, String>,
    _all_spell_names: &[String],
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

    let output_glb_path = format!("characters/{}/skins/{}.glb", champ_name, skin_id);

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
        // println!("{:?}", skin_mesh_properties.material_override);
    }

    // 获取 scale 和 bar_type
    let scale = skin_mesh_properties.skin_scale.unwrap_or(1.0);
    let bar_type = skin_data
        .health_bar_data
        .as_ref()
        .and_then(|h| h.unit_health_bar_style)
        .unwrap_or(0);
    let avatar_name = skin_data
        .icon_avatar
        .as_ref()
        .or(skin_data.icon_circle.as_ref());

    // 导出头像纹理
    let avatar = if let Some(name) = avatar_name {
        extract_texture(loader, name)
    } else {
        String::new()
    };

    // 构建皮肤场景 skin.ron
    let mut app = App::new();

    app.add_plugins(AssetPlugin::default());
    app.add_plugins(TaskPoolPlugin::default());

    app.init_asset::<AnimationGraph>();
    app.init_asset::<AnimationClip>();
    app.init_asset::<WorldAsset>();
    app.init_asset::<LOLAnimationGraph>();
    // 注册 ConfigVfx 资产与 ConfigVfxHandle 资源类型，使 skin{N}.ron 能承载指向 skin{N}_vfx.ron 的句柄
    app.init_asset::<ConfigVfx>();
    app.register_type::<ConfigVfxHandle>();
    // 注册 ConfigAudio 资产，使 skin{N}.ron 能承载指向 skin{N}_audio.ron 的 AudioBank 句柄
    app.init_asset::<ConfigAudio>();

    app.finish();
    app.cleanup();

    let world = app.world_mut();

    let asset_server = world.resource::<AssetServer>().clone();
    let skin_handle: Handle<WorldAsset> = asset_server.load(
        AssetPath::from(format!("characters/{}/skins/{}.glb", champ_name, skin_id))
            .with_label(GltfAssetLabel::Scene(0).to_string()),
    );

    // 导出动画 Asset（保留独立的 ron 文件用于运行时加载）
    let gltf_path = format!("characters/{}/skins/{}.glb", champ_name, skin_id);
    let animation_ron_path = export_animation_for_skin(
        &asset_server,
        champ_name,
        skin_bin_path,
        &skin_prop_group,
        &skin_data,
        hashes,
        &gltf_path,
        &hash_to_glb_index,
    );

    // 导出粒子系统/VFX 配置，作为 Resource 单独序列化为 skin{N}_vfx.ron
    let config_vfx = export_vfx_for_skin(
        loader,
        champ_name,
        &skin_prop_group,
        &skin_data,
        hashes,
    );

    // 导出音效配置，序列化为 skin{N}_audio.ron，并通过 AudioBank 组件挂到皮肤实体
    let config_audio = export_audio_for_skin(
        loader,
        champ_name,
        skin_id,
        &skin_data,
    );
    let output_audio_path = format!("characters/{}/skins/{}_audio.ron", champ_name, skin_id);
    let serialized_audio = to_string_pretty(&config_audio, PrettyConfig::default()).unwrap();
    super::utils::write_to_file(&output_audio_path, &serialized_audio);
    let audio_handle: Handle<ConfigAudio> = asset_server.load(&output_audio_path);

    // 如果有动画，创建 AnimationHandler
    let animation_handler = animation_ron_path.map(|anim_path| {
        let anim_handle = asset_server.load(&anim_path);
        let anim_graph_handle = asset_server.load(&anim_path);
        (
            LOLAnimationGraphHandle(anim_handle),
            AnimationGraphHandle(anim_graph_handle),
        )
    });

    let resolver_key = skin_data.m_resource_resolver.unwrap_or(0);

    let mut entity_builder = world.spawn((
        Skin {
            scale,
            avatar,
            resolver_key,
        },
        HealthBar { bar_type },
        Visibility::default(),
        WorldAssetRoot(skin_handle),
        AudioBank(audio_handle),
    ));
    if let Some(handler) = animation_handler {
        entity_builder.insert(handler);
    }
    let _entity = entity_builder.id();

    // 先把 ConfigVfx 以纯 RON（serde）写入 skin{N}_vfx.ron，运行时由 ConfigVfxLoader 加载
    let output_vfx_path = format!("characters/{}/skins/{}_vfx.ron", champ_name, skin_id);
    let serialized_vfx = to_string_pretty(&config_vfx, PrettyConfig::default()).unwrap();
    super::utils::write_to_file(&output_vfx_path, &serialized_vfx);

    // 在皮肤场景中放入指向 skin{N}_vfx.ron 的 Handle<ConfigVfx> 资源，
    // 皮肤场景反序列化写入主世界时会触发 ConfigVfxLoader 加载粒子定义
    let vfx_handle: Handle<ConfigVfx> = asset_server.load(&output_vfx_path);
    world.insert_resource(ConfigVfxHandle(vfx_handle));

    // 皮肤场景 skin{N}.ron：实体 + ConfigVfxHandle 资源
    {
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

        let output_skin_path = format!("characters/{}/skins/{}.ron", champ_name, skin_id);
        super::utils::write_to_file(&output_skin_path, serialized_scene);
    }
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

    // println!("{:?}", hash_to_glb_index);

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
                    node_index_map.insert(*hash, AnimationNodeIndex::new(glb_index + 1));
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
    let anim_path = format!("characters/{}/animations/{}.ron", champ_name, skin_id);
    let serialized = to_string_pretty(&config_animation, PrettyConfig::default()).unwrap();
    super::utils::write_to_file(&anim_path, &serialized);

    Some(anim_path)
}

/// 提取皮肤的粒子系统/VFX 配置，返回 ConfigVfx Resource（单独序列化为 skin{N}_vfx.ron）
fn export_vfx_for_skin(
    loader: &LeagueLoader,
    champ_name: &str,
    skin_prop_group: &PropGroup,
    skin_data: &SkinCharacterDataProperties,
    hashes: &HashMap<u32, String>,
) -> lol_base_render::particle::ConfigVfx {
    use lol_base_render::particle::{ConfigResourceResolver, ConfigVfx, VfxTexture};

    use crate::extract::vfx::convert_system_definition;

    let mut config_vfx_main = ConfigVfx::default();

    // 提取贴图并返回 VfxTexture（磁盘只存路径，运行时由 ConfigVfxLoader 填充 handle）
    let mut load_texture = |path: &str| -> VfxTexture {
        extract_particle_texture(loader, path);
        VfxTexture::from_path(get_texture_path(path))
    };

    let mut resolvers = Vec::new();
    if let Some(resolver_hash) = skin_data.m_resource_resolver {
        if let Some(resolver) =
            skin_prop_group.get_data_option::<league_core::extract::ResourceResolver>(resolver_hash)
        {
            resolvers.push((resolver_hash, resolver));
        }
    }
    if let Some(additional_hashes) = &skin_data.m_additional_resource_resolvers {
        for hash in additional_hashes {
            if let Some(resolver) =
                skin_prop_group.get_data_option::<league_core::extract::ResourceResolver>(*hash)
            {
                resolvers.push((*hash, resolver));
            }
        }
    }

    for (resolver_hash, resolver) in resolvers {
        // Convert and save ResourceResolver with String keys
        let mut mapped_resource_map = std::collections::BTreeMap::new();
        if let Some(ref resource_map) = resolver.resource_map {
            for (&trigger_hash, &vfx_hash) in resource_map {
                let key = hashes
                    .get(&trigger_hash)
                    .cloned()
                    .unwrap_or_else(|| format!("unk_0x{:x}", trigger_hash));
                mapped_resource_map.insert(key, vfx_hash);
            }
        }

        let config_resolver = ConfigResourceResolver {
            resource_map: mapped_resource_map,
        };
        config_vfx_main
            .resolvers
            .insert(resolver_hash, config_resolver);

        // Convert and save VfxSystemDefinitionData entries
        if let Some(ref resource_map) = resolver.resource_map {
            for (&_trigger_hash, &vfx_hash) in resource_map {
                if let Some(vfx_system) =
                    skin_prop_group
                        .get_data_option::<league_core::extract::VfxSystemDefinitionData>(vfx_hash)
                {
                    let mut config_vfx = convert_system_definition(&vfx_system, &mut load_texture);

                    // Extract textures and meshes referenced by this VFX system
                    extract_assets_for_vfx(loader, &mut config_vfx);

                    config_vfx_main.systems.insert(vfx_hash, config_vfx);
                }
            }
        }
    }

    // Clean up old fragmented vfx directory if exists
    let old_vfx_dir = std::path::Path::new("assets").join(format!("characters/{}/vfx", champ_name));
    if old_vfx_dir.exists() {
        let _ = std::fs::remove_dir_all(&old_vfx_dir);
    }

    // 清理旧的独立 vfx.ron（已改为单独的 skin{N}_vfx.ron 场景序列化）
    let legacy_vfx_ron =
        std::path::Path::new("assets").join(format!("characters/{}/vfx.ron", champ_name));
    if legacy_vfx_ron.exists() {
        let _ = std::fs::remove_file(&legacy_vfx_ron);
    }

    config_vfx_main
}

fn extract_assets_for_vfx(
    loader: &LeagueLoader,
    config_vfx: &mut lol_base_render::particle::ConfigVfxSystemDefinition,
) {
    if let Some(emitters) = config_vfx.complex_emitter_definition_data.as_mut() {
        for emitter in emitters {
            extract_assets_for_emitter(loader, emitter);
        }
    }
    if let Some(emitters) = config_vfx.simple_emitter_definition_data.as_mut() {
        for emitter in emitters {
            extract_assets_for_emitter(loader, emitter);
        }
    }
}

fn extract_assets_for_emitter(
    loader: &LeagueLoader,
    emitter: &mut lol_base_render::particle::ConfigVfxEmitterDefinition,
) {
    use lol_base_render::particle::ConfigVfxPrimitive;

    // 所有纹理（texture/particle_color_texture/normal_map_texture/texture_mult/base_texture）
    // 已在 convert_* 的 load_texture 闭包中提取并转为 Handle<Image>，这里仅处理静态网格

    // mesh file (.scb) in primitive
    if let Some(primitive) = emitter.primitive.as_ref() {
        match primitive {
            ConfigVfxPrimitive::VfxPrimitiveMesh {
                simple_mesh_name, ..
            }
            | ConfigVfxPrimitive::VfxPrimitiveAttachedMesh {
                simple_mesh_name, ..
            } => {
                if let Some(mesh_path) = simple_mesh_name.as_ref() {
                    if !mesh_path.is_empty() {
                        let target_exists = std::path::Path::new("assets").join(mesh_path).exists();
                        if !target_exists {
                            if let Ok(buf) = loader.get_wad_entry_buffer_by_path(mesh_path) {
                                write_to_file(mesh_path, buf);
                                println!("[EXTRACT] 已提取静态网格: {}", mesh_path);
                            } else {
                                println!("[WARN] 无法加载静态网格: {}", mesh_path);
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
}
