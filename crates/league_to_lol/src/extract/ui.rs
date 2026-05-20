use std::collections::{BTreeMap, HashMap};

use bevy::prelude::*;
use image::ImageFormat;
use league_core::extract::{
    AbilitiesUiData, AbilityResourceBarData, BarTypeMap, EnumData, EnumResourceMeter, EnumUiMetric,
    EnumUiPosition, FloatingInfoBarViewController, HealthBarData, HealthBarExtraBarsData,
    HealthBarFadeData, HealthBarTextData, HealthMeter, HeroFloatingInfoBarData,
    HeroFloatingInfoBorderData, HudAbilityResourceThresholdIndicator, HudPlayerResourceBars,
    LolGameStateViewController, PlayerFrameViewController, PlayerInventoryViewController,
    PlayerPortraitUiData, SpellLevelUpUiData, SpellPipsUiData, SpellRankPipsUiData,
    SpellSlotDetailedUiDefinition, StructureFloatingInfoBarData, UiElementEffectAnimationData,
    UiElementEffectDesaturateData, UiElementEffectFillPercentageData, UiElementEffectInstancedData,
    UiElementGroupButtonData, UiElementIconData, UiElementMeterSkin, UiElementRegionData,
    UiElementTextData, UiLevelUp,
    UiPositionRect, UiPropertyLoadable, UiSceneData, UnitFloatingInfoBarData, Unk0x1e2d1428,
    Unk0xc3f95838,
};
use league_loader::game::{Data, LeagueLoader};
use league_loader::prop_bin::LeagueWadLoaderTrait;
use league_utils::hash_bin;
use lol_base::ui::{
    LOLAbilitiesUiData, LOLAbilityResourceBarData, LOLAtlasData, LOLAtlasData3SliceH,
    LOLAtlasData3SliceV, LOLAtlasData9Slice, LOLCooldownEffectUiData, LOLCooldownGemUiData,
    LOLDrawAreaList, LOLEnumAnchor, LOLEnumData, LOLEnumResourceMeter, LOLEnumUiMetric,
    LOLEnumUiPosition, LOLFloatingInfoBarViewController, LOLHealthMeter,
    LOLHeroFloatingInfoBarData, LOLHudAbilityResourceThresholdIndicator, LOLHudPlayerResourceBars,
    LOLHudShopButton, LOLItemSlotDetailedUiData, LOLLolGameStateViewController,
    LOLLooseUiTextureData, LOLLooseUiTextureData3SliceH, LOLLooseUiTextureData3SliceV,
    LOLLooseUiTextureData9Slice, LOLPlayerFrameViewController, LOLPlayerInventoryViewController,
    LOLPlayerPortraitUiData, LOLResourceMeterGroupData, LOLResourceMeterIconData,
    LOLResourceMeterSkinData, LOLSpellLevelUpUiData, LOLSpellPipsUiData, LOLSpellRankPipsUiData,
    LOLSpellSlotBuffTimerData, LOLSpellSlotDetailedUiDefinition, LOLStatPageCategoryData,
    LOLStatPageViewController, LOLStructureFloatingInfoBarData, LOLUiClashTeam,
    LOLUiElementEffectAnimationData, LOLUiElementEffectDesaturateData,
    LOLUiElementEffectFillPercentageData, LOLUiElementEffectInstancedData,
    LOLUiElementIconData, LOLUiElementMeterSkin,
    LOLUiElementTextData, LOLUiFile, LOLUiLevelUp, LOLUiMetricClash, LOLUiMetricCreepScore,
    LOLUiMetricFps, LOLUiMetricGameTime, LOLUiMetricKda, LOLUiMetricLatencyText,
    LOLUiMetricTeamKills, LOLUiMetricTeamScoreMeters, LOLUiPaths, LOLUiPositionRect,
    LOLUnitFloatingInfoBarData, LOLUnk0x5ab5b20f, LOLUnk0x7a19656, LOLUnk0x767adcf7,
    LOLUnk0xa8c6f5f0, LOLUnk0xb8a49c96, LOLUnk0xb62c8675, LOLUnk0xe228ce4a, LOLUnk0xf43ad1ce,
};

