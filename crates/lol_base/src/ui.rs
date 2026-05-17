use std::collections::{BTreeMap, HashMap};

use bevy::asset::Asset;
use bevy::math::{Vec2, Vec4};
use bevy::prelude::{Handle, Resource};
use bevy::reflect::TypePath;
use serde::{Deserialize, Serialize};

use crate::hash::hash_bin;
use crate::hash_key::HashKey;

#[derive(Serialize, Deserialize, Debug, Clone, Resource, Asset, TypePath)]
pub struct LOLPlayerFrameViewController {
    pub abilities_ui_data: LOLAbilitiesUiData,
    pub portrait_ui_data: LOLPlayerPortraitUiData,
    pub resource_bars: LOLHudPlayerResourceBars,
    pub level_up_display: LOLUiLevelUp,
    pub root_scene: HashKey<LOLUiSceneData>,
    pub stat_pages: Vec<LOLStatPageViewController>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LOLSpellPipsUiData {
    pub empty_pips: Vec<HashKey<LOLUiElementIconData>>,
    pub full_pips: Vec<HashKey<LOLUiElementIconData>>,
    pub pip_target_rect: HashKey<LOLUiElementRegionData>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LOLSpellRankPipsUiData {
    pub rank_pips: Vec<LOLSpellPipsUiData>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LOLAbilitiesUiData {
    pub champion_spells: Vec<LOLSpellSlotDetailedUiDefinition>,
    pub passive: LOLSpellSlotDetailedUiDefinition,
    pub spell_rank_pips: LOLSpellRankPipsUiData,
    pub summoner_spells: Vec<LOLSpellSlotDetailedUiDefinition>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LOLSpellSlotDetailedUiDefinition {
    pub ammo_fx: Option<u32>,
    pub ammo_text: Option<u32>,
    pub border_disabled: HashKey<LOLUiElementIconData>,
    pub border_enabled: HashKey<LOLUiElementIconData>,
    pub buff_timer: Option<LOLSpellSlotBuffTimerData>,
    pub cooldown: HashKey<LOLUiElementTextData>,
    pub cooldown_gem: Option<LOLCooldownGemUiData>,
    pub content_element: Option<HashKey<LOLUiElementEffectDesaturateData>>,
    pub cost: Option<u32>,
    pub cost_bg: Option<u32>,
    pub hotkey: Option<u32>,
    pub mouseover_region: Option<HashKey<LOLUiElementRegionData>>,
    pub overlay_cced: Option<HashKey<LOLUiElementIconData>>,
    pub overlay_disabled: Option<HashKey<LOLUiElementIconData>>,
    pub overlay_oom: Option<HashKey<LOLUiElementIconData>>,
    pub reset_flash_fx_attention: Option<u32>,
    pub toggle_fx: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LOLSpellSlotBuffTimerData {
    pub timer_bar_bg: u32,
    pub timer_bar_fill: u32,
    pub timer_border_bg: u32,
    pub timer_border_fx: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LOLCooldownEffectUiData {
    pub cooldown_complete_effect: u32,
    pub cooldown_jump_effect: Option<u32>,
    pub cooldown_text: Option<u32>,
    pub radial_effect: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LOLCooldownGemUiData {
    pub ally_gem: u32,
    pub cooldown_effects: LOLCooldownEffectUiData,
    pub gem_background: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LOLPlayerPortraitUiData {
    pub icon: HashKey<LOLUiElementIconData>,
    pub level_text: u32,
    pub respawn_timer: u32,
    pub tooltip_region: HashKey<LOLUiElementRegionData>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LOLHudPlayerResourceBars {
    pub experience_bar: HashKey<LOLUiElementIconData>,
    pub health_hit_region: HashKey<LOLUiElementRegionData>,
    pub health_regen_text: u32,
    pub par_hit_region: HashKey<LOLUiElementRegionData>,
    pub par_regen_text: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LOLUiLevelUp {
    pub buttons_scene: u32,
    pub fx_in_scene: u32,
    pub spells: Option<Vec<LOLSpellLevelUpUiData>>,
    pub title: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LOLSpellLevelUpUiData {
    pub ability_fx_in: u32,
    pub button_fx_in: u32,
    pub button_fx_out_selected: u32,
    pub button_fx_out_unselected: u32,
    pub button_idle_glow_fx: u32,
    pub button_idle_sheen_fx: u32,
    pub button_post_fx_in: u32,
    pub skill_up_button: HashKey<LOLUiElementGroupButtonData>,
}

/// UI 资源路径
#[derive(Clone, Resource, Default)]
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
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LOLUiSceneExport {
    pub elements: HashMap<u32, String>, // hash -> element asset path
    pub scenes: HashMap<u32, LOLSceneDataExport>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LOLSceneDataExport {
    pub name: String,
    pub enabled: Option<bool>,
    pub elements: Vec<u32>,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct LOLUiFile {
    pub elements: BTreeMap<u32, LOLUiElementIconData>,
    pub animation_elements: BTreeMap<u32, LOLUiElementEffectAnimationData>,
    pub desaturate_elements: BTreeMap<u32, LOLUiElementEffectDesaturateData>,
    pub instanced_elements: BTreeMap<u32, LOLUiElementEffectInstancedData>,
    pub button_elements: BTreeMap<u32, LOLUiElementGroupButtonData>,
    pub region_elements: BTreeMap<u32, LOLUiElementRegionData>,
    pub text_elements: BTreeMap<u32, LOLUiElementTextData>,
    pub scenes: BTreeMap<u32, LOLUiSceneData>,
    pub floating_info_bar_view_controller: Option<LOLFloatingInfoBarViewController>,
    pub player_frame_view_controller: Option<LOLPlayerFrameViewController>,
    pub unit_floating_info_bars: BTreeMap<u32, LOLUnitFloatingInfoBarData>,
    pub hero_floating_info_bars: BTreeMap<u32, LOLHeroFloatingInfoBarData>,
    pub structure_floating_info_bars: BTreeMap<u32, LOLStructureFloatingInfoBarData>,
    pub stat_page_view_controller: Option<LOLStatPageViewController>,
}

#[derive(Debug, Clone)]
pub enum LOLFloatingInfoBarDataHandle {
    Unit(Handle<LOLUnitFloatingInfoBarData>),
    Hero(Handle<LOLHeroFloatingInfoBarData>),
    Structure(Handle<LOLStructureFloatingInfoBarData>),
}

#[derive(Serialize, Deserialize, Debug, Clone, Asset, TypePath)]
pub struct LOLUiSceneData {
    pub name: String,
    pub enabled: bool,
    pub parent_scene: Option<u32>,
    pub layer: Option<u32>,
}

impl LOLUiFile {
    pub fn add(&mut self, element: LOLUiElementIconData) {
        self.elements.insert(hash_bin(&element.name), element);
    }

    pub fn add_animation(&mut self, element: LOLUiElementEffectAnimationData) {
        self.animation_elements
            .insert(hash_bin(&element.name), element);
    }

    pub fn add_desaturate(&mut self, element: LOLUiElementEffectDesaturateData) {
        self.desaturate_elements
            .insert(hash_bin(&element.name), element);
    }

    pub fn add_instanced(&mut self, element: LOLUiElementEffectInstancedData) {
        self.instanced_elements
            .insert(hash_bin(&element.name), element);
    }

    pub fn add_button(&mut self, element: LOLUiElementGroupButtonData) {
        self.button_elements
            .insert(hash_bin(&element.name), element);
    }

    pub fn add_region(&mut self, element: LOLUiElementRegionData, hash: u32) {
        self.region_elements.insert(hash, element);
    }

    pub fn add_text(&mut self, element: LOLUiElementTextData) {
        self.text_elements.insert(hash_bin(&element.name), element);
    }

    pub fn set_floating_info_bar_view_controller(
        &mut self,
        controller: LOLFloatingInfoBarViewController,
    ) {
        self.floating_info_bar_view_controller = Some(controller);
    }

    pub fn add_unit_floating_info_bar(&mut self, hash: u32, data: LOLUnitFloatingInfoBarData) {
        self.unit_floating_info_bars.insert(hash, data);
    }

    pub fn add_hero_floating_info_bar(&mut self, hash: u32, data: LOLHeroFloatingInfoBarData) {
        self.hero_floating_info_bars.insert(hash, data);
    }

    pub fn add_structure_floating_info_bar(
        &mut self,
        hash: u32,
        data: LOLStructureFloatingInfoBarData,
    ) {
        self.structure_floating_info_bars.insert(hash, data);
    }

    pub fn get_position(&self, hash: u32) -> Option<&LOLEnumUiPosition> {
        if let Some(element) = self.elements.get(&hash) {
            return Some(&element.position);
        }
        if let Some(region) = self.region_elements.get(&hash) {
            return region.position.as_ref();
        }
        if let Some(desaturate) = self.desaturate_elements.get(&hash) {
            return Some(&desaturate.position);
        }
        None
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Resource, Asset, TypePath)]
pub struct LOLFloatingInfoBarViewController {
    pub info_bar_style_source_map: BTreeMap<u8, u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Resource, Asset, TypePath)]
pub struct LOLStatPageViewController {
    pub button: HashKey<LOLUiElementGroupButtonData>,
    pub stat_page_view_controller: u32,
    pub categories: Option<BTreeMap<u32, LOLStatPageCategoryData>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LOLStatPageCategoryData {
    pub button: HashKey<LOLUiElementGroupButtonData>,
    pub stat_page_view_controller: u32,
    pub is_selected: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Asset, TypePath)]
pub struct LOLHeroFloatingInfoBarData {
    pub anchor: HashKey<LOLUiElementRegionData>,
    pub borders: LOLHeroFloatingInfoBorderData,
    pub health_bar: LOLHealthBarData,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LOLHeroFloatingInfoBorderData {
    pub additional_status_icons: Option<BTreeMap<u32, HashKey<LOLUiElementIconData>>>,
    pub default_border: LOLHeroFloatingInfoBorderTypeData,
    pub defense_modifier_icons: Option<LOLHeroFloatingInfoBorderDefenseIconData>,
    pub executable_border: LOLHeroFloatingInfoBorderTypeData,
    pub has_attached_ally_border: Option<LOLHeroFloatingInfoBorderTypeData>,
    pub invulnerable_border: Option<LOLHeroFloatingInfoBorderTypeData>,
    pub level_text: HashKey<LOLUiElementTextData>,
    pub level_text_color_ally: Option<[u8; 4]>,
    pub level_text_color_enemy: Option<[u8; 4]>,
    pub level_text_color_self_colorblind: Option<[u8; 4]>,
    pub spell_shield_border: Option<LOLHeroFloatingInfoBorderTypeData>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LOLHeroFloatingInfoBorderDefenseIconData {
    pub defense_down_icons: Vec<LOLHeroFloatingInfoBorderDefenseIconThresholdData>,
    pub defense_up_icon: LOLHeroFloatingInfoBorderDefenseIconThresholdData,
    pub left_icon_region: HashKey<LOLUiElementRegionData>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LOLHeroFloatingInfoBorderDefenseIconThresholdData {
    pub armor_icon: HashKey<LOLUiElementIconData>,
    pub combo_icon: HashKey<LOLUiElementIconData>,
    pub defense_modifier_threshold: f32,
    pub magic_resist_icon: HashKey<LOLUiElementIconData>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LOLHeroFloatingInfoBorderTypeData {
    pub border: HashKey<LOLUiElementIconData>,
    pub level_box_overlay_ally: Option<HashKey<LOLUiElementIconData>>,
    pub level_box_overlay_enemy: Option<HashKey<LOLUiElementIconData>>,
    pub level_box_overlay_self: Option<HashKey<LOLUiElementIconData>>,
    pub level_box_overlay_self_colorblind: Option<HashKey<LOLUiElementIconData>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LOLHealthBarData {
    pub extra_bars: Option<LOLHealthBarExtraBarsData>,
    pub fade_data: Option<LOLHealthBarFadeData>,
    pub health_bar: LOLBarTypeMap,
    pub incoming_damage_bar: Option<HashKey<LOLUiElementIconData>>,
    pub max_hp_penalty_bar: Option<HashKey<LOLUiElementIconData>>,
    pub max_hp_penalty_divider: Option<HashKey<LOLUiElementIconData>>,
    pub text_data: Option<LOLHealthBarTextData>,
    pub tick_style: Option<LOLEnumHealthBarTickStyle>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LOLHealthBarExtraBarsData {
    pub all_shield_bar: HashKey<LOLUiElementIconData>,
    pub champ_specific_bar: Option<LOLBarTypeMap>,
    pub disguise_health_bar: Option<HashKey<LOLUiElementIconData>>,
    pub incoming_heal_bar: Option<LOLBarTypeMap>,
    pub magic_shield_bar: Option<HashKey<LOLUiElementIconData>>,
    pub physical_shield_bar: Option<HashKey<LOLUiElementIconData>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LOLHealthBarFadeData {
    pub fade_bar: LOLBarTypeMap,
    pub fade_speed: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LOLHealthBarTextData {
    pub health_value_text: HashKey<LOLUiElementTextData>,
    pub include_max_health: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum LOLEnumHealthBarTickStyle {
    HealthBarTickStyleHero(LOLHealthBarTickStyleHero),
    HealthBarTickStyleTftCompanion(LOLHealthBarTickStyleTftCompanion),
    HealthBarTickStyleUnit(LOLHealthBarTickStyleUnit),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LOLHealthBarTickStyleHero {
    pub micro_tick: HashKey<LOLUiElementEffectInstancedData>,
    pub micro_tick_per_standard_tick_data: Vec<LOLMicroTicksPerStandardTickData>,
    pub standard_tick: HashKey<LOLUiElementEffectInstancedData>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LOLHealthBarTickStyleTftCompanion {
    pub standard_tick: HashKey<LOLUiElementEffectInstancedData>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LOLHealthBarTickStyleUnit {
    pub standard_tick: HashKey<LOLUiElementEffectInstancedData>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LOLMicroTicksPerStandardTickData {
    pub micro_ticks_between: u32,
    pub min_health: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LOLBarTypeMap {
    pub additional_bar_types: Option<BTreeMap<u32, HashKey<LOLUiElementIconData>>>,
    pub default_bar: HashKey<LOLUiElementIconData>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Asset, TypePath)]
pub struct LOLStructureFloatingInfoBarData {
    pub anchor: HashKey<LOLUiElementRegionData>,
    pub border: HashKey<LOLUiElementIconData>,
    pub health_bar: LOLHealthBarData,
}

#[derive(Serialize, Deserialize, Debug, Clone, Asset, TypePath)]
pub struct LOLUnitFloatingInfoBarData {
    pub anchor: HashKey<LOLUiElementRegionData>,
    pub border: HashKey<LOLUiElementIconData>,
    pub health_bar: LOLHealthBarData,
}

#[derive(Serialize, Deserialize, Debug, Clone, Asset, TypePath)]
pub struct LOLUiElementIconData {
    pub name: String,
    pub position: LOLEnumUiPosition,
    pub layer: Option<u32>,
    pub texture_data: Option<LOLEnumData>,
    pub enabled: bool,
    pub scene: HashKey<LOLUiSceneData>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Asset, TypePath)]
pub struct LOLUiElementRegionData {
    pub name: String,
    pub position: Option<LOLEnumUiPosition>,
    pub scene: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Asset, TypePath)]
pub struct LOLUiElementGroupButtonData {
    pub name: String,
    pub is_enabled: Option<bool>,
    pub hit_region_element: HashKey<LOLUiElementRegionData>,
    pub elements: Vec<HashKey<LOLUiElementIconData>>,
    pub clicked_state_elements: Option<LOLUiElementGroupButtonState>,
    pub hover_state_elements: Option<LOLUiElementGroupButtonState>,
    pub default_state_elements: Option<LOLUiElementGroupButtonState>,
    pub scene: HashKey<LOLUiSceneData>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LOLUiElementGroupButtonState {
    pub display_element_list: Option<Vec<u32>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Asset, TypePath)]
pub struct LOLUiElementEffectAnimationData {
    pub name: String,
    pub position: LOLEnumUiPosition,
    pub layer: Option<u32>,
    pub texture_data: Option<LOLEnumData>,
    pub frames_per_second: Option<f32>,
    pub total_number_of_frames: Option<f32>,
    pub number_of_frames_per_row_in_atlas: Option<f32>,
    pub finish_behavior: Option<u8>,
    pub scene: u32,
    pub enabled: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Asset, TypePath)]
pub struct LOLUiElementEffectDesaturateData {
    pub name: String,
    pub position: LOLEnumUiPosition,
    pub layer: Option<u32>,
    pub texture_data: Option<LOLEnumData>,
    pub scene: u32,
    pub enabled: bool,
    pub minimum_saturation: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Asset, TypePath)]
pub struct LOLUiElementEffectInstancedData {
    pub name: String,
    pub position: LOLEnumUiPosition,
    pub layer: u32,
    pub texture_data: Option<LOLAtlasData>,
    pub color: Option<[u8; 4]>,
    pub scene: u32,
    pub enabled: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Asset, TypePath)]
pub struct LOLUiElementTextData {
    pub name: String,
    pub position: LOLEnumUiPosition,
    pub layer: Option<u32>,
    pub font_description: u32,
    pub text_alignment_horizontal: Option<u8>,
    pub text_alignment_vertical: Option<u8>,
    pub tra_key: Option<String>,
    pub color: Option<[u8; 4]>,
    pub scene: u32,
    pub enabled: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum LOLEnumUiPosition {
    UiPositionRect(LOLUiPositionRect),
    UiPositionPolygon(LOLUiPositionPolygon),
    UiPositionFullScreen,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LOLUiPositionPolygon {
    pub anchors: LOLEnumAnchor,
    pub ui_rect: Option<LOLUiElementRect>,
    pub polygon_vertices: Vec<Vec2>,
    pub disable_pixel_snapping_x: Option<bool>,
    pub disable_pixel_snapping_y: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LOLUiPositionRect {
    pub anchors: Option<LOLEnumAnchor>,
    pub ui_rect: Option<LOLUiElementRect>,
    pub disable_pixel_snapping_x: Option<bool>,
    pub disable_pixel_snapping_y: Option<bool>,
    pub disable_resolution_downscale: Option<bool>,
    pub ignore_global_scale: Option<bool>,
    pub ignore_safe_zone: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LOLUiElementRect {
    pub position: Option<Vec2>,
    pub size: Option<Vec2>,
    pub source_resolution_height: Option<u16>,
    pub source_resolution_width: Option<u16>,
}

impl LOLEnumUiPosition {
    pub fn ignore_global_scale(&self) -> bool {
        match self {
            LOLEnumUiPosition::UiPositionRect(rect) => rect.ignore_global_scale.unwrap_or(false),
            _ => false,
        }
    }

    pub fn disable_resolution_downscale(&self) -> bool {
        match self {
            LOLEnumUiPosition::UiPositionRect(rect) => {
                rect.disable_resolution_downscale.unwrap_or(false)
            }
            _ => false,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum LOLEnumAnchor {
    AnchorSingle(LOLAnchorSingle),
    AnchorDouble(LOLAnchorDouble),
    Unk0xf090d2e7,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LOLAnchorDouble {
    pub anchor_left: Option<Vec2>,
    pub anchor_right: Option<Vec2>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LOLAnchorSingle {
    pub anchor: Vec2,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum LOLEnumData {
    AtlasData(LOLAtlasData),
    AtlasData3SliceH(LOLAtlasData3SliceH),
    AtlasData3SliceV(LOLAtlasData3SliceV),
    AtlasData9Slice(LOLAtlasData9Slice),
    LooseUiTextureData(LOLLooseUiTextureData),
    LooseUiTextureData3SliceH(LOLLooseUiTextureData3SliceH),
    LooseUiTextureData3SliceV(LOLLooseUiTextureData3SliceV),
    LooseUiTextureData9Slice(LOLLooseUiTextureData9Slice),
    Unk0x5eaead1a,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LOLAtlasData3SliceH {
    pub m_texture_name: String,
    pub texture_us: Vec4,
    pub texture_vs: Vec2,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LOLAtlasData3SliceV {
    pub m_texture_name: String,
    pub texture_us: Vec2,
    pub texture_vs: Vec4,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LOLAtlasData9Slice {
    pub m_texture_name: String,
    pub texture_us: Vec4,
    pub texture_vs: Vec4,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LOLLooseUiTextureData3SliceH {
    pub texture_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LOLLooseUiTextureData3SliceV {
    pub texture_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LOLLooseUiTextureData9Slice {
    pub texture_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LOLAtlasData {
    pub m_texture_name: String,
    pub m_texture_uv: Option<Vec4>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LOLLooseUiTextureData {
    pub texture_name: String,
}
