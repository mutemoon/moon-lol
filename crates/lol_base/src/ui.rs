use std::collections::{BTreeMap, HashMap};

use bevy::asset::Asset;
use bevy::math::{Vec2, Vec4};
use bevy::prelude::{Handle, Reflect, ReflectResource, Resource};
use league_utils::hash_bin;
use serde::{Deserialize, Serialize};

/// UI 资源路径
#[derive(Clone, Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct LOLUiPaths;

impl LOLUiPaths {
    pub fn ui_elements_dir(&self) -> String {
        "assets/ui/elements".to_string()
    }

    pub fn ui_scenes_dir(&self) -> String {
        "assets/ui/scenes".to_string()
    }

    pub fn ui_ron(&self) -> String {
        "assets/ui/ui.ron".to_string()
    }

    pub fn ui_scene_ron(&self) -> String {
        "assets/ui/ui_scene.ron".to_string()
    }

    pub fn player_frame_ron(&self) -> String {
        "ui/gameplay.playerframe.ron".to_string()
    }

    pub fn floating_info_bars_ron(&self) -> String {
        "ui/gameplay.lolfloatinginfobars.ron".to_string()
    }
}

/// UI 场景数据（用于序列化到 .ron）
#[derive(Serialize, Deserialize, Debug, Clone, Reflect)]
#[serde(rename_all = "camelCase")]
pub struct LOLUiSceneExport {
    pub elements: HashMap<u32, String>, // hash -> element asset path
    pub scenes: HashMap<u32, LOLSceneDataExport>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect)]
#[serde(rename_all = "camelCase")]
pub struct LOLSceneDataExport {
    pub name: String,
    pub enabled: Option<bool>,
    pub elements: Vec<u32>,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LOLUiFile {
    pub elements: BTreeMap<u32, LOLUiElementIconData>,
    pub animation_elements: BTreeMap<u32, LOLUiElementEffectAnimationData>,
    pub button_elements: BTreeMap<u32, LOLUiElementGroupButtonData>,
    pub region_elements: BTreeMap<u32, LOLUiElementRegionData>,
    pub scenes: BTreeMap<u32, LOLUiSceneData>,
}

#[derive(Reflect, Resource, Default)]
#[reflect(Resource)]
pub struct LOLUiScenes {
    pub scenes: HashMap<u32, LOLUiSceneData>,
}

#[derive(Reflect, Resource, Default)]
#[reflect(Resource)]
pub struct LOLUiHandles {
    pub icon_handles: HashMap<u32, Handle<LOLUiElementIconData>>,
    pub animation_handles: HashMap<u32, Handle<LOLUiElementEffectAnimationData>>,
    pub button_handles: HashMap<u32, Handle<LOLUiElementGroupButtonData>>,
    pub region_handles: HashMap<u32, Handle<LOLUiElementRegionData>>,
}

#[derive(Reflect, Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LOLUiSceneData {
    pub name: String,
    pub enabled: bool,
}

impl LOLUiFile {
    pub fn add(&mut self, element: LOLUiElementIconData) {
        self.elements.insert(hash_bin(&element.name), element);
    }

    pub fn add_animation(&mut self, element: LOLUiElementEffectAnimationData) {
        self.animation_elements
            .insert(hash_bin(&element.name), element);
    }

    pub fn add_button(&mut self, element: LOLUiElementGroupButtonData) {
        self.button_elements
            .insert(hash_bin(&element.name), element);
    }

    pub fn add_region(&mut self, element: LOLUiElementRegionData, hash: u32) {
        self.region_elements.insert(hash, element);
    }

    pub fn get_position(&self, hash: u32) -> Option<&LOLEnumUiPosition> {
        if let Some(element) = self.elements.get(&hash) {
            return Some(&element.position);
        }
        if let Some(region) = self.region_elements.get(&hash) {
            return region.position.as_ref();
        }
        None
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Asset, Reflect)]
#[serde(rename_all = "camelCase")]
pub struct LOLFloatingInfoBarViewController {
    pub info_bar_style_source_map: BTreeMap<u8, u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Asset, Reflect)]
