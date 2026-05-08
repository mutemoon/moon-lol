use std::collections::HashMap;

use bevy::prelude::*;
use league_core::extract::{
    EnumData, EnumUiPosition, UiElementEffectAnimationData, UiElementGroupButtonData,
    UiElementIconData, UiElementRegionData, UiPositionRect, UiPropertyLoadable, UiSceneData,
};
use league_loader::game::{Data, LeagueLoader};
use league_loader::prop_bin::LeagueWadLoaderTrait;
use league_utils::hash_bin;
use lol_base::ui::{
    LOLAtlasData, LOLEnumAnchor, LOLEnumData, LOLEnumUiPosition, LOLUiElementEffectAnimationData,
    LOLUiElementIconData, LOLUiFile, LOLUiPaths, LOLUiPositionRect,
};

use crate::extract::utils::write_to_file;

/// UI 元素提取结果
pub struct UiExtractResult {
    pub texture_names: Vec<String>, // 所有用到的 m_texture_name
}

/// Phase UI 1: 提取 UI 元素数据
pub fn extract_ui_data(
    loader: &LeagueLoader,
    bin_path: &str,
    assets: &mut LOLUiFile,
) -> UiExtractResult {
    let ui_prop_group = match loader.get_prop_group_by_paths(vec![bin_path]) {
        Ok(group) => group,
        Err(e) => {
            println!("[WARN] 无法加载 UI bin 文件 {}: {:?}", bin_path, e);
            return UiExtractResult {
                texture_names: Vec::new(),
            };
        }
    };

    // 获取所有 UiPropertyLoadable 获取真正的 prop 文件路径
    let ui_property_loadables: Vec<UiPropertyLoadable> =
        ui_prop_group.get_all_by_class::<UiPropertyLoadable>();

    // 通过 filepath_hash 加载真正的 UI 元素 bin 文件
    let mut all_ui_element_datas: Vec<UiElementIconData> = Vec::new();
    let mut all_ui_animation_datas: Vec<UiElementEffectAnimationData> = Vec::new();
    let mut all_ui_button_datas: Vec<UiElementGroupButtonData> = Vec::new();
    let mut all_ui_region_datas: Vec<UiElementRegionData> = Vec::new();
    let mut all_scene_datas: HashMap<u32, UiSceneData> = HashMap::new();

    for loadable in &ui_property_loadables {
        let filepath_hash = loadable.filepath_hash;

        match loader.get_prop_bin_by_hash(filepath_hash) {
            Ok(prop_file) => {
                for entry in prop_file.iter_class_hash_and_entry() {
                    if let Ok(ui_data) = league_property::from_entry::<UiElementIconData>(entry.1) {
                        all_ui_element_datas.push(ui_data);
                    }
                    if let Ok(anim_data) =
                        league_property::from_entry::<UiElementEffectAnimationData>(entry.1)
                    {
                        all_ui_animation_datas.push(anim_data);
                    }
                    if let Ok(button_data) =
                        league_property::from_entry::<UiElementGroupButtonData>(entry.1)
                    {
                        all_ui_button_datas.push(button_data);
                    }
                    if let Ok(region_data) =
                        league_property::from_entry::<UiElementRegionData>(entry.1)
                    {
                        all_ui_region_datas.push(region_data);
                    }
                }
                for entry in prop_file.iter_class_hash_and_entry() {
                    if let Ok(scene_data) = league_property::from_entry::<UiSceneData>(entry.1) {
                        let hash = hash_bin(&scene_data.name);
                        all_scene_datas.insert(hash, scene_data);
                    }
                }
            }
            Err(_) => {}
        }
    }

    let mut all_texture_names: Vec<String> = Vec::new();
    for ui_data in all_ui_element_datas {
        // 收集原始纹理路径用于提取
        if let Some(EnumData::AtlasData(atlas)) = &ui_data.texture_data {
            if !atlas.m_texture_name.is_empty() && !all_texture_names.contains(&atlas.m_texture_name)
            {
                all_texture_names.push(atlas.m_texture_name.clone());
            }
        }

        let lol_ui: LOLUiElementIconData = convert_ui_element_icon_data(&ui_data, &all_scene_datas);
        assets.add(lol_ui);
    }

    for anim_data in all_ui_animation_datas {
        // 收集原始纹理路径用于提取
        if let EnumData::AtlasData(atlas) = &anim_data.texture_data {
            if !atlas.m_texture_name.is_empty() && !all_texture_names.contains(&atlas.m_texture_name)
            {
                all_texture_names.push(atlas.m_texture_name.clone());
            }
        }

        let lol_anim = convert_ui_animation_data(&anim_data);
        assets.add_animation(lol_anim);
    }

    for button_data in all_ui_button_datas {
        let lol_button = convert_ui_button_data(&button_data);
        assets.add_button(lol_button);
    }

    for region_data in all_ui_region_datas {
        let hash = hash_bin(&region_data.name);
        let lol_region = convert_ui_region_data(&region_data);
        assets.add_region(lol_region, hash);
    }

    for (hash, scene_data) in &all_scene_datas {
        assets.scenes.insert(
            *hash,
            lol_base::ui::LOLUiSceneData {
                name: scene_data.name.clone(),
                enabled: scene_data.enabled.unwrap_or(false),
            },
        );
    }

    UiExtractResult {
        texture_names: all_texture_names,
    }
}

