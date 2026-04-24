use bevy::asset::{AssetPath, Handle};
use bevy::ecs::archetype;
use bevy::prelude::*;
use league_core::extract::SkinCharacterDataProperties;
use league_file::mesh_skinned::LeagueSkinnedMesh;
use league_loader::game::LeagueLoader;
use league_loader::prop_bin::LeagueWadLoaderTrait;
use lol_base::character::{HealthBar, Skin};

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

    // 加载皮肤 bin 文件获取 SkinCharacterDataProperties
    let Ok(skin_prop_group) = loader.get_prop_group_by_paths(vec![skin_bin_path]) else {
        println!("[WARN] 无法加载皮肤 bin 文件: {}", skin_bin_path);
        return;
    };

    let Some(skin_data) = skin_prop_group.get_by_class::<SkinCharacterDataProperties>() else {
        println!(
            "[WARN] 无法获取 SkinCharacterDataProperties: {}",
            skin_bin_path
        );
        return;
    };

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
    if let Err(e) = export_skin_to_glb(&skinned_mesh, texture_png, &output_glb_path) {
        println!("[WARN] 皮肤 GLB 导出失败: {}", e);
        return;
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

    app.init_asset::<WorldAsset>();

    app.finish();
    app.cleanup();

    let world = app.world_mut();

    let asset_server = world.resource::<AssetServer>();
    let skin_handle: Handle<WorldAsset> = asset_server.load(
        AssetPath::from(format!("characters/{}/skin.glb", champ_name.to_lowercase()))
            .with_label(GltfAssetLabel::Scene(0).to_string()),
    );

    let _entity = world
        .spawn((
            Skin { scale },
            HealthBar { bar_type },
            Visibility::default(),
            WorldAssetRoot(skin_handle),
        ))
        .id();

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
