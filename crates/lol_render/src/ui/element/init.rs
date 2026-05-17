use std::collections::HashMap;

use bevy::prelude::*;
use lol_base::hash_key::LoadHashKeyTrait;
use lol_base::ui::{
    LOLHeroFloatingInfoBarData, LOLStructureFloatingInfoBarData, LOLUiElementEffectAnimationData,
    LOLUiElementEffectDesaturateData, LOLUiElementEffectInstancedData, LOLUiElementGroupButtonData,
    LOLUiElementIconData, LOLUiElementRegionData, LOLUiElementTextData, LOLUiFile, LOLUiPaths,
    LOLUiSceneData, LOLUnitFloatingInfoBarData,
};
use lol_base::ui_components::{UIButton, UIElement};

use super::{UIElementEntity, UIState};
use crate::ui::text::UiTextState;

pub type IconAssets = Assets<LOLUiElementIconData>;
pub type AnimAssets = Assets<LOLUiElementEffectAnimationData>;
pub type DesaturateAssets = Assets<LOLUiElementEffectDesaturateData>;
pub type ButtonAssets = Assets<LOLUiElementGroupButtonData>;
pub type RegionAssets = Assets<LOLUiElementRegionData>;
pub type TextAssets = Assets<LOLUiElementTextData>;
pub type InstancedAssets = Assets<LOLUiElementEffectInstancedData>;
pub type SceneAssets = Assets<LOLUiSceneData>;

