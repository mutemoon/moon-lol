use std::collections::HashMap;

use bevy::prelude::*;
use league_core::extract::{
    AbilitiesUiData, BarTypeMap, EnumData, EnumUiPosition, FloatingInfoBarViewController,
    HealthBarData, HealthBarExtraBarsData, HealthBarFadeData, HealthBarTextData,
    HeroFloatingInfoBarData, HeroFloatingInfoBorderData, HudPlayerResourceBars,
    PlayerFrameViewController, PlayerPortraitUiData, SpellLevelUpUiData,
    SpellSlotDetailedUiDefinition, StructureFloatingInfoBarData, UiElementEffectAnimationData,
    UiElementEffectDesaturateData, UiElementEffectInstancedData, UiElementGroupButtonData,
    UiElementIconData, UiElementRegionData, UiElementTextData, UiLevelUp, UiPositionRect,
    UiPropertyLoadable, UiSceneData, UnitFloatingInfoBarData, Unk0x1e2d1428, Unk0xc3f95838,
};
use league_loader::game::{Data, LeagueLoader};
use league_loader::prop_bin::LeagueWadLoaderTrait;
use league_utils::hash_bin;
use lol_base::ui::{
    LOLAbilitiesUiData, LOLAtlasData, LOLAtlasData3SliceH, LOLAtlasData3SliceV, LOLAtlasData9Slice,
    LOLCooldownEffectUiData, LOLCooldownGemUiData, LOLEnumAnchor, LOLEnumData, LOLEnumUiPosition,
    LOLFloatingInfoBarViewController, LOLHeroFloatingInfoBarData, LOLHudPlayerResourceBars,
    LOLLooseUiTextureData, LOLLooseUiTextureData3SliceH, LOLLooseUiTextureData3SliceV,
    LOLLooseUiTextureData9Slice, LOLPlayerFrameViewController, LOLPlayerPortraitUiData,
    LOLSpellLevelUpUiData, LOLSpellSlotBuffTimerData, LOLSpellSlotDetailedUiDefinition,
    LOLStatPageCategoryData, LOLStatPageViewController, LOLStructureFloatingInfoBarData,
    LOLUiElementEffectAnimationData, LOLUiElementEffectDesaturateData,
    LOLUiElementEffectInstancedData, LOLUiElementIconData,
    LOLUiElementTextData, LOLUiFile, LOLUiLevelUp, LOLUiPaths, LOLUiPositionRect,
    LOLUnitFloatingInfoBarData,
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
    let mut all_ui_desaturate_datas: Vec<UiElementEffectDesaturateData> = Vec::new();
    let mut all_ui_instanced_datas: Vec<UiElementEffectInstancedData> = Vec::new();
    let mut all_ui_button_datas: Vec<UiElementGroupButtonData> = Vec::new();
    let mut all_ui_region_datas: Vec<UiElementRegionData> = Vec::new();
    let mut all_ui_text_datas: Vec<UiElementTextData> = Vec::new();
    let mut all_scene_datas: HashMap<u32, UiSceneData> = HashMap::new();

    for loadable in &ui_property_loadables {
        let filepath_hash = loadable.filepath_hash;

        match loader.get_prop_bin_by_hash(filepath_hash) {
            Ok(prop_file) => {
                let anim_hash = league_utils::type_name_to_hash("UiElementEffectAnimationData");
                let desaturate_hash =
                    league_utils::type_name_to_hash("UiElementEffectDesaturateData");
                let instanced_hash = league_utils::type_name_to_hash("UiElementEffectInstancedData");
                let button_hash = league_utils::type_name_to_hash("UiElementGroupButtonData");
                let icon_hash = league_utils::type_name_to_hash("UiElementIconData");
                let region_hash = league_utils::type_name_to_hash("UiElementRegionData");
                let text_hash = league_utils::type_name_to_hash("UiElementTextData");
                let scene_hash = league_utils::type_name_to_hash("UiSceneData");

                for entry in prop_file.iter_entry_by_class(anim_hash) {
                    if let Ok(anim_data) =
                        league_property::from_entry::<UiElementEffectAnimationData>(entry)
                    {
                        all_ui_animation_datas.push(anim_data);
                    }
                }
                for entry in prop_file.iter_entry_by_class(desaturate_hash) {
                    if let Ok(desaturate_data) =
                        league_property::from_entry::<UiElementEffectDesaturateData>(entry)
                    {
                        all_ui_desaturate_datas.push(desaturate_data);
                    }
                }
                for entry in prop_file.iter_entry_by_class(instanced_hash) {
                    if let Ok(instanced_data) =
                        league_property::from_entry::<UiElementEffectInstancedData>(entry)
                    {
                        all_ui_instanced_datas.push(instanced_data);
                    }
                }
                for entry in prop_file.iter_entry_by_class(button_hash) {
                    if let Ok(button_data) =
                        league_property::from_entry::<UiElementGroupButtonData>(entry)
                    {
                        all_ui_button_datas.push(button_data);
                    }
                }
                for entry in prop_file.iter_entry_by_class(icon_hash) {
                    if let Ok(ui_data) = league_property::from_entry::<UiElementIconData>(entry) {
                        all_ui_element_datas.push(ui_data);
                    }
                }
                for entry in prop_file.iter_entry_by_class(region_hash) {
                    if let Ok(region_data) =
                        league_property::from_entry::<UiElementRegionData>(entry)
                    {
                        all_ui_region_datas.push(region_data);
                    }
                }
                for entry in prop_file.iter_entry_by_class(text_hash) {
                    if let Ok(text_data) = league_property::from_entry::<UiElementTextData>(entry) {
                        all_ui_text_datas.push(text_data);
                    }
                }
                for entry in prop_file.iter_entry_by_class(scene_hash) {
                    if let Ok(scene_data) = league_property::from_entry::<UiSceneData>(entry) {
                        let hash = hash_bin(&scene_data.name);
                        all_scene_datas.insert(hash, scene_data);
                    }
                }
            }
            Err(_) => {}
        }
    }

    // 提取浮动信息条数据
    let view_controllers = ui_prop_group.get_all_by_class::<FloatingInfoBarViewController>();
    if let Some(vc) = view_controllers.first() {
        assets.set_floating_info_bar_view_controller(convert_floating_info_bar_view_controller(vc));
    }

    let unit_info_bars = ui_prop_group.get_all_by_class_with_hash::<UnitFloatingInfoBarData>();
    for (hash, data) in unit_info_bars {
        assets.add_unit_floating_info_bar(hash, convert_unit_floating_info_bar_data(&data));
    }

    let hero_info_bars = ui_prop_group.get_all_by_class_with_hash::<HeroFloatingInfoBarData>();
    for (hash, data) in hero_info_bars {
        assets.add_hero_floating_info_bar(hash, convert_hero_floating_info_bar_data(&data));
    }

    let structure_info_bars =
        ui_prop_group.get_all_by_class_with_hash::<StructureFloatingInfoBarData>();
    for (hash, data) in structure_info_bars {
        assets
            .add_structure_floating_info_bar(hash, convert_structure_floating_info_bar_data(&data));
    }

    let player_frame_view_controller =
        ui_prop_group.get_all_by_class::<PlayerFrameViewController>();
    if let Some(vc) = player_frame_view_controller.first() {
        assets.player_frame_view_controller = Some(convert_player_frame_view_controller(vc));
    }

    let mut all_texture_names: Vec<String> = Vec::new();
    for ui_data in all_ui_element_datas {
        // 收集原始纹理路径用于提取
        if let Some(texture_data) = &ui_data.texture_data {
            match texture_data {
                EnumData::AtlasData(atlas) => {
                    if !atlas.m_texture_name.is_empty()
                        && !all_texture_names.contains(&atlas.m_texture_name)
                    {
                        all_texture_names.push(atlas.m_texture_name.clone());
                    }
                }
                EnumData::AtlasData3SliceH(atlas) => {
                    if !atlas.m_texture_name.is_empty()
                        && !all_texture_names.contains(&atlas.m_texture_name)
                    {
                        all_texture_names.push(atlas.m_texture_name.clone());
                    }
                }
                EnumData::AtlasData3SliceV(atlas) => {
                    if !atlas.m_texture_name.is_empty()
                        && !all_texture_names.contains(&atlas.m_texture_name)
                    {
                        all_texture_names.push(atlas.m_texture_name.clone());
                    }
                }
                EnumData::AtlasData9Slice(atlas) => {
                    if !atlas.m_texture_name.is_empty()
                        && !all_texture_names.contains(&atlas.m_texture_name)
                    {
                        all_texture_names.push(atlas.m_texture_name.clone());
                    }
                }
                EnumData::LooseUiTextureData(loose) => {
                    if !loose.texture_name.is_empty()
                        && !all_texture_names.contains(&loose.texture_name)
                    {
                        all_texture_names.push(loose.texture_name.clone());
                    }
                }
                EnumData::LooseUiTextureData3SliceH(loose) => {
                    if !loose.texture_name.is_empty()
                        && !all_texture_names.contains(&loose.texture_name)
                    {
                        all_texture_names.push(loose.texture_name.clone());
                    }
                }
                EnumData::LooseUiTextureData3SliceV(loose) => {
                    if !loose.texture_name.is_empty()
                        && !all_texture_names.contains(&loose.texture_name)
                    {
                        all_texture_names.push(loose.texture_name.clone());
                    }
                }
                EnumData::LooseUiTextureData9Slice(loose) => {
                    if !loose.texture_name.is_empty()
                        && !all_texture_names.contains(&loose.texture_name)
                    {
                        all_texture_names.push(loose.texture_name.clone());
                    }
                }
                _ => panic!("UI资源类型 {:?} 渲染器暂不支持", ui_data.texture_data),
            }
        }

        let lol_ui: LOLUiElementIconData = convert_ui_element_icon_data(&ui_data, &all_scene_datas);
        assets.add(lol_ui);
    }

    for anim_data in all_ui_animation_datas {
        // 收集原始纹理路径用于提取
        match &anim_data.texture_data {
            EnumData::AtlasData(atlas) => {
                if !atlas.m_texture_name.is_empty()
                    && !all_texture_names.contains(&atlas.m_texture_name)
                {
                    all_texture_names.push(atlas.m_texture_name.clone());
                }
            }
            EnumData::AtlasData3SliceH(atlas) => {
                if !atlas.m_texture_name.is_empty()
                    && !all_texture_names.contains(&atlas.m_texture_name)
                {
                    all_texture_names.push(atlas.m_texture_name.clone());
                }
            }
            EnumData::AtlasData3SliceV(atlas) => {
                if !atlas.m_texture_name.is_empty()
                    && !all_texture_names.contains(&atlas.m_texture_name)
                {
                    all_texture_names.push(atlas.m_texture_name.clone());
                }
            }
            EnumData::AtlasData9Slice(atlas) => {
                if !atlas.m_texture_name.is_empty()
                    && !all_texture_names.contains(&atlas.m_texture_name)
                {
                    all_texture_names.push(atlas.m_texture_name.clone());
                }
            }
            EnumData::LooseUiTextureData(loose) => {
                if !loose.texture_name.is_empty()
                    && !all_texture_names.contains(&loose.texture_name)
                {
                    all_texture_names.push(loose.texture_name.clone());
                }
            }
            EnumData::LooseUiTextureData3SliceH(loose) => {
                if !loose.texture_name.is_empty()
                    && !all_texture_names.contains(&loose.texture_name)
                {
                    all_texture_names.push(loose.texture_name.clone());
                }
            }
            EnumData::LooseUiTextureData3SliceV(loose) => {
                if !loose.texture_name.is_empty()
                    && !all_texture_names.contains(&loose.texture_name)
                {
                    all_texture_names.push(loose.texture_name.clone());
                }
            }
            EnumData::LooseUiTextureData9Slice(loose) => {
                if !loose.texture_name.is_empty()
                    && !all_texture_names.contains(&loose.texture_name)
                {
                    all_texture_names.push(loose.texture_name.clone());
                }
            }
            _ => panic!("UI资源类型 {:?} 渲染器暂不支持", anim_data.texture_data),
        }

        let lol_anim = convert_ui_animation_data(&anim_data);
        assets.add_animation(lol_anim);
    }

    for desaturate_data in all_ui_desaturate_datas {
        // 收集原始纹理路径用于提取
        if let Some(texture_data) = &desaturate_data.texture_data {
            match texture_data {
                EnumData::AtlasData(atlas) => {
                    if !atlas.m_texture_name.is_empty()
                        && !all_texture_names.contains(&atlas.m_texture_name)
                    {
                        all_texture_names.push(atlas.m_texture_name.clone());
                    }
                }
                EnumData::AtlasData3SliceH(atlas) => {
                    if !atlas.m_texture_name.is_empty()
                        && !all_texture_names.contains(&atlas.m_texture_name)
                    {
                        all_texture_names.push(atlas.m_texture_name.clone());
                    }
                }
                EnumData::AtlasData3SliceV(atlas) => {
                    if !atlas.m_texture_name.is_empty()
                        && !all_texture_names.contains(&atlas.m_texture_name)
                    {
                        all_texture_names.push(atlas.m_texture_name.clone());
                    }
                }
                EnumData::AtlasData9Slice(atlas) => {
                    if !atlas.m_texture_name.is_empty()
                        && !all_texture_names.contains(&atlas.m_texture_name)
                    {
                        all_texture_names.push(atlas.m_texture_name.clone());
                    }
                }
                EnumData::LooseUiTextureData(loose) => {
                    if !loose.texture_name.is_empty()
                        && !all_texture_names.contains(&loose.texture_name)
                    {
                        all_texture_names.push(loose.texture_name.clone());
                    }
                }
                EnumData::LooseUiTextureData3SliceH(loose) => {
                    if !loose.texture_name.is_empty()
                        && !all_texture_names.contains(&loose.texture_name)
                    {
                        all_texture_names.push(loose.texture_name.clone());
                    }
                }
                EnumData::LooseUiTextureData3SliceV(loose) => {
                    if !loose.texture_name.is_empty()
                        && !all_texture_names.contains(&loose.texture_name)
                    {
                        all_texture_names.push(loose.texture_name.clone());
                    }
                }
                EnumData::LooseUiTextureData9Slice(loose) => {
                    if !loose.texture_name.is_empty()
                        && !all_texture_names.contains(&loose.texture_name)
                    {
                        all_texture_names.push(loose.texture_name.clone());
                    }
                }
                _ => panic!(
                    "UI资源类型 {:?} 渲染器暂不支持",
                    desaturate_data.texture_data
                ),
            }
        }

        let lol_desaturate = convert_ui_desaturate_data(&desaturate_data);
        assets.add_desaturate(lol_desaturate);
    }

    for instanced_data in all_ui_instanced_datas {
        // 收集原始纹理路径用于提取
        if let Some(atlas) = &instanced_data.texture_data {
            if !atlas.m_texture_name.is_empty()
                && !all_texture_names.contains(&atlas.m_texture_name)
            {
                all_texture_names.push(atlas.m_texture_name.clone());
            }
        }

        let lol_instanced = convert_ui_instanced_data(&instanced_data);
        assets.add_instanced(lol_instanced);
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

    for text_data in all_ui_text_datas {
        let lol_text = convert_ui_text_data(&text_data);
        assets.add_text(lol_text);
    }

    for (hash, scene_data) in &all_scene_datas {
        assets.scenes.insert(
            *hash,
            lol_base::ui::LOLUiSceneData {
                name: scene_data.name.clone(),
                enabled: scene_data.enabled.unwrap_or(false),
                parent_scene: scene_data.parent_scene,
                layer: scene_data.layer,
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
        scene: ui.scene.into(),
    }
}

/// 转换 UiElementEffectAnimationData 到 lol_base 稳定类型
fn convert_ui_animation_data(
    anim: &UiElementEffectAnimationData,
) -> LOLUiElementEffectAnimationData {
    LOLUiElementEffectAnimationData {
        name: anim.name.clone(),
        position: convert_ui_position(&anim.position),
        layer: anim.layer,
        texture_data: convert_texture_data(&anim.texture_data),
        frames_per_second: anim.frames_per_second,
        total_number_of_frames: anim.total_number_of_frames,
        number_of_frames_per_row_in_atlas: anim.number_of_frames_per_row_in_atlas,
        finish_behavior: anim.m_finish_behavior,
        scene: anim.scene,
        enabled: anim.enabled.unwrap_or(false),
    }
}

/// 转换 UiElementEffectDesaturateData 到 lol_base 稳定类型
fn convert_ui_desaturate_data(
    desaturate: &UiElementEffectDesaturateData,
) -> LOLUiElementEffectDesaturateData {
    LOLUiElementEffectDesaturateData {
        name: desaturate.name.clone(),
        position: convert_ui_position(&desaturate.position),
        layer: desaturate.layer,
        texture_data: desaturate
            .texture_data
            .as_ref()
            .and_then(convert_texture_data),
        scene: desaturate.scene,
        enabled: desaturate.enabled.unwrap_or(false),
        minimum_saturation: desaturate.minimum_saturation,
    }
}

/// 转换 UiElementEffectInstancedData 到 lol_base 稳定类型
fn convert_ui_instanced_data(
    instanced: &UiElementEffectInstancedData,
) -> LOLUiElementEffectInstancedData {
    LOLUiElementEffectInstancedData {
        name: instanced.name.clone(),
        position: convert_ui_position(&instanced.position),
        layer: instanced.layer,
        texture_data: instanced.texture_data.as_ref().map(convert_atlas_data),
        color: instanced.m_color,
        scene: instanced.scene,
        enabled: instanced.enabled.unwrap_or(false),
    }
}

fn convert_atlas_data(atlas: &league_core::extract::AtlasData) -> LOLAtlasData {
    LOLAtlasData {
        m_texture_name: crate::extract::utils::get_texture_path(&atlas.m_texture_name),
        m_texture_uv: atlas.m_texture_uv,
    }
}

/// 转换 UiElementGroupButtonData 到 lol_base 稳定类型
fn convert_ui_button_data(
    button: &UiElementGroupButtonData,
) -> lol_base::ui::LOLUiElementGroupButtonData {
    lol_base::ui::LOLUiElementGroupButtonData {
        name: button.name.clone(),
        is_enabled: button.is_enabled,
        hit_region_element: button.hit_region_element.into(),
        elements: button.elements.iter().map(|&h| h.into()).collect(),
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
        scene: button.scene.into(),
    }
}

/// 转换 UiElementRegionData 到 lol_base 稳定类型
fn convert_ui_region_data(region: &UiElementRegionData) -> lol_base::ui::LOLUiElementRegionData {
    lol_base::ui::LOLUiElementRegionData {
        name: region.name.clone(),
        position: region.position.as_ref().map(convert_ui_position),
        scene: region.scene,
    }
}

/// 转换 UiElementTextData 到 lol_base 稳定类型
fn convert_ui_text_data(text: &UiElementTextData) -> LOLUiElementTextData {
    LOLUiElementTextData {
        name: text.name.clone(),
        position: text
            .position
            .as_ref()
            .map(|rect| LOLEnumUiPosition::UiPositionRect(convert_ui_position_rect(rect)))
            .unwrap_or(LOLEnumUiPosition::UiPositionFullScreen),
        layer: text.layer,
        font_description: text.font_description,
        text_alignment_horizontal: text.text_alignment_horizontal,
        text_alignment_vertical: text.text_alignment_vertical,
        tra_key: text.tra_key.clone(),
        color: text.color,
        scene: text.scene,
        enabled: text.enabled.unwrap_or(false),
    }
}

/// 转换 EnumUiPosition 到 LOLEnumUiPosition
fn convert_ui_position(pos: &EnumUiPosition) -> LOLEnumUiPosition {
    match pos {
        EnumUiPosition::UiPositionRect(rect) => {
            LOLEnumUiPosition::UiPositionRect(convert_ui_position_rect(rect))
        }
        EnumUiPosition::UiPositionFullScreen => LOLEnumUiPosition::UiPositionFullScreen,
        EnumUiPosition::UiPositionPolygon(poly) => {
            LOLEnumUiPosition::UiPositionPolygon(lol_base::ui::LOLUiPositionPolygon {
                anchors: convert_anchor(&league_core::extract::EnumAnchor::AnchorSingle(
                    poly.anchors.clone(),
                )),
                ui_rect: Some(lol_base::ui::LOLUiElementRect {
                    position: poly.ui_rect.position,
                    size: poly.ui_rect.size,
                    source_resolution_height: poly.ui_rect.source_resolution_height,
                    source_resolution_width: poly.ui_rect.source_resolution_width,
                }),
                polygon_vertices: poly.polygon_vertices.clone(),
                disable_pixel_snapping_x: poly.disable_pixel_snapping_x,
                disable_pixel_snapping_y: poly.disable_pixel_snapping_y,
            })
        }
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
        disable_pixel_snapping_x: rect.disable_pixel_snapping_x,
        disable_pixel_snapping_y: rect.disable_pixel_snapping_y,
        disable_resolution_downscale: rect.disable_resolution_downscale,
        ignore_global_scale: rect.ignore_global_scale,
        ignore_safe_zone: rect.ignore_safe_zone,
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
        league_core::extract::EnumAnchor::AnchorDouble(dual) => {
            LOLEnumAnchor::AnchorDouble(lol_base::ui::LOLAnchorDouble {
                anchor_left: dual.anchor_left,
                anchor_right: dual.anchor_right,
            })
        }
        league_core::extract::EnumAnchor::Unk0xf090d2e7(_) => LOLEnumAnchor::Unk0xf090d2e7,
    }
}

/// 转换 texture data
fn convert_texture_data(data: &EnumData) -> Option<LOLEnumData> {
    match data {
        EnumData::AtlasData(atlas) => Some(LOLEnumData::AtlasData(LOLAtlasData {
            m_texture_name: crate::extract::utils::get_texture_path(&atlas.m_texture_name),
            m_texture_uv: atlas.m_texture_uv,
        })),
        EnumData::AtlasData3SliceH(atlas) => {
            Some(LOLEnumData::AtlasData3SliceH(LOLAtlasData3SliceH {
                m_texture_name: crate::extract::utils::get_texture_path(&atlas.m_texture_name),
                texture_us: atlas.texture_us,
                texture_vs: atlas.texture_vs,
            }))
        }
        EnumData::AtlasData3SliceV(atlas) => {
            Some(LOLEnumData::AtlasData3SliceV(LOLAtlasData3SliceV {
                m_texture_name: crate::extract::utils::get_texture_path(&atlas.m_texture_name),
                texture_us: atlas.texture_us,
                texture_vs: atlas.texture_vs,
            }))
        }
        EnumData::AtlasData9Slice(atlas) => {
            Some(LOLEnumData::AtlasData9Slice(LOLAtlasData9Slice {
                m_texture_name: crate::extract::utils::get_texture_path(&atlas.m_texture_name),
                texture_us: atlas.texture_us,
                texture_vs: atlas.texture_vs,
            }))
        }
        EnumData::LooseUiTextureData(loose) => {
            Some(LOLEnumData::LooseUiTextureData(LOLLooseUiTextureData {
                texture_name: crate::extract::utils::get_texture_path(&loose.texture_name),
            }))
        }
        EnumData::LooseUiTextureData3SliceH(loose) => Some(LOLEnumData::LooseUiTextureData3SliceH(
            LOLLooseUiTextureData3SliceH {
                texture_name: crate::extract::utils::get_texture_path(&loose.texture_name),
            },
        )),
        EnumData::LooseUiTextureData3SliceV(loose) => Some(LOLEnumData::LooseUiTextureData3SliceV(
            LOLLooseUiTextureData3SliceV {
                texture_name: crate::extract::utils::get_texture_path(&loose.texture_name),
            },
        )),
        EnumData::LooseUiTextureData9Slice(loose) => Some(LOLEnumData::LooseUiTextureData9Slice(
            LOLLooseUiTextureData9Slice {
                texture_name: crate::extract::utils::get_texture_path(&loose.texture_name),
            },
        )),
        EnumData::Unk0x5eaead1a => Some(LOLEnumData::Unk0x5eaead1a),
    }
}

fn convert_floating_info_bar_view_controller(
    vc: &FloatingInfoBarViewController,
) -> LOLFloatingInfoBarViewController {
    LOLFloatingInfoBarViewController {
        info_bar_style_source_map: vc.info_bar_style_source_map.clone(),
    }
}

fn convert_player_frame_view_controller(
    vc: &PlayerFrameViewController,
) -> LOLPlayerFrameViewController {
    LOLPlayerFrameViewController {
        abilities_ui_data: convert_abilities_ui_data(&vc.abilities_ui_data),
        portrait_ui_data: convert_player_portrait_ui_data(&vc.portrait_ui_data),
        resource_bars: convert_hud_player_resource_bars(&vc.resource_bars),
        level_up_display: convert_ui_level_up(&vc.level_up_display),
        root_scene: vc.root_scene.into(),
        stat_pages: vc
            .unk_0x1c05ee9d
            .iter()
            .map(convert_stat_page_view_controller)
            .collect(),
    }
}

fn convert_stat_page_view_controller(data: &Unk0xc3f95838) -> LOLStatPageViewController {
    LOLStatPageViewController {
        button: data.button.into(),
        stat_page_view_controller: data.stat_page_view_controller,
        categories: data.unk_0xcfb90a94.as_ref().map(|map| {
            map.iter()
                .map(|(&k, v)| (k, convert_stat_page_category_data(v)))
                .collect()
        }),
    }
}

fn convert_stat_page_category_data(data: &Unk0x1e2d1428) -> LOLStatPageCategoryData {
    LOLStatPageCategoryData {
        button: data.button.into(),
        stat_page_view_controller: data.stat_page_view_controller,
        is_selected: data.unk_0x5b5f63b5,
    }
}

fn convert_abilities_ui_data(data: &AbilitiesUiData) -> LOLAbilitiesUiData {
    LOLAbilitiesUiData {
        champion_spells: data
            .champion_spells
            .iter()
            .map(convert_spell_slot_detailed_ui_definition)
            .collect(),
        passive: convert_spell_slot_detailed_ui_definition(&data.passive),
        summoner_spells: data
            .summoner_spells
            .iter()
            .map(convert_spell_slot_detailed_ui_definition)
            .collect(),
    }
}

fn convert_spell_slot_detailed_ui_definition(
    data: &SpellSlotDetailedUiDefinition,
) -> LOLSpellSlotDetailedUiDefinition {
    LOLSpellSlotDetailedUiDefinition {
        ammo_fx: data.ammo_fx,
        ammo_text: data.ammo_text,
        border_disabled: data.border_disabled.into(),
        border_enabled: data.border_enabled.into(),
        buff_timer: data
            .buff_timer
            .as_ref()
            .map(convert_spell_slot_buff_timer_data),
        cooldown: data
            .cooldown_ui_data
            .as_ref()
            .and_then(|d| d.cooldown_text)
            .unwrap_or(0)
            .into(),
        cooldown_gem: data.cooldown_gem.as_ref().map(convert_cooldown_gem_ui_data),
        content_element: data.content_element.map(|h| h.into()),
        cost: data.cost,
        cost_bg: data.cost_bg,
        hotkey: data.hotkey,
        mouseover_region: data.mouseover_region.map(|h| h.into()),
        overlay_cced: data.overlay_cced.map(|h| h.into()),
        overlay_disabled: data.overlay_disabled.map(|h| h.into()),
        overlay_oom: data.overlay_oom.map(|h| h.into()),
        reset_flash_fx_attention: data.reset_flash_fx_attention,
        toggle_fx: data.toggle_fx,
    }
}

fn convert_spell_slot_buff_timer_data(
    data: &league_core::extract::SpellSlotBuffTimerData,
) -> LOLSpellSlotBuffTimerData {
    LOLSpellSlotBuffTimerData {
        timer_bar_bg: data.timer_bar_bg,
        timer_bar_fill: data.timer_bar_fill,
        timer_border_bg: data.timer_border_bg,
        timer_border_fx: data.timer_border_fx,
    }
}

fn convert_cooldown_gem_ui_data(
    data: &league_core::extract::CooldownGemUiData,
) -> LOLCooldownGemUiData {
    LOLCooldownGemUiData {
        ally_gem: data.ally_gem,
        cooldown_effects: convert_cooldown_effect_ui_data(&data.cooldown_effects),
        gem_background: data.gem_background,
    }
}

fn convert_cooldown_effect_ui_data(
    data: &league_core::extract::CooldownEffectUiData,
) -> LOLCooldownEffectUiData {
    LOLCooldownEffectUiData {
        cooldown_complete_effect: data.cooldown_complete_effect,
        cooldown_jump_effect: data.cooldown_jump_effect,
        cooldown_text: data.cooldown_text,
        radial_effect: data.radial_effect,
    }
}

fn convert_player_portrait_ui_data(data: &PlayerPortraitUiData) -> LOLPlayerPortraitUiData {
    LOLPlayerPortraitUiData {
        icon: data.icon.into(),
        level_text: data.level_text,
        respawn_timer: data.respawn_timer,
        tooltip_region: data.tooltip_region.into(),
    }
}

fn convert_hud_player_resource_bars(data: &HudPlayerResourceBars) -> LOLHudPlayerResourceBars {
    LOLHudPlayerResourceBars {
        experience_bar: data.experience_bar.into(),
        health_hit_region: data.health_hit_region.into(),
        health_regen_text: data.health_regen_text,
        par_hit_region: data.par_hit_region.into(),
        par_regen_text: data.par_regen_text,
    }
}

fn convert_ui_level_up(data: &UiLevelUp) -> LOLUiLevelUp {
    LOLUiLevelUp {
        buttons_scene: data.buttons_scene,
        fx_in_scene: data.fx_in_scene,
        spells: data
            .spells
            .as_ref()
            .map(|v| v.iter().map(convert_spell_level_up_ui_data).collect()),
        title: data.title,
    }
}

fn convert_spell_level_up_ui_data(data: &SpellLevelUpUiData) -> LOLSpellLevelUpUiData {
    LOLSpellLevelUpUiData {
        ability_fx_in: data.ability_fx_in,
        button_fx_in: data.button_fx_in,
        button_fx_out_selected: data.button_fx_out_selected,
        button_fx_out_unselected: data.button_fx_out_unselected,
        button_idle_glow_fx: data.button_idle_glow_fx,
        button_idle_sheen_fx: data.button_idle_sheen_fx,
        button_post_fx_in: data.button_post_fx_in,
        skill_up_button: data.skill_up_button.into(),
    }
}

fn convert_hero_floating_info_bar_data(
    data: &HeroFloatingInfoBarData,
) -> LOLHeroFloatingInfoBarData {
    LOLHeroFloatingInfoBarData {
        anchor: data.anchor.into(),
        borders: convert_hero_floating_info_border_data(&data.borders),
        health_bar: convert_health_bar_data(&data.health_bar),
    }
}

fn convert_hero_floating_info_border_type_data(
    data: &league_core::extract::HeroFloatingInfoBorderTypeData,
) -> lol_base::ui::LOLHeroFloatingInfoBorderTypeData {
    lol_base::ui::LOLHeroFloatingInfoBorderTypeData {
        border: data.border.into(),
        level_box_overlay_ally: data.level_box_overlay_ally.map(|v| v.into()),
        level_box_overlay_enemy: data.level_box_overlay_enemy.map(|v| v.into()),
        level_box_overlay_self: data.level_box_overlay_self.map(|v| v.into()),
        level_box_overlay_self_colorblind: data.level_box_overlay_self_colorblind.map(|v| v.into()),
    }
}

fn convert_hero_floating_info_border_defense_icon_data(
    data: &league_core::extract::HeroFloatingInfoBorderDefenseIconData,
) -> lol_base::ui::LOLHeroFloatingInfoBorderDefenseIconData {
    lol_base::ui::LOLHeroFloatingInfoBorderDefenseIconData {
        defense_down_icons: data.defense_down_icons.iter().map(convert_hero_floating_info_border_defense_icon_threshold_data).collect(),
        defense_up_icon: convert_hero_floating_info_border_defense_icon_threshold_data(&data.defense_up_icon),
        left_icon_region: data.left_icon_region.into(),
    }
}

fn convert_hero_floating_info_border_defense_icon_threshold_data(
    data: &league_core::extract::HeroFloatingInfoBorderDefenseIconThresholdData,
) -> lol_base::ui::LOLHeroFloatingInfoBorderDefenseIconThresholdData {
    lol_base::ui::LOLHeroFloatingInfoBorderDefenseIconThresholdData {
        armor_icon: data.armor_icon.into(),
        combo_icon: data.combo_icon.into(),
        defense_modifier_threshold: data.defense_modifier_threshold,
        magic_resist_icon: data.magic_resist_icon.into(),
    }
}

fn convert_hero_floating_info_border_data(
    data: &HeroFloatingInfoBorderData,
) -> lol_base::ui::LOLHeroFloatingInfoBorderData {
    lol_base::ui::LOLHeroFloatingInfoBorderData {
        additional_status_icons: data.additional_status_icons.as_ref().map(|map| {
            map.iter().map(|(&k, &v)| (k, v.into())).collect()
        }),
        default_border: convert_hero_floating_info_border_type_data(&data.default_border),
        defense_modifier_icons: data.defense_modifier_icons.as_ref().map(convert_hero_floating_info_border_defense_icon_data),
        executable_border: convert_hero_floating_info_border_type_data(&data.executable_border),
        has_attached_ally_border: data.has_attached_ally_border.as_ref().map(convert_hero_floating_info_border_type_data),
        invulnerable_border: data.invulnerable_border.as_ref().map(convert_hero_floating_info_border_type_data),
        level_text: data.level_text.into(),
        level_text_color_ally: data.level_text_color_ally,
        level_text_color_enemy: data.level_text_color_enemy,
        level_text_color_self_colorblind: data.level_text_color_self_colorblind,
        spell_shield_border: data.spell_shield_border.as_ref().map(convert_hero_floating_info_border_type_data),
    }
}

fn convert_unit_floating_info_bar_data(
    data: &UnitFloatingInfoBarData,
) -> LOLUnitFloatingInfoBarData {
    LOLUnitFloatingInfoBarData {
        anchor: data.anchor.into(),
        border: data.border.into(),
        health_bar: convert_health_bar_data(&data.health_bar),
    }
}

fn convert_structure_floating_info_bar_data(
    data: &StructureFloatingInfoBarData,
) -> LOLStructureFloatingInfoBarData {
    LOLStructureFloatingInfoBarData {
        anchor: data.anchor.into(),
        border: data.border.into(),
        health_bar: convert_health_bar_data(&data.health_bar),
    }
}

fn convert_health_bar_data(data: &HealthBarData) -> lol_base::ui::LOLHealthBarData {
    lol_base::ui::LOLHealthBarData {
        extra_bars: data
            .extra_bars
            .as_ref()
            .map(convert_health_bar_extra_bars_data),
        fade_data: data.fade_data.as_ref().map(convert_health_bar_fade_data),
        health_bar: convert_bar_type_map(&data.health_bar),
        incoming_damage_bar: data.incoming_damage_bar.map(|v| v.into()),
        max_hp_penalty_bar: data.max_hp_penalty_bar.map(|v| v.into()),
        max_hp_penalty_divider: data.max_hp_penalty_divider.map(|v| v.into()),
        text_data: data.text_data.as_ref().map(convert_health_bar_text_data),
        tick_style: Some(convert_health_bar_tick_style(&data.tick_style)),
    }
}

fn convert_health_bar_extra_bars_data(
    data: &HealthBarExtraBarsData,
) -> lol_base::ui::LOLHealthBarExtraBarsData {
    lol_base::ui::LOLHealthBarExtraBarsData {
        all_shield_bar: data.all_shield_bar.into(),
        champ_specific_bar: data.champ_specific_bar.as_ref().map(convert_bar_type_map),
        disguise_health_bar: data.disguise_health_bar.map(|v| v.into()),
        incoming_heal_bar: data.incoming_heal_bar.as_ref().map(convert_bar_type_map),
        magic_shield_bar: data.magic_shield_bar.map(|v| v.into()),
        physical_shield_bar: data.physical_shield_bar.map(|v| v.into()),
    }
}

fn convert_health_bar_fade_data(data: &HealthBarFadeData) -> lol_base::ui::LOLHealthBarFadeData {
    lol_base::ui::LOLHealthBarFadeData {
        fade_bar: convert_bar_type_map(&data.fade_bar),
        fade_speed: data.fade_speed,
    }
}

fn convert_health_bar_text_data(data: &HealthBarTextData) -> lol_base::ui::LOLHealthBarTextData {
    lol_base::ui::LOLHealthBarTextData {
        health_value_text: data.health_value_text.into(),
        include_max_health: data.include_max_health,
    }
}

fn convert_health_bar_tick_style(
    data: &league_core::extract::EnumHealthBarTickStyle,
) -> lol_base::ui::LOLEnumHealthBarTickStyle {
    match data {
        league_core::extract::EnumHealthBarTickStyle::HealthBarTickStyleHero(hero) => {
            lol_base::ui::LOLEnumHealthBarTickStyle::HealthBarTickStyleHero(
                lol_base::ui::LOLHealthBarTickStyleHero {
                    micro_tick: hero.micro_tick.into(),
                    micro_tick_per_standard_tick_data: hero
                        .micro_tick_per_standard_tick_data
                        .iter()
                        .map(|d| lol_base::ui::LOLMicroTicksPerStandardTickData {
                            micro_ticks_between: d.micro_ticks_between,
                            min_health: d.min_health,
                        })
                        .collect(),
                    standard_tick: hero.standard_tick.into(),
                },
            )
        }
        league_core::extract::EnumHealthBarTickStyle::HealthBarTickStyleTftCompanion(tft) => {
            lol_base::ui::LOLEnumHealthBarTickStyle::HealthBarTickStyleTftCompanion(
                lol_base::ui::LOLHealthBarTickStyleTftCompanion {
                    standard_tick: tft.standard_tick.into(),
                },
            )
        }
        league_core::extract::EnumHealthBarTickStyle::HealthBarTickStyleUnit(unit) => {
            lol_base::ui::LOLEnumHealthBarTickStyle::HealthBarTickStyleUnit(
                lol_base::ui::LOLHealthBarTickStyleUnit {
                    standard_tick: unit.standard_tick.into(),
                },
            )
        }
    }
}

fn convert_bar_type_map(data: &BarTypeMap) -> lol_base::ui::LOLBarTypeMap {
    lol_base::ui::LOLBarTypeMap {
        additional_bar_types: data.additional_bar_types.as_ref().map(|m| {
            m.iter()
                .map(|(&k, &v)| (k, v.into()))
                .collect::<std::collections::BTreeMap<_, _>>()
        }),
        default_bar: data.default_bar.into(),
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
