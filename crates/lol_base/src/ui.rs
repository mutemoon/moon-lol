use std::collections::{BTreeMap, HashMap};

use bevy::asset::Asset;
use bevy::math::{Vec2, Vec4};
use bevy::prelude::{Handle, Resource, Reflect, ReflectResource};
use serde::{Deserialize, Serialize};

use crate::hash::hash_bin;
use crate::hash_key::HashKey;

#[derive(Serialize, Deserialize, Debug, Clone, Resource, Asset, Reflect, Default)]
#[reflect(Resource)]
pub struct LOLPlayerFrameViewController {
    pub abilities_ui_data: LOLAbilitiesUiData,
    pub portrait_ui_data: LOLPlayerPortraitUiData,
    pub resource_bars: LOLHudPlayerResourceBars,
    pub level_up_display: LOLUiLevelUp,
    pub root_scene: HashKey<LOLUiSceneData>,
    pub stat_pages: Vec<LOLStatPageViewController>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect, Default)]
pub struct LOLSpellPipsUiData {
    pub empty_pips: Vec<HashKey<LOLUiElementIconData>>,
    pub full_pips: Vec<HashKey<LOLUiElementIconData>>,
    pub pip_target_rect: HashKey<LOLUiElementRegionData>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect, Default)]
pub struct LOLSpellRankPipsUiData {
    pub rank_pips: Vec<LOLSpellPipsUiData>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect, Default)]
pub struct LOLAbilitiesUiData {
    pub champion_spells: Vec<LOLSpellSlotDetailedUiDefinition>,
    pub passive: LOLSpellSlotDetailedUiDefinition,
    pub spell_rank_pips: LOLSpellRankPipsUiData,
    pub summoner_spells: Vec<LOLSpellSlotDetailedUiDefinition>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect, Default)]
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

#[derive(Serialize, Deserialize, Debug, Clone, Reflect, Default)]
pub struct LOLSpellSlotBuffTimerData {
    pub timer_bar_bg: u32,
    pub timer_bar_fill: u32,
    pub timer_border_bg: u32,
    pub timer_border_fx: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect, Default)]
pub struct LOLCooldownEffectUiData {
    pub cooldown_complete_effect: u32,
    pub cooldown_jump_effect: Option<u32>,
    pub cooldown_text: Option<u32>,
    pub radial_effect: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect, Default)]
pub struct LOLCooldownGemUiData {
    pub ally_gem: u32,
    pub cooldown_effects: LOLCooldownEffectUiData,
    pub gem_background: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect, Default)]
pub struct LOLPlayerPortraitUiData {
    pub icon: HashKey<LOLUiElementIconData>,
    pub level_text: u32,
    pub respawn_timer: u32,
    pub tooltip_region: HashKey<LOLUiElementRegionData>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect, Default)]
pub struct LOLHudAbilityResourceThresholdIndicator {
    pub threshold_indicator_elements: Vec<HashKey<LOLUiElementIconData>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect, Default)]
pub struct LOLUiElementMeterSkin {
    pub bar_elements: Vec<HashKey<LOLUiElementIconData>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect, Default)]
pub struct LOLHealthMeter {
    pub fade_bar: HashKey<LOLUiElementIconData>,
    pub meter: HashKey<LOLUiElementIconData>,
    pub value_text: HashKey<LOLUiElementTextData>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect, Default)]
pub struct LOLAbilityResourceBarData {
    pub ability_resource_bars: LOLEnumResourceMeter,
    pub backdrop: Option<HashKey<LOLUiElementIconData>>,
    pub standard_tick: Option<HashKey<LOLUiElementIconData>>,
    pub use_animated_skins: Option<bool>,
    pub value_text: Option<HashKey<LOLUiElementTextData>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect)]
pub enum LOLEnumResourceMeter {
    ResourceMeterGroupData(LOLResourceMeterGroupData),
    ResourceMeterIconData(LOLResourceMeterIconData),
}

impl Default for LOLEnumResourceMeter {
    fn default() -> Self {
        Self::ResourceMeterGroupData(LOLResourceMeterGroupData::default())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect, Default)]