#[serde(rename_all = "camelCase")]
pub struct LOLHeroFloatingInfoBarData {
    pub anchor: u32,
    pub borders: LOLHeroFloatingInfoBorderData,
    pub health_bar: LOLHealthBarData,
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect)]
#[serde(rename_all = "camelCase")]
pub struct LOLHeroFloatingInfoBorderData {
    pub default_border: LOLHeroFloatingInfoBorderTypeData,
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect)]
#[serde(rename_all = "camelCase")]
pub struct LOLHeroFloatingInfoBorderTypeData {
    pub border: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect)]
#[serde(rename_all = "camelCase")]
pub struct LOLHealthBarData {
    pub health_bar: LOLBarTypeMap,
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect)]
#[serde(rename_all = "camelCase")]
pub struct LOLBarTypeMap {
    pub additional_bar_types: Option<BTreeMap<u32, u32>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Asset, Reflect)]
#[serde(rename_all = "camelCase")]
pub struct LOLStructureFloatingInfoBarData {
    pub anchor: u32,
    pub border: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Asset, Reflect)]
#[serde(rename_all = "camelCase")]
pub struct LOLUnitFloatingInfoBarData {
    pub anchor: u32,
    pub border: u32,
}

#[derive(Reflect, Serialize, Deserialize, Debug, Clone, Asset)]
#[serde(rename_all = "camelCase")]
pub struct LOLUiElementIconData {
    pub name: String,
    pub position: LOLEnumUiPosition,
    pub layer: Option<u32>,
    pub texture_data: Option<LOLEnumData>,
    pub enabled: bool,
    pub scene: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Asset, Reflect)]
#[serde(rename_all = "camelCase")]
pub struct LOLUiElementRegionData {
    pub name: String,
    pub position: Option<LOLEnumUiPosition>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Asset, Reflect)]
#[serde(rename_all = "camelCase")]
pub struct LOLUiElementGroupButtonData {
    pub name: String,
    pub is_enabled: Option<bool>,
    pub hit_region_element: u32,
    pub elements: Vec<u32>,
    pub clicked_state_elements: Option<LOLUiElementGroupButtonState>,
    pub hover_state_elements: Option<LOLUiElementGroupButtonState>,
    pub default_state_elements: Option<LOLUiElementGroupButtonState>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect)]
#[serde(rename_all = "camelCase")]
pub struct LOLUiElementGroupButtonState {
    pub display_element_list: Option<Vec<u32>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Asset, Reflect)]
#[serde(rename_all = "camelCase")]
pub struct LOLUiElementEffectAnimationData {
    pub name: String,
    pub position: LOLUiPositionRect,
    pub layer: Option<u32>,
    pub texture_data: Option<LOLEnumData>,
    pub frames_per_second: Option<f32>,
    pub total_number_of_frames: Option<f32>,
    pub number_of_frames_per_row_in_atlas: Option<f32>,
    pub finish_behavior: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect)]
#[serde(rename_all = "camelCase")]
pub enum LOLEnumUiPosition {
    UiPositionRect(LOLUiPositionRect),
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect)]
#[serde(rename_all = "camelCase")]
pub struct LOLUiPositionRect {
    pub anchors: Option<LOLEnumAnchor>,
    pub ui_rect: Option<LOLUiElementRect>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect)]
#[serde(rename_all = "camelCase")]
pub struct LOLUiElementRect {
    pub position: Option<Vec2>,
    pub size: Option<Vec2>,
    pub source_resolution_height: Option<u16>,
    pub source_resolution_width: Option<u16>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect)]
#[serde(rename_all = "camelCase")]
pub enum LOLEnumAnchor {
    AnchorSingle(LOLAnchorSingle),
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect)]
#[serde(rename_all = "camelCase")]
pub struct LOLAnchorSingle {
    pub anchor: Vec2,
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect)]
#[serde(rename_all = "camelCase")]
pub enum LOLEnumData {
    AtlasData(LOLAtlasData),
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect)]
#[serde(rename_all = "camelCase")]
pub struct LOLAtlasData {
    pub m_texture_name: String,
    pub m_texture_uv: Option<Vec4>,
}
