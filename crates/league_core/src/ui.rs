use std::collections::HashMap;

use bevy::math::{Vec2, Vec4};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum UiElementEffectAnimationDataTextureData {
    LooseUiTextureData9Slice(LooseUiTextureData9Slice),
    LooseUiTextureData(LooseUiTextureData),
    AtlasData(AtlasData),
    AtlasData3SliceV(AtlasData3SliceV),
    LooseUiTextureData3SliceH(LooseUiTextureData3SliceH),
    AtlasData9Slice(AtlasData9Slice),
    LooseUiTextureData3SliceV(LooseUiTextureData3SliceV),
    AtlasData3SliceH(AtlasData3SliceH),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum UiElementGroupMeterDataTipStyle {
    DoubleSidedTipStyle(DoubleSidedTipStyle),
    GlowCenteredOverlayTipStyle(GlowCenteredOverlayTipStyle),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum OptionItemGroupFilter {
    OptionItemFilterWindows,
    OptionItemFilterFeatureToggle(OptionItemFilterFeatureToggle),
    OptionItemFilterHwRequirement(OptionItemFilterHwRequirement),
    Unk0xfca7e1ca,
    OptionItemFilterNot(OptionItemFilterNot),
    OptionItemFilterMutator(OptionItemFilterMutator),
    Unk0xd4737a04,
    Unk0xef2cc9a6,
    OptionItemFilterClassicMusicAllowed,
    OptionItemFilterGameStyle(OptionItemFilterGameStyle),
    Unk0x4e771e24,
    OptionItemFilterVoiceChat,
    Unk0x70749bea,
    Unk0x6b5fc3eb,
    OptionItemFilterAnd(OptionItemFilterAnd),
    OptionItemFilterOsx,
    OptionItemFilterReplayApi,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum OptionItemGroupItems {
    OptionItemVoiceInputDeviceDropdown(OptionItemVoiceInputDeviceDropdown),
    Unk0x165c0117(Unk0x165c0117),
    OptionItemSliderGraphicsQuality(OptionItemSliderGraphicsQuality),
    OptionItemHotkeys(OptionItemHotkeys),
    OptionItemDropdown(OptionItemDropdown),
    OptionItemSliderFloat(OptionItemSliderFloat),
    Unk0x9ef1e737(Unk0x9ef1e737),
    OptionItemBorder(OptionItemBorder),
    OptionItemLabel(OptionItemLabel),
    Unk0xa9d60c77(Unk0xa9d60c77),
    OptionItemResolutionDropdown(OptionItemResolutionDropdown),
    Unk0x81580a34(Unk0x81580a34),
    OptionItemCheckbox(OptionItemCheckbox),
    OptionItemGroup(OptionItemGroup),
    OptionItemSliderInt(OptionItemSliderInt),
    OptionItemSliderVolume(OptionItemSliderVolume),
    OptionItemButton(OptionItemButton),
    OptionItemColumns(OptionItemColumns),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum UiElementIconDataDragType {
    UiDraggableElementGroupDrag(UiDraggableElementGroupDrag),
    UiDraggableProxyElementDrag(UiDraggableProxyElementDrag),
    UiDraggableBasic(UiDraggableBasic),
    UiDraggableSceneDrag(UiDraggableSceneDrag),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum UiElementIconDataPosition {
    UiPositionRect(UiPositionRect),
    UiPositionFullScreen,
    UiPositionPolygon(UiPositionPolygon),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum UiElementIconDataExtension {
    IconElementGradientExtension,
    IconElementCircleMaskeExtension,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum UiElementGroupManagedLayoutDataLayoutStyle {
    LayoutStyleGrid(LayoutStyleGrid),
    LayoutStyleVerticalList(LayoutStyleVerticalList),
    LayoutStyleHorizontalList(LayoutStyleHorizontalList),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum UiSceneDataSceneTransitionOut {
    SceneScreenEdgeTransitionData(SceneScreenEdgeTransitionData),
    SceneAlphaTransitionData(SceneAlphaTransitionData),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Unk0x104afcdaUnknownEnumField {
    Unk0xd2fb821b(Unk0xd2fb821b),
    Unk0x7acf50f9,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum OptionItemButtonFilter {
    OptionItemFilterIos,
    OptionItemFilterPromoteAccount,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum UiPositionRectAnchors {
    AnchorSingle(AnchorSingle),
    AnchorDouble(AnchorDouble),
    Unk0x0,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AnchorDouble {
    pub anchor_right: Option<Vec2>,
    pub anchor_left: Option<Vec2>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UiElementGroupSliderState {
    pub slider_icon: u32,
    pub bar_backdrop: Option<u32>,
    pub bar_fill: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UiElementGroupFramedData {
    pub scene: u32,
    pub name: String,
    pub elements: Vec<u32>,
    pub frame_element: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UiElementEffectAnimationData {
    pub layer: Option<u32>,
    pub m_finish_behavior: Option<u8>,
    pub total_number_of_frames: Option<f32>,
    pub m_per_pixel_uvs_x: Option<bool>,
    pub enabled: Option<bool>,
    pub frames_per_second: Option<f32>,
    pub number_of_frames_per_row_in_atlas: Option<f32>,
    pub texture_data: UiElementEffectAnimationDataTextureData,
    pub scene: u32,
    pub name: String,
    pub position: UiPositionRect,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UiElementGroupMeterData {
    pub tip_style: Option<UiElementGroupMeterDataTipStyle>,
    pub fill_direction: Option<u8>,
    pub elements: Vec<u32>,
    pub is_enabled: Option<bool>,
    pub bar_elements: Vec<u32>,
    pub start_percentage: Option<f32>,
    pub name: String,
    pub scene: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OptionItemGroup {
    pub label_tra_key: String,
    pub filter: Option<OptionItemGroupFilter>,
    pub items: Box<Vec<OptionItemGroupItems>>,
    pub live_update: Option<bool>,
    pub template: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xa9d60c77 {
    pub unk_0x3e58e16: u32,
    pub filter: OptionItemGroupFilter,
    pub column1_label_tra_key: String,
    pub unk_0xd7150c53: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LayoutStyleGrid {
    pub vertical_justification: Option<u8>,
    pub row_horizontal_alignment: Option<u8>,
    pub horizontal_justification: Option<u8>,
    pub row_vertical_alignment: Option<u8>,
    pub horizontal_fill_direction: Option<u8>,
    pub vertical_fill_direction: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xd31bbf89 {
    pub unk_0x69ae9918: Option<u32>,
    pub animation_name: Option<String>,
    pub unk_0x4a174502: Option<f32>,
    pub r#loop: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LooseUiTextureData {
    pub texture_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OptionItemResolutionDropdown {
    pub label_tra_key: String,
    pub template: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x65955db8 {
    pub unk_0xb02e568a: u32,
    pub unk_0xf60f3af2: u32,
    pub unk_0x2a781f11: u32,
    pub bounds_element: u32,
    pub unk_0xf0692ff1: u32,
    pub unk_0xd3d87bbf: u32,
    pub unk_0xcaacc388: u32,
    pub unk_0xf6b809cd: u32,
    pub unk_0x25b31bc: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UiElementIconData {
    pub flip_x: Option<bool>,
    pub texture_data: Option<UiElementEffectAnimationDataTextureData>,
    pub name: String,
    pub flip_y: Option<bool>,
    pub unk_0x8c262820: Option<u32>,
    pub drag_type: Option<UiElementIconDataDragType>,
    pub use_alpha: Option<bool>,
    pub block_input_events: Option<bool>,
    pub position: UiElementIconDataPosition,
    pub layer: Option<u32>,
    pub scene: u32,
    pub material: Option<u32>,
    pub per_pixel_uvs_x: Option<bool>,
    pub extension: Option<UiElementIconDataExtension>,
    pub color: Option<[u8; 4]>,
    pub enabled: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xd75bce6a {
    pub name: String,
    pub layer: u32,
    pub scene: u32,
    pub texture_data: LooseUiTextureData,
    pub position: UiPositionRect,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HudItemShopItemIconDefinition {
    pub ornn_frame: u32,
    pub item_icon: u32,
    pub cost_text: Option<u32>,
    pub hover_frame_icon: u32,
    pub offset_region: Option<u32>,
    pub purchaseable_vfx: Option<u32>,
    pub mythic_purchased_vfx: Option<u32>,
    pub mythic_frame_icon: u32,
    pub cost_text_unpurchaseable: Option<u32>,
    pub mythic_purchaseable_vfx: Option<u32>,
    pub hit_region: u32,
    pub popular_icon: Option<u32>,
    pub recently_changed_icon: Option<u32>,
    pub name_text: Option<u32>,
    pub locked_icon: Option<u32>,
    pub locked_hover_icon: Option<u32>,
    pub selected_vfx: Option<u32>,
    pub purchased_overlay: Option<u32>,
    pub cost_text_selected: Option<u32>,
    pub frame_icon: u32,
    pub selected_icon: u32,
    pub unpurchaseable_overlay: Option<u32>,
    pub hover_icon: Option<u32>,
    pub cooldown_effect_data: Option<CooldownEffectUiData>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OptionItemSliderVolume {
    pub label_tra_key: String,
    pub live_update: Option<bool>,
    pub mute_button_template: u32,
    pub template: u32,
    pub mute_option: u16,
    pub option: u16,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UiElementEffectCooldownData {
    pub m_effect_color0: [u8; 4],
    pub m_effect_color1: [u8; 4],
    pub layer: Option<u32>,
    pub position: UiPositionRect,
    pub scene: u32,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x9984b4a {
    pub unk_0xf60f3af2: u32,
    pub unk_0xd3d87bbf: u32,
    pub unk_0xf0692ff1: u32,
    pub unk_0xb02e568a: u32,
    pub bounds_element: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xd2fb821b {
    pub anchor: Vec2,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UiElementGroupManagedLayoutData {
    pub scene: u32,
    pub layout_style: UiElementGroupManagedLayoutDataLayoutStyle,
    pub region: u32,
    pub name: String,
    pub elements: Vec<u32>,
    pub unk_0x6e4f45c5: Option<bool>,
    pub ignore_disabled_elements: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UiPositionPolygon {
    pub ui_rect: UiElementRect,
    pub anchors: AnchorSingle,
    pub polygon_vertices: Vec<Vec2>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UiComboBoxDefinition {
    pub object_path: u32,
    pub dropdown_display_tra_key: Option<String>,
    pub dropdown_backdrop_element_data: u32,
    pub sound_events: Option<UiComboBoxSoundEvents>,
    pub dropdown_hover_element_data: u32,
    pub list_display_direction: Option<u8>,
    pub list_option_text_element_data: u32,
    pub button_definition: u32,
    pub selected_highlight_element_data: u32,
    pub list_option_hit_area_element_data: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UiElementEffectInstancedData {
    pub enabled: Option<bool>,
    pub m_per_pixel_uvs_x: Option<bool>,
    pub m_color: Option<[u8; 4]>,
    pub layer: u32,
    pub name: String,
    pub texture_data: Option<AtlasData>,
    pub position: UiPositionRect,
    pub scene: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OptionItemHotkeys {
    pub filter: Option<OptionItemGroupFilter>,
    pub template: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CooldownEffectUiData {
    pub radial_effect: u32,
    pub cooldown_text: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UiSceneViewPaneData {
    pub scroll_direction: Option<u8>,
    pub drag_region_element: Option<u32>,
    pub handle_input_during_pause: Option<bool>,
    pub parent_scene: Option<u32>,
    pub enabled: Option<bool>,
    pub scroll_region_element: Option<u32>,
    pub name: String,
    pub layer: u32,
    pub scissor_region_element: u32,
    pub slider: Option<u32>,
    pub scrolling_scene: u32,
    pub inherit_scissoring: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UiElementParticleSystemData {
    pub name: String,
    pub play_during_transition: Option<bool>,
    pub enabled: Option<bool>,
    pub texture_overrides: Option<HashMap<u32, String>>,
    pub vfx_adjustment_scale: Option<f32>,
    pub position: UiElementIconDataPosition,
    pub scene: u32,
    pub layer: Option<u32>,
    pub max_play_count: Option<u32>,
    pub vfx_system: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OptionItemFilterAnd {
    pub filters: Box<Vec<OptionItemGroupFilter>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UiSceneData {
    pub unk_0x49d8f2c4: Option<bool>,
    pub enabled: Option<bool>,
    pub scene_transition_out: Option<UiSceneDataSceneTransitionOut>,
    pub handle_input_during_pause: Option<bool>,
    pub name: String,
    pub layer: Option<u32>,
    pub parent_scene: Option<u32>,
    pub inherit_scissoring: Option<bool>,
    pub scene_transition_in: Option<UiSceneDataSceneTransitionOut>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HudItemShopRecItemCardDefinition {
    pub advice_empty_icon_non_mythic_hover: u32,
    pub bundle_item_frame_hover_icon: u32,
    pub advice_empty_icon_mythic_hover: u32,
    pub card_refresh_mythic_vfx: u32,
    pub advice_char_icon1: u32,
    pub card_selected_non_mythic: u32,
    pub unpurchaseable_overlay: u32,
    pub bundle_item_frame_icon: u32,
    pub bundle_item_frame_unpurchasable: u32,
    pub advice_empty_text_hover: u32,
    pub advice_icon_hover_non_mythic: u32,
    pub brief_text: u32,
    pub advice_char_border_hover1: u32,
    pub frame_icon: u32,
    pub hit_region: u32,
    pub advice_char_border1: u32,
    pub recently_changed_icon: u32,
    pub card_selected_mythic_vfx: u32,
    pub advice_icon_hover_mythic: u32,
    pub advice_char_border0: u32,
    pub bundle_item_primary_icon: u32,
    pub cost_text_unpurchaseable: u32,
    pub advice_char_icon0: u32,
    pub card_refresh_non_mythic_vfx: u32,
    pub advice_empty_text: u32,
    pub card_hover_mythic: u32,
    pub bundle_item_more_text: u32,
    pub ornn_frame: u32,
    pub purchased_overlay: u32,
    pub cost_text_selected: u32,
    pub advice_icon_default: u32,
    pub bundle_item_more_tag: u32,
    pub brief_text_backdrop: u32,
    pub advice_label: u32,
    pub bundle_stack_secondary_text: u32,
    pub hover_frame_icon: u32,
    pub advice_char_border_hover0: u32,
    pub card_default: u32,
    pub advice_empty_icon_default: u32,
    pub locked_hover_icon: u32,
    pub bundle_item_secondary_icon: u32,
    pub bundle_stack_primary_text: u32,
    pub cost_text: u32,
    pub card_selected_non_mythic_vfx: u32,
    pub card_hover_mythic_vfx: u32,
    pub locked_icon: u32,
    pub item_icon: u32,
    pub bundle_item_more_tag_hover: u32,
    pub card_selected_mythic: u32,
    pub card_hover_non_mythic_vfx: u32,
    pub card_hover_non_mythic: u32,
    pub name_text: u32,
    pub mythic_frame_icon: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UiElementEffectAnimatedRotatingIconData {
    pub number_of_frames_per_row_in_atlas: f32,
    pub scene: u32,
    pub name: String,
    pub frames_per_second: f32,
    pub total_number_of_frames: f32,
    pub position: UiPositionRect,
    pub texture_data: AtlasData,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UiElementGroupData {
    pub scene: u32,
    pub name: String,
    pub elements: Option<Vec<u32>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VoiceChatViewSelfSlot {
    pub connection_button: u32,
    pub name_text: u32,
    pub mic_volume_text: u32,
    pub mic_volume_slider_bar: u32,
    pub portrait: u32,
    pub mute_button: u32,
    pub halo: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UiElementEffectArcFillData {
    pub name: String,
    pub m_flip_x: Option<bool>,
    pub enabled: Option<bool>,
    pub scene: u32,
    pub layer: u32,
    pub position: UiPositionRect,
    pub texture_data: UiElementEffectAnimationDataTextureData,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OptionTemplateLabel {
    pub label1: u32,
    pub bounds_element: u32,
    pub label2: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OptionItemSecondaryHotkeys2Column {
    pub unk_0x98f86e0: Vec<OptionItemSecondaryHotkeys2ColumnRow>,
    pub template: u32,
    pub header: Option<OptionItemSecondaryHotkeys2ColumnHeader>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x30ed049a {
    pub position: u32,
    pub event_name_tra_key: String,
    pub unk_0xa7429aa5: bool,
    pub event_name: String,
    pub unk_0x1cb3d492: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TooltipViewController {
    pub unk_0x56716e4a: String,
    pub base_loadable: u32,
    pub tooltip_popup_delay_time: f32,
    pub unk_0xf0ae6ff1: Unk0xf0ae6ff1,
    pub per_locale_adjustments: HashMap<String, PerLocaleTooltipAdjustments>,
    pub tooltip_popup_timeout: f32,
    pub path_hash_to_self: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DoubleSidedTipStyle {
    pub directional_tip_elements: Vec<u32>,
    pub sliver: u32,
    pub reverse_directional_tip_elements: Vec<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LayoutStyleVerticalList {
    pub column_horizontal_alignment: Option<u8>,
    pub vertical_fill_direction: Option<u8>,
    pub vertical_justification: Option<u8>,
    pub horizontal_justification: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UiDraggableBasic {
    pub use_sticky_drag: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x165c0117 {
    pub filter: OptionItemGroupFilter,
    pub template: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UiElementEffectCircleMaskDesaturateData {
    pub position: UiPositionRect,
    pub name: String,
    pub scene: u32,
    pub enabled: Option<bool>,
    pub m_flip_x: Option<bool>,
    pub minimum_saturation: Option<f32>,
    pub layer: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VoiceChatViewController {
    pub self_slot: VoiceChatViewSelfSlot,
    pub connected_bg_region_handle: u32,
    pub unk_0x70f56833: Option<[u8; 4]>,
    pub player_grid: u32,
    pub player_slot_region_handle: u32,
    pub unk_0xe38d9a41: [u8; 4],
    pub player_slot_data: VoiceChatViewPlayerSlotData,
    pub error_text: u32,
    pub base_loadable: u32,
    pub backdrop: u32,
    pub panel_scene_handle: u32,
    pub path_hash_to_self: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UiDraggableElementGroupDrag {
    pub use_sticky_drag: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AnchorSingle {
    pub anchor: Vec2,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StatFilterDefinition {
    pub matching_categories: Option<Vec<u32>>,
    pub button_definition: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AtlasData3SliceV {
    pub m_texture_source_resolution_width: u32,
    pub m_texture_name: String,
    pub texture_us: Vec2,
    pub top_bottom_heights: Vec2,
    pub m_texture_source_resolution_height: u32,
    pub texture_vs: Vec4,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LooseUiTextureData3SliceH {
    pub texture_name: String,
    pub edge_sizes_left_right: Option<Vec2>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OptionItemSecondaryHotkeys1ColumnHeader {
    pub column1_label_tra_key: String,
    pub column0_label_tra_key: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OptionsTab {
    pub show_on: Option<u8>,
    pub unk_0xa4b002d7: Vec<OptionItemGroupItems>,
    pub filter: Option<OptionItemGroupFilter>,
    pub add_padding_after_last_item: Option<bool>,
    pub tab_name_tra_key: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x104afcda {
    pub scene: u32,
    pub layer: Option<u32>,
    pub position: UiElementIconDataPosition,
    pub name: String,
    pub unk_0x374b50fc: Option<HashMap<u32, u32>>,
    pub enabled: Option<bool>,
    pub unk_0x68dace96: Option<Unk0x104afcdaUnknownEnumField>,
    pub unk_0x2181f0dd: Option<bool>,
    pub material: Option<u32>,
    pub unk_0xb5985cf7: Vec<Unk0xd31bbf89>,
    pub unk_0xc31d847f: Option<bool>,
    pub unk_0x81c3985d: String,
    pub unk_0x4fc07890: Option<String>,
    pub unk_0x9edb827d: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x3a8e6763 {
    pub hotkey_button_text_small: u32,
    pub labels: Vec<Unk0x6a04facb>,
    pub hotkey_quick_cast_button_definition: u32,
    pub unk_0xd8b966a9: Vec<Unk0x30ed049a>,
    pub normal_cast_button_pos: u32,
    pub quick_cast_button_pos: u32,
    pub unk_0xbda33073: u32,
    pub hotkey_modifier_text: u32,
    pub bounds_element: u32,
    pub cast_all_button_definition: u32,
    pub hotkey_button_definition: u32,
    pub unk_0xb712cd3d: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UiElementGroupSliderSoundEvents {
    pub on_bar_clicked_event: String,
    pub on_drag_end_event: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UiElementEffectAmmoData {
    pub layer: u32,
    pub position: UiPositionRect,
    pub scene: u32,
    pub m_effect_color1: [u8; 4],
    pub name: String,
    pub m_effect_color0: [u8; 4],
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AtlasData3SliceH {
    pub m_texture_source_resolution_height: u32,
    pub m_texture_source_resolution_width: u32,
    pub left_right_widths: Vec2,
    pub texture_us: Vec4,
    pub m_texture_name: String,
    pub texture_vs: Vec2,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PerLocaleTooltipAdjustments {
    pub bottom_hr_y_pre_adjustment: Option<i32>,
    pub bottom_y_padding_adjustment: Option<i32>,
    pub bottom_hr_y_post_adjustment: Option<i32>,
    pub top_hr_y_post_adjustment: Option<i32>,
    pub title_y_adjustment: Option<i32>,
    pub top_hr_y_pre_adjustment: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VoiceChatViewPlayerSlotData {
    pub group: u32,
    pub portrait: u32,
    pub volume_text: u32,
    pub halo: u32,
    pub name_text: u32,
    pub mute_button: u32,
    pub volume_slider_bar: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GlowCenteredOverlayTipStyle {
    pub directional_tip_elements: Vec<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LooseUiTextureData9Slice {
    pub texture_name: String,
    pub edge_sizes_left_right: Option<Vec2>,
    pub edge_sizes_top_bottom: Option<Vec2>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UiElementMeterSkin {
    pub tip_elements: Vec<u32>,
    pub bar_elements: Vec<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UiElementEffectRotatingIconData {
    pub layer: u32,
    pub texture_data: UiElementEffectAnimationDataTextureData,
    pub enabled: Option<bool>,
    pub scene: u32,
    pub position: UiPositionRect,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UiElementGroupButtonState {
    pub display_element_list: Option<Vec<u32>>,
    pub text_element: Option<u32>,
    pub text_frame_element: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OptionTemplateHotkeysLabel {
    pub label: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AtlasData {
    pub m_texture_uv: Option<Vec4>,
    pub m_texture_name: String,
    pub m_texture_source_resolution_width: Option<u32>,
    pub m_texture_source_resolution_height: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UiDraggableSceneDrag {
    pub use_sticky_drag: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UiElementEffectDesaturateData {
    pub name: String,
    pub enabled: Option<bool>,
    pub minimum_saturation: Option<f32>,
    pub scene: u32,
    pub layer: Option<u32>,
    pub position: UiPositionRect,
    pub texture_data: Option<UiElementEffectAnimationDataTextureData>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OptionTemplateSecondaryHotkeysLabel {
    pub background_element: u32,
    pub text_element: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SceneScreenEdgeTransitionData {
    pub transition_start_delay_secs: Option<f32>,
    pub transition_time: Option<f32>,
    pub easing_type: Option<u8>,
    pub start_alpha: Option<u8>,
    pub edge: Option<u8>,
    pub end_alpha: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OptionItemSecondaryHotkeys1Column {
    pub filter: Option<OptionItemGroupFilter>,
    pub template: u32,
    pub unk_0x98f86e0: Vec<OptionItemSecondaryHotkeys1ColumnRow>,
    pub live_update: Option<bool>,
    pub header: OptionItemSecondaryHotkeys1ColumnHeader,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xf0ae6ff1 {
    pub scene: u32,
    pub post_script_title_element: u32,
    pub hr_top: u32,
    pub hr_bottom_sub_scene: u32,
    pub subtitle_right_element: u32,
    pub click_absorbing_region_element: u32,
    pub subtitle_left_element: u32,
    pub icon_overlay_element: u32,
    pub backdrop: u32,
    pub main_text_element: u32,
    pub title_right_element: u32,
    pub hr_top_sub_scene: u32,
    pub icon_element: u32,
    pub title_left_element: u32,
    pub hr_bottom: u32,
    pub post_script_right_element: u32,
    pub post_script_left_element: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OptionItemFilterFeatureToggle {
    pub toggle_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OptionItemSecondaryHotkeys2ColumnRow {
    pub event_name: String,
    pub filter: Option<OptionItemGroupFilter>,
    pub label_tra_key: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x81580a34 {
    pub template: u32,
    pub filter: OptionItemGroupFilter,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OptionTemplateGroup {
    pub bounds_element: u32,
    pub expand_button_definition: Option<u32>,
    pub post_group_padding_region: Option<u32>,
    pub label_element: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReconnectDialogViewController {
    pub base_loadable: u32,
    pub scene: u32,
    pub meter: u32,
    pub mobile_override: u32,
    pub path_hash_to_self: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LayoutStyleHorizontalList {
    pub vertical_justification: Option<u8>,
    pub row_vertical_alignment: Option<u8>,
    pub horizontal_justification: Option<u8>,
    pub horizontal_fill_direction: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OptionTemplateSlider {
    pub bounds_element: u32,
    pub slider_bar_definition: u32,
    pub label_element: u32,
    pub value_element: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UiElementEffectCircleMaskCooldownData {
    pub layer: u32,
    pub m_effect_color1: [u8; 4],
    pub scene: u32,
    pub position: UiPositionRect,
    pub m_effect_color0: [u8; 4],
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TooltipFormat {
    pub m_uses_list_values: Option<bool>,
    pub m_output_strings: HashMap<String, String>,
    pub m_list_styles: Option<HashMap<u32, String>>,
    pub m_list_grid_prefix: Option<String>,
    pub m_input_loc_keys_with_defaults: HashMap<String, String>,
    pub m_list_type_choices: Option<HashMap<String, String>>,
    pub m_list_grid_postfix: Option<String>,
    pub m_list_names: Option<Vec<String>>,
    pub m_list_value_separator: Option<String>,
    pub m_list_grid_separator: Option<String>,
    pub m_object_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UiDraggableProxyElementDrag {
    pub proxy_element: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StatFilterButtonDefinitions {
    pub mana: StatFilterDefinition,
    pub armor: StatFilterDefinition,
    pub ability_power: StatFilterDefinition,
    pub disable_stat_filters: StatFilterDefinition,
    pub attack_speed: StatFilterDefinition,
    pub life_steal_and_vamp: StatFilterDefinition,
    pub ability_haste: StatFilterDefinition,
    pub health: StatFilterDefinition,
    pub physical_damage: StatFilterDefinition,
    pub magic_resist: StatFilterDefinition,
    pub magic_penetration: StatFilterDefinition,
    pub move_speed: StatFilterDefinition,
    pub on_hit: StatFilterDefinition,
    pub armor_penetration: StatFilterDefinition,
    pub critical_strike: StatFilterDefinition,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OptionTemplateHotkeys {
    pub hotkey_button_text_small: u32,
    pub normal_cast_button_pos: u32,
    pub labels: Vec<OptionTemplateHotkeysLabel>,
    pub cast_all_button_definition: u32,
    pub hotkey_quick_cast_button_definition: u32,
    pub hotkey_button_definition: u32,
    pub hotkey_modifier_text: u32,
    pub unk_0xd8b966a9: Vec<OptionTemplateHotkeysKey>,
    pub quick_cast_button_pos: u32,
    pub bounds_element: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UiElementEffectLineData {
    pub m_thickness: f32,
    pub layer: u32,
    pub scene: u32,
    pub texture_data: AtlasData,
    pub position: UiPositionRect,
    pub name: String,
    pub enabled: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AtlasData9Slice {
    pub m_texture_source_resolution_width: u32,
    pub m_texture_name: String,
    pub texture_us: Vec4,
    pub top_bottom_heights: Vec2,
    pub left_right_widths: Vec2,
    pub m_texture_source_resolution_height: u32,
    pub texture_vs: Vec4,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OptionTemplateDropdown {
    pub bounds_element: u32,
    pub combo_box_definition: u32,
    pub label_element: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OptionItemFilterHwRequirement {
    pub requires_alienware: Option<bool>,
    pub requires_shader_model3: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UiElementRegionData {
    pub scene: u32,
    pub enabled: Option<bool>,
    pub name: String,
    pub layer: Option<u32>,
    pub position: Option<UiElementIconDataPosition>,
    pub drag_type: Option<UiElementIconDataDragType>,
    pub block_input_events: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UiElementRect {
    pub source_resolution_height: Option<u16>,
    pub size: Option<Vec2>,
    pub position: Option<Vec2>,
    pub source_resolution_width: Option<u16>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UiPropertyLoadable {
    pub filepath_hash: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OptionItemDropdownItem {
    pub tra_key: String,
    pub value: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OptionTemplateSecondaryHotkeys1Column {
    pub heading_row_label1: OptionTemplateSecondaryHotkeysLabel,
    pub row_button_column1: OptionTemplateSecondaryHotkeysButton,
    pub heading_row_label0: OptionTemplateSecondaryHotkeysLabel,
    pub bounds_element: u32,
    pub row_label_column0: OptionTemplateSecondaryHotkeysLabel,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OptionTemplateCheckbox {
    pub button_definition: u32,
    pub bounds_element: u32,
    pub label_element: u32,
    pub unk_0x60e0943: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OptionsViewController {
    pub tabs: Vec<u32>,
    pub base_loadable: u32,
    pub mobile_menu_button_tra_keys: Vec<String>,
    pub mobile_override_loadable: u32,
    pub path_hash_to_self: u64,
    pub korea_ratings_icon_element: u32,
    pub default_menu_button_tra_keys: Vec<String>,
    pub surrender_hit_region: u32,
    pub view_pane_link: u32,
    pub button2_definition: u32,
    pub button1_definition: u32,
    pub okay_hit_region: u32,
    pub tab_button_definition: u32,
    pub exit_hit_region: u32,
    pub cancel_hit_region: u32,
    pub last_item_padding: u32,
    pub restore_defaults_hit_region: u32,
    pub close_button_definition: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OptionTemplateSecondaryHotkeys2Column {
    pub row_label_column0: OptionTemplateSecondaryHotkeysLabel,
    pub bounds_element: u32,
    pub heading_row_label1: OptionTemplateSecondaryHotkeysLabel,
    pub heading_row_label2: OptionTemplateSecondaryHotkeysLabel,
    pub heading_row_label0: OptionTemplateSecondaryHotkeysLabel,
    pub row_button_column1: OptionTemplateSecondaryHotkeysButton,
    pub row_button_column2: OptionTemplateSecondaryHotkeysButton,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OptionItemButton {
    pub show_on_platform: u8,
    pub action: u16,
    pub template: u32,
    pub label_tra_key: String,
    pub filter: Option<OptionItemButtonFilter>,
    pub description_tra_key: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UiPositionRect {
    pub ui_rect: Option<UiElementRect>,
    pub anchors: Option<UiPositionRectAnchors>,
    pub disable_pixel_snapping_y: Option<bool>,
    pub disable_pixel_snapping_x: Option<bool>,
    pub unk_0x981fbd00: Option<bool>,
    pub disable_resolution_downscale: Option<bool>,
    pub ignore_global_scale: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LooseUiTextureData3SliceV {
    pub texture_name: String,
    pub edge_sizes_top_bottom: Vec2,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OptionItemLabel {
    pub template: u32,
    pub show_on_platform: Option<u8>,
    pub label1_tra_key: String,
    pub label2_tra_key: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SceneAlphaTransitionData {
    pub start_alpha: Option<u8>,
    pub end_alpha: Option<u8>,
    pub easing_type: Option<u8>,
    pub transition_time: Option<f32>,
    pub transition_start_delay_secs: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UiElementScissorRegionData {
    pub enabled: Option<bool>,
    pub scene_to_scissor: u32,
    pub name: String,
    pub position: UiPositionRect,
    pub layer: Option<u32>,
    pub scene: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OptionItemSliderFloat {
    pub live_update: Option<bool>,
    pub filter: Option<OptionItemGroupFilter>,
    pub scale: Option<f32>,
    pub option: u16,
    pub template: u32,
    pub update_on_drag: Option<bool>,
    pub show_on_platform: Option<u8>,
    pub label_tra_key: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x6a04facb {
    pub label: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OptionTemplateBorder {
    pub border: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x31bf21b0 {
    pub unk_0x25b31bc: u32,
    pub unk_0xcaacc388: u32,
    pub unk_0x2d1115b1: u32,
    pub unk_0x2a781f11: u32,
    pub bounds_element: u32,
    pub unk_0xbaa5364a: u32,
    pub unk_0xf6b809cd: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UiElementTextData {
    pub wrapping_mode: Option<u8>,
    pub layer: Option<u32>,
    pub unk_0xc1d0e91a: Option<f32>,
    pub text_alignment_vertical: Option<u8>,
    pub tra_key: Option<String>,
    pub block_input_events: Option<bool>,
    pub font_description: u32,
    pub html_style_sheet: Option<u32>,
    pub position: UiPositionRect,
    pub text_alignment_horizontal: Option<u8>,
    pub name: String,
    pub unk_0x6e4f45c5: Option<bool>,
    pub enabled: Option<bool>,
    pub icon_scale: Option<f32>,
    pub scene: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UiElementGroupSliderData {
    pub name: String,
    pub default_state: UiElementGroupSliderState,
    pub bar_hit_region: u32,
    pub slider_hovered_state: UiElementGroupSliderState,
    pub slider_clicked_state: UiElementGroupSliderState,
    pub scene: u32,
    pub direction: Option<u8>,
    pub sound_events: Option<UiElementGroupSliderSoundEvents>,
    pub is_enabled: Option<bool>,
    pub slider_hit_region: u32,
    pub bar_hovered_state: UiElementGroupSliderState,
    pub elements: Vec<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UiElementEffectCooldownRadialData {
    pub texture_data: UiElementEffectAnimationDataTextureData,
    pub layer: Option<u32>,
    pub position: UiPositionRect,
    pub name: String,
    pub m_is_fill: Option<bool>,
    pub enabled: Option<bool>,
    pub scene: u32,
    pub m_flip_x: Option<bool>,
    pub m_flip_y: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OptionItemSecondaryHotkeys1ColumnRow {
    pub event_name: String,
    pub label_tra_key: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x35317d3f {
    pub enabled: Option<bool>,
    pub layer: u32,
    pub scrolling_scene: u32,
    pub slider: u32,
    pub buffer_region_element: u32,
    pub name: String,
    pub drag_region_element: u32,
    pub scissor_region_element: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UiElementEffectFillPercentageData {
    pub scene: u32,
    pub m_per_pixel_uvs_x: bool,
    pub layer: u32,
    pub name: String,
    pub position: UiPositionRect,
    pub enabled: Option<bool>,
    pub texture_data: AtlasData,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UiElementEffectGlowingRotatingIconData {
    pub name: String,
    pub position: UiPositionRect,
    pub brightness_mod: f32,
    pub scene: u32,
    pub texture_data: AtlasData,
    pub cycle_time: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OptionItemFilterNot {
    pub filter: Box<OptionItemGroupFilter>,
    pub unk_0x93bef4c5: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OptionTemplateHotkeysKey {
    pub event_name: String,
    pub position: u32,
    pub event_name_tra_key: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OptionItemBorder {
    pub unk_0xa4b002d7: Box<Vec<OptionItemGroupItems>>,
    pub template: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OptionItemFilterMutator {
    pub mutator: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OptionTemplateMuteButton {
    pub button_definition: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OptionItemColumns {
    pub items_right: Box<Option<Vec<OptionItemGroupItems>>>,
    pub filter: Option<OptionItemFilterGameStyle>,
    pub items_left: Box<Option<Vec<OptionItemGroupItems>>>,
    pub items_either: Box<Option<Vec<OptionItemGroupItems>>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UiElementGroupButtonSoundEvents {
    pub mouse_down_event: Option<String>,
    pub mouse_up_event: Option<String>,
    pub mouse_up_selected: Option<String>,
    pub mouse_down_selected: Option<String>,
    pub roll_over_event: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OptionItemVoiceInputDeviceDropdown {
    pub label_tra_key: String,
    pub template: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OptionTemplateButton {
    pub button_definition: u32,
    pub bounds_element: u32,
    pub unk_0x244362ff: u32,
    pub description_definition: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OptionItemFilterGameStyle {
    pub unk_0x64bc3430: Option<bool>,
    pub show_in_lol_game: Option<bool>,
    pub show_in_tft_pregame: Option<bool>,
    pub show_in_tft_replay: Option<bool>,
    pub unk_0x47600fd: Option<bool>,
    pub show_in_tft_game: Option<bool>,
    pub show_in_lol_replay: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UiComboBoxSoundEvents {
    pub on_selection_event: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OptionItemSliderGraphicsQuality {
    pub template: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UiElementEffectGlowData {
    pub scene: u32,
    pub base_scale: f32,
    pub position: UiPositionRect,
    pub layer: u32,
    pub minimum_alpha: Option<f32>,
    pub name: String,
    pub texture_data: UiElementEffectAnimationDataTextureData,
    pub cycle_time: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OptionTemplateSecondaryHotkeysButton {
    pub button_definition: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OptionItemDropdown {
    pub option: u16,
    pub label_tra_key: String,
    pub unk_0xa4b002d7: Vec<OptionItemDropdownItem>,
    pub template: u32,
    pub filter: Option<OptionItemGroupFilter>,
    pub live_update: Option<bool>,
    pub tooltip_tra_key: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x9ef1e737 {
    pub template: u32,
    pub filter: OptionItemFilterGameStyle,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OptionItemSecondaryHotkeys2ColumnHeader {
    pub column2_label_tra_key: String,
    pub column1_label_tra_key: String,
    pub column0_label_tra_key: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UiElementGroupButtonData {
    pub elements: Vec<u32>,
    pub is_focusable: Option<bool>,
    pub inactive_selected_state_elements: Option<UiElementGroupButtonState>,
    pub selected_clicked_state_elements: Option<UiElementGroupButtonState>,
    pub selected_hover_state_elements: Option<UiElementGroupButtonState>,
    pub hover_state_elements: Option<UiElementGroupButtonState>,
    pub clicked_state_elements: Option<UiElementGroupButtonState>,
    pub selected_state_elements: Option<UiElementGroupButtonState>,
    pub add_text_size_to_hit_region: Option<bool>,
    pub is_selected: Option<bool>,
    pub name: String,
    pub inactive_state_elements: Option<UiElementGroupButtonState>,
    pub inactive_tooltip_tra_key: Option<String>,
    pub hit_region_element: u32,
    pub click_release_particle_element: Option<u32>,
    pub default_state_elements: Option<UiElementGroupButtonState>,
    pub active_tooltip_tra_key: Option<String>,
    pub is_enabled: Option<bool>,
    pub scene: u32,
    pub sound_events: Option<UiElementGroupButtonSoundEvents>,
    pub tab_order: Option<u32>,
    pub is_active: Option<bool>,
    pub selected_tooltip_tra_key: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UiPropertyOverrideLoadable {
    pub filepath_hash: u64,
    pub override_src_folder: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OptionItemCheckbox {
    pub live_update: Option<bool>,
    pub negate: Option<bool>,
    pub label_tra_key: String,
    pub option: u16,
    pub filter: Option<OptionItemGroupFilter>,
    pub tooltip_tra_key: Option<String>,
    pub template: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OptionItemSliderInt {
    pub label_tra_key: String,
    pub live_update: bool,
    pub option: u16,
    pub option_scale: Option<u32>,
    pub template: u32,
}