pub struct LOLResourceMeterGroupData {
    pub meter: HashKey<LOLUiElementIconData>,
    pub meter_skins: LOLResourceMeterSkinData,
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect, Default)]
pub struct LOLResourceMeterSkinData {
    pub additional_meter_skins: BTreeMap<u32, LOLUiElementMeterSkin>,
    pub default_meter_skin: LOLUiElementMeterSkin,
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect, Default)]
pub struct LOLResourceMeterIconData {
    pub additional_bar_types: Option<BTreeMap<u32, HashKey<LOLUiElementIconData>>>,
    pub default_bar: HashKey<LOLUiElementIconData>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect, Default)]
pub struct LOLHudPlayerResourceBars {
    pub ar_threshold_indicator: Option<LOLHudAbilityResourceThresholdIndicator>,
    pub experience_bar: HashKey<LOLUiElementIconData>,
    pub experience_hit_region: HashKey<LOLUiElementIconData>,
    pub health_animated_meter_skin: LOLUiElementMeterSkin,
    pub health_hit_region: HashKey<LOLUiElementRegionData>,
    pub health_meter: LOLHealthMeter,
    pub health_regen_text: HashKey<LOLUiElementTextData>,
    pub par_hit_region: HashKey<LOLUiElementRegionData>,
    pub par_meter_data: LOLAbilityResourceBarData,
    pub par_regen_text: HashKey<LOLUiElementTextData>,
    pub sar_text: HashKey<LOLUiElementTextData>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect, Default)]
pub struct LOLUiLevelUp {
    pub buttons_scene: u32,
    pub fx_in_scene: u32,
    pub spells: Option<Vec<LOLSpellLevelUpUiData>>,
    pub title: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect, Default)]
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
        "ui/ui.ron".to_string()
    }

    pub fn ui_scene_ron(&self) -> String {
        "ui/ui_scene.ron".to_string()
    }

    pub fn player_frame_ron(&self) -> String {
        "ui/gameplay.playerframe.ron".to_string()
    }

    pub fn player_inventory_ron(&self) -> String {
        "ui/gameplay.playerinventory.ron".to_string()
    }

    pub fn player_augments_ron(&self) -> String {
        "ui/gameplay.playeraugments.ron".to_string()
    }

    pub fn player_mute_ron(&self) -> String {
        "ui/gameplay.playermute.ron".to_string()
    }

    pub fn player_perks_ron(&self) -> String {
        "ui/gameplay.playerperks.ron".to_string()
    }

    pub fn player_report_ron(&self) -> String {
        "ui/gameplay.playerreport.ron".to_string()
    }

    pub fn player_stats_ron(&self) -> String {
        "ui/gameplay.playerstats.ron".to_string()
    }

    pub fn player_statstones_ron(&self) -> String {
        "ui/gameplay.playerstatstones.ron".to_string()
    }

    pub fn floating_info_bars_ron(&self) -> String {
        "ui/gameplay.lolfloatinginfobars.ron".to_string()
    }

    pub fn lol_game_header_ron(&self) -> String {
        "ui/gameplay.lolgameheader.ron".to_string()
    }
}

/// UI 场景数据（用于序列化到 .ron）
#[derive(Serialize, Deserialize, Debug, Clone, Reflect, Default)]
pub struct LOLUiSceneExport {
    pub elements: HashMap<u32, String>, // hash -> element asset path
    pub scenes: HashMap<u32, LOLSceneDataExport>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect, Default)]
pub struct LOLSceneDataExport {
    pub name: String,
    pub enabled: Option<bool>,
    pub elements: Vec<u32>,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone, Reflect)]
pub struct LOLUiFile {
    pub elements: BTreeMap<u32, LOLUiElementIconData>,
    pub animation_elements: BTreeMap<u32, LOLUiElementEffectAnimationData>,
    pub desaturate_elements: BTreeMap<u32, LOLUiElementEffectDesaturateData>,
    pub instanced_elements: BTreeMap<u32, LOLUiElementEffectInstancedData>,
    pub fill_percentage_elements: BTreeMap<u32, LOLUiElementEffectFillPercentageData>,
    pub button_elements: BTreeMap<u32, LOLUiElementGroupButtonData>,
    pub region_elements: BTreeMap<u32, LOLUiElementRegionData>,
    pub text_elements: BTreeMap<u32, LOLUiElementTextData>,
    pub scenes: BTreeMap<u32, LOLUiSceneData>,
    pub unit_floating_info_bars: BTreeMap<u32, LOLUnitFloatingInfoBarData>,
    pub hero_floating_info_bars: BTreeMap<u32, LOLHeroFloatingInfoBarData>,
    pub structure_floating_info_bars: BTreeMap<u32, LOLStructureFloatingInfoBarData>,
}