/// 根据 SceneData 决定可见性
/// 转换 UiElementIconData 到 lol_base 稳定类型
fn convert_ui_element_icon_data(
    ui: &UiElementIconData,
    _scene_datas: &HashMap<u32, UiSceneData>,
) -> LOLUiElementIconData {
    LOLUiElementIconData {
        name: ui.name.clone(),
        position: convert_ui_position(&ui.position),
        layer: ui.layer,
        texture_data: ui.texture_data.as_ref().and_then(convert_texture_data),
        enabled: ui.enabled.unwrap_or(false),
        scene: ui.scene,
    }
}

/// 转换 UiElementEffectAnimationData 到 lol_base 稳定类型
fn convert_ui_animation_data(
    anim: &UiElementEffectAnimationData,
) -> LOLUiElementEffectAnimationData {
    LOLUiElementEffectAnimationData {
        name: anim.name.clone(),
        position: convert_ui_position_rect(&anim.position),
        layer: anim.layer,
        texture_data: convert_texture_data(&anim.texture_data),
        frames_per_second: anim.frames_per_second,
        total_number_of_frames: anim.total_number_of_frames,
        number_of_frames_per_row_in_atlas: anim.number_of_frames_per_row_in_atlas,
        finish_behavior: anim.m_finish_behavior,
    }
}

/// 转换 UiElementGroupButtonData 到 lol_base 稳定类型
fn convert_ui_button_data(
    button: &UiElementGroupButtonData,
) -> lol_base::ui::LOLUiElementGroupButtonData {
    lol_base::ui::LOLUiElementGroupButtonData {
        name: button.name.clone(),
        is_enabled: button.is_enabled,
        hit_region_element: button.hit_region_element,
        elements: button.elements.clone(),
        clicked_state_elements: button.clicked_state_elements.as_ref().map(|v| {
            lol_base::ui::LOLUiElementGroupButtonState {
                display_element_list: v.display_element_list.clone(),
            }
        }),
        hover_state_elements: button.hover_state_elements.as_ref().map(|v| {
            lol_base::ui::LOLUiElementGroupButtonState {
                display_element_list: v.display_element_list.clone(),
            }
        }),
        default_state_elements: button.default_state_elements.as_ref().map(|v| {
            lol_base::ui::LOLUiElementGroupButtonState {
                display_element_list: v.display_element_list.clone(),
            }
        }),
    }
}

/// 转换 UiElementRegionData 到 lol_base 稳定类型
fn convert_ui_region_data(region: &UiElementRegionData) -> lol_base::ui::LOLUiElementRegionData {
    lol_base::ui::LOLUiElementRegionData {
        name: region.name.clone(),
        position: region.position.as_ref().map(convert_ui_position),
    }
}

/// 转换 EnumUiPosition 到 LOLEnumUiPosition
fn convert_ui_position(pos: &EnumUiPosition) -> LOLEnumUiPosition {
    match pos {
        EnumUiPosition::UiPositionRect(rect) => {
            LOLEnumUiPosition::UiPositionRect(convert_ui_position_rect(rect))
        }
        _ => LOLEnumUiPosition::UiPositionRect(LOLUiPositionRect {
            anchors: None,
            ui_rect: None,
        }),
    }
}

/// 转换 UiPositionRect 到 LOLUiPositionRect
fn convert_ui_position_rect(rect: &UiPositionRect) -> LOLUiPositionRect {
    LOLUiPositionRect {
        anchors: rect.anchors.as_ref().map(convert_anchor),
        ui_rect: rect
            .ui_rect
            .as_ref()
            .map(|r| lol_base::ui::LOLUiElementRect {
                position: r.position,
                size: r.size,
                source_resolution_height: r.source_resolution_height,
                source_resolution_width: r.source_resolution_width,
            }),
    }
}

/// 转换 anchors
fn convert_anchor(anchor: &league_core::extract::EnumAnchor) -> LOLEnumAnchor {
    match anchor {
        league_core::extract::EnumAnchor::AnchorSingle(single) => {
            LOLEnumAnchor::AnchorSingle(lol_base::ui::LOLAnchorSingle {
                anchor: single.anchor,
            })
        }
        _ => LOLEnumAnchor::AnchorSingle(lol_base::ui::LOLAnchorSingle { anchor: Vec2::ZERO }),
    }
}

/// 转换 texture data
fn convert_texture_data(data: &EnumData) -> Option<LOLEnumData> {
    match data {
        EnumData::AtlasData(atlas) => Some(LOLEnumData::AtlasData(LOLAtlasData {
            m_texture_name: crate::extract::utils::get_texture_path(&atlas.m_texture_name),
            m_texture_uv: atlas.m_texture_uv,
        })),
        _ => None,
    }
}

/// 一键提取 UI
pub fn extract_ui_all(game_path: &str) {
    let wad_files: Vec<&str> = vec!["DATA/FINAL/UI.wad.client"];
    let loader = LeagueLoader::from_relative_path(game_path, wad_files);
    let ui_paths = LOLUiPaths::default();

    let export_configs = vec![
        ("gameplay.playerframe.bin", ui_paths.player_frame_ron()),
        (
            "gameplay.lolfloatinginfobars.bin",
            ui_paths.floating_info_bars_ron(),
        ),
    ];

    for (bin_path, ron_path) in export_configs {
        let mut assets = LOLUiFile::default();
        let result = extract_ui_data(&loader, bin_path, &mut assets);

        // 序列化到 RON
        let ron_config = ron::ser::PrettyConfig::default();
        let serialized = ron::ser::to_string_pretty(&assets, ron_config).unwrap();
        write_to_file(&ron_path, &serialized);
        println!("[UI] 已导出 {} 到 {}", bin_path, ron_path);

        // 提取纹理图片
        for texture_name in &result.texture_names {
            crate::extract::utils::extract_texture(&loader, texture_name);
        }
    }
}