use crate::extract::utils::write_to_file;

/// UI 元素提取结果
pub struct UiExtractResult {
    pub texture_names: Vec<String>, // 所有用到的 m_texture_name
    pub floating_info_bar_vc: Option<LOLFloatingInfoBarViewController>,
    pub player_frame_vc: Option<LOLPlayerFrameViewController>,
    pub player_inventory_vc: Option<LOLPlayerInventoryViewController>,
    pub lol_game_state_vc: Option<LOLLolGameStateViewController>,
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
                floating_info_bar_vc: None,
                player_frame_vc: None,
                player_inventory_vc: None,
                lol_game_state_vc: None,
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
    let mut all_ui_fill_percentage_datas: Vec<UiElementEffectFillPercentageData> = Vec::new();
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
                let instanced_hash =
                    league_utils::type_name_to_hash("UiElementEffectInstancedData");
                let fill_percentage_hash =
                    league_utils::type_name_to_hash("UiElementEffectFillPercentageData");
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
                for entry in prop_file.iter_entry_by_class(fill_percentage_hash) {
                    if let Ok(fill_percentage_data) =
                        league_property::from_entry::<UiElementEffectFillPercentageData>(entry)
                    {
                        all_ui_fill_percentage_datas.push(fill_percentage_data);
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
    let floating_info_bar_vc = view_controllers
        .first()
        .map(convert_floating_info_bar_view_controller);

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
    let player_frame_vc = player_frame_view_controller
        .first()
        .map(convert_player_frame_view_controller);

    let player_inventory_view_controller =
        ui_prop_group.get_all_by_class::<PlayerInventoryViewController>();
    let player_inventory_vc = player_inventory_view_controller
        .first()
        .map(convert_player_inventory_view_controller);

    let game_state_view_controllers =
        ui_prop_group.get_all_by_class::<LolGameStateViewController>();
    let lol_game_state_vc = game_state_view_controllers
        .first()
        .map(convert_lol_game_state_view_controller);

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

    for fill_percentage_data in all_ui_fill_percentage_datas {
        // 收集原始纹理路径用于提取
        let atlas = &fill_percentage_data.texture_data;
        if !atlas.m_texture_name.is_empty()
            && !all_texture_names.contains(&atlas.m_texture_name)
        {
            all_texture_names.push(atlas.m_texture_name.clone());
        }

        let lol_fill_percentage = convert_ui_fill_percentage_data(&fill_percentage_data);
        assets.add_fill_percentage(lol_fill_percentage);
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
        floating_info_bar_vc,
        player_frame_vc,
        player_inventory_vc,
        lol_game_state_vc,
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
        position: convert_ui_position_to_enum(&anim.position),
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
        position: convert_ui_position_to_enum(&desaturate.position),
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
        position: convert_ui_position_to_enum(&instanced.position),
        layer: instanced.layer,
        texture_data: instanced.texture_data.as_ref().map(convert_atlas_data),
        color: instanced.m_color,
        scene: instanced.scene,
        enabled: instanced.enabled.unwrap_or(false),
    }
}

/// 转换 UiElementEffectFillPercentageData 到 lol_base 稳定类型
fn convert_ui_fill_percentage_data(
    fill_percentage: &UiElementEffectFillPercentageData,
) -> LOLUiElementEffectFillPercentageData {
    LOLUiElementEffectFillPercentageData {
        name: fill_percentage.name.clone(),
        position: convert_ui_position_to_enum(&fill_percentage.position),
        layer: fill_percentage.layer,
        texture_data: convert_atlas_data(&fill_percentage.texture_data),
        scene: fill_percentage.scene,
        enabled: fill_percentage.enabled.unwrap_or(false),
        m_per_pixel_uvs_x: fill_percentage.m_per_pixel_uvs_x,
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

/// 转换 EnumUiPosition 到 LOLEnumUiPosition
fn convert_ui_position_to_enum(pos: &UiPositionRect) -> LOLEnumUiPosition {
    LOLEnumUiPosition::UiPositionRect(convert_ui_position_rect(pos))
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

fn convert_player_inventory_view_controller(
    vc: &PlayerInventoryViewController,
) -> LOLPlayerInventoryViewController {
    LOLPlayerInventoryViewController {
        item_slot_ui_data: vc
            .item_slot_ui_data
            .iter()
            .map(convert_item_slot_detailed_ui_data)
            .collect(),
        scene: vc.scene.into(),
        shop_button: convert_hud_shop_button(&vc.shop_button),
    }
}

fn convert_item_slot_detailed_ui_data(
    data: &league_core::extract::ItemSlotDetailedUiData,
) -> LOLItemSlotDetailedUiData {
    LOLItemSlotDetailedUiData {
        ammo_fx: data.ammo_fx,
        backdrop: data.backdrop.into(),
        border_default: data.border_default.into(),
        border_disabled: data.border_disabled.into(),
        border_enabled: data.border_enabled.into(),
        border_selected: data.border_selected.map(|v| v.into()),
        complete_fx: data.complete_fx,
        cooldown_effects: data
            .cooldown_effects
            .as_ref()
            .map(convert_cooldown_effect_ui_data),
        hit_area: data.hit_area.into(),
        hotkey_text: data.hotkey_text.into(),
        icon: data.icon.into(),
        major_active: data.major_active,
        overlay_disabled: data.overlay_disabled.into(),
        overlay_hover: data.overlay_hover.into(),
        overlay_loc: data.overlay_loc.into(),
        overlay_oom: data.overlay_oom.map(|v| v.into()),
        stack_text: data.stack_text.map(|v| v.into()),
        toggle_fx: Some(data.toggle_fx),
    }
}

fn convert_hud_shop_button(data: &league_core::extract::HudShopButton) -> LOLHudShopButton {
    LOLHudShopButton {
        inactive_icon: data.inactive_icon.into(),
        shop_button: data.shop_button.into(),
        text_link: data.text_link.into(),
        unk_0x34a1434b: data.unk_0x34a1434b,
        unk_0x40aa9d58: data.unk_0x40aa9d58,
        unk_0x697f8b6b: data.unk_0x697f8b6b,
        unk_0x778e26c6: data.unk_0x778e26c6,
        unk_0x7dffe581: data.unk_0x7dffe581.clone(),
        unk_0x8031b7a0: data.unk_0x8031b7a0,
        unk_0xb77375ae: data.unk_0xb77375ae,
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
        spell_rank_pips: convert_spell_rank_pips_ui_data(&data.spell_rank_pips),
        summoner_spells: data
            .summoner_spells
            .iter()
            .map(convert_spell_slot_detailed_ui_definition)
            .collect(),
    }
}

fn convert_spell_rank_pips_ui_data(data: &SpellRankPipsUiData) -> LOLSpellRankPipsUiData {
    LOLSpellRankPipsUiData {
        rank_pips: data
            .rank_pips
            .iter()
            .map(convert_spell_pips_ui_data)
            .collect(),
    }
}

fn convert_spell_pips_ui_data(data: &SpellPipsUiData) -> LOLSpellPipsUiData {
    LOLSpellPipsUiData {
        empty_pips: data.empty_pips.iter().map(|&h| h.into()).collect(),
        full_pips: data.full_pips.iter().map(|&h| h.into()).collect(),
        pip_target_rect: data.pip_target_rect.into(),
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
        ar_threshold_indicator: data
            .ar_threshold_indicator
            .as_ref()
            .map(convert_hud_ability_resource_threshold_indicator),
        experience_bar: data.experience_bar.into(),
        experience_hit_region: data.experience_hit_region.into(),
        health_animated_meter_skin: convert_ui_element_meter_skin(&data.health_animated_meter_skin),
        health_hit_region: data.health_hit_region.into(),
        health_meter: convert_health_meter(&data.health_meter),
        health_regen_text: data.health_regen_text.into(),
        par_hit_region: data.par_hit_region.into(),
        par_meter_data: convert_ability_resource_bar_data(&data.par_meter_data),
        par_regen_text: data.par_regen_text.into(),
        sar_text: data.sar_text.into(),
    }
}

fn convert_hud_ability_resource_threshold_indicator(
    data: &HudAbilityResourceThresholdIndicator,
) -> LOLHudAbilityResourceThresholdIndicator {
    LOLHudAbilityResourceThresholdIndicator {
        threshold_indicator_elements: data
            .threshold_indicator_elements
            .iter()
            .map(|&v| v.into())
            .collect(),
    }
}

fn convert_ui_element_meter_skin(data: &UiElementMeterSkin) -> LOLUiElementMeterSkin {
    LOLUiElementMeterSkin {
        bar_elements: data.bar_elements.iter().map(|&v| v.into()).collect(),
    }
}

fn convert_health_meter(data: &HealthMeter) -> LOLHealthMeter {
    LOLHealthMeter {
        fade_bar: data.fade_bar.into(),
        meter: data.meter.into(),
        value_text: data.value_text.into(),
    }
}

fn convert_ability_resource_bar_data(data: &AbilityResourceBarData) -> LOLAbilityResourceBarData {
    LOLAbilityResourceBarData {
        ability_resource_bars: convert_enum_resource_meter(&data.ability_resource_bars),
        backdrop: data.backdrop.map(|v| v.into()),
        standard_tick: data.standard_tick.map(|v| v.into()),
        use_animated_skins: data.use_animated_skins,
        value_text: data.value_text.map(|v| v.into()),
    }
}

fn convert_enum_resource_meter(data: &EnumResourceMeter) -> LOLEnumResourceMeter {
    match data {
        EnumResourceMeter::ResourceMeterGroupData(m) => {
            LOLEnumResourceMeter::ResourceMeterGroupData(LOLResourceMeterGroupData {
                meter: m.meter.into(),
                meter_skins: LOLResourceMeterSkinData {
                    additional_meter_skins: m
                        .meter_skins
                        .additional_meter_skins
                        .iter()
                        .map(|(&k, v)| (k, convert_ui_element_meter_skin(v)))
                        .collect(),
                    default_meter_skin: convert_ui_element_meter_skin(
                        &m.meter_skins.default_meter_skin,
                    ),
                },
            })
        }
        EnumResourceMeter::ResourceMeterIconData(m) => {
            LOLEnumResourceMeter::ResourceMeterIconData(LOLResourceMeterIconData {
                additional_bar_types: m
                    .additional_bar_types
                    .as_ref()
                    .map(|map| map.iter().map(|(&k, &v)| (k, v.into())).collect()),
                default_bar: m.default_bar.into(),
            })
        }
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
        defense_down_icons: data
            .defense_down_icons
            .iter()
            .map(convert_hero_floating_info_border_defense_icon_threshold_data)
            .collect(),
        defense_up_icon: convert_hero_floating_info_border_defense_icon_threshold_data(
            &data.defense_up_icon,
        ),
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
        additional_status_icons: data
            .additional_status_icons
            .as_ref()
            .map(|map| map.iter().map(|(&k, &v)| (k, v.into())).collect()),
        default_border: convert_hero_floating_info_border_type_data(&data.default_border),
        defense_modifier_icons: data
            .defense_modifier_icons
            .as_ref()
            .map(convert_hero_floating_info_border_defense_icon_data),
        executable_border: convert_hero_floating_info_border_type_data(&data.executable_border),
        has_attached_ally_border: data
            .has_attached_ally_border
            .as_ref()
            .map(convert_hero_floating_info_border_type_data),
        invulnerable_border: data
            .invulnerable_border
            .as_ref()
            .map(convert_hero_floating_info_border_type_data),
        level_text: data.level_text.into(),
        level_text_color_ally: data.level_text_color_ally,
        level_text_color_enemy: data.level_text_color_enemy,
        level_text_color_self_colorblind: data.level_text_color_self_colorblind,
        spell_shield_border: data
            .spell_shield_border
            .as_ref()
            .map(convert_hero_floating_info_border_type_data),
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
            "gameplay.playerinventory.bin",
            ui_paths.player_inventory_ron(),
        ),
        (
            "gameplay.playeraugments.bin",
            ui_paths.player_augments_ron(),
        ),
        ("gameplay.playermute.bin", ui_paths.player_mute_ron()),
        ("gameplay.playerperks.bin", ui_paths.player_perks_ron()),
        ("gameplay.playerreport.bin", ui_paths.player_report_ron()),
        ("gameplay.playerstats.bin", ui_paths.player_stats_ron()),
        (
            "gameplay.playerstatstones.bin",
            ui_paths.player_statstones_ron(),
        ),
        (
            "gameplay.lolfloatinginfobars.bin",
            ui_paths.floating_info_bars_ron(),
        ),
        ("gameplay.lolgameheader.bin", ui_paths.lol_game_header_ron()),
    ];

    let mut player_frame_vc = None;
    let mut player_inventory_vc = None;
    let mut lol_game_state_vc = None;
    let mut floating_info_bar_vc = None;

    for (bin_path, ron_path) in export_configs {
        let mut assets = LOLUiFile::default();
        let result = extract_ui_data(&loader, bin_path, &mut assets);

        // 收集控制器数据
        if let Some(vc) = result.player_frame_vc.clone() {
            player_frame_vc = Some(vc);
        }
        if let Some(vc) = result.player_inventory_vc.clone() {
            player_inventory_vc = Some(vc);
        }
        if let Some(vc) = result.lol_game_state_vc.clone() {
            lol_game_state_vc = Some(vc);
        }
        if let Some(vc) = result.floating_info_bar_vc.clone() {
            floating_info_bar_vc = Some(vc);
        }

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

    // 将所有视图控制器导出到统一的 ui.ron 场景文件
    let mut resources = std::collections::BTreeMap::new();
    let ron_config = ron::ser::PrettyConfig::default();

    if let Some(vc) = &player_frame_vc {
        let serialized = ron::ser::to_string_pretty(vc, ron_config.clone()).unwrap();
        resources.insert("lol_base::ui::LOLPlayerFrameViewController".to_string(), serialized);
    }
    if let Some(vc) = &player_inventory_vc {
        let serialized = ron::ser::to_string_pretty(vc, ron_config.clone()).unwrap();
        resources.insert("lol_base::ui::LOLPlayerInventoryViewController".to_string(), serialized);
    }
    if let Some(vc) = &lol_game_state_vc {
        let serialized = ron::ser::to_string_pretty(vc, ron_config.clone()).unwrap();
        resources.insert("lol_base::ui::LOLLolGameStateViewController".to_string(), serialized);
    }
    if let Some(vc) = &floating_info_bar_vc {
        let serialized = ron::ser::to_string_pretty(vc, ron_config.clone()).unwrap();
        resources.insert("lol_base::ui::LOLFloatingInfoBarViewController".to_string(), serialized);
    }

    let mut scene_str = String::new();
    scene_str.push_str("(\n  resources: {\n");
    for (type_name, value) in &resources {
        scene_str.push_str(&format!("    \"{}\": {},\n", type_name, value));
    }
    scene_str.push_str("  },\n  entities: {},\n)\n");

    let ui_ron_path = ui_paths.ui_ron();
    write_to_file(&ui_ron_path, &scene_str);
    println!("[UI] 已成功整合并导出所有核心 UI 控制器到 {}", ui_ron_path);

    // 额外提取 Cursors 资源 (ASSETS/UX/Cursors/hand1.tga)
    let cursor_tga_path = "ASSETS/UX/Cursors/hand1.tga";
    if let Ok(buf) = loader.get_wad_entry_buffer_by_path(cursor_tga_path) {
        write_to_file(cursor_tga_path, &buf);
        println!("[UI] 已提取 Cursors: {}", cursor_tga_path);

        // 如果可能，同时转换为 PNG 以便其他用途/兼容性
        if let Ok(img) = image::load_from_memory(&buf) {
            let mut png_buf = std::io::Cursor::new(Vec::new());
            if img.write_to(&mut png_buf, ImageFormat::Png).is_ok() {
                let cursor_png_path = "ASSETS/UX/Cursors/hand1.png";
                write_to_file(cursor_png_path, png_buf.into_inner());
                println!("[UI] 已将 Cursors 转换为 PNG 并保存: {}", cursor_png_path);
            }
        }
    } else {
        println!("[WARN] WAD 中未找到 Cursors 资源: {}", cursor_tga_path);
    }
}

fn convert_lol_game_state_view_controller(
    vc: &LolGameStateViewController,
) -> LOLLolGameStateViewController {
    LOLLolGameStateViewController {
        base_loadable: vc.base_loadable,
        draw_area_list: vc.draw_area_list.as_ref().map(|d| LOLDrawAreaList {
            draw_regions: d.draw_regions.iter().map(|&h| h.into()).collect(),
        }),
        metrics: vc.metrics.iter().map(convert_enum_ui_metric).collect(),
        path_hash_to_self: vc.path_hash_to_self,
        scene: vc.scene.into(),
    }
}

fn convert_enum_ui_metric(metric: &EnumUiMetric) -> LOLEnumUiMetric {
    match metric {
        EnumUiMetric::UiMetricClash(c) => LOLEnumUiMetric::UiMetricClash(LOLUiMetricClash {
            clash_frame: c.clash_frame.into(),
            clash_frame_mirror: c.clash_frame_mirror.into(),
            clash_round_icon: c.clash_round_icon.into(),
            clash_round_text: c.clash_round_text.into(),
            device_ux: c.device_ux,
            team1: LOLUiClashTeam {
                logo_icon: c.team1.logo_icon.into(),
                tag_text: c.team1.tag_text.into(),
            },
            team2: LOLUiClashTeam {
                logo_icon: c.team2.logo_icon.into(),
                tag_text: c.team2.tag_text.into(),
            },
        }),
        EnumUiMetric::UiMetricCreepScore(c) => {
            LOLEnumUiMetric::UiMetricCreepScore(LOLUiMetricCreepScore {
                device_ux: c.device_ux,
                icon: c.icon.into(),
                text: c.text.into(),
            })
        }
        EnumUiMetric::UiMetricFps(f) => LOLEnumUiMetric::UiMetricFps(LOLUiMetricFps {
            device_ux: f.device_ux,
            fps_text: f.fps_text.into(),
        }),
        EnumUiMetric::UiMetricGameTime(gt) => {
            LOLEnumUiMetric::UiMetricGameTime(LOLUiMetricGameTime {
                device_ux: gt.device_ux,
                time_text: gt.time_text.into(),
            })
        }
        EnumUiMetric::UiMetricKda(k) => LOLEnumUiMetric::UiMetricKda(LOLUiMetricKda {
            device_ux: k.device_ux,
            icon: k.icon.into(),
            text: k.text.into(),
        }),
        EnumUiMetric::UiMetricLatencyText(l) => {
            LOLEnumUiMetric::UiMetricLatencyText(LOLUiMetricLatencyText {
                device_ux: l.device_ux,
                latency_text: l.latency_text.into(),
            })
        }
        EnumUiMetric::UiMetricTeamKills(tk) => {
            LOLEnumUiMetric::UiMetricTeamKills(LOLUiMetricTeamKills {
                device_ux: tk.device_ux,
                team1_kill_text: tk.team1_kill_text.into(),
                team2_kill_text: tk.team2_kill_text.into(),
                team_kills_icon: tk.team_kills_icon.into(),
            })
        }
        EnumUiMetric::UiMetricTeamScoreMeters(tsm) => {
            LOLEnumUiMetric::UiMetricTeamScoreMeters(LOLUiMetricTeamScoreMeters {
                device_ux: tsm.device_ux,
                frame: tsm.frame.into(),
                team1_meter: tsm.team1_meter.into(),
                team1_meter_blue_skin: tsm.team1_meter_blue_skin.into(),
                team1_meter_red_skin: tsm.team1_meter_red_skin.into(),
                team2_meter: tsm.team2_meter.into(),
                team2_meter_blue_skin: tsm.team2_meter_blue_skin.into(),
                team2_meter_red_skin: tsm.team2_meter_red_skin.into(),
            })
        }
        EnumUiMetric::Unk0x5ab5b20f(u) => LOLEnumUiMetric::Unk0x5ab5b20f(LOLUnk0x5ab5b20f {
            device_ux: u.device_ux,
            time_text: u.time_text.into(),
            unk_0xadbcc5ee: u.unk_0xadbcc5ee.into(),
        }),
        EnumUiMetric::Unk0x767adcf7(u) => LOLEnumUiMetric::Unk0x767adcf7(LOLUnk0x767adcf7 {
            device_ux: u.device_ux,
            frame: u.frame.into(),
            time_text: u.time_text.into(),
        }),
        EnumUiMetric::Unk0xb62c8675(u) => LOLEnumUiMetric::Unk0xb62c8675(LOLUnk0xb62c8675 {
            base_loadable: u.base_loadable,
            crown_icons: LOLUnk0xa8c6f5f0 {
                unk_0x1793d323: u.crown_icons.unk_0x1793d323.into(),
                unk_0x4297f4f9: u.crown_icons.unk_0x4297f4f9.into(),
                unk_0x5329572e: u.crown_icons.unk_0x5329572e.into(),
                unk_0xb80015ba: u.crown_icons.unk_0xb80015ba.into(),
            },
            details_panel: LOLUnk0x7a19656 {
                detail_panel: u.details_panel.detail_panel.into(),
                detail_text_t1: u.details_panel.detail_text_t1.into(),
                detail_text_t2: u.details_panel.detail_text_t2.into(),
                timer_panel: u.details_panel.timer_panel.into(),
                timer_text: u.details_panel.timer_text.into(),
                unk_0x6188e7b7: u.details_panel.unk_0x6188e7b7.into(),
            },
            device_ux: u.device_ux,
            meters_panel: LOLUnk0xf43ad1ce {
                frame: u.meters_panel.frame.into(),
                icon_shadow_t1: u.meters_panel.icon_shadow_t1.into(),
                icon_shadow_t2: u.meters_panel.icon_shadow_t2.into(),
                team1_meter: LOLUnk0xb8a49c96 {
                    blue_skin: u.meters_panel.team1_meter.blue_skin.into(),
                    meter: u.meters_panel.team1_meter.meter.into(),
                    red_skin: u.meters_panel.team1_meter.red_skin.into(),
                },
                team2_meter: LOLUnk0xb8a49c96 {
                    blue_skin: u.meters_panel.team2_meter.blue_skin.into(),
                    meter: u.meters_panel.team2_meter.meter.into(),
                    red_skin: u.meters_panel.team2_meter.red_skin.into(),
                },
            },
            scene: u.scene,
            soraka_icons: LOLUnk0xa8c6f5f0 {
                unk_0x1793d323: u.soraka_icons.unk_0x1793d323.into(),
                unk_0x4297f4f9: u.soraka_icons.unk_0x4297f4f9.into(),
                unk_0x5329572e: u.soraka_icons.unk_0x5329572e.into(),
                unk_0xb80015ba: u.soraka_icons.unk_0xb80015ba.into(),
            },
            tower_icons: LOLUnk0xa8c6f5f0 {
                unk_0x1793d323: u.tower_icons.unk_0x1793d323.into(),
                unk_0x4297f4f9: u.tower_icons.unk_0x4297f4f9.into(),
                unk_0x5329572e: u.tower_icons.unk_0x5329572e.into(),
                unk_0xb80015ba: u.tower_icons.unk_0xb80015ba.into(),
            },
            unk_0x462800b7: LOLUnk0xa8c6f5f0 {
                unk_0x1793d323: u.unk_0x462800b7.unk_0x1793d323.into(),
                unk_0x4297f4f9: u.unk_0x462800b7.unk_0x4297f4f9.into(),
                unk_0x5329572e: u.unk_0x462800b7.unk_0x5329572e.into(),
                unk_0xb80015ba: u.unk_0x462800b7.unk_0xb80015ba.into(),
            },
            unk_0xb057cf4b: u.unk_0xb057cf4b,
        }),
        EnumUiMetric::Unk0xe228ce4a(u) => LOLEnumUiMetric::Unk0xe228ce4a(LOLUnk0xe228ce4a {
            device_ux: u.device_ux,
            frame: u.frame.into(),
            team1_text: u.team1_text.into(),
            team2_text: u.team2_text.into(),
            unk_0x3a568777: u.unk_0x3a568777.clone(),
        }),
    }
}