#[derive(Debug, Clone, Reflect)]
pub enum LOLFloatingInfoBarDataHandle {
    Unit(Handle<LOLUnitFloatingInfoBarData>),
    Hero(Handle<LOLHeroFloatingInfoBarData>),
    Structure(Handle<LOLStructureFloatingInfoBarData>),
}

#[derive(Serialize, Deserialize, Debug, Clone, Asset, Reflect, Default)]
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

    pub fn add_fill_percentage(&mut self, element: LOLUiElementEffectFillPercentageData) {
        self.fill_percentage_elements
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
        if let Some(fill_percentage) = self.fill_percentage_elements.get(&hash) {
            return Some(&fill_percentage.position);
        }
        None
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Resource, Asset, Reflect, Default)]
#[reflect(Resource)]
pub struct LOLFloatingInfoBarViewController {
    pub info_bar_style_source_map: BTreeMap<u8, u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Resource, Asset, Reflect, Default)]
#[reflect(Resource)]
pub struct LOLStatPageViewController {
    pub button: HashKey<LOLUiElementGroupButtonData>,
    pub stat_page_view_controller: u32,
    pub categories: Option<BTreeMap<u32, LOLStatPageCategoryData>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect, Default)]
pub struct LOLStatPageCategoryData {
    pub button: HashKey<LOLUiElementGroupButtonData>,
    pub stat_page_view_controller: u32,
    pub is_selected: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Asset, Reflect, Default)]
pub struct LOLHeroFloatingInfoBarData {
    pub anchor: HashKey<LOLUiElementRegionData>,
    pub borders: LOLHeroFloatingInfoBorderData,
    pub health_bar: LOLHealthBarData,
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect, Default)]
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

#[derive(Serialize, Deserialize, Debug, Clone, Reflect, Default)]
pub struct LOLHeroFloatingInfoBorderDefenseIconData {
    pub defense_down_icons: Vec<LOLHeroFloatingInfoBorderDefenseIconThresholdData>,
    pub defense_up_icon: LOLHeroFloatingInfoBorderDefenseIconThresholdData,
    pub left_icon_region: HashKey<LOLUiElementRegionData>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect, Default)]
pub struct LOLHeroFloatingInfoBorderDefenseIconThresholdData {
    pub armor_icon: HashKey<LOLUiElementIconData>,
    pub combo_icon: HashKey<LOLUiElementIconData>,
    pub defense_modifier_threshold: f32,
    pub magic_resist_icon: HashKey<LOLUiElementIconData>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect, Default)]
pub struct LOLHeroFloatingInfoBorderTypeData {
    pub border: HashKey<LOLUiElementIconData>,
    pub level_box_overlay_ally: Option<HashKey<LOLUiElementIconData>>,
    pub level_box_overlay_enemy: Option<HashKey<LOLUiElementIconData>>,
    pub level_box_overlay_self: Option<HashKey<LOLUiElementIconData>>,
    pub level_box_overlay_self_colorblind: Option<HashKey<LOLUiElementIconData>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect, Default)]
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

#[derive(Serialize, Deserialize, Debug, Clone, Reflect, Default)]
pub struct LOLHealthBarExtraBarsData {
    pub all_shield_bar: HashKey<LOLUiElementIconData>,
    pub champ_specific_bar: Option<LOLBarTypeMap>,
    pub disguise_health_bar: Option<HashKey<LOLUiElementIconData>>,
    pub incoming_heal_bar: Option<LOLBarTypeMap>,
    pub magic_shield_bar: Option<HashKey<LOLUiElementIconData>>,
    pub physical_shield_bar: Option<HashKey<LOLUiElementIconData>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect, Default)]