pub fn startup_load_ui_data(
    mut commands: Commands,
    mut res_ui_element_entity: ResMut<UIElementEntity>,
    mut icon_assets: ResMut<IconAssets>,
    mut anim_assets: ResMut<AnimAssets>,
    mut desaturate_assets: ResMut<DesaturateAssets>,
    mut button_assets: ResMut<ButtonAssets>,
    mut region_assets: ResMut<RegionAssets>,
    mut text_assets: ResMut<TextAssets>,
    mut instanced_assets: ResMut<InstancedAssets>,
    mut scene_assets: ResMut<SceneAssets>,
    mut unit_floating_info_bar_assets: ResMut<Assets<LOLUnitFloatingInfoBarData>>,
    mut hero_floating_info_bar_assets: ResMut<Assets<LOLHeroFloatingInfoBarData>>,
    mut structure_floating_info_bar_assets: ResMut<Assets<LOLStructureFloatingInfoBarData>>,
) {
    let ui_paths = LOLUiPaths::default();
    let ron_paths = vec![
        ui_paths.player_frame_ron(),
        ui_paths.floating_info_bars_ron(),
    ];

    let mut all_ui_files = Vec::new();

    for path in ron_paths {
        let Ok(content) = std::fs::read_to_string(format!("assets/{}", path)) else {
            warn!("无法读取 UI 数据文件: {}", path);
            continue;
        };

        let Ok(data) = ron::from_str::<LOLUiFile>(&content) else {
            warn!("无法解析 UI 数据文件: {}", path);
            continue;
        };
        all_ui_files.push(data);
    }

    let mut combined_scenes = HashMap::new();
    let mut combined_elements = HashMap::new();
    let mut combined_buttons = HashMap::new();
    let mut combined_animations = HashMap::new();
    let mut combined_desaturates = HashMap::new();
    let mut combined_regions = HashMap::new();
    let mut combined_texts = HashMap::new();
    let mut combined_instanceds = HashMap::new();

    // 第一阶段：创建所有 Asset 并收集数据
    for data in &all_ui_files {
        for (hash, scene_data) in &data.scenes {
            scene_assets.add_hash(*hash, scene_data.clone());
            combined_scenes.insert(*hash, scene_data.clone());
        }

        for (hash, region_data) in &data.region_elements {
            region_assets.add_hash(*hash, region_data.clone());
            combined_regions.insert(*hash, region_data.clone());
        }

        for (hash, icon_data) in &data.elements {
            icon_assets.add_hash(*hash, icon_data.clone());
            combined_elements.insert(*hash, icon_data.clone());
        }

        for (hash, button_data) in &data.button_elements {
            button_assets.add_hash(*hash, button_data.clone());
            combined_buttons.insert(*hash, button_data.clone());
        }

        for (hash, text_data) in &data.text_elements {
            text_assets.add_hash(*hash, text_data.clone());
            combined_texts.insert(*hash, text_data.clone());
        }

        for (hash, anim_data) in &data.animation_elements {
            anim_assets.add_hash(*hash, anim_data.clone());
            combined_animations.insert(*hash, anim_data.clone());
        }

        for (hash, desaturate_data) in &data.desaturate_elements {
            desaturate_assets.add_hash(*hash, desaturate_data.clone());
            combined_desaturates.insert(*hash, desaturate_data.clone());
        }

        for (hash, instanced_data) in &data.instanced_elements {
            instanced_assets.add_hash(*hash, instanced_data.clone());
            combined_instanceds.insert(*hash, instanced_data.clone());
        }

        for (hash, bar_data) in &data.unit_floating_info_bars {
            unit_floating_info_bar_assets.add_hash(*hash, bar_data.clone());
        }

        for (hash, bar_data) in &data.hero_floating_info_bars {
            hero_floating_info_bar_assets.add_hash(*hash, bar_data.clone());
        }

        for (hash, bar_data) in &data.structure_floating_info_bars {
            structure_floating_info_bar_assets.add_hash(*hash, bar_data.clone());
        }

        if let Some(vc) = &data.floating_info_bar_view_controller {
            commands.insert_resource(vc.clone());
        }

        if let Some(vc) = &data.player_frame_view_controller {
            commands.insert_resource(vc.clone());
        }
    }

    // 1. 创建场景实体
    for (hash, scene_data) in &combined_scenes {
        let entity = commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    ..default()
                },
                if scene_data.enabled {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                },
                Name::new(format!("Scene: {}", scene_data.name)),
                ZIndex(scene_data.layer.unwrap_or(0) as i32),
            ))
            .id();
        res_ui_element_entity.add(*hash, entity);
    }

    // 2. 链接场景父子关系
    for (hash, scene_data) in &combined_scenes {
        if let Some(parent_hash) = scene_data.parent_scene {
            let parent_entity = res_ui_element_entity.get(parent_hash);
            let entity = res_ui_element_entity.get(*hash);
            commands.entity(parent_entity).add_child(entity);
        }
    }

    // 3. 创建 UI 元素实体 (Icon) 并挂载到场景
    for (hash, icon_data) in &combined_elements {
        let entity = commands
            .spawn((
                UIElement::Icon(hash.into()),
                if icon_data.enabled {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                },
                Name::new(format!("Icon: {}", icon_data.name)),
            ))
            .id();

        let scene_entity = res_ui_element_entity.get(icon_data.scene.0.0);
        commands.entity(scene_entity).add_child(entity);
        res_ui_element_entity.add(*hash, entity);
    }

    // 4. 创建 Button 实体并挂载到场景
    for (hash, button_data) in &combined_buttons {
        let entity = commands
            .spawn((
                UIButton(hash.into()),
                if button_data.is_enabled.unwrap_or(false) {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                },
                Name::new(format!("Button: {}", button_data.name)),
            ))
            .id();

        let scene_entity = res_ui_element_entity.get(button_data.scene.0.0);
        commands.entity(scene_entity).add_child(entity);
        res_ui_element_entity.add(*hash, entity);
    }

    // 5. 创建 Animation 实体并挂载到场景
    for (hash, anim_data) in &combined_animations {
        let entity = commands
            .spawn((
                UIElement::Animation(hash.into()),
                crate::ui::animation::UiAnimationState {
                    handle: hash.into(),
                    current_frame: 0,
                    timer: 0.0,
                },
                if anim_data.enabled {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                },
                Name::new(format!("Anim: {}", anim_data.name)),
            ))
            .id();

        let scene_entity = res_ui_element_entity.get(anim_data.scene);
        commands.entity(scene_entity).add_child(entity);
        res_ui_element_entity.add(*hash, entity);
    }

    // 6. 创建 Desaturate 实体并挂载到场景
    for (hash, desaturate_data) in &combined_desaturates {
        let entity = commands
            .spawn((
                UIElement::Desaturate(hash.into()),
                if desaturate_data.enabled {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                },
                Name::new(format!("Desaturate: {}", desaturate_data.name)),
            ))
            .id();

        let scene_entity = res_ui_element_entity.get(desaturate_data.scene);
        commands.entity(scene_entity).add_child(entity);
        res_ui_element_entity.add(*hash, entity);
    }

    // 7. 创建 Region 实体并挂载到场景
    for (hash, region_data) in &combined_regions {
        let entity = commands
            .spawn((
                UIElement::Region(hash.into()),
                Name::new(format!("Region: {}", region_data.name)),
            ))
            .id();

        let scene_entity = res_ui_element_entity.get(region_data.scene);
        commands.entity(scene_entity).add_child(entity);
        res_ui_element_entity.add(*hash, entity);
    }

    // 8. 创建 Text 实体并挂载到场景
    for (hash, text_data) in &combined_texts {
        let entity = commands
            .spawn((
                UIElement::Text(hash.into()),
                if text_data.enabled {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                },
                Name::new(format!("Text: {}", text_data.name)),
                UiTextState {
                    text: "10".to_string(),
                },
            ))
            .id();

        let scene_entity = res_ui_element_entity.get(text_data.scene);
        commands.entity(scene_entity).add_child(entity);
        res_ui_element_entity.add(*hash, entity);
    }

    info!(
        "UI 元素初始化完成，一共 {} 个实体",
        res_ui_element_entity.map.len()
    );

    // save_ui_tree_to_json(
    //     &combined_scenes,
    //     &combined_elements,
    //     &combined_buttons,
    //     &combined_animations,
    //     &combined_regions,
    //     &combined_texts,
    //     &combined_instanceds,
    // );

    commands.set_state(UIState::Loaded);
}