pub struct LOLHealthBarFadeData {
    pub fade_bar: LOLBarTypeMap,
    pub fade_speed: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect, Default)]
pub struct LOLHealthBarTextData {
    pub health_value_text: HashKey<LOLUiElementTextData>,
    pub include_max_health: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect)]
pub enum LOLEnumHealthBarTickStyle {
    HealthBarTickStyleHero(LOLHealthBarTickStyleHero),
    HealthBarTickStyleTftCompanion(LOLHealthBarTickStyleTftCompanion),
    HealthBarTickStyleUnit(LOLHealthBarTickStyleUnit),
}

impl Default for LOLEnumHealthBarTickStyle {
    fn default() -> Self {
        Self::HealthBarTickStyleHero(LOLHealthBarTickStyleHero::default())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect, Default)]
pub struct LOLHealthBarTickStyleHero {
    pub micro_tick: HashKey<LOLUiElementEffectInstancedData>,
    pub micro_tick_per_standard_tick_data: Vec<LOLMicroTicksPerStandardTickData>,
    pub standard_tick: HashKey<LOLUiElementEffectInstancedData>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect, Default)]
pub struct LOLHealthBarTickStyleTftCompanion {
    pub standard_tick: HashKey<LOLUiElementEffectInstancedData>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect, Default)]
pub struct LOLHealthBarTickStyleUnit {
    pub standard_tick: HashKey<LOLUiElementEffectInstancedData>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect, Default)]
pub struct LOLMicroTicksPerStandardTickData {
    pub micro_ticks_between: u32,
    pub min_health: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect, Default)]
pub struct LOLBarTypeMap {
    pub additional_bar_types: Option<BTreeMap<u32, HashKey<LOLUiElementIconData>>>,
    pub default_bar: HashKey<LOLUiElementIconData>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Asset, Reflect, Default)]
pub struct LOLStructureFloatingInfoBarData {
    pub anchor: HashKey<LOLUiElementRegionData>,
    pub border: HashKey<LOLUiElementIconData>,
    pub health_bar: LOLHealthBarData,
}

#[derive(Serialize, Deserialize, Debug, Clone, Asset, Reflect, Default)]
pub struct LOLUnitFloatingInfoBarData {
    pub anchor: HashKey<LOLUiElementRegionData>,
    pub border: HashKey<LOLUiElementIconData>,
    pub health_bar: LOLHealthBarData,
}

#[derive(Serialize, Deserialize, Debug, Clone, Asset, Reflect, Default)]
pub struct LOLUiElementIconData {
    pub name: String,
    pub position: LOLEnumUiPosition,
    pub layer: Option<u32>,
    pub texture_data: Option<LOLEnumData>,
    pub enabled: bool,
    pub scene: HashKey<LOLUiSceneData>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Asset, Reflect, Default)]
pub struct LOLUiElementRegionData {
    pub name: String,
    pub position: Option<LOLEnumUiPosition>,
    pub scene: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Asset, Reflect, Default)]
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

#[derive(Serialize, Deserialize, Debug, Clone, Reflect, Default)]
pub struct LOLUiElementGroupButtonState {
    pub display_element_list: Option<Vec<u32>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Asset, Reflect, Default)]
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

#[derive(Serialize, Deserialize, Debug, Clone, Asset, Reflect, Default)]
pub struct LOLUiElementEffectDesaturateData {
    pub name: String,
    pub position: LOLEnumUiPosition,
    pub layer: Option<u32>,
    pub texture_data: Option<LOLEnumData>,
    pub scene: u32,
    pub enabled: bool,
    pub minimum_saturation: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Asset, Reflect, Default)]
pub struct LOLUiElementEffectInstancedData {
    pub name: String,
    pub position: LOLEnumUiPosition,
    pub layer: u32,
    pub texture_data: Option<LOLAtlasData>,
    pub color: Option<[u8; 4]>,
    pub scene: u32,
    pub enabled: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Asset, Reflect, Default)]
pub struct LOLUiElementEffectFillPercentageData {
    pub name: String,
    pub position: LOLEnumUiPosition,
    pub layer: u32,
    pub texture_data: LOLAtlasData,
    pub scene: u32,
    pub enabled: bool,
    pub m_per_pixel_uvs_x: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Asset, Reflect, Default)]
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

#[derive(Serialize, Deserialize, Debug, Clone, Reflect, Default)]
pub enum LOLEnumUiPosition {
    UiPositionRect(LOLUiPositionRect),
    UiPositionPolygon(LOLUiPositionPolygon),
    #[default]
    UiPositionFullScreen,
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect, Default)]
pub struct LOLUiPositionPolygon {
    pub anchors: LOLEnumAnchor,
    pub ui_rect: Option<LOLUiElementRect>,
    pub polygon_vertices: Vec<Vec2>,
    pub disable_pixel_snapping_x: Option<bool>,
    pub disable_pixel_snapping_y: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect, Default)]
pub struct LOLUiPositionRect {
    pub anchors: Option<LOLEnumAnchor>,
    pub ui_rect: Option<LOLUiElementRect>,
    pub disable_pixel_snapping_x: Option<bool>,
    pub disable_pixel_snapping_y: Option<bool>,
    pub disable_resolution_downscale: Option<bool>,
    pub ignore_global_scale: Option<bool>,
    pub ignore_safe_zone: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect, Default)]
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

#[derive(Serialize, Deserialize, Debug, Clone, Reflect, Default)]
pub enum LOLEnumAnchor {
    AnchorSingle(LOLAnchorSingle),
    AnchorDouble(LOLAnchorDouble),
    #[default]
    Unk0xf090d2e7,
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect, Default)]
pub struct LOLAnchorDouble {
    pub anchor_left: Option<Vec2>,
    pub anchor_right: Option<Vec2>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect, Default)]
pub struct LOLAnchorSingle {
    pub anchor: Vec2,
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect, Default)]
pub enum LOLEnumData {
    AtlasData(LOLAtlasData),
    AtlasData3SliceH(LOLAtlasData3SliceH),
    AtlasData3SliceV(LOLAtlasData3SliceV),
    AtlasData9Slice(LOLAtlasData9Slice),
    LooseUiTextureData(LOLLooseUiTextureData),
    LooseUiTextureData3SliceH(LOLLooseUiTextureData3SliceH),
    LooseUiTextureData3SliceV(LOLLooseUiTextureData3SliceV),
    LooseUiTextureData9Slice(LOLLooseUiTextureData9Slice),
    #[default]
    Unk0x5eaead1a,
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect, Default)]
pub struct LOLAtlasData3SliceH {
    pub m_texture_name: String,
    pub texture_us: Vec4,
    pub texture_vs: Vec2,
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect, Default)]
pub struct LOLAtlasData3SliceV {
    pub m_texture_name: String,
    pub texture_us: Vec2,
    pub texture_vs: Vec4,
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect, Default)]
pub struct LOLAtlasData9Slice {
    pub m_texture_name: String,
    pub texture_us: Vec4,
    pub texture_vs: Vec4,
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect, Default)]
pub struct LOLLooseUiTextureData3SliceH {
    pub texture_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect, Default)]
pub struct LOLLooseUiTextureData3SliceV {
    pub texture_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect, Default)]
pub struct LOLLooseUiTextureData9Slice {
    pub texture_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect, Default)]
pub struct LOLAtlasData {
    pub m_texture_name: String,
    pub m_texture_uv: Option<Vec4>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect, Default)]
pub struct LOLLooseUiTextureData {
    pub texture_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Resource, Asset, Reflect, Default)]
#[reflect(Resource)]
pub struct LOLPlayerInventoryViewController {
    pub item_slot_ui_data: Vec<LOLItemSlotDetailedUiData>,
    pub scene: HashKey<LOLUiSceneData>,
    pub shop_button: LOLHudShopButton,
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect, Default)]
pub struct LOLItemSlotDetailedUiData {
    pub ammo_fx: Option<u32>,
    pub backdrop: HashKey<LOLUiElementIconData>,
    pub border_default: HashKey<LOLUiElementIconData>,
    pub border_disabled: HashKey<LOLUiElementIconData>,
    pub border_enabled: HashKey<LOLUiElementIconData>,
    pub border_selected: Option<HashKey<LOLUiElementIconData>>,
    pub complete_fx: Option<u32>,
    pub cooldown_effects: Option<LOLCooldownEffectUiData>,
    pub hit_area: HashKey<LOLUiElementRegionData>,
    pub hotkey_text: HashKey<LOLUiElementTextData>,
    pub icon: HashKey<LOLUiElementIconData>,
    pub major_active: Option<u32>,
    pub overlay_disabled: HashKey<LOLUiElementIconData>,
    pub overlay_hover: HashKey<LOLUiElementIconData>,
    pub overlay_loc: HashKey<LOLUiElementIconData>,
    pub overlay_oom: Option<HashKey<LOLUiElementIconData>>,
    pub stack_text: Option<HashKey<LOLUiElementTextData>>,
    pub toggle_fx: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect, Default)]
pub struct LOLHudShopButton {
    pub inactive_icon: HashKey<LOLUiElementIconData>,
    pub shop_button: HashKey<LOLUiElementGroupButtonData>,
    pub text_link: HashKey<LOLUiElementTextData>,
    pub unk_0x34a1434b: Option<u32>,
    pub unk_0x40aa9d58: Option<u8>,
    pub unk_0x697f8b6b: Option<u32>,
    pub unk_0x778e26c6: Option<u32>,
    pub unk_0x7dffe581: Option<String>,
    pub unk_0x8031b7a0: Option<u32>,
    pub unk_0xb77375ae: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect, Default)]
pub struct LOLDrawAreaList {
    pub draw_regions: Vec<HashKey<LOLUiElementRegionData>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect)]
pub enum LOLEnumUiMetric {
    UiMetricFps(LOLUiMetricFps),
    UiMetricClash(LOLUiMetricClash),
    UiMetricCreepScore(LOLUiMetricCreepScore),
    UiMetricGameTime(LOLUiMetricGameTime),
    UiMetricKda(LOLUiMetricKda),
    UiMetricLatencyText(LOLUiMetricLatencyText),
    UiMetricTeamKills(LOLUiMetricTeamKills),
    UiMetricTeamScoreMeters(LOLUiMetricTeamScoreMeters),
    Unk0x5ab5b20f(LOLUnk0x5ab5b20f),
    Unk0x767adcf7(LOLUnk0x767adcf7),
    Unk0xb62c8675(LOLUnk0xb62c8675),
    Unk0xe228ce4a(LOLUnk0xe228ce4a),
}

impl Default for LOLEnumUiMetric {
    fn default() -> Self {
        Self::UiMetricFps(LOLUiMetricFps::default())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect, Default)]
pub struct LOLUiClashTeam {
    pub logo_icon: HashKey<LOLUiElementIconData>,
    pub tag_text: HashKey<LOLUiElementTextData>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect, Default)]
pub struct LOLUiMetricClash {
    pub clash_frame: HashKey<LOLUiElementIconData>,
    pub clash_frame_mirror: HashKey<LOLUiElementIconData>,
    pub clash_round_icon: HashKey<LOLUiElementIconData>,
    pub clash_round_text: HashKey<LOLUiElementTextData>,
    pub device_ux: i32,
    pub team1: LOLUiClashTeam,
    pub team2: LOLUiClashTeam,
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect, Default)]
pub struct LOLUiMetricCreepScore {
    pub device_ux: i32,
    pub icon: HashKey<LOLUiElementIconData>,
    pub text: HashKey<LOLUiElementTextData>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect, Default)]
pub struct LOLUiMetricFps {
    pub device_ux: i32,
    pub fps_text: HashKey<LOLUiElementTextData>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect, Default)]
pub struct LOLUiMetricGameTime {
    pub device_ux: i32,
    pub time_text: HashKey<LOLUiElementTextData>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect, Default)]
pub struct LOLUiMetricKda {
    pub device_ux: i32,
    pub icon: HashKey<LOLUiElementIconData>,
    pub text: HashKey<LOLUiElementTextData>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect, Default)]
pub struct LOLUiMetricLatencyText {
    pub device_ux: i32,
    pub latency_text: HashKey<LOLUiElementTextData>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect, Default)]
pub struct LOLUiMetricTeamKills {
    pub device_ux: i32,
    pub team1_kill_text: HashKey<LOLUiElementTextData>,
    pub team2_kill_text: HashKey<LOLUiElementTextData>,
    pub team_kills_icon: HashKey<LOLUiElementIconData>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect, Default)]
pub struct LOLUiMetricTeamScoreMeters {
    pub device_ux: Option<i32>,
    pub frame: HashKey<LOLUiElementIconData>,
    pub team1_meter: HashKey<LOLUiElementIconData>,
    pub team1_meter_blue_skin: HashKey<LOLUiElementIconData>,
    pub team1_meter_red_skin: HashKey<LOLUiElementIconData>,
    pub team2_meter: HashKey<LOLUiElementIconData>,
    pub team2_meter_blue_skin: HashKey<LOLUiElementIconData>,
    pub team2_meter_red_skin: HashKey<LOLUiElementIconData>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect, Default)]
pub struct LOLUnk0x5ab5b20f {
    pub device_ux: i32,
    pub time_text: HashKey<LOLUiElementTextData>,
    pub unk_0xadbcc5ee: HashKey<LOLUiElementIconData>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect, Default)]
pub struct LOLUnk0x767adcf7 {
    pub device_ux: i32,
    pub frame: HashKey<LOLUiElementIconData>,
    pub time_text: HashKey<LOLUiElementTextData>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect, Default)]
pub struct LOLUnk0xa8c6f5f0 {
    pub unk_0x1793d323: HashKey<LOLUiElementIconData>,
    pub unk_0x4297f4f9: HashKey<LOLUiElementIconData>,
    pub unk_0x5329572e: HashKey<LOLUiElementIconData>,
    pub unk_0xb80015ba: HashKey<LOLUiElementIconData>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect, Default)]
pub struct LOLUnk0x7a19656 {
    pub detail_panel: HashKey<LOLUiElementRegionData>,
    pub detail_text_t1: HashKey<LOLUiElementTextData>,
    pub detail_text_t2: HashKey<LOLUiElementTextData>,
    pub timer_panel: HashKey<LOLUiElementRegionData>,
    pub timer_text: HashKey<LOLUiElementTextData>,
    pub unk_0x6188e7b7: HashKey<LOLUiElementIconData>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect, Default)]
pub struct LOLUnk0xb8a49c96 {
    pub blue_skin: HashKey<LOLUiElementIconData>,
    pub meter: HashKey<LOLUiElementIconData>,
    pub red_skin: HashKey<LOLUiElementIconData>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect, Default)]
pub struct LOLUnk0xf43ad1ce {
    pub frame: HashKey<LOLUiElementIconData>,
    pub icon_shadow_t1: HashKey<LOLUiElementIconData>,
    pub icon_shadow_t2: HashKey<LOLUiElementIconData>,
    pub team1_meter: LOLUnk0xb8a49c96,
    pub team2_meter: LOLUnk0xb8a49c96,
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect, Default)]
pub struct LOLUnk0xb62c8675 {
    pub base_loadable: u32,
    pub crown_icons: LOLUnk0xa8c6f5f0,
    pub details_panel: LOLUnk0x7a19656,
    pub device_ux: i32,
    pub meters_panel: LOLUnk0xf43ad1ce,
    pub scene: u32,
    pub soraka_icons: LOLUnk0xa8c6f5f0,
    pub tower_icons: LOLUnk0xa8c6f5f0,
    pub unk_0x462800b7: LOLUnk0xa8c6f5f0,
    pub unk_0xb057cf4b: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone, Reflect, Default)]
pub struct LOLUnk0xe228ce4a {
    pub device_ux: i32,
    pub frame: HashKey<LOLUiElementIconData>,
    pub team1_text: HashKey<LOLUiElementTextData>,
    pub team2_text: HashKey<LOLUiElementTextData>,
    pub unk_0x3a568777: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Resource, Asset, Reflect, Default)]
#[reflect(Resource)]
pub struct LOLLolGameStateViewController {
    pub base_loadable: u32,
    pub draw_area_list: Option<LOLDrawAreaList>,
    pub metrics: Vec<LOLEnumUiMetric>,
    pub path_hash_to_self: u64,
    pub scene: HashKey<LOLUiSceneData>,
}
