use std::collections::HashMap;

use bevy::asset::Asset;
use bevy::math::{Mat4, Vec2, Vec3, Vec4};
use bevy::reflect::TypePath;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AbilityResourceBarData {
    pub ability_resource_bars: ResourceMeterIconData,
    pub backdrop: Option<u32>,
    pub standard_tick: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AbilityResourceByCoefficientCalculationPart {
    pub m_ability_resource: Option<u8>,
    pub m_coefficient: f32,
    pub m_stat_formula: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AbilityResourceDynamicMaterialFloatDriver {
    pub slot: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AbilityResourcePipSpacerTypeMap {
    pub additional_pip_spacer_types: HashMap<u32, u32>,
    pub default_pip_spacer: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AbilityResourcePipTypeMap {
    pub additional_pip_types: HashMap<u32, u32>,
    pub default_empty_pip: u32,
    pub default_large_pip: u32,
    pub default_medium_pip: u32,
    pub default_small_pip: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AbilityResourcePipsData {
    pub backdrop: u32,
    pub pip_spacer: AbilityResourcePipSpacerTypeMap,
    pub pips: AbilityResourcePipTypeMap,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AbilityResourceSlotInfo {
    pub ar_allow_max_value_to_be_overridden: Option<bool>,
    pub ar_base: Option<f32>,
    pub ar_base_factor_regen: Option<f32>,
    pub ar_base_static_regen: Option<f32>,
    pub ar_display_as_pips: Option<bool>,
    pub ar_has_regen_text: Option<bool>,
    pub ar_increments: Option<f32>,
    pub ar_is_shown: Option<bool>,
    pub ar_max_segments: Option<i32>,
    pub ar_negative_spacer: Option<bool>,
    pub ar_override_empty_pip_name: Option<String>,
    pub ar_override_large_pip_name: Option<String>,
    pub ar_override_medium_pip_name: Option<String>,
    pub ar_override_small_pip_name: Option<String>,
    pub ar_override_spacer_name: Option<String>,
    pub ar_per_level: Option<f32>,
    pub ar_regen_per_level: Option<f32>,
    pub ar_type: Option<u8>,
    pub hide_empty_pips: Option<bool>,
    pub unk_0x4eb6a404: Option<u8>,
    pub visibility_flags: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AcceleratingMovement {
    pub m_acceleration: Option<f32>,
    pub m_initial_speed: Option<f32>,
    pub m_max_speed: f32,
    pub m_min_speed: Option<f32>,
    pub m_offset_initial_target_height: Option<f32>,
    pub m_project_target_to_cast_range: Option<bool>,
    pub m_start_bone_name: Option<String>,
    pub m_start_bone_skin_overrides: Option<HashMap<u32, String>>,
    pub m_target_bone_name: Option<String>,
    pub m_target_height_augment: Option<f32>,
    pub m_tracks_target: Option<bool>,
    pub m_use_ground_height_at_target: Option<bool>,
    pub m_use_height_offset_at_end: Option<bool>,
    pub m_visuals_track_hidden_targets: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AiSpellData {
    pub m_block_level: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AllTrueMaterialDriver {
    pub m_drivers: Option<Vec<Box<EnumDriver>>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AnchorDouble {
    pub anchor_left: Option<Vec2>,
    pub anchor_right: Option<Vec2>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AnchorSingle {
    pub anchor: Vec2,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AnimationFractionDynamicMaterialFloatDriver {
    pub m_animation_name: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Asset, TypePath)]
#[serde(rename_all = "camelCase")]
pub struct AnimationGraphData {
    pub m_blend_data_table: Option<HashMap<u64, EnumBlendData>>,
    pub m_cascade_blend_value: Option<f32>,
    pub m_clip_data_map: Option<HashMap<u32, EnumClipData>>,
    pub m_mask_data_map: Option<HashMap<u32, MaskData>>,
    pub m_sync_group_data_map: Option<HashMap<u32, SyncGroupData>>,
    pub m_track_data_map: HashMap<u32, TrackData>,
    pub m_use_cascade_blend: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AnimationResourceData {
    pub m_animation_file_path: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AtlasData {
    pub m_texture_name: String,
    pub m_texture_source_resolution_height: Option<u32>,
    pub m_texture_source_resolution_width: Option<u32>,
    pub m_texture_uv: Option<Vec4>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AtlasData3SliceH {
    pub left_right_widths: Vec2,
    pub m_texture_name: String,
    pub m_texture_source_resolution_height: u32,
    pub m_texture_source_resolution_width: u32,
    pub texture_us: Vec4,
    pub texture_vs: Vec2,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AtlasData3SliceV {
    pub m_texture_name: String,
    pub m_texture_source_resolution_height: u32,
    pub m_texture_source_resolution_width: u32,
    pub texture_us: Vec2,
    pub texture_vs: Vec4,
    pub top_bottom_heights: Vec2,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AtlasData9Slice {
    pub left_right_widths: Vec2,
    pub m_texture_name: String,
    pub m_texture_source_resolution_height: u32,
    pub m_texture_source_resolution_width: u32,
    pub texture_us: Vec4,
    pub texture_vs: Vec4,
    pub top_bottom_heights: Vec2,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AtomicClipData {
    pub accessorylist: Option<Vec<KeyFrameFloatMapClipAccessoryData>>,
    pub end_frame: Option<f32>,
    pub m_animation_interruption_group_names: Option<Vec<u32>>,
    pub m_animation_resource_data: AnimationResourceData,
    pub m_event_data_map: Option<HashMap<u32, EnumEventData>>,
    pub m_flags: Option<u32>,
    pub m_mask_data_name: Option<u32>,
    pub m_sync_group_data_name: Option<u32>,
    pub m_tick_duration: Option<f32>,
    pub m_track_data_name: u32,
    pub m_updater_resource_data: Option<UpdaterResourceData>,
    pub start_frame: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AttackEvents {
    pub roll_for_critical_hit_result: bool,
    pub trigger_launch_attack: bool,
    pub trigger_once_per_enemy_of_parent: bool,
    pub trigger_once_per_parent: bool,
    pub trigger_only_once: bool,
    pub trigger_pre_attack: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AttackSlotData {
    pub m_attack_cast_time: Option<f32>,
    pub m_attack_delay_cast_offset_percent: Option<f32>,
    pub m_attack_delay_cast_offset_percent_attack_speed_ratio: Option<f32>,
    pub m_attack_name: Option<String>,
    pub m_attack_probability: Option<f32>,
    pub m_attack_total_time: Option<f32>,
    pub m_override_autoattack_cast_time: Option<OverrideAutoAttackCastTimeData>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BankUnit {
    pub asynchrone: Option<bool>,
    pub bank_path: Option<Vec<String>>,
    pub events: Option<Vec<String>>,
    pub name: String,
    pub voice_over: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BarTypeMap {
    pub additional_bar_types: Option<HashMap<u32, u32>>,
    pub default_bar: u32,
    pub min_display_percent_override: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Asset, TypePath)]
#[serde(rename_all = "camelCase")]
pub struct BarracksConfig {
    pub exp_radius: f32,
    pub gold_radius: f32,
    pub initial_spawn_time_secs: f32,
    pub minion_spawn_interval_secs: f32,
    pub move_speed_increase_increment: i32,
    pub move_speed_increase_initial_delay_secs: f32,
    pub move_speed_increase_interval_secs: f32,
    pub move_speed_increase_max_times: i32,
    pub units: Vec<BarracksMinionConfig>,
    pub upgrade_interval_secs: f32,
    pub upgrades_before_late_game_scaling: i32,
    pub wave_spawn_interval_secs: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BarracksMinionConfig {
    pub minion_type: u8,
    pub minion_upgrade_stats: MinionUpgradeConfig,
    pub unk_0xfee040bc: u32,
    pub wave_behavior: EnumWaveBehavior,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BlendingSwitchMaterialDriver {
    pub m_blend_time: Option<f32>,
    pub m_default_value: Box<EnumDriver>,
    pub m_elements: Vec<Box<SwitchMaterialDriverElement>>,
    pub m_override_blend_times: Option<Vec<f32>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BotsSpellData {
    pub damage_tag: Option<u32>,
    pub unk_0x38382c53: Option<Vec<Unk0x150d1b92>>,
    pub unk_0x591f8423: Option<f32>,
    pub unk_0x6d548702: Option<GameCalculation>,
    pub unk_0xec17e271: Option<Vec<Unk0xb09016f6>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Breakpoint {
    pub m_additional_bonus_at_this_level: Option<f32>,
    pub m_bonus_per_level_at_and_after: Option<f32>,
    pub m_level: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BuffCounterByCoefficientCalculationPart {
    pub m_buff_name: u32,
    pub m_coefficient: f32,
    pub m_icon_key: Option<String>,
    pub m_scaling_tag_key: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BuffCounterByNamedDataValueCalculationPart {
    pub m_buff_name: u32,
    pub m_data_value: u32,
    pub m_icon_key: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BuffCounterDynamicMaterialFloatDriver {
    pub m_script_name: Option<String>,
    pub spell: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BuffData {
    pub can_timeout_while_casting: Option<bool>,
    pub m_buff_attribute_flag: Option<u8>,
    pub m_description: Option<String>,
    pub m_float_vars_decimals: Option<Vec<i32>>,
    pub m_show_accumulated_duration: Option<bool>,
    pub m_show_duration: Option<bool>,
    pub m_tooltip_data: Option<TooltipInstanceBuff>,
    pub persistent_effect_conditions: Option<Vec<PersistentEffectConditionData>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ByCharLevelBreakpointsCalculationPart {
    pub m_breakpoints: Option<Vec<Breakpoint>>,
    pub m_initial_bonus_per_level: Option<f32>,
    pub m_level1_value: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ByCharLevelFormulaCalculationPart {
    pub m_values: Vec<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ByCharLevelInterpolationCalculationPart {
    pub m_end_value: Option<f32>,
    pub m_scale_by_stat_progression_multiplier: Option<bool>,
    pub m_scale_past_default_max_level: Option<bool>,
    pub m_start_value: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Cast {
    pub roll_for_critical_hit_result: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CastOnMovementComplete {
    pub roll_for_critical_hit_result: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CcBehaviorData {
    pub cc_behavior: TargetingPriorityList,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CensoredImage {
    pub image: String,
    pub uncensored_images: Option<HashMap<u32, String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChangeMissileSpeed {
    pub m_speed_change_type: Option<u32>,
    pub m_speed_value: f32,
    pub trigger_only_once: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChangeMissileWidth {
    pub width_change_type: u32,
    pub width_value: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CharacterHealthBarDataRecord {
    pub alpha_out_while_untargetable: Option<bool>,
    pub attach_to_bone: Option<String>,
    pub character_state_indicator_max_count: Option<u32>,
    pub hp_per_tick: Option<f32>,
    pub show_character_state_indicator_to_allies: Option<bool>,
    pub show_character_state_indicator_to_enemies: Option<bool>,
    pub show_while_untargetable: Option<bool>,
    pub unit_health_bar_style: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CharacterLevelRequirement {
    pub m_level: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CharacterPassiveData {
    pub m_allow_on_clones: Option<bool>,
    pub m_child_spells: Option<Vec<u32>>,
    pub m_component_buffs: Option<Vec<u32>>,
    pub m_display_flags: Option<u8>,
    pub m_parent_passive_buff: u32,
    pub skin_filter: Option<SkinFilterData>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Asset, TypePath)]
#[serde(rename_all = "camelCase")]
pub struct CharacterRecord {
    pub acquisition_range: Option<f32>,
    pub ally_champ_specific_health_suffix: Option<String>,
    pub area_indicator_max_distance: Option<f32>,
    pub area_indicator_min_distance: Option<f32>,
    pub area_indicator_min_radius: Option<f32>,
    pub area_indicator_radius: Option<f32>,
    pub area_indicator_target_distance: Option<f32>,
    pub area_indicator_texture_size: Option<f32>,
    pub armor_per_level: Option<f32>,
    pub attack_auto_interrupt_percent: Option<f32>,
    pub attack_range: Option<f32>,
    pub attack_speed: Option<f32>,
    pub attack_speed_per_level: Option<f32>,
    pub attack_speed_ratio: Option<f32>,
    pub base_armor: Option<f32>,
    pub base_crit_chance: Option<f32>,
    pub base_damage: Option<f32>,
    pub base_factor_hp_regen: Option<f32>,
    pub base_hp: Option<f32>,
    pub base_move_speed: Option<f32>,
    pub base_spell_block: Option<f32>,
    pub base_static_hp_regen: Option<f32>,
    pub basic_attack: Option<AttackSlotData>,
    pub char_audio_name_override: Option<String>,
    pub character_tool_data: Option<CharacterToolData>,
    pub crit_attacks: Option<Vec<AttackSlotData>>,
    pub crit_damage_multiplier: Option<f32>,
    pub crit_per_level: Option<f32>,
    pub critical_attack: Option<String>,
    pub damage_per_level: Option<f32>,
    pub death_event_listening_radius: Option<f32>,
    pub death_time: Option<f32>,
    pub disabled_target_laser_effects: Option<TargetLaserComponentEffects>,
    pub disguise_minimap_icon_override: Option<String>,
    pub enemy_champ_specific_health_suffix: Option<String>,
    pub enemy_tooltip: Option<String>,
    pub evolution_data: Option<EvolutionDescription>,
    pub exp_given_on_death: Option<f32>,
    pub experience_radius: Option<f32>,
    pub extra_attacks: Option<Vec<AttackSlotData>>,
    pub extra_spells: Option<Vec<String>>,
    pub first_acquisition_range: Option<f32>,
    pub flags: Option<u32>,
    pub friendly_tooltip: Option<String>,
    pub friendly_ux_override_exclude_tags_string: Option<String>,
    pub friendly_ux_override_include_tags_string: Option<String>,
    pub friendly_ux_override_team: Option<u32>,
    pub global_exp_given_on_death: Option<f32>,
    pub global_gold_given_on_death: Option<f32>,
    pub gold_given_on_death: Option<f32>,
    pub gold_radius: Option<f32>,
    pub health_bar_full_parallax: Option<bool>,
    pub health_bar_height: Option<f32>,
    pub highlight_healthbar_icons: Option<bool>,
    pub hit_fx_scale: Option<f32>,
    pub hover_indicator_minimap_override: Option<String>,
    pub hover_indicator_radius: Option<f32>,
    pub hover_indicator_radius_minimap: Option<f32>,
    pub hover_indicator_rotate_to_player: Option<bool>,
    pub hover_indicator_texture_name: Option<String>,
    pub hover_line_indicator_base_texture_name: Option<String>,
    pub hover_line_indicator_target_texture_name: Option<String>,
    pub hover_line_indicator_width: Option<f32>,
    pub hover_line_indicator_width_minimap: Option<f32>,
    pub hp_per_level: Option<f32>,
    pub hp_regen_per_level: Option<f32>,
    pub joint_for_anim_adjusted_selection: Option<String>,
    pub launch_area_data: Option<LaunchAreaData>,
    pub local_exp_given_on_death: Option<f32>,
    pub local_gold_given_on_death: Option<f32>,
    pub local_gold_split_with_last_hitter: Option<bool>,
    pub m_abilities: Option<Vec<u32>>,
    pub m_ability_slot_cc: Option<Vec<i32>>,
    pub m_adaptive_force_to_ability_power_weight: Option<f32>,
    pub m_character_calculations: Option<HashMap<u32, GameCalculation>>,
    pub m_character_name: String,
    pub m_character_passive_buffs: Option<Vec<CharacterPassiveData>>,
    pub m_character_passive_spell: Option<u32>,
    pub m_client_side_item_inventory: Option<Vec<u32>>,
    pub m_education_tool_data: Option<ToolEducationData>,
    pub m_fallback_character_name: Option<String>,
    pub m_perk_replacements: Option<PerkReplacementList>,
    pub m_preferred_perk_style: Option<u32>,
    pub m_use_cc_animations: Option<bool>,
    pub minimap_icon_override: Option<String>,
    pub minion_flags: Option<u32>,
    pub minion_score_value: Option<f32>,
    pub name: Option<String>,
    pub occluded_unit_selectable_distance: Option<f32>,
    pub on_kill_event: Option<u32>,
    pub on_kill_event_for_spectator: Option<u32>,
    pub on_kill_event_steal: Option<u32>,
    pub outline_b_box_expansion: Option<f32>,
    pub override_gameplay_collision_radius: Option<f32>,
    pub pack_manager_data: Option<PackManagerData>,
    pub par_name: Option<String>,
    pub passive1_icon_name: Option<String>,
    pub passive_lua_name: Option<String>,
    pub passive_name: Option<String>,
    pub passive_range: Option<f32>,
    pub passive_spell: Option<String>,
    pub passive_tool_tip: Option<String>,
    pub pathfinding_collision_radius: Option<f32>,
    pub perception_bounding_box_size: Option<Vec3>,
    pub perception_bubble_radius: Option<f32>,
    pub platform_enabled: Option<bool>,
    pub primary_ability_resource: Option<AbilityResourceSlotInfo>,
    pub purchase_identities: Option<Vec<u32>>,
    pub rec_spell_rank_up_info_list: Option<RecSpellRankUpInfoList>,
    pub record_as_ward: Option<bool>,
    pub secondary_ability_resource: Option<AbilityResourceSlotInfo>,
    pub selection_height: Option<f32>,
    pub selection_radius: Option<f32>,
    pub self_cb_champ_specific_health_suffix: Option<String>,
    pub self_champ_specific_health_suffix: Option<String>,
    pub significance: Option<f32>,
    pub silhouette_attachment_anim: Option<String>,
    pub spell_block_per_level: Option<f32>,
    pub spell_level_up_info: Option<Vec<SpellLevelUpInfo>>,
    pub spell_names: Option<Vec<String>>,
    pub spells: Option<Vec<u32>>,
    pub target_laser_effects: Option<TargetLaserComponentEffects>,
    pub tower_targeting_priority_boost: Option<f32>,
    pub treat_auto_attacks_as_normal_spells: Option<TreatAutoAttacksAsNormalSpells>,
    pub unit_tags_string: Option<String>,
    pub unk_0x3f975e4a: Option<bool>,
    pub unk_0x43135375: Option<f32>,
    pub unk_0x6854087e: Option<Vec<Unk0x47f13ab0>>,
    pub unk_0x9836cd87: Option<u8>,
    pub unk_0xc1984296: Option<Vec<u32>>,
    pub unk_0xc5c48b41: Option<u8>,
    pub unk_0xdd661aab: Option<Unk0x280745b1>,
    pub untargetable_spawn_time: Option<f32>,
    pub use_riot_relationships: Option<bool>,
    pub useable_data: Option<UseableData>,
    pub wake_up_range: Option<f32>,
    pub weapon_materials: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CharacterToolData {
    pub alternate_forms: Option<Vec<ToolAlternateForm>>,
    pub attack_rank: Option<i32>,
    pub attack_speed: Option<f32>,
    pub base_attack_speed_bonus: Option<f32>,
    pub base_spell_effectiveness: Option<f32>,
    pub bot_default_spell1: Option<String>,
    pub bot_default_spell2: Option<String>,
    pub bot_enabled: Option<bool>,
    pub bot_enabled_mm: Option<bool>,
    pub cast_shadows: Option<bool>,
    pub champion_id: Option<i32>,
    pub chasing_attack_range_percent: Option<f32>,
    pub classification: Option<String>,
    pub defense_rank: Option<i32>,
    pub description: Option<String>,
    pub difficulty_rank: Option<i32>,
    pub inherits: Option<ToolInheritsData>,
    pub level_spell_effectiveness: Option<f32>,
    pub lore2: Option<String>,
    pub magic_rank: Option<i32>,
    pub map_ai_presence: Option<HashMap<u32, ToolAiPresence>>,
    pub par_fade_color: Option<String>,
    pub pass_lev1_desc: Option<Vec<String>>,
    pub passive_data: Option<Vec<ToolPassiveData>>,
    pub post_attack_move_delay: Option<f32>,
    pub rec_items: Option<Vec<String>>,
    pub roles: Option<String>,
    pub search_tags: Option<String>,
    pub search_tags_secondary: Option<String>,
    pub soul_given_on_death: Option<f32>,
    pub sound: Option<ToolSoundData>,
    pub spell_data: Option<Vec<ToolSpellDesc>>,
    pub tips3: Option<String>,
    pub tutorial_rec_items: Option<Vec<String>>,
    pub unk_0xaa75da9d: Option<bool>,
    pub weapon_material_crit: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CircleMovement {
    pub m_angular_velocity: Option<f32>,
    pub m_lifetime: f32,
    pub m_linear_velocity: Option<f32>,
    pub m_start_bone_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ClampSubPartsCalculationPart {
    pub m_ceiling: Option<f32>,
    pub m_floor: Option<f32>,
    pub m_subparts: Vec<Box<EnumAbilityResourceByCoefficientCalculationPart>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ClearTargetAndKeepMoving {
    pub let_server_drive_target_position: Option<bool>,
    pub m_override_height_augment: Option<f32>,
    pub m_override_movement: Option<FixedSpeedMovement>,
    pub m_override_range: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ColorChooserMaterialDriver {
    pub m_bool_driver: Box<EnumDriver>,
    pub m_color_off: Option<Vec4>,
    pub m_color_on: Option<Vec4>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ColorGraphMaterialDriver {
    pub colors: VfxAnimatedColorVariableData,
    pub driver: Box<EnumDriver>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConditionBoolClipData {
    pub dont_stomp_transition_clip: Option<bool>,
    pub m_change_animation_mid_play: Option<bool>,
    pub m_child_anim_delay_switch_time: Option<f32>,
    pub m_false_condition_clip_name: u32,
    pub m_flags: Option<u32>,
    pub m_play_anim_change_from_beginning: Option<bool>,
    pub m_true_condition_clip_name: u32,
    pub sync_frame_on_change_anim: Option<bool>,
    pub updater: EnumParametricUpdater,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConditionFloatClipData {
    pub dont_stomp_transition_clip: Option<bool>,
    pub m_change_animation_mid_play: Option<bool>,
    pub m_child_anim_delay_switch_time: Option<f32>,
    pub m_condition_float_pair_data_list: Vec<ConditionFloatPairData>,
    pub m_flags: Option<u32>,
    pub m_play_anim_change_from_beginning: Option<bool>,
    pub sync_frame_on_change_anim: Option<bool>,
    pub updater: EnumParametricUpdater,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConditionFloatPairData {
    pub m_clip_name: u32,
    pub m_hold_animation_to_higher: Option<f32>,
    pub m_hold_animation_to_lower: Option<f32>,
    pub m_value: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConformToPathEventData {
    pub m_blend_in_time: Option<f32>,
    pub m_blend_out_time: Option<f32>,
    pub m_end_frame: Option<f32>,
    pub m_fire_if_animation_ends_early: Option<bool>,
    pub m_is_self_only: Option<bool>,
    pub m_mask_data_name: u32,
    pub m_name: Option<u32>,
    pub m_start_frame: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConformToPathRigPoseModifierData {
    pub activation_angle: Option<f32>,
    pub activation_distance: Option<f32>,
    pub blend_distance: Option<f32>,
    pub m_damping_value: Option<f32>,
    pub m_default_mask_name: Option<u32>,
    pub m_ending_joint_name: u32,
    pub m_frequency: Option<f32>,
    pub m_max_bone_angle: Option<f32>,
    pub m_starting_joint_name: u32,
    pub m_vel_multiplier: Option<f32>,
    pub only_activate_in_turns: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConstantWaveBehavior {
    pub spawn_count: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CursorData {
    pub m_texture_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CurveTheDifferenceHeightSolver {
    pub m_initial_target_height_offset: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CustomTargeterDefinitions {
    pub m_targeter_definitions: Vec<EnumTargeterDefinition>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DecelToLocationMovement {
    pub m_acceleration: f32,
    pub m_initial_speed: f32,
    pub m_max_speed: f32,
    pub m_min_speed: f32,
    pub m_project_target_to_cast_range: bool,
    pub m_start_bone_name: Option<String>,
    pub m_target_bone_name: Option<String>,
    pub m_target_height_augment: Option<f32>,
    pub m_tracks_target: bool,
    pub m_use_height_offset_at_end: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Defaultvisibility {
    pub m_perception_bubble_radius: Option<f32>,
    pub m_target_controls_visibility: Option<bool>,
    pub m_visible_to_owner_team_only: Option<bool>,
    pub trail_time_to_consider_for_visibility: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DelayStart {
    pub m_delay_time: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DelayedBoolMaterialDriver {
    pub m_bool_driver: Box<EnumDriver>,
    pub m_delay_off: Option<f32>,
    pub m_delay_on: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DestroyOnMovementComplete {
    pub m_delay: Option<i32>,
    pub render_particles_after_destroy: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DistanceToPlayerMaterialFloatDriver {
    pub max_distance: f32,
    pub min_distance: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DrawablePositionLocator {
    pub angle_offset_radian: Option<f32>,
    pub base_position: Option<u32>,
    pub distance_offset: Option<f32>,
    pub orientation_type: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DynamicMaterialDef {
    pub parameters: Option<Vec<DynamicMaterialParameterDef>>,
    pub static_switch: Option<DynamicMaterialStaticSwitch>,
    pub textures: Option<Vec<DynamicMaterialTextureSwapDef>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DynamicMaterialParameterDef {
    pub driver: EnumDriver,
    pub enabled: Option<bool>,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DynamicMaterialStaticSwitch {
    pub driver: EnumDriver,
    pub enabled: Option<bool>,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DynamicMaterialTextureSwapDef {
    pub enabled: Option<bool>,
    pub name: String,
    pub options: Option<Vec<DynamicMaterialTextureSwapOption>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DynamicMaterialTextureSwapOption {
    pub driver: EnumDriver,
    pub texture_name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EffectValueCalculationPart {
    pub m_effect_index: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EnableLookAtEventData {
    pub m_enable_look_at: Option<bool>,
    pub m_end_frame: Option<f32>,
    pub m_lock_current_values: Option<bool>,
    pub m_name: Option<u32>,
    pub m_start_frame: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EnterFowVisibility {
    pub m_missile_client_exit_fow_prediction: Option<bool>,
    pub m_missile_client_wait_for_target_update_before_missile_show: Option<bool>,
    pub m_perception_bubble_radius: Option<f32>,
    pub m_target_controls_visibility: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EnumAbilityResourceByCoefficientCalculationPart {
    AbilityResourceByCoefficientCalculationPart(AbilityResourceByCoefficientCalculationPart),
    BuffCounterByCoefficientCalculationPart(BuffCounterByCoefficientCalculationPart),
    BuffCounterByNamedDataValueCalculationPart(BuffCounterByNamedDataValueCalculationPart),
    ByCharLevelBreakpointsCalculationPart(ByCharLevelBreakpointsCalculationPart),
    ByCharLevelFormulaCalculationPart(ByCharLevelFormulaCalculationPart),
    ByCharLevelInterpolationCalculationPart(ByCharLevelInterpolationCalculationPart),
    ClampSubPartsCalculationPart(ClampSubPartsCalculationPart),
    CooldownMultiplierCalculationPart,
    EffectValueCalculationPart(EffectValueCalculationPart),
    ExponentSubPartsCalculationPart(ExponentSubPartsCalculationPart),
    NamedDataValueCalculationPart(NamedDataValueCalculationPart),
    NumberCalculationPart(NumberCalculationPart),
    PercentageOfBuffNameElapsed(PercentageOfBuffNameElapsed),
    ProductOfSubPartsCalculationPart(ProductOfSubPartsCalculationPart),
    StatByCoefficientCalculationPart(StatByCoefficientCalculationPart),
    StatByNamedDataValueCalculationPart(StatByNamedDataValueCalculationPart),
    StatBySubPartCalculationPart(StatBySubPartCalculationPart),
    StatEfficiencyPerHundred(StatEfficiencyPerHundred),
    SubPartScaledProportionalToStat(SubPartScaledProportionalToStat),
    SumOfSubPartsCalculationPart(SumOfSubPartsCalculationPart),
    Unk0x1d452085(Unk0x1d452085),
    Unk0x382277da(Unk0x382277da),
    Unk0x8a96ea3c(Unk0x8a96ea3c),
    Unk0x9e9e2e5c(Unk0x9e9e2e5c),
    Unk0xb22609db(Unk0xb22609db),
    Unk0xba007871(Unk0xba007871),
    Unk0xee18a47b(Unk0xee18a47b),
    Unk0xf3cbe7b2(Unk0xf3cbe7b2),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EnumAnchor {
    AnchorDouble(AnchorDouble),
    AnchorSingle(AnchorSingle),
    Unk0xf090d2e7(Unk0xf090d2e7),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EnumArea {
    Area,
    AreaClamped,
    Cone,
    Direction,
    DragDirection,
    Location,
    LocationClamped,
    MySelf,
    SelfAoe,
    Target(Target),
    TargetOrLocation,
    TerrainLocation,
    TerrainType(TerrainType),
    WallDetection(WallDetection),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EnumAttackEvents {
    AttackEvents(AttackEvents),
    CallOnMissileBounce,
    Cast(Cast),
    ChangeMissileSpeed(ChangeMissileSpeed),
    ChangeMissileWidth(ChangeMissileWidth),
    ClearAlreadyHitTracking,
    ClearTargetAndKeepMoving(ClearTargetAndKeepMoving),
    Destroy,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EnumBehavior {
    FadeOverTimeBehavior(FadeOverTimeBehavior),
    FadeToExplicitValueBehavior(FadeToExplicitValueBehavior),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EnumBlendData {
    TimeBlendData(TimeBlendData),
    TransitionClipBlendData(TransitionClipBlendData),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EnumCastOnHit {
    CastOnHit,
    CastOnMovementComplete(CastOnMovementComplete),
    DelayStart(DelayStart),
    DestroyOnExitMap,
    DestroyOnHit,
    DestroyOnMovementComplete(DestroyOnMovementComplete),
    FixedDistanceIgnoringTerrain(FixedDistanceIgnoringTerrain),
    ResimulateTrailVfxOnEnterVisibility(ResimulateTrailVfxOnEnterVisibility),
    ReturnToCasterOnMovementComplete(ReturnToCasterOnMovementComplete),
    TriggerFromScript(TriggerFromScript),
    TriggerOnDelay(TriggerOnDelay),
    TriggerOnHit(TriggerOnHit),
    TriggerOnMovementComplete(TriggerOnMovementComplete),
    Unk0x72f86c81(Unk0x72f86c81),
    Unk0x91fd0920,
    WidthPerSecond(WidthPerSecond),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EnumClipData {
    AtomicClipData(AtomicClipData),
    ConditionBoolClipData(ConditionBoolClipData),
    ConditionFloatClipData(ConditionFloatClipData),
    ParallelClipData(ParallelClipData),
    ParametricClipData(ParametricClipData),
    SelectorClipData(SelectorClipData),
    SequencerClipData(SequencerClipData),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EnumData {
    AtlasData(AtlasData),
    AtlasData3SliceH(AtlasData3SliceH),
    AtlasData3SliceV(AtlasData3SliceV),
    AtlasData9Slice(AtlasData9Slice),
    LooseUiTextureData(LooseUiTextureData),
    LooseUiTextureData3SliceH(LooseUiTextureData3SliceH),
    LooseUiTextureData3SliceV(LooseUiTextureData3SliceV),
    LooseUiTextureData9Slice(LooseUiTextureData9Slice),
    Unk0x5eaead1a,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EnumDefaultvisibility {
    Defaultvisibility(Defaultvisibility),
    EnterFowVisibility(EnterFowVisibility),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EnumDriver {
    AbilityResourceDynamicMaterialFloatDriver(AbilityResourceDynamicMaterialFloatDriver),
    AllTrueMaterialDriver(AllTrueMaterialDriver),
    AnimationFractionDynamicMaterialFloatDriver(AnimationFractionDynamicMaterialFloatDriver),
    BlendingSwitchMaterialDriver(BlendingSwitchMaterialDriver),
    BuffCounterDynamicMaterialFloatDriver(BuffCounterDynamicMaterialFloatDriver),
    ColorChooserMaterialDriver(ColorChooserMaterialDriver),
    ColorGraphMaterialDriver(ColorGraphMaterialDriver),
    DelayedBoolMaterialDriver(DelayedBoolMaterialDriver),
    DistanceToPlayerMaterialFloatDriver(DistanceToPlayerMaterialFloatDriver),
    FixedDurationTriggeredBoolDriver(FixedDurationTriggeredBoolDriver),
    Float4LiteralMaterialDriver(Float4LiteralMaterialDriver),
    FloatComparisonMaterialDriver(FloatComparisonMaterialDriver),
    FloatGraphMaterialDriver(FloatGraphMaterialDriver),
    FloatLiteralMaterialDriver(FloatLiteralMaterialDriver),
    HasBuffDynamicMaterialBoolDriver(HasBuffDynamicMaterialBoolDriver),
    HasBuffOfTypeBoolDriver(HasBuffOfTypeBoolDriver),
    HasBuffWithAttributeBoolDriver,
    HasGearDynamicMaterialBoolDriver(HasGearDynamicMaterialBoolDriver),
    HealthDynamicMaterialFloatDriver,
    IsAnimationPlayingDynamicMaterialBoolDriver(IsAnimationPlayingDynamicMaterialBoolDriver),
    IsAttackingBoolDriver,
    IsCastingBoolDriver(IsCastingBoolDriver),
    IsDeadDynamicMaterialBoolDriver,
    IsEnemyDynamicMaterialBoolDriver,
    IsInGrassDynamicMaterialBoolDriver,
    IsLocalPlayerBoolDriver,
    IsMovingBoolDriver,
    KeyFrameFloatClipReaderDriver(KeyFrameFloatClipReaderDriver),
    LerpMaterialDriver(LerpMaterialDriver),
    LerpVec4LogicDriver(LerpVec4LogicDriver),
    MaxMaterialDriver(MaxMaterialDriver),
    MinMaterialDriver(MinMaterialDriver),
    NotMaterialDriver(NotMaterialDriver),
    OneTrueMaterialDriver(OneTrueMaterialDriver),
    PlayerPositionDynamicMaterialDriver,
    RemapFloatMaterialDriver(RemapFloatMaterialDriver),
    RemapVec4MaterialDriver(RemapVec4MaterialDriver),
    SineMaterialDriver(SineMaterialDriver),
    SpecificColorMaterialDriver(SpecificColorMaterialDriver),
    SpellRankIntDriver(SpellRankIntDriver),
    SubmeshVisibilityBoolDriver(SubmeshVisibilityBoolDriver),
    SwitchMaterialDriver(SwitchMaterialDriver),
    TimeMaterialDriver(TimeMaterialDriver),
    Unk0x5b2fdd66(Unk0x5b2fdd66),
    Unk0x635d04b7(Unk0x635d04b7),
    Unk0x77b42f3f,
    Unk0x83a9f4f8,
    Unk0x9bc366ca(Unk0x9bc366ca),
    Unk0xb7b43e1d(Unk0xb7b43e1d),
    Unk0xf5821f8b,
    Unk0xfe70e9c4(Unk0xfe70e9c4),
    UvScaleBiasFromAnimationDynamicMaterialDriver(UvScaleBiasFromAnimationDynamicMaterialDriver),
    VelocityDynamicMaterialFloatDriver,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EnumEventData {
    ConformToPathEventData(ConformToPathEventData),
    EnableLookAtEventData(EnableLookAtEventData),
    FaceTargetEventData(FaceTargetEventData),
    FadeEventData(FadeEventData),
    IdleParticlesVisibilityEventData(IdleParticlesVisibilityEventData),
    JointOrientationEventData(JointOrientationEventData),
    JointSnapEventData(JointSnapEventData),
    LockRootOrientationEventData(LockRootOrientationEventData),
    ParticleEventData(ParticleEventData),
    SoundEventData(SoundEventData),
    SpringPhysicsEventData(SpringPhysicsEventData),
    StopAnimationEventData(StopAnimationEventData),
    SubmeshVisibilityEventData(SubmeshVisibilityEventData),
    SyncedAnimationEventData(SyncedAnimationEventData),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EnumFacing {
    VeritcalFacingMatchVelocity,
    VerticalFacingFaceTarget,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EnumGameCalculation {
    GameCalculation(GameCalculation),
    GameCalculationConditional(GameCalculationConditional),
    GameCalculationModified(GameCalculationModified),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EnumHasAllSubRequirementsCastRequirement {
    HasAllSubRequirementsCastRequirement(HasAllSubRequirementsCastRequirement),
    HasAtleastNSubRequirementsCastRequirement(HasAtleastNSubRequirementsCastRequirement),
    HasBuffCastRequirement(HasBuffCastRequirement),
    HasNNearbyUnitsRequirement(HasNNearbyUnitsRequirement),
    HasNNearbyVisibleUnitsRequirement(HasNNearbyVisibleUnitsRequirement),
    HasTypeAndStatusFlags(HasTypeAndStatusFlags),
    HasUnitTagsCastRequirement(HasUnitTagsCastRequirement),
    IsSpecifiedUnitCastRequirement(IsSpecifiedUnitCastRequirement),
    SameTeamCastRequirement(SameTeamCastRequirement),
    Unk0x303c66f4(Unk0x303c66f4),
    Unk0xe2ef74d0(Unk0xe2ef74d0),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EnumHealthBarTickStyle {
    HealthBarTickStyleHero(HealthBarTickStyleHero),
    HealthBarTickStyleTftCompanion(HealthBarTickStyleTftCompanion),
    HealthBarTickStyleUnit(HealthBarTickStyleUnit),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EnumHeightSolver {
    BlendedLinearHeightSolver,
    CurveTheDifferenceHeightSolver(CurveTheDifferenceHeightSolver),
    FollowTerrainHeightSolver(FollowTerrainHeightSolver),
    GravityHeightSolver(GravityHeightSolver),
    SinusoidalHeightSolver(SinusoidalHeightSolver),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EnumIconElement {
    IconElementCircleMaskeExtension,
    IconElementGradientExtension,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EnumIndicatorType {
    IndicatorTypeGlobal(IndicatorTypeGlobal),
    IndicatorTypeLocal,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EnumMap {
    GdsMapObject(GdsMapObject),
    MapAnimatedProp(MapAnimatedProp),
    MapAudio(MapAudio),
    MapBakeProperties(MapBakeProperties),
    MapCubemapProbe(MapCubemapProbe),
    MapGroup(MapGroup),
    MapLocator(MapLocator),
    MapNavGrid(MapNavGrid),
    MapParticle(MapParticle),
    MapScriptLocator(MapScriptLocator),
    MapSunProperties(MapSunProperties),
    MapTerrainPaint(MapTerrainPaint),
    Unk0x1f1f50f2(Unk0x1f1f50f2),
    Unk0x25e3f5d0(Unk0x25e3f5d0),
    Unk0x3c995caf(Unk0x3c995caf),
    Unk0x4cf74021(Unk0x4cf74021),
    Unk0x9aa5b4bc(Unk0x9aa5b4bc),
    Unk0xad65d8c4(Unk0xad65d8c4),
    Unk0xba138ae3(Unk0xba138ae3),
    Unk0xcdb1c8f6(Unk0xcdb1c8f6),
    Unk0xcf4a55da(Unk0xcf4a55da),
    Unk0xd178749c(Unk0xd178749c),
    Unk0xeb997689(Unk0xeb997689),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EnumMovement {
    AcceleratingMovement(AcceleratingMovement),
    CircleMovement(CircleMovement),
    DecelToLocationMovement(DecelToLocationMovement),
    FixedSpeedMovement(FixedSpeedMovement),
    FixedSpeedSplineMovement(FixedSpeedSplineMovement),
    FixedTimeMovement(FixedTimeMovement),
    FixedTimeSplineMovement(FixedTimeSplineMovement),
    ParametricMovement(ParametricMovement),
    PhysicsMovement(PhysicsMovement),
    SyncCircleMovement(SyncCircleMovement),
    TrackMouseMovement(TrackMouseMovement),
    WallFollowMovement(WallFollowMovement),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EnumOverLifeMaterialDriver {
    VfxColorOverLifeMaterialDriver(VfxColorOverLifeMaterialDriver),
    VfxFloatOverLifeMaterialDriver(VfxFloatOverLifeMaterialDriver),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EnumParametricUpdater {
    AttackSpeedParametricUpdater,
    DisplacementParametricUpdater,
    EquippedGearParametricUpdater,
    FacingAndMovementAngleParametricUpdater,
    FacingParametricUpdater,
    IsAllyParametricUpdater,
    IsHomeguardParametricUpdater,
    IsInTerrainParametricUpdater,
    IsMovingParametricUpdater,
    IsRangedParametricUpdater,
    IsTurningParametricUpdater,
    LogicDriverBoolParametricUpdater(LogicDriverBoolParametricUpdater),
    LogicDriverFloatParametricUpdater(LogicDriverFloatParametricUpdater),
    LookAtGoldRedirectTargetAngleParametricUpdater,
    LookAtInterestAngleParametricUpdater,
    LookAtInterestDistanceParametricUpdater,
    LookAtSpellTargetAngleParametricUpdater,
    LookAtSpellTargetDistanceParametricUpdater,
    LookAtSpellTargetHeightOffsetParametricUpdater,
    MoveSpeedParametricUpdater,
    MovementDirectionParametricUpdater,
    ParBarPercentParametricUpdater,
    SkinScaleParametricUpdater,
    SlopeAngleParametricUpdater,
    TotalTurnAngleParametricUpdater,
    TurnAngleParametricUpdater,
    TurnAngleRemainingParametricUpdater,
    Unk0xe7b61183(Unk0xe7b61183),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EnumPersistentEffectConditionData {
    PersistentEffectConditionData(PersistentEffectConditionData),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EnumRequirement {
    CharacterLevelRequirement(CharacterLevelRequirement),
    HasBuffRequirement(HasBuffRequirement),
    HasSkillPointRequirement,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EnumRigPoseModifierData {
    ConformToPathRigPoseModifierData(ConformToPathRigPoseModifierData),
    SpringPhysicsRigPoseModifierData(SpringPhysicsRigPoseModifierData),
    SyncedAnimationRigPoseModifierData,
    Unk0xe6147387(Unk0xe6147387),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EnumTargeterDefinition {
    TargeterDefinitionAoe(TargeterDefinitionAoe),
    TargeterDefinitionArc(TargeterDefinitionArc),
    TargeterDefinitionCone(TargeterDefinitionCone),
    TargeterDefinitionLine(TargeterDefinitionLine),
    TargeterDefinitionMinimap(TargeterDefinitionMinimap),
    TargeterDefinitionMultiAoe(TargeterDefinitionMultiAoe),
    TargeterDefinitionRange(TargeterDefinitionRange),
    TargeterDefinitionSpline(TargeterDefinitionSpline),
    TargeterDefinitionWall(TargeterDefinitionWall),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EnumTargetingRangeValue {
    TargetingRangeValue(TargetingRangeValue),
    Unk0x9d62f7e(Unk0x9d62f7e),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EnumTransitionData {
    SceneAlphaTransitionData(SceneAlphaTransitionData),
    SceneScreenEdgeTransitionData(SceneScreenEdgeTransitionData),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EnumUiDraggable {
    UiDraggableBasic(UiDraggableBasic),
    UiDraggableElementGroupDrag(UiDraggableElementGroupDrag),
    UiDraggableProxyElementDrag(UiDraggableProxyElementDrag),
    UiDraggableSceneDrag(UiDraggableSceneDrag),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EnumUiPosition {
    UiPositionFullScreen,
    UiPositionPolygon(UiPositionPolygon),
    UiPositionRect(UiPositionRect),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EnumUnk0x1aae122 {
    Unk0x1aae122(Unk0x1aae122),
    Unk0x1ddfbeeb,
    Unk0x93adc5b3(Unk0x93adc5b3),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EnumUnk0x51445de9 {
    Unk0x51445de9(Unk0x51445de9),
    Unk0x557bb273(Unk0x557bb273),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EnumUnk0x6bbc3db6 {
    Unk0x6bbc3db6(Unk0x6bbc3db6),
    Unk0xf00f3333,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EnumUnk0x797fe1c2 {
    Unk0x797fe1c2,
    Unk0xcdf661db(Unk0xcdf661db),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EnumUnk0xc96d9140 {
    Unk0xc96d9140(Unk0xc96d9140),
    Unk0xe7ee4f28(Unk0xe7ee4f28),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EnumVfxPrimitive {
    Unk0x8df5fcf7,
    VfxPrimitiveArbitraryQuad,
    VfxPrimitiveArbitraryTrail(VfxPrimitiveArbitraryTrail),
    VfxPrimitiveAttachedMesh(VfxPrimitiveAttachedMesh),
    VfxPrimitiveBeam(VfxPrimitiveBeam),
    VfxPrimitiveCameraSegmentBeam(VfxPrimitiveCameraSegmentBeam),
    VfxPrimitiveCameraTrail(VfxPrimitiveCameraTrail),
    VfxPrimitiveCameraUnitQuad,
    VfxPrimitiveMesh(VfxPrimitiveMesh),
    VfxPrimitivePlanarProjection(VfxPrimitivePlanarProjection),
    VfxPrimitiveRay,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EnumVfxShape {
    Unk0xee39916f(Unk0xee39916f),
    VfxShapeBox(VfxShapeBox),
    VfxShapeCylinder(VfxShapeCylinder),
    VfxShapeLegacy(VfxShapeLegacy),
    VfxShapeSphere(VfxShapeSphere),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EnumWaveBehavior {
    ConstantWaveBehavior(ConstantWaveBehavior),
    InhibitorWaveBehavior(InhibitorWaveBehavior),
    RotatingWaveBehavior(RotatingWaveBehavior),
    TimedVariableWaveBehavior(TimedVariableWaveBehavior),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EsportsBannerMaterialController {}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EvolutionDescription {
    pub m_flags: Option<u32>,
    pub m_icon_names: Vec<String>,
    pub m_title: Option<String>,
    pub m_tooltips: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExponentSubPartsCalculationPart {
    pub part1: NamedDataValueCalculationPart,
    pub part2: NumberCalculationPart,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FaceTargetEventData {
    pub blend_in_time: Option<f32>,
    pub blend_out_time: Option<f32>,
    pub face_target: Option<u8>,
    pub m_end_frame: Option<f32>,
    pub m_fire_if_animation_ends_early: Option<bool>,
    pub m_is_self_only: Option<bool>,
    pub m_name: Option<u32>,
    pub m_start_frame: Option<f32>,
    pub y_rotation_degrees: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FadeEventData {
    pub m_end_frame: Option<f32>,
    pub m_fire_if_animation_ends_early: Option<bool>,
    pub m_name: Option<u32>,
    pub m_start_frame: Option<f32>,
    pub m_target_alpha: f32,
    pub m_time_to_fade: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FadeOverTimeBehavior {
    pub m_end_alpha: Option<f32>,
    pub m_start_alpha: Option<f32>,
    pub m_time_end: f32,
    pub m_time_start: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FadeToExplicitValueBehavior {
    pub m_alpha: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FixedDistanceIgnoringTerrain {
    pub m_maximum_distance: f32,
    pub m_maximum_terrain_walls_to_skip: Option<u32>,
    pub m_minimum_gap_between_terrain_walls: Option<f32>,
    pub m_targeter_definition: TargeterDefinitionSkipTerrain,
    pub scan_width_override: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FixedDurationTriggeredBoolDriver {
    pub m_bool_driver: Box<EnumDriver>,
    pub m_custom_duration: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FixedSpeedMovement {
    pub add_bonus_attack_range_to_cast_range: Option<bool>,
    pub m_infer_direction_from_facing_if_needed: Option<bool>,
    pub m_offset_initial_target_height: Option<f32>,
    pub m_project_target_to_cast_range: Option<bool>,
    pub m_speed: Option<f32>,
    pub m_start_bone_name: Option<String>,
    pub m_start_bone_skin_overrides: Option<HashMap<u32, String>>,
    pub m_target_bone_name: Option<String>,
    pub m_target_height_augment: Option<f32>,
    pub m_tracks_target: Option<bool>,
    pub m_use_ground_height_at_target: Option<bool>,
    pub m_use_height_offset_at_end: Option<bool>,
    pub m_visuals_track_hidden_targets: Option<bool>,
    pub unk_0x3046674: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FixedSpeedSplineMovement {
    pub m_offset_initial_target_height: Option<f32>,
    pub m_speed: f32,
    pub m_spline_info: HermiteSplineInfo,
    pub m_start_bone_name: Option<String>,
    pub m_target_bone_name: Option<String>,
    pub m_target_height_augment: Option<f32>,
    pub m_tracks_target: Option<bool>,
    pub m_use_height_offset_at_end: Option<bool>,
    pub m_use_missile_position_as_origin: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FixedTimeMovement {
    pub m_infer_direction_from_facing_if_needed: Option<bool>,
    pub m_offset_initial_target_height: Option<f32>,
    pub m_project_target_to_cast_range: Option<bool>,
    pub m_start_bone_name: Option<String>,
    pub m_start_bone_skin_overrides: Option<HashMap<u32, String>>,
    pub m_target_bone_name: Option<String>,
    pub m_target_height_augment: Option<f32>,
    pub m_tracks_target: Option<bool>,
    pub m_travel_time: f32,
    pub m_use_ground_height_at_target: Option<bool>,
    pub m_use_height_offset_at_end: Option<bool>,
    pub m_visuals_track_hidden_targets: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FixedTimeSplineMovement {
    pub m_spline_info: HermiteSplineInfo,
    pub m_start_bone_name: String,
    pub m_target_bone_name: Option<String>,
    pub m_target_height_augment: Option<f32>,
    pub m_tracks_target: Option<bool>,
    pub m_travel_time: f32,
    pub m_use_missile_position_as_origin: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FlexTypeFloat {
    pub m_flex_id: Option<u32>,
    pub m_value: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FlexValueFloat {
    pub m_flex_id: Option<u32>,
    pub m_value: Option<ValueFloat>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FlexValueVector2 {
    pub m_flex_id: Option<u32>,
    pub m_value: Option<ValueVector2>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FlexValueVector3 {
    pub m_value: Option<ValueVector3>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Float4LiteralMaterialDriver {
    pub value: Option<Vec4>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FloatComparisonMaterialDriver {
    pub m_operator: Option<u32>,
    pub m_value_a: Box<EnumDriver>,
    pub m_value_b: Box<EnumDriver>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FloatGraphMaterialDriver {
    pub driver: Box<EnumDriver>,
    pub graph: VfxAnimatedFloatVariableData,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FloatLiteralMaterialDriver {
    pub m_value: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FloatPerSpellLevel {
    pub m_per_level_values: Option<Vec<f32>>,
    pub m_value_type: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FloatingHealthBarBurstData {
    pub burst_trigger_percent: f32,
    pub flash_trigger_percent: f32,
    pub shake_trigger_percent: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Asset, TypePath)]
#[serde(rename_all = "camelCase")]
pub struct FloatingInfoBarViewController {
    pub base_loadable: u32,
    pub info_bar_style_source_map: HashMap<u8, u32>,
    pub path_hash_to_self: u64,
    pub unit_status_priority_list: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FollowTerrainHeightSolver {
    pub m_height_offset: Option<f32>,
    pub m_max_slope: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GameCalculation {
    pub m_display_as_percent: Option<bool>,
    pub m_expanded_tooltip_calculation_display: Option<u8>,
    pub m_formula_parts: Option<Vec<EnumAbilityResourceByCoefficientCalculationPart>>,
    pub m_multiplier: Option<EnumAbilityResourceByCoefficientCalculationPart>,
    pub m_precision: Option<i32>,
    pub m_simple_tooltip_calculation_display: Option<u8>,
    pub result_modifier: Option<u8>,
    pub tooltip_only: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GameCalculationConditional {
    pub m_conditional_calculation_requirements: HasBuffCastRequirement,
    pub m_conditional_game_calculation: u32,
    pub m_default_game_calculation: u32,
    pub m_expanded_tooltip_calculation_display: Option<u8>,
    pub m_simple_tooltip_calculation_display: Option<u8>,
    pub tooltip_only: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GameCalculationModified {
    pub m_expanded_tooltip_calculation_display: Option<u8>,
    pub m_modified_game_calculation: u32,
    pub m_multiplier: EnumAbilityResourceByCoefficientCalculationPart,
    pub m_override_spell_level: Option<i32>,
    pub m_simple_tooltip_calculation_display: Option<u8>,
    pub tooltip_only: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GdsMapObject {
    pub box_max: Option<Vec3>,
    pub box_min: Option<Vec3>,
    pub extra_info: Option<Vec<GdsMapObjectBannerInfo>>,
    pub eye_candy: Option<bool>,
    pub m_visibility_flags: Option<u8>,
    pub name: String,
    pub r#type: Option<u8>,
    pub transform: Mat4,
    pub visibility_controller: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GdsMapObjectBannerInfo {
    pub banner_data: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GravityHeightSolver {
    pub m_gravity: Option<f32>,
    pub unk_0x922c17e5: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HasAllSubRequirementsCastRequirement {
    pub m_sub_requirements: Vec<Box<EnumHasAllSubRequirementsCastRequirement>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HasAtleastNSubRequirementsCastRequirement {
    pub m_sub_requirements: Vec<Box<EnumHasAllSubRequirementsCastRequirement>>,
    pub m_successes_required: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HasBuffCastRequirement {
    pub m_buff_name: Option<u32>,
    pub m_from_anyone: Option<bool>,
    pub m_invert_result: Option<bool>,
    pub unk_0x7b66f15d: Option<bool>,
    pub unk_0xd6b6109c: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HasBuffDynamicMaterialBoolDriver {
    pub m_deactivate_early_seconds: Option<f32>,
    pub m_script_name: Option<String>,
    pub spell: Option<u32>,
    pub unk_0x149271dd: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HasBuffOfTypeBoolDriver {
    pub buff_type: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HasBuffRequirement {
    pub m_buff_name: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HasGearDynamicMaterialBoolDriver {
    pub m_gear_index: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HasNNearbyUnitsRequirement {
    pub m_distance_type: Option<u32>,
    pub m_range: f32,
    pub m_units_required: u32,
    pub m_units_requirements: Vec<Box<EnumHasAllSubRequirementsCastRequirement>>,
    pub unk_0x675c02b0: Option<u32>,
    pub unk_0xcec18a6a: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HasNNearbyVisibleUnitsRequirement {
    pub m_distance_type: Option<u32>,
    pub m_range: f32,
    pub m_units_required: u32,
    pub m_units_requirements: Vec<Box<EnumHasAllSubRequirementsCastRequirement>>,
    pub unk_0x990ecf03: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HasTypeAndStatusFlags {
    pub m_affects_type_flags: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HasUnitTagsCastRequirement {
    pub m_unit_tags: ObjectTags,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HealthBarData {
    pub extra_bars: Option<HealthBarExtraBarsData>,
    pub fade_data: Option<HealthBarFadeData>,
    pub health_bar: BarTypeMap,
    pub incoming_damage_bar: Option<u32>,
    pub max_hp_penalty_bar: Option<u32>,
    pub max_hp_penalty_divider: Option<u32>,
    pub text_data: Option<HealthBarTextData>,
    pub tick_style: EnumHealthBarTickStyle,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HealthBarExtraBarsData {
    pub all_shield_bar: u32,
    pub champ_specific_bar: Option<BarTypeMap>,
    pub disguise_health_bar: Option<u32>,
    pub incoming_heal_bar: Option<BarTypeMap>,
    pub magic_shield_bar: Option<u32>,
    pub physical_shield_bar: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HealthBarFadeData {
    pub fade_bar: BarTypeMap,
    pub fade_speed: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HealthBarTextData {
    pub health_value_text: u32,
    pub include_max_health: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HealthBarTickStyleHero {
    pub micro_tick: u32,
    pub micro_tick_per_standard_tick_data: Vec<MicroTicksPerStandardTickData>,
    pub standard_tick: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HealthBarTickStyleTftCompanion {
    pub standard_tick: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HealthBarTickStyleUnit {
    pub standard_tick: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HermiteSplineInfo {
    pub m_control_point1: Option<Vec3>,
    pub m_control_point2: Option<Vec3>,
    pub m_start_position_offset: Option<Vec3>,
    pub m_use_missile_position_as_origin: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Asset, TypePath)]
#[serde(rename_all = "camelCase")]
pub struct HeroFloatingInfoBarData {
    pub anchor: u32,
    pub borders: HeroFloatingInfoBorderData,
    pub burst_fade_meter_other: u32,
    pub burst_fade_meter_self: u32,
    pub burst_heal_meter_ally: u32,
    pub burst_heal_meter_enemy: u32,
    pub burst_heal_meter_self: u32,
    pub burst_heal_meter_self_colorblind: u32,
    pub character_state_indicators: Option<HeroFloatingInfoCharacterStateIndicatorData>,
    pub damage_flash_meter: u32,
    pub death_anim_ally: u32,
    pub death_anim_enemy: u32,
    pub divider: u32,
    pub health_bar: HealthBarData,
    pub icons: HeroFloatingInfoIconsData,
    pub par_bar: Option<AbilityResourceBarData>,
    pub sar_bar: Option<AbilityResourceBarData>,
    pub sar_pips: Option<AbilityResourcePipsData>,
    pub scene: u32,
    pub scripted_threshold_types: HashMap<u32, u32>,
    pub unit_status: UiUnitStatusData,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HeroFloatingInfoBorderData {
    pub additional_status_icons: Option<HashMap<u32, u32>>,
    pub default_border: HeroFloatingInfoBorderTypeData,
    pub defense_modifier_icons: Option<HeroFloatingInfoBorderDefenseIconData>,
    pub executable_border: HeroFloatingInfoBorderTypeData,
    pub has_attached_ally_border: Option<HeroFloatingInfoBorderTypeData>,
    pub invulnerable_border: Option<HeroFloatingInfoBorderTypeData>,
    pub level_text: u32,
    pub level_text_color_ally: Option<[u8; 4]>,
    pub level_text_color_enemy: Option<[u8; 4]>,
    pub level_text_color_self_colorblind: Option<[u8; 4]>,
    pub spell_shield_border: Option<HeroFloatingInfoBorderTypeData>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HeroFloatingInfoBorderDefenseIconData {
    pub defense_down_icons: Vec<HeroFloatingInfoBorderDefenseIconThresholdData>,
    pub defense_up_icon: HeroFloatingInfoBorderDefenseIconThresholdData,
    pub left_icon_region: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HeroFloatingInfoBorderDefenseIconThresholdData {
    pub armor_icon: u32,
    pub combo_icon: u32,
    pub defense_modifier_threshold: f32,
    pub magic_resist_icon: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HeroFloatingInfoBorderTypeData {
    pub anim_in: Option<u32>,
    pub border: u32,
    pub highlight: u32,
    pub level_box_overlay_ally: Option<u32>,
    pub level_box_overlay_enemy: Option<u32>,
    pub level_box_overlay_self: Option<u32>,
    pub level_box_overlay_self_colorblind: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HeroFloatingInfoCharacterStateIndicatorData {
    pub character_states_map: HashMap<u32, HeroFloatingInfoCharacterStateIndicatorList>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HeroFloatingInfoCharacterStateIndicatorList {
    pub state_indicator_list: Vec<Unk0x85a6a05c>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HeroFloatingInfoIconData {
    pub border: u32,
    pub highlight_anim: Option<u32>,
    pub icon: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct HeroFloatingInfoIconsData {
    pub icons: Vec<HeroFloatingInfoIconData>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IdleParticlesVisibilityEventData {
    pub m_fire_if_animation_ends_early: Option<bool>,
    pub m_is_self_only: Option<bool>,
    pub m_show: Option<bool>,
    pub m_start_frame: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IndicatorTypeGlobal {
    pub m_is_floating: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InhibitorWaveBehavior {
    pub spawn_count_per_inhibitor_down: Vec<i32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IntegratedValueFloat {
    pub constant_value: Option<f32>,
    pub dynamics: VfxAnimatedFloatVariableData,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IntegratedValueVector2 {
    pub constant_value: Option<Vec2>,
    pub dynamics: VfxAnimatedVector2fVariableData,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IntegratedValueVector3 {
    pub constant_value: Option<Vec3>,
    pub dynamics: VfxAnimatedVector3fVariableData,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IsAnimationPlayingDynamicMaterialBoolDriver {
    pub m_animation_names: Vec<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IsCastingBoolDriver {
    pub spell_slot: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IsSpecifiedUnitCastRequirement {
    pub m_invert_result: Option<bool>,
    pub m_unit: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct JointOrientationEventData {
    pub blend_data: Unk0x125a3586,
    pub m_end_frame: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct JointSnapEventData {
    pub m_end_frame: Option<f32>,
    pub m_fire_if_animation_ends_early: Option<bool>,
    pub m_is_self_only: Option<bool>,
    pub m_joint_name_to_override: Option<u32>,
    pub m_joint_name_to_snap_to: Option<u32>,
    pub m_name: Option<u32>,
    pub m_start_frame: Option<f32>,
    pub offset: Option<Vec3>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct KeyFrameFloatClipReaderDriver {
    pub clip_accessory_to_read: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct KeyFrameFloatMapClipAccessoryData {
    pub key_frame_floatmap: Option<HashMap<u32, f32>>,
    pub name: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LaunchAreaData {
    pub indicator_texture_name: String,
    pub inner_area_target_distance: f32,
    pub inner_radius: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LerpMaterialDriver {
    pub m_bool_driver: Box<EnumDriver>,
    pub m_off_value: Option<f32>,
    pub m_on_value: Option<f32>,
    pub m_turn_off_time_sec: Option<f32>,
    pub m_turn_on_time_sec: Option<f32>,
    pub m_use_broken_old_interpolation: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LerpVec4LogicDriver {
    pub bool_driver: Box<EnumDriver>,
    pub off_value: Option<Vec4>,
    pub on_value: Option<Vec4>,
    pub turn_off_time_sec: Option<f32>,
    pub turn_on_time_sec: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LinearTransformProcessorData {
    pub m_increment: Option<f32>,
    pub m_multiplier: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LockRootOrientationEventData {
    pub blend_out_time: Option<f32>,
    pub joint_name: Option<u32>,
    pub m_end_frame: Option<f32>,
    pub m_start_frame: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LogicDriverBoolParametricUpdater {
    pub driver: Option<EnumDriver>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LogicDriverFloatParametricUpdater {
    pub driver: EnumDriver,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LooseUiTextureData {
    pub texture_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LooseUiTextureData3SliceH {
    pub edge_sizes_left_right: Option<Vec2>,
    pub texture_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LooseUiTextureData3SliceV {
    pub edge_sizes_top_bottom: Vec2,
    pub texture_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LooseUiTextureData9Slice {
    pub edge_sizes_left_right: Option<Vec2>,
    pub edge_sizes_top_bottom: Option<Vec2>,
    pub texture_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MapAnimatedProp {
    pub idle_animation_name: String,
    pub m_visibility_flags: Option<u8>,
    pub name: String,
    pub play_idle_animation: bool,
    pub prop_name: String,
    pub skin_id: Option<u32>,
    pub transform: Mat4,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MapAudio {
    pub event_name: String,
    pub name: String,
    pub transform: Mat4,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MapBakeProperties {
    pub light_grid_size: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Asset, TypePath)]
#[serde(rename_all = "camelCase")]
pub struct MapContainer {
    pub bounds_max: Vec2,
    pub bounds_min: Option<Vec2>,
    pub chunks: HashMap<u32, u32>,
    pub components: Vec<EnumMap>,
    pub convert_streams_to_half_float: Option<bool>,
    pub lowest_walkable_height: f32,
    pub map_path: String,
    pub mesh_combine_radius: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MapCubemapProbe {
    pub cubemap_probe_path: String,
    pub cubemap_probe_scale: f32,
    pub cubemap_region: Option<MapCubemapRegion>,
    pub m_visibility_flags: Option<u8>,
    pub name: String,
    pub transform: Mat4,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MapCubemapRegion {
    pub max: Vec3,
    pub min: Vec3,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MapGroup {
    pub name: String,
    pub transform: Mat4,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MapLocator {
    pub name: String,
    pub transform: Mat4,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MapNavGrid {
    pub nav_grid_config: u32,
    pub nav_grid_path: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MapParticle {
    pub color_modulate: Option<Vec4>,
    pub eye_candy: Option<bool>,
    pub group_name: Option<String>,
    pub m_visibility_flags: Option<u8>,
    pub name: String,
    pub start_disabled: Option<bool>,
    pub system: u32,
    pub transform: Mat4,
    pub transitional: Option<bool>,
    pub visibility_controller: Option<u32>,
    pub visibility_mode: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Asset, TypePath)]
#[serde(rename_all = "camelCase")]
pub struct MapPlaceableContainer {
    pub items: Option<HashMap<u32, EnumMap>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MapScriptLocator {
    pub name: String,
    pub script_name: String,
    pub transform: Mat4,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MapSunProperties {
    pub fog_alternate_color: Option<Vec4>,
    pub fog_color: Option<Vec4>,
    pub fog_enabled: Option<bool>,
    pub fog_start_and_end: Vec2,
    pub ground_color: Vec4,
    pub horizon_color: Vec4,
    pub light_map_color_scale: f32,
    pub sky_light_color: Vec4,
    pub sky_light_scale: f32,
    pub sun_direction: Vec3,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MapTerrainPaint {
    pub terrain_paint_texture_path: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MaskData {
    pub m_weight_list: Vec<f32>,
    pub mid: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MaxMaterialDriver {
    pub m_drivers: Vec<Box<EnumDriver>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MicroTicksPerStandardTickData {
    pub micro_ticks_between: u32,
    pub min_health: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MinMaterialDriver {
    pub m_drivers: Vec<Box<EnumDriver>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MinionUpgradeConfig {
    pub armor_max: Option<f32>,
    pub armor_upgrade: Option<f32>,
    pub armor_upgrade_growth: Option<f32>,
    pub damage_max: f32,
    pub damage_upgrade: Option<f32>,
    pub damage_upgrade_late: Option<f32>,
    pub gold_max: Option<f32>,
    pub hp_max_bonus: f32,
    pub hp_upgrade: f32,
    pub hp_upgrade_late: Option<f32>,
    pub magic_resistance_upgrade: Option<f32>,
    pub unk_0x726ae049: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MissileAttachedTargetingDefinition {
    pub m_end_position_type: u8,
    pub m_line_end_texture_height: f32,
    pub m_line_end_texture_name: String,
    pub m_line_end_texture_width: f32,
    pub m_line_texture_width: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MissileGroupSpawnerSpec {
    pub m_child_missile_spell: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MissileSpecification {
    pub behaviors: Option<Vec<EnumCastOnHit>>,
    pub height_solver: Option<EnumHeightSolver>,
    pub m_missile_width: Option<f32>,
    pub missile_group_spawners: Option<Vec<MissileGroupSpawnerSpec>>,
    pub movement_component: EnumMovement,
    pub unk_0xc195fba6: Option<bool>,
    pub vertical_facing: Option<EnumFacing>,
    pub visibility_component: Option<EnumDefaultvisibility>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NamedDataValueCalculationPart {
    pub m_data_value: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NotMaterialDriver {
    pub m_driver: Box<EnumDriver>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NumberCalculationPart {
    pub m_number: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ObjectTags {
    pub m_object_tag_list: Vec<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OneTrueMaterialDriver {
    pub m_drivers: Option<Vec<Box<EnumDriver>>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OverrideAttackTimeData {
    pub m_cast_time_percent: Option<f32>,
    pub m_total_attack_time_secs: Option<GameCalculation>,
    pub set_override_attack_delay: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OverrideAutoAttackCastTimeData {
    pub m_override_autoattack_cast_time_calculation: GameCalculation,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PackFormationData {
    pub formation_positions: Vec<Vec2>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PackManagerData {
    pub attack_move_target_forgiveness_range: Option<f32>,
    pub buff_overrides: Option<Vec<u32>>,
    pub follower_crossover_animation: u32,
    pub leash_distance: Option<f32>,
    pub on_leader_move_follower_animation: u32,
    pub order_trailing_delay: Option<f32>,
    pub rank_to_formation_map: Option<HashMap<u32, PackFormationData>>,
    pub ui_target_forgiveness_range: Option<f32>,
    pub unk_0x377491e8: EnumUnk0x1aae122,
    pub unk_0xb97a9b92: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ParallelClipData {
    pub m_clip_name_list: Vec<u32>,
    pub m_flags: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ParametricClipData {
    pub m_animation_interruption_group_names: Option<Vec<u32>>,
    pub m_event_data_map: Option<HashMap<u32, EnumEventData>>,
    pub m_flags: Option<u32>,
    pub m_mask_data_name: Option<u32>,
    pub m_parametric_pair_data_list: Vec<ParametricPairData>,
    pub m_sync_group_data_name: Option<u32>,
    pub m_track_data_name: u32,
    pub unk_0x69de8fca: Option<bool>,
    pub updater: EnumParametricUpdater,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ParametricMovement {
    pub m_offset_initial_target_height: f32,
    pub m_start_bone_name: String,
    pub m_target_height_augment: f32,
    pub movement_entries: Vec<ParametricMovementEntry>,
    pub parametric_movement_type: ParametricMovementTypeAngleFromTarget,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ParametricMovementEntry {
    pub movement_spec: FixedSpeedSplineMovement,
    pub value: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ParametricMovementTypeAngleFromTarget {}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ParametricPairData {
    pub m_clip_name: u32,
    pub m_value: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ParticleEventData {
    pub m_effect_key: Option<u32>,
    pub m_effect_name: Option<String>,
    pub m_end_frame: Option<f32>,
    pub m_enemy_effect_key: Option<u32>,
    pub m_fire_if_animation_ends_early: Option<bool>,
    pub m_is_detachable: Option<bool>,
    pub m_is_kill_event: Option<bool>,
    pub m_is_loop: Option<bool>,
    pub m_is_self_only: Option<bool>,
    pub m_name: Option<u32>,
    pub m_particle_event_data_pair_list: Option<Vec<ParticleEventDataPair>>,
    pub m_scale_play_speed_with_animation: Option<bool>,
    pub m_start_frame: Option<f32>,
    pub scale: Option<f32>,
    pub skip_if_past_end_frame: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ParticleEventDataPair {
    pub m_bone_name: Option<u32>,
    pub m_target_bone_name: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PercentageOfBuffNameElapsed {
    pub buff_name: u32,
    pub coefficient: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PerkReplacement {
    pub m_replace_target: u32,
    pub m_replace_with: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PerkReplacementList {
    pub m_replacements: Vec<PerkReplacement>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PersistentEffectConditionData {
    pub force_render_vfx: Option<bool>,
    pub owner_condition: Option<EnumDriver>,
    pub persistent_vfxs: Option<Vec<PersistentVfxData>>,
    pub source_condition: Option<EnumDriver>,
    pub submeshes_to_hide: Option<Vec<u32>>,
    pub submeshes_to_show: Option<Vec<u32>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PersistentVfxData {
    pub attach_to_camera: Option<bool>,
    pub bone_name: Option<String>,
    pub effect_key: u32,
    pub effect_key_for_other_team: Option<u32>,
    pub play_speed_modifier: Option<f32>,
    pub scale: Option<f32>,
    pub show_to_owner_only: Option<bool>,
    pub specific_team: Option<u32>,
    pub target_bone_name: Option<String>,
    pub target_pos_is_owner: Option<bool>,
    pub use_different_key_for_other_team: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PhysicsMovement {
    pub m_drag: f32,
    pub m_initial_speed: f32,
    pub m_lifetime: f32,
    pub m_offset_initial_target_height: Option<f32>,
    pub m_start_bone_name: String,
    pub m_target_height_augment: Option<f32>,
    pub m_tracks_target: bool,
    pub m_wall_sliding: bool,
    pub m_wall_sliding_friction_multiplier: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlatformSpellInfo {
    pub m_avatar_level_required: Option<i32>,
    pub m_game_modes: Option<Vec<String>>,
    pub m_platform_enabled: Option<bool>,
    pub m_spell_id: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProductOfSubPartsCalculationPart {
    pub m_part1: Box<EnumAbilityResourceByCoefficientCalculationPart>,
    pub m_part2: Box<EnumAbilityResourceByCoefficientCalculationPart>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RecSpellRankUpInfo {
    pub m_default_priority: Vec<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RecSpellRankUpInfoList {
    pub rec_spell_rank_up_infos: Vec<RecSpellRankUpInfo>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RemapFloatMaterialDriver {
    pub m_driver: Box<EnumDriver>,
    pub m_max_value: Option<f32>,
    pub m_min_value: Option<f32>,
    pub m_output_max_value: Option<f32>,
    pub m_output_min_value: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RemapVec4MaterialDriver {
    pub driver: AbilityResourceDynamicMaterialFloatDriver,
    pub output_max_value: Vec4,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ResimulateTrailVfxOnEnterVisibility {
    pub cycles: u32,
    pub simulation_frames: u32,
    pub time_to_resimulate: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ResourceMeterIconData {
    pub additional_bar_types: Option<HashMap<u32, u32>>,
    pub default_bar: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Asset, TypePath)]
#[serde(rename_all = "camelCase")]
pub struct ResourceResolver {
    pub resource_map: Option<HashMap<u32, u32>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReturnToCasterOnMovementComplete {
    pub m_override_spec: AcceleratingMovement,
    pub m_preserve_speed: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RotatingWaveBehavior {
    pub spawn_counts_by_wave: Vec<i32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SameTeamCastRequirement {
    pub m_invert_result: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SceneAlphaTransitionData {
    pub easing_type: Option<u8>,
    pub end_alpha: Option<u8>,
    pub start_alpha: Option<u8>,
    pub transition_start_delay_secs: Option<f32>,
    pub transition_time: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SceneScreenEdgeTransitionData {
    pub easing_type: Option<u8>,
    pub edge: Option<u8>,
    pub end_alpha: Option<u8>,
    pub start_alpha: Option<u8>,
    pub transition_start_delay_secs: Option<f32>,
    pub transition_time: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SecondaryResourceDisplayFractional {}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SelectorClipData {
    pub m_flags: Option<u32>,
    pub m_selector_pair_data_list: Vec<SelectorPairData>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SelectorPairData {
    pub m_clip_name: u32,
    pub m_probability: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SequencerClipData {
    pub m_clip_name_list: Vec<u32>,
    pub m_event_data_map: Option<HashMap<u32, EnumEventData>>,
    pub m_flags: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SineMaterialDriver {
    pub m_bias: Option<f32>,
    pub m_driver: Box<EnumDriver>,
    pub m_frequency: Option<f32>,
    pub m_scale: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SinusoidalHeightSolver {
    pub m_amplitude: f32,
    pub m_number_of_periods: f32,
    pub m_vertical_offset: Option<f32>,
    pub unk_0x827af87a: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkinAnimationProperties {
    pub animation_graph_data: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkinAudioProperties {
    pub bank_units: Option<Vec<BankUnit>>,
    pub plays_vo: Option<bool>,
    pub tag_event_list: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkinAugmentCategories {
    pub basic_augments: Option<Vec<Unk0xe1555e0a>>,
    pub border_augments: Vec<Unk0x4a70b12c>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Asset, TypePath)]
#[serde(rename_all = "camelCase")]
pub struct SkinCharacterDataProperties {
    pub alternate_icons_circle: Option<Vec<String>>,
    pub alternate_icons_square: Option<Vec<String>>,
    pub armor_material: Option<String>,
    pub attribute_flags: Option<u32>,
    pub can_share_theme_music: Option<bool>,
    pub champion_skin_name: Option<String>,
    pub default_animations: Option<Vec<String>>,
    pub emote_buffbone: Option<String>,
    pub emote_loadout: Option<u32>,
    pub emote_y_offset: Option<f32>,
    pub extra_action_button_count: Option<u32>,
    pub extra_character_preloads: Option<Vec<String>>,
    pub godray_f_xbone: Option<String>,
    pub health_bar_data: Option<CharacterHealthBarDataRecord>,
    pub hud_mute_event: Option<String>,
    pub hud_unmute_event: Option<String>,
    pub icon_avatar: Option<String>,
    pub icon_circle: Option<String>,
    pub icon_circle_scale: Option<f32>,
    pub icon_square: Option<String>,
    pub idle_particles_effects: Option<Vec<SkinCharacterDataPropertiesCharacterIdleEffect>>,
    pub loadscreen: Option<CensoredImage>,
    pub loadscreen_vintage: Option<CensoredImage>,
    pub m_contextual_action_data: Option<u32>,
    pub m_emblems: Option<Vec<SkinEmblem>>,
    pub m_resource_resolver: Option<u32>,
    pub m_spawn_particle_name: Option<String>,
    pub meta_data_tags: Option<String>,
    pub override_on_screen_name: Option<String>,
    pub particle_override_champion_kill_death_particle: Option<String>,
    pub particle_override_death_particle: Option<String>,
    pub persistent_effect_conditions: Option<Vec<EnumPersistentEffectConditionData>>,
    pub secondary_resource_hud_display_data: Option<SecondaryResourceDisplayFractional>,
    pub skin_animation_properties: SkinAnimationProperties,
    pub skin_audio_properties: Option<SkinAudioProperties>,
    pub skin_classification: Option<u32>,
    pub skin_mesh_properties: Option<SkinMeshDataProperties>,
    pub skin_parent: Option<i32>,
    pub skin_upgrade_data: Option<SkinUpgradeData>,
    pub theme_music: Option<Vec<String>>,
    pub uncensored_icon_circles: Option<HashMap<u32, String>>,
    pub uncensored_icon_squares: Option<HashMap<u32, String>>,
    pub unk_0x2ac577e2: Option<bool>,
    pub unk_0xb67a2dd8: Option<Vec<Unk0x9c1d99c0>>,
    pub unk_0xc3a944e7: Option<EnumUnk0xc96d9140>,
    pub unk_0xe484edc4: Option<u32>,
    pub unk_0xeda7817e: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkinCharacterDataPropertiesCharacterIdleEffect {
    pub bone_name: Option<String>,
    pub effect_key: Option<u32>,
    pub effect_name: Option<String>,
    pub position: Option<Vec3>,
    pub target_bone_name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkinEmblem {
    pub m_emblem_data: u32,
    pub m_loading_screen_anchor: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkinFilterData {
    pub filter_type: Option<u32>,
    pub skin_ids: Vec<u32>,
    pub use_valid_parent_for_chroma: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkinMeshDataProperties {
    pub bounding_cylinder_height: Option<f32>,
    pub bounding_cylinder_radius: Option<f32>,
    pub bounding_sphere_radius: Option<f32>,
    pub brush_alpha_override: Option<f32>,
    pub cast_shadows: Option<bool>,
    pub emissive_texture: Option<String>,
    pub emitter_submesh_avatar_to_hide: Option<String>,
    pub enable_picking: Option<bool>,
    pub force_draw_last: Option<bool>,
    pub fresnel: Option<f32>,
    pub fresnel_color: Option<[u8; 4]>,
    pub gloss_texture: Option<String>,
    pub initial_submesh_avatar_to_hide: Option<String>,
    pub initial_submesh_mouse_overs_to_hide: Option<String>,
    pub initial_submesh_shadows_to_hide: Option<String>,
    pub initial_submesh_to_hide: Option<String>,
    pub material: Option<u32>,
    pub material_controller: Option<EsportsBannerMaterialController>,
    pub material_override: Option<Vec<SkinMeshDataPropertiesMaterialOverride>>,
    pub override_bounding_box: Option<Vec3>,
    pub reduced_bone_skinning: Option<bool>,
    pub reflection_fresnel: Option<f32>,
    pub reflection_fresnel_color: Option<[u8; 4]>,
    pub reflection_map: Option<String>,
    pub reflection_opacity_direct: Option<f32>,
    pub reflection_opacity_glancing: Option<f32>,
    pub rig_pose_modifier_data: Option<Vec<EnumRigPoseModifierData>>,
    pub self_illumination: Option<f32>,
    pub simple_skin: Option<String>,
    pub skeleton: Option<String>,
    pub skin_scale: Option<f32>,
    pub submesh_render_order: Option<String>,
    pub texture: Option<String>,
    pub uses_skin_vo: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkinMeshDataPropertiesMaterialOverride {
    pub gloss_texture: Option<String>,
    pub material: Option<u32>,
    pub submesh: String,
    pub texture: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkinUpgradeData {
    pub m_gear_skin_upgrades: Option<Vec<u32>>,
    pub skin_augment_categories: Option<SkinAugmentCategories>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SoundEventData {
    pub condition_clip_transition_type: Option<u16>,
    pub m_end_frame: Option<f32>,
    pub m_fire_if_animation_ends_early: Option<bool>,
    pub m_is_kill_event: Option<bool>,
    pub m_is_loop: Option<bool>,
    pub m_is_self_only: Option<bool>,
    pub m_name: Option<u32>,
    pub m_skip_if_past_end_frame: Option<bool>,
    pub m_sound_name: Option<String>,
    pub m_start_frame: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SpawningUiDefinition {
    pub buff_name_filter: String,
    pub max_number_of_units: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SpecificColorMaterialDriver {
    pub m_color: Option<Vec4>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct SpellDataResource {
    pub always_snap_facing: Option<bool>,
    pub audio_bank_paths: Option<Vec<String>>,
    pub b_have_hit_bone: Option<bool>,
    pub b_have_hit_effect: Option<bool>,
    pub b_is_toggle_spell: Option<bool>,
    pub can_cast_or_queue_while_casting: Option<bool>,
    pub can_cast_while_disabled: Option<bool>,
    pub can_only_cast_while_dead: Option<bool>,
    pub cannot_be_suppressed: Option<bool>,
    pub cant_cast_while_rooted: Option<bool>,
    pub cast_cone_angle: Option<f32>,
    pub cast_cone_distance: Option<f32>,
    pub cast_frame: Option<f32>,
    pub cast_radius: Option<Vec<f32>>,
    pub cast_radius_secondary: Option<Vec<f32>>,
    pub cast_range: Option<Vec<f32>>,
    pub cast_range_display_override: Option<Vec<f32>>,
    pub cast_range_use_bounding_boxes: Option<bool>,
    pub cast_target_additional_units_radius: Option<f32>,
    pub cooldown_time: Option<Vec<f32>>,
    pub data_values: Option<Vec<SpellDataValue>>,
    pub data_values_mode_override: Option<HashMap<u32, SpellDataValueVector>>,
    pub delay_cast_offset_percent: Option<f32>,
    pub delay_total_time_percent: Option<f32>,
    pub flags: Option<u32>,
    pub img_icon_path: Option<String>,
    pub loadable: Option<u32>,
    pub lua_on_missile_update_distance_interval: Option<f32>,
    pub m_affects_status_flags: Option<u32>,
    pub m_affects_type_flags: Option<u32>,
    pub m_after_effect_key: Option<u32>,
    pub m_after_effect_name: Option<String>,
    pub m_ai_data: Option<AiSpellData>,
    pub m_alternate_name: Option<String>,
    pub m_ammo_count_hidden_in_ui: Option<bool>,
    pub m_ammo_not_affected_by_cdr: Option<bool>,
    pub m_ammo_recharge_time: Option<Vec<f32>>,
    pub m_ammo_used: Option<Vec<i32>>,
    pub m_animation_loop_name: Option<String>,
    pub m_animation_name: Option<String>,
    pub m_animation_winddown_name: Option<String>,
    pub m_apply_attack_damage: Option<bool>,
    pub m_apply_attack_effect: Option<bool>,
    pub m_apply_material_on_hit_sound: Option<bool>,
    pub m_belongs_to_avatar: Option<bool>,
    pub m_can_move_while_channeling: Option<bool>,
    pub m_can_trigger_charge_spell_while_disabled: Option<bool>,
    pub m_cancel_charge_on_recast_time: Option<f32>,
    pub m_cant_cancel_while_channeling: Option<bool>,
    pub m_cant_cancel_while_winding_up: Option<bool>,
    pub m_cant_cancel_while_winding_up_targeting_champion: Option<bool>,
    pub m_cast_range_growth_duration: Option<Vec<f32>>,
    pub m_cast_range_growth_max: Option<Vec<f32>>,
    pub m_cast_range_growth_start_time: Option<Vec<f32>>,
    pub m_cast_requirements_caster: Option<Vec<EnumHasAllSubRequirementsCastRequirement>>,
    pub m_cast_requirements_target: Option<Vec<EnumHasAllSubRequirementsCastRequirement>>,
    pub m_cast_time: Option<f32>,
    pub m_cast_type: Option<u32>,
    pub m_caster_position_end_of_cast_update: Option<u8>,
    pub m_casting_breaks_stealth: Option<bool>,
    pub m_casting_breaks_stealth_while_attached: Option<bool>,
    pub m_casting_reveals_caster_stealth: Option<bool>,
    pub m_channel_duration: Option<Vec<f32>>,
    pub m_channel_is_interrupted_by_attacking: Option<bool>,
    pub m_channel_is_interrupted_by_disables: Option<bool>,
    pub m_character_passive_buffs: Option<Vec<CharacterPassiveData>>,
    pub m_charge_update_interval: Option<f32>,
    pub m_client_data: Option<SpellDataResourceClient>,
    pub m_coefficient: Option<f32>,
    pub m_coefficient2: Option<f32>,
    pub m_considered_as_auto_attack: Option<bool>,
    pub m_cooldown_not_affected_by_cdr: Option<bool>,
    pub m_cost_always_shown_in_ui: Option<bool>,
    pub m_cursor_changes_in_grass: Option<bool>,
    pub m_cursor_changes_in_terrain: Option<bool>,
    pub m_dimension_behavior: Option<u8>,
    pub m_disable_cast_bar: Option<bool>,
    pub m_do_not_need_to_face_target: Option<bool>,
    pub m_does_not_consume_cooldown: Option<bool>,
    pub m_does_not_consume_mana: Option<bool>,
    pub m_doesnt_break_channels: Option<bool>,
    pub m_effect_amount: Option<Vec<SpellEffectAmount>>,
    pub m_excluded_unit_tags: Option<ObjectTags>,
    pub m_float_vars_decimals: Option<Vec<i32>>,
    pub m_hide_range_indicator_when_casting: Option<bool>,
    pub m_hit_bone_name: Option<String>,
    pub m_hit_effect_key: Option<u32>,
    pub m_hit_effect_name: Option<String>,
    pub m_hit_effect_orient_type: Option<u32>,
    pub m_hit_effect_player_key: Option<u32>,
    pub m_hit_effect_player_name: Option<String>,
    pub m_ignore_anim_continue_until_cast_frame: Option<bool>,
    pub m_ignore_range_check: Option<bool>,
    pub m_img_icon_name: Option<Vec<String>>,
    pub m_is_delayed_by_cast_locked: Option<bool>,
    pub m_is_disabled_while_dead: Option<bool>,
    pub m_keyword_when_acquired: Option<String>,
    pub m_line_drag_length: Option<f32>,
    pub m_line_width: Option<f32>,
    pub m_locked_spell_origination_cast_id: Option<bool>,
    pub m_look_at_policy: Option<u32>,
    pub m_max_ammo: Option<Vec<i32>>,
    pub m_minimap_icon_display_flag: Option<u16>,
    pub m_minimap_icon_name: Option<String>,
    pub m_minimap_icon_rotation: Option<bool>,
    pub m_missile_effect_enemy_key: Option<u32>,
    pub m_missile_effect_key: Option<u32>,
    pub m_missile_effect_name: Option<String>,
    pub m_missile_effect_player_key: Option<u32>,
    pub m_missile_effect_player_name: Option<String>,
    pub m_missile_spec: Option<MissileSpecification>,
    pub m_no_winddown_if_cancelled: Option<bool>,
    pub m_orient_radius_texture_from_player: Option<bool>,
    pub m_override_attack_time: Option<OverrideAttackTimeData>,
    pub m_particle_start_offset: Option<Vec3>,
    pub m_pingable_while_disabled: Option<bool>,
    pub m_platform_spell_info: Option<PlatformSpellInfo>,
    pub m_post_cast_lockout_delta_time: Option<f32>,
    pub m_pre_cast_lockout_delta_time: Option<f32>,
    pub m_pre_cast_lockout_delta_time_data: Option<SpellLockDeltaTimeData>,
    pub m_project_target_to_cast_range: Option<bool>,
    pub m_required_unit_tags: Option<ObjectTags>,
    pub m_resource_resolvers: Option<Vec<u32>>,
    pub m_roll_for_critical_hit: Option<bool>,
    pub m_show_channel_bar: Option<bool>,
    pub m_spell_calculations: Option<HashMap<u32, EnumGameCalculation>>,
    pub m_spell_cooldown_or_sealed_queue_threshold: Option<f32>,
    pub m_spell_reveals_champion: Option<bool>,
    pub m_spell_tags: Option<Vec<String>>,
    pub m_start_cooldown: Option<f32>,
    pub m_targeting_type_data: Option<EnumArea>,
    pub m_turn_speed_scalar: Option<f32>,
    pub m_update_rotation_when_casting: Option<bool>,
    pub m_use_autoattack_cast_time_data: Option<UseAutoattackCastTimeData>,
    pub m_use_charge_channeling: Option<bool>,
    pub m_use_minimap_targeting: Option<bool>,
    pub mana: Option<Vec<f32>>,
    pub mana_ui_override: Option<Vec<f32>>,
    pub missile_effect_max_turn_speed_degrees_per_second: Option<f32>,
    pub missile_effect_maximum_angle_degrees: Option<f32>,
    pub missile_speed: Option<f32>,
    pub passive_spell_affected_by_cooldown: Option<bool>,
    pub selection_priority: Option<u32>,
    pub should_receive_input_events: Option<bool>,
    pub show_channel_bar_per_spell_level_override: Option<Vec<bool>>,
    pub spell_event_to_audio_event_suffix: Option<HashMap<u32, String>>,
    pub targeting_forgiveness_definitions: Option<Vec<TargetingForgivenessDefinitions>>,
    pub unk_0x288b8edc: Option<EnumUnk0x6bbc3db6>,
    pub unk_0x48201b0d: Option<f32>,
    pub unk_0x66769fb4: Option<bool>,
    pub unk_0x8958fee2: Option<Unk0x8958fee2>,
    pub unk_0xabe507b9: Option<u32>,
    pub unk_0xb08bc498: Option<HashMap<u32, SpellEffectAmount>>,
    pub unk_0xf4ca428f: Option<u8>,
    pub unk_0xf9c2333e: Option<HashMap<u32, SpellEffectAmount>>,
    pub use_animator_framerate: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SpellDataResourceClient {
    pub m_custom_targeter_definitions: Option<HashMap<u32, CustomTargeterDefinitions>>,
    pub m_left_click_spell_action: Option<u32>,
    pub m_missile_targeter_definitions: Option<Vec<MissileAttachedTargetingDefinition>>,
    pub m_right_click_spell_action: Option<u32>,
    pub m_spawning_ui_definition: Option<SpawningUiDefinition>,
    pub m_targeter_definitions: Option<Vec<EnumTargeterDefinition>>,
    pub m_tooltip_data: Option<TooltipInstanceSpell>,
    pub m_use_death_recap_tooltip_from_another_spell: Option<u32>,
    pub m_use_tooltip_from_another_spell: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SpellDataValue {
    pub m_name: String,
    pub m_values: Option<Vec<f32>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SpellDataValueVector {
    pub spell_data_values: Option<Vec<SpellDataValue>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SpellEffectAmount {
    pub value: Option<Vec<f32>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SpellLevelUpInfo {
    pub m_requirements: Vec<SpellRankUpRequirements>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SpellLockDeltaTimeData {
    pub m_spell_lock_delta_time_calculation: GameCalculation,
}

#[derive(Serialize, Deserialize, Debug, Clone, Asset, TypePath)]
#[serde(rename_all = "camelCase")]
pub struct SpellObject {
    pub bot_data: Option<BotsSpellData>,
    pub cc_behavior_data: Option<CcBehaviorData>,
    pub m_buff: Option<BuffData>,
    pub m_script_name: String,
    pub m_spell: Option<SpellDataResource>,
    pub object_name: String,
    pub script: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SpellRankIntDriver {
    pub spell_slot: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SpellRankUpRequirements {
    pub m_requirements: Option<Vec<EnumRequirement>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SpringPhysicsEventData {
    pub blend_out_time: Option<f32>,
    pub m_end_frame: Option<f32>,
    pub m_fire_if_animation_ends_early: Option<bool>,
    pub spring_to_affect: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SpringPhysicsRigPoseModifierData {
    pub damping: Option<f32>,
    pub default_on: Option<bool>,
    pub do_rotation: bool,
    pub do_translation: Option<bool>,
    pub joint: u32,
    pub mass: Option<f32>,
    pub max_angle: Option<f32>,
    pub max_distance: Option<f32>,
    pub name: Option<u32>,
    pub spring_stiffness: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StatByCoefficientCalculationPart {
    pub m_coefficient: Option<f32>,
    pub m_stat: Option<u8>,
    pub m_stat_formula: Option<u8>,
    pub unk_0xa8cb9c14: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StatByNamedDataValueCalculationPart {
    pub m_data_value: u32,
    pub m_stat: Option<u8>,
    pub m_stat_formula: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StatBySubPartCalculationPart {
    pub m_stat: Option<u8>,
    pub m_stat_formula: Option<u8>,
    pub m_subpart: Box<EnumAbilityResourceByCoefficientCalculationPart>,
    pub unk_0xa8cb9c14: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StatEfficiencyPerHundred {
    pub m_bonus_stat_for_efficiency: f32,
    pub m_data_value: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StaticMaterialChildTechniqueDef {
    pub name: String,
    pub parent_name: String,
    pub shader_macros: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Asset, TypePath)]
#[serde(rename_all = "camelCase")]
pub struct StaticMaterialDef {
    pub child_techniques: Option<Vec<StaticMaterialChildTechniqueDef>>,
    pub dynamic_material: Option<DynamicMaterialDef>,
    pub name: String,
    pub param_values: Option<Vec<StaticMaterialShaderParamDef>>,
    pub r#type: Option<u32>,
    pub sampler_values: Option<Vec<StaticMaterialShaderSamplerDef>>,
    pub shader_macros: Option<HashMap<String, String>>,
    pub switches: Option<Vec<StaticMaterialSwitchDef>>,
    pub techniques: Vec<StaticMaterialTechniqueDef>,
    pub unk_0xe251b20a: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StaticMaterialPassDef {
    pub blend_enable: Option<bool>,
    pub cull_enable: Option<bool>,
    pub depth_compare_func: Option<u32>,
    pub depth_enable: Option<bool>,
    pub depth_offset_bias: Option<f32>,
    pub depth_offset_slope: Option<f32>,
    pub dst_alpha_blend_factor: Option<u32>,
    pub dst_color_blend_factor: Option<u32>,
    pub polygon_depth_bias_enable: Option<bool>,
    pub shader: u32,
    pub shader_macros: Option<HashMap<String, String>>,
    pub src_alpha_blend_factor: Option<u32>,
    pub src_color_blend_factor: Option<u32>,
    pub stencil_compare_func: Option<u32>,
    pub stencil_enable: Option<bool>,
    pub stencil_mask: Option<u32>,
    pub stencil_reference_val: Option<u8>,
    pub winding_to_cull: Option<u32>,
    pub write_mask: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StaticMaterialShaderParamDef {
    pub name: String,
    pub value: Option<Vec4>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StaticMaterialShaderSamplerDef {
    pub address_u: Option<u32>,
    pub address_v: Option<u32>,
    pub address_w: Option<u32>,
    pub sampler_name: Option<String>,
    pub texture_name: String,
    pub texture_path: Option<String>,
    pub uncensored_textures: Option<HashMap<u32, String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StaticMaterialSwitchDef {
    pub group: Option<String>,
    pub name: String,
    pub on: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StaticMaterialTechniqueDef {
    pub name: String,
    pub passes: Vec<StaticMaterialPassDef>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StopAnimationEventData {
    pub m_end_frame: Option<f32>,
    pub m_name: Option<u32>,
    pub m_start_frame: Option<f32>,
    pub m_stop_animation_name: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Asset, TypePath)]
#[serde(rename_all = "camelCase")]
pub struct StructureFloatingInfoBarData {
    pub aggro: u32,
    pub anchor: u32,
    pub border: u32,
    pub burst_data: FloatingHealthBarBurstData,
    pub burst_fade_meter: u32,
    pub damage_flash_meter: u32,
    pub death_anim_ally: u32,
    pub death_anim_enemy: u32,
    pub health_bar: HealthBarData,
    pub highlight: u32,
    pub objective_bounty_ally: u32,
    pub objective_bounty_enemy: u32,
    pub par_bar: Option<AbilityResourceBarData>,
    pub scene: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SubPartScaledProportionalToStat {
    pub m_ratio: f32,
    pub m_stat: Option<u8>,
    pub m_style_tag: Option<String>,
    pub m_style_tag_if_scaled: Option<String>,
    pub m_subpart: Box<EnumAbilityResourceByCoefficientCalculationPart>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SubmeshVisibilityBoolDriver {
    pub any_submesh: Option<bool>,
    pub submeshes: Vec<u32>,
    pub visible: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SubmeshVisibilityEventData {
    pub apply_only_to_avatar_meshes: Option<bool>,
    pub m_end_frame: Option<f32>,
    pub m_fire_if_animation_ends_early: Option<bool>,
    pub m_hide_submesh_list: Option<Vec<u32>>,
    pub m_is_self_only: Option<bool>,
    pub m_name: Option<u32>,
    pub m_show_submesh_list: Option<Vec<u32>>,
    pub m_start_frame: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SumOfSubPartsCalculationPart {
    pub m_subparts: Vec<Box<EnumAbilityResourceByCoefficientCalculationPart>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SwitchMaterialDriver {
    pub m_default_value: Box<EnumDriver>,
    pub m_elements: Option<Vec<Box<SwitchMaterialDriverElement>>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SwitchMaterialDriverElement {
    pub m_condition: Box<EnumDriver>,
    pub m_value: Box<EnumDriver>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SyncCircleMovement {
    pub m_angular_velocity: f32,
    pub m_axis_of_rotation: Option<u8>,
    pub m_lifetime: Option<f32>,
    pub m_offset_initial_target_height: Option<f32>,
    pub m_rotate_around_caster_facing_direction: Option<bool>,
    pub m_start_bone_name: Option<String>,
    pub m_target_bone_name: Option<String>,
    pub m_visuals_track_hidden_targets: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SyncGroupData {
    pub m_type: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SyncedAnimationEventData {
    pub m_lerp_time: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Target {
    pub m_can_complete_cast_without_target: Option<bool>,
    pub unk_0xfb5bbd7: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TargetLaserComponentEffects {
    pub beam_effect_definition: SkinCharacterDataPropertiesCharacterIdleEffect,
    pub champ_targeting_effect_definition: Option<SkinCharacterDataPropertiesCharacterIdleEffect>,
    pub tower_targeting_effect_definition: Option<SkinCharacterDataPropertiesCharacterIdleEffect>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TargeterDefinitionAoe {
    pub center_locator: Option<DrawablePositionLocator>,
    pub constraint_pos_locator: Option<DrawablePositionLocator>,
    pub constraint_range: Option<FloatPerSpellLevel>,
    pub dynamic_game_calc_size_scalar: Option<GameCalculationModified>,
    pub is_constrained_to_range: Option<bool>,
    pub max_range_size_scalar: Option<TargeterDefinitionAoeScalar>,
    pub override_radius: Option<FloatPerSpellLevel>,
    pub texture_orientation: Option<u32>,
    pub texture_radius_override_name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TargeterDefinitionAoeScalar {
    pub scalar: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TargeterDefinitionArc {
    pub constraint_range: FloatPerSpellLevel,
    pub end_locator: DrawablePositionLocator,
    pub is_constrained_to_range: bool,
    pub override_radius: FloatPerSpellLevel,
    pub start_locator: Option<DrawablePositionLocator>,
    pub texture_arc_override_name: Option<String>,
    pub thickness_offset: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TargeterDefinitionCone {
    pub cone_angle_degrees: Option<f32>,
    pub cone_follows_end: Option<bool>,
    pub cone_range: Option<f32>,
    pub end_locator: DrawablePositionLocator,
    pub fallback_direction: Option<u32>,
    pub start_locator: Option<DrawablePositionLocator>,
    pub texture_cone_override_name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TargeterDefinitionLine {
    pub always_draw: Option<bool>,
    pub arrow_size: Option<f32>,
    pub end_locator: Option<DrawablePositionLocator>,
    pub facing_line: Option<bool>,
    pub fade: Option<bool>,
    pub fallback_direction: Option<u32>,
    pub indicator_type: Option<EnumIndicatorType>,
    pub line_stops_at_end_position: Option<bool>,
    pub line_width: Option<FloatPerSpellLevel>,
    pub m_center_arrow_to_end_point: Option<bool>,
    pub m_fade_behavior: Option<FadeOverTimeBehavior>,
    pub max_angle: Option<f32>,
    pub minimum_displayed_range: Option<f32>,
    pub override_base_range: Option<FloatPerSpellLevel>,
    pub range_growth_duration: Option<FloatPerSpellLevel>,
    pub range_growth_max: Option<FloatPerSpellLevel>,
    pub range_growth_start_time: Option<FloatPerSpellLevel>,
    pub start_locator: Option<DrawablePositionLocator>,
    pub texture_base_max_grow_name: Option<String>,
    pub texture_base_override_name: Option<String>,
    pub texture_target_max_grow_name: Option<String>,
    pub texture_target_override_name: Option<String>,
    pub use_global_line_indicator: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TargeterDefinitionMinimap {
    pub center_locator: Option<DrawablePositionLocator>,
    pub override_base_range: Option<FloatPerSpellLevel>,
    pub use_caster_bounding_box: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TargeterDefinitionMultiAoe {
    pub angel_offset_radian: Option<f32>,
    pub center_locator: DrawablePositionLocator,
    pub inner_texture_name: String,
    pub left_texture_name: String,
    pub num_of_inner_aoe: Option<u32>,
    pub override_aoe_radius: Option<FloatPerSpellLevel>,
    pub override_max_cast_range: FloatPerSpellLevel,
    pub override_min_cast_range: FloatPerSpellLevel,
    pub right_texture_name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TargeterDefinitionRange {
    pub center_locator: Option<DrawablePositionLocator>,
    pub has_max_grow_range: Option<bool>,
    pub hide_with_line_indicator: Option<bool>,
    pub m_fade_behavior: Option<EnumBehavior>,
    pub override_base_range: Option<FloatPerSpellLevel>,
    pub range_growth_duration: Option<FloatPerSpellLevel>,
    pub range_growth_max: Option<FloatPerSpellLevel>,
    pub range_growth_start_time: Option<FloatPerSpellLevel>,
    pub texture_max_grow_name: Option<String>,
    pub texture_orientation: Option<u32>,
    pub texture_override_name: Option<String>,
    pub use_caster_bounding_box: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TargeterDefinitionSkipTerrain {
    pub m_base_texture_name: String,
    pub m_end_locator: DrawablePositionLocator,
    pub m_terrain_texture_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TargeterDefinitionSpline {
    pub base_texture_name: String,
    pub constraint_range: FloatPerSpellLevel,
    pub end_locator: DrawablePositionLocator,
    pub front_texture_name: String,
    pub is_constrained_to_range: bool,
    pub override_spline: HermiteSplineInfo,
    pub spline_width: FloatPerSpellLevel,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TargeterDefinitionWall {
    pub center_locator: DrawablePositionLocator,
    pub length: FloatPerSpellLevel,
    pub texture_wall_override_name: Option<String>,
    pub thickness: FloatPerSpellLevel,
    pub wall_rotation: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TargetingForgivenessDefinitions {
    pub caster_forgiveness_definitions: Option<Vec<SameTeamCastRequirement>>,
    pub forgiveness_range: f32,
    pub m_affects_type_override: Option<u32>,
    pub override_affects_flags: Option<bool>,
    pub target_forgiveness_definitions: Option<Vec<EnumHasAllSubRequirementsCastRequirement>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TargetingParameters {
    pub exit_conditions: Option<Vec<u8>>,
    pub m_affects_status_flags: Option<u32>,
    pub m_affects_type_flags: u32,
    pub m_spell_flags: Option<u32>,
    pub range_value: EnumTargetingRangeValue,
    pub targeting_param_name: Option<String>,
    pub unit_object_tags: Option<ObjectTags>,
    pub unk_0x791c5fa3: Option<bool>,
    pub unk_0x9845aa67: Option<bool>,
    pub unk_0xfc462d60: Option<Vec<Unk0xe90af953>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TargetingPriorityList {
    pub m_spell_flags: u32,
    pub targeting_parameters_list: Vec<TargetingParameters>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TargetingRangeValue {
    pub range: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TerrainType {
    pub m_brush_cursor: CursorData,
    pub m_river_cursor: CursorData,
    pub m_wall_cursor: CursorData,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TimeBlendData {
    pub m_time: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TimeMaterialDriver {
    pub loop_duration: Option<f32>,
    pub loop_time_as_fraction: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TimedVariableWaveBehavior {
    pub behaviors: Vec<Box<TimedWaveBehaviorInfo>>,
    pub default_spawn_count: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TimedWaveBehaviorInfo {
    pub behavior: Box<EnumWaveBehavior>,
    pub start_time_secs: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ToolAiPresence {
    pub easy: Option<bool>,
    pub hard: Option<bool>,
    pub intro: Option<bool>,
    pub medium: Option<bool>,
    pub unk_0x42ac598e: Option<bool>,
    pub unk_0x6175bb7b: Option<bool>,
    pub unk_0xb66d0e47: Option<bool>,
    pub unk_0xb75b2ab8: Option<bool>,
    pub unk_0xca762bfc: Option<bool>,
    pub unk_0xeba3cb5: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ToolAlternateForm {
    pub champion: Option<String>,
    pub name: String,
    pub spells: Option<Vec<String>>,
    pub the_switch: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ToolEducationData {
    pub first_item: i32,
    pub skill_order: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ToolInheritsData {
    pub recommended: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ToolPassiveData {
    pub effect: Option<Vec<String>>,
    pub level: Option<Vec<i32>>,
    pub name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ToolSoundData {
    pub attack: Option<Vec<String>>,
    pub click: Option<Vec<String>>,
    pub death: Option<String>,
    pub r#move: Option<Vec<String>>,
    pub ready: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ToolSpellDesc {
    pub desc: Option<String>,
    pub display_name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TooltipInstanceBuff {
    pub m_format: u32,
    pub m_loc_keys: Option<HashMap<String, String>>,
    pub m_object_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TooltipInstanceList {
    pub elements: Option<Vec<TooltipInstanceListElement>>,
    pub level_count: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TooltipInstanceListElement {
    pub multiplier: Option<f32>,
    pub name_override: Option<String>,
    pub r#type: String,
    pub style: Option<u32>,
    pub type_index: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TooltipInstanceSpell {
    pub enable_extended_tooltip: Option<bool>,
    pub m_format: u32,
    pub m_lists: Option<HashMap<String, TooltipInstanceList>>,
    pub m_loc_keys: Option<HashMap<String, String>>,
    pub m_object_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TrackData {
    pub m_blend_mode: Option<u8>,
    pub m_blend_weight: Option<f32>,
    pub m_priority: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TrackMouseMovement {
    pub m_acceleration: f32,
    pub m_anti_lag_delay: f32,
    pub m_initial_speed: f32,
    pub m_max_speed: f32,
    pub m_min_speed: f32,
    pub m_start_bone_name: String,
    pub m_target_bone_name: String,
    pub m_tracks_target: bool,
    pub m_turn_radius_by_level: Vec<f32>,
    pub m_use_ground_height_at_target: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TransitionClipBlendData {
    pub m_clip_name: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TreatAutoAttacksAsNormalSpells {
    pub auto_attack_spells_use_spell_source: bool,
    pub override_queryable_attack_range: GameCalculation,
    pub skip_sequenced_attack_events: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TriggerFromScript {
    pub m_actions: Vec<EnumAttackEvents>,
    pub m_delay: Option<f32>,
    pub m_trigger_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TriggerOnDelay {
    pub m_actions: Vec<EnumAttackEvents>,
    pub m_delay: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TriggerOnHit {
    pub m_actions: Vec<EnumAttackEvents>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TriggerOnMovementComplete {
    pub m_actions: Vec<EnumAttackEvents>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UiDraggableBasic {
    pub use_sticky_drag: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UiDraggableElementGroupDrag {
    pub use_sticky_drag: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UiDraggableProxyElementDrag {
    pub proxy_element: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UiDraggableSceneDrag {
    pub use_sticky_drag: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Asset, TypePath)]
#[serde(rename_all = "camelCase")]
pub struct UiElementEffectAnimationData {
    pub enabled: Option<bool>,
    pub frames_per_second: Option<f32>,
    pub layer: Option<u32>,
    pub m_finish_behavior: Option<u8>,
    pub m_per_pixel_uvs_x: Option<bool>,
    pub name: String,
    pub number_of_frames_per_row_in_atlas: Option<f32>,
    pub position: UiPositionRect,
    pub scene: u32,
    pub texture_data: EnumData,
    pub total_number_of_frames: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Asset, TypePath)]
#[serde(rename_all = "camelCase")]
pub struct UiElementGroupButtonData {
    pub active_tooltip_tra_key: Option<String>,
    pub add_text_size_to_hit_region: Option<bool>,
    pub click_release_particle_element: Option<u32>,
    pub clicked_state_elements: Option<UiElementGroupButtonState>,
    pub default_state_elements: Option<UiElementGroupButtonState>,
    pub elements: Vec<u32>,
    pub hit_region_element: u32,
    pub hover_state_elements: Option<UiElementGroupButtonState>,
    pub inactive_selected_state_elements: Option<UiElementGroupButtonState>,
    pub inactive_state_elements: Option<UiElementGroupButtonState>,
    pub inactive_tooltip_tra_key: Option<String>,
    pub is_active: Option<bool>,
    pub is_enabled: Option<bool>,
    pub is_focusable: Option<bool>,
    pub is_selected: Option<bool>,
    pub name: String,
    pub scene: u32,
    pub selected_clicked_state_elements: Option<UiElementGroupButtonState>,
    pub selected_hover_state_elements: Option<UiElementGroupButtonState>,
    pub selected_state_elements: Option<UiElementGroupButtonState>,
    pub selected_tooltip_tra_key: Option<String>,
    pub sound_events: Option<UiElementGroupButtonSoundEvents>,
    pub tab_order: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UiElementGroupButtonSoundEvents {
    pub mouse_down_event: Option<String>,
    pub mouse_down_on_inactive: Option<String>,
    pub mouse_down_selected: Option<String>,
    pub mouse_up_event: Option<String>,
    pub mouse_up_selected: Option<String>,
    pub roll_over_event: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UiElementGroupButtonState {
    pub display_element_list: Option<Vec<u32>>,
    pub text_element: Option<u32>,
    pub text_frame_element: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Asset, TypePath)]
#[serde(rename_all = "camelCase")]
pub struct UiElementIconData {
    pub block_input_events: Option<bool>,
    pub color: Option<[u8; 4]>,
    pub drag_type: Option<EnumUiDraggable>,
    pub enabled: Option<bool>,
    pub extension: Option<EnumIconElement>,
    pub fill_type: Option<u32>,
    pub flip_x: Option<bool>,
    pub flip_y: Option<bool>,
    pub layer: Option<u32>,
    pub material: Option<u32>,
    pub name: String,
    pub per_pixel_uvs_x: Option<bool>,
    pub position: EnumUiPosition,
    pub scene: u32,
    pub texture_data: Option<EnumData>,
    pub use_alpha: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UiElementRect {
    pub position: Option<Vec2>,
    pub size: Option<Vec2>,
    pub source_resolution_height: Option<u16>,
    pub source_resolution_width: Option<u16>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Asset, TypePath)]
#[serde(rename_all = "camelCase")]
pub struct UiElementRegionData {
    pub block_input_events: Option<bool>,
    pub drag_type: Option<EnumUiDraggable>,
    pub enabled: Option<bool>,
    pub layer: Option<u32>,
    pub name: String,
    pub position: Option<EnumUiPosition>,
    pub scene: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UiPositionPolygon {
    pub anchors: AnchorSingle,
    pub polygon_vertices: Vec<Vec2>,
    pub ui_rect: UiElementRect,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UiPositionRect {
    pub anchors: Option<EnumAnchor>,
    pub disable_pixel_snapping_x: Option<bool>,
    pub disable_pixel_snapping_y: Option<bool>,
    pub disable_resolution_downscale: Option<bool>,
    pub ignore_global_scale: Option<bool>,
    pub ignore_safe_zone: Option<bool>,
    pub ui_rect: Option<UiElementRect>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Asset, TypePath)]
#[serde(rename_all = "camelCase")]
pub struct UiPropertyLoadable {
    pub filepath_hash: u64,
    pub unk_0xe50d4da6: Option<EnumUnk0x797fe1c2>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Asset, TypePath)]
#[serde(rename_all = "camelCase")]
pub struct UiSceneData {
    pub enabled: Option<bool>,
    pub handle_input_during_pause: Option<bool>,
    pub inherit_scissoring: Option<bool>,
    pub layer: Option<u32>,
    pub name: String,
    pub parent_scene: Option<u32>,
    pub scene_transition_in: Option<EnumTransitionData>,
    pub scene_transition_out: Option<EnumTransitionData>,
    pub unk_0x49d8f2c4: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UiUnitStatusData {
    pub center_justify_status_icon_and_text: bool,
    pub name_text: u32,
    pub status_duration_bar_data: Option<UiUnitStatusDurationBarData>,
    pub status_icon: u32,
    pub status_text: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UiUnitStatusDurationBarData {
    pub status_duration_bar: u32,
    pub tenacity_bar: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Asset, TypePath)]
#[serde(rename_all = "camelCase")]
pub struct UnitFloatingInfoBarData {
    pub aggro: u32,
    pub anchor: u32,
    pub border: u32,
    pub death_anim_ally: Option<u32>,
    pub death_anim_enemy: Option<u32>,
    pub health_bar: HealthBarData,
    pub highlight: u32,
    pub icons: Option<HeroFloatingInfoIconsData>,
    pub objective_bounty_ally: Option<u32>,
    pub objective_bounty_enemy: Option<u32>,
    pub par_bar: Option<AbilityResourceBarData>,
    pub scene: u32,
    pub scripted_threshold_types: Option<HashMap<u32, u32>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UnitStatusData {
    pub attackable_unit_status_type: Option<u32>,
    pub loc_type: Option<u32>,
    pub status_name: String,
    pub texture_uvs: Vec4,
}

#[derive(Serialize, Deserialize, Debug, Clone, Asset, TypePath)]
#[serde(rename_all = "camelCase")]
pub struct UnitStatusPriorityList {
    pub m_prioritized_unit_status_data: Vec<UnitStatusData>,
    pub texture_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x125a3586 {
    pub unk_0xe61bf09e: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x150d1b92 {
    pub unk_0x717e686: Option<bool>,
    pub unk_0xe38f54f7: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x19da44b2 {
    pub unk_0x44146c9d: Vec<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x1aae122 {
    pub max_distance: f32,
    pub max_offset_delta: Option<f32>,
    pub min_distance: f32,
    pub unk_0x7863785e: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x1d452085 {
    pub m_stat: Option<u8>,
    pub unk_0x137cf12a: u32,
    pub unk_0xa519b194: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x1f1f50f2 {
    pub definition: Unk0x8ad25772,
    pub name: u32,
    pub transform: Mat4,
    pub unk_0xbbe68da1: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x1f2e5fd0 {
    pub group: Option<u32>,
    pub unk_0x752ff961: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x25e3f5d0 {
    pub definition: Unk0xf775806c,
    pub name: u32,
    pub transform: Mat4,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x280745b1 {
    pub params: Unk0xc7e628b9,
    pub unk_0x50aad250: Vec<Unk0xdd661aab>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x303c66f4 {
    pub buff_type: u8,
    pub unk_0xe93cd19c: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x313c0076 {
    pub enabled: bool,
    pub icon: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x382277da {
    pub m_subparts: Vec<Box<EnumAbilityResourceByCoefficientCalculationPart>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x3c995caf {
    pub name: String,
    pub segments: Vec<Vec3>,
    pub transform: Mat4,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x429a2180 {
    pub camp_level: Option<u16>,
    pub minimap_icon: Option<String>,
    pub minimap_icon_offset: Option<Vec3>,
    pub reveal_event: Option<u16>,
    pub scoreboard_timer: Option<u16>,
    pub stop_spawn_time_secs: Option<f32>,
    pub tags: Option<Vec<u32>>,
    pub team: u32,
    pub unk_0x1f2e5fd0: Option<Unk0x1f2e5fd0>,
    pub unk_0x5a4ef4e7: u32,
    pub unk_0x7d27af7f: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x47f13ab0 {
    pub unk_0xcf19cb5d: Unk0x770f7888,
    pub unk_0xe4f7105d: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x4a70b12c {
    pub augment_group: Vec<u32>,
    pub unk_0x9a676645: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x4cf74021 {
    pub definition: Unk0xfbbe5989,
    pub name: u32,
    pub transform: Mat4,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x51445de9 {
    pub value: Vec4,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x557bb273 {
    pub value: Option<Vec4>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x55f6bf86 {
    pub effect_key: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x56bb851 {
    pub unk_0xe6d60f41: Option<HashMap<u8, Unk0xc76c1b9a>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x5b2fdd66 {
    pub add: FloatLiteralMaterialDriver,
    pub value: Box<EnumDriver>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x6355dd6f {
    pub chunk: u32,
    pub visibility_controller: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x635d04b7 {
    pub champion_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x671b7351 {
    pub vfx_group_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x6bbc3db6 {
    pub spell_objects: Vec<u32>,
    pub unk_0xda28e4c: [u8; 4],
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x72f86c81 {
    pub unk_0xccfd27e6: Option<f32>,
    pub unk_0xdc9124b1: Option<[u8; 4]>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x75e34c40 {
    pub unk_0x1dcc5270: Vec<Unk0xd5c9eb1>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x770f7888 {
    pub armor_per_level: Option<f32>,
    pub attack_speed_per_level: Option<f32>,
    pub base_armor: Option<f32>,
    pub base_hp: Option<f32>,
    pub damage_per_level: Option<f32>,
    pub hp_per_level: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x7a1a2d27 {
    pub absorbed_damage_format: u32,
    pub combinable_damage_format: u32,
    pub critical_magical_damage_format: u32,
    pub critical_physical_damage_format: u32,
    pub critical_true_damage_format: u32,
    pub default_magical_damage_format: u32,
    pub default_physical_damage_format: u32,
    pub default_true_damage_format: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x7faa90a0 {
    pub character_record: String,
    pub idle_animation_name: String,
    pub play_idle_animation: Option<bool>,
    pub skin: String,
    pub team: Option<u32>,
    pub unk_0x86f3c70: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x7fb92f53 {
    pub unk_0x28de30d6: f32,
    pub unk_0x3c475337: f32,
    pub unk_0xc865acd9: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x82cab1b3 {
    pub lane: u16,
    pub position: Option<u16>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x85a6a05c {
    pub unk_0xffeb3531: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x8958fee2 {
    pub unk_0x79a2e7aa: Option<f32>,
    pub unk_0xffcbd9e2: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x8a96ea3c {
    pub m_subparts: Vec<Box<EnumAbilityResourceByCoefficientCalculationPart>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x8ad25772 {
    pub system: u32,
    pub unk_0x63176011: Option<bool>,
    pub unk_0x86f3c70: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x92024c11 {
    pub max: Vec3,
    pub min: Vec3,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x93adc5b3 {
    pub distance_between: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x9aa5b4bc {
    pub definition: Unk0x7faa90a0,
    pub name: u32,
    pub transform: Mat4,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x9bc366ca {
    pub skin_id: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x9c1d99c0 {
    pub spells: Vec<u32>,
    pub unk_0x80cf3335: Unk0x7a1a2d27,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x9d62f7e {
    pub named_data_value: u32,
    pub spell: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x9d9f60d2 {
    pub character_record: String,
    pub r#type: Option<u16>,
    pub skin: String,
    pub tags: Option<Vec<u32>>,
    pub team: Option<u32>,
    pub unk_0x33c6fd60: Option<Unk0x313c0076>,
    pub unk_0x397fe037: Option<bool>,
    pub unk_0xdbde2288: Option<Vec<Unk0x82cab1b3>>,
    pub unk_0xde46f1d8: Option<String>,
    pub unk_0xf1d3a034: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0x9e9e2e5c {
    pub source_object: u32,
    pub unk_0x137cf12a: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Asset, TypePath)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xad65d8c4 {
    pub definition: Unk0x9d9f60d2,
    pub name: u32,
    pub transform: Option<Mat4>,
    pub unk_0xbbe68da1: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xb09016f6 {
    pub effect_calculation: GameCalculation,
    pub effect_tag: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xb22609db {
    pub unk_0x91d404a5: u32,
    pub unk_0xb2cd0eb0: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xb7b43e1d {
    pub bool_driver: IsAnimationPlayingDynamicMaterialBoolDriver,
    pub percentage: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xba007871 {
    pub source_object: u32,
    pub unk_0x3de6ce2d: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xba138ae3 {
    pub definition: Unk0xfde6a2d7,
    pub name: u32,
    pub transform: Mat4,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xc76c1b9a {
    pub modifiers: Vec<EnumUnk0x51445de9>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xc7e628b9 {
    pub spell: u32,
    pub unk_0x877e4953: u32,
    pub unk_0xa2877ddb: u32,
    pub unk_0xd00e123a: u32,
    pub unk_0xe5bc4229: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xc96d9140 {
    pub unk_0x1418c47f: u32,
    pub unk_0xa2cb8e03: Option<HashMap<String, u32>>,
    pub unk_0xc19c58be: Option<HashMap<String, String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xcd5a34f5 {
    pub animation_name: Option<String>,
    pub mesh_name: Option<String>,
    pub mesh_scale: Option<f32>,
    pub skeleton_name: Option<String>,
    pub submeshes: Option<Vec<u32>>,
    pub use_surface_normal_for_birth_physics: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xcdb1c8f6 {
    pub unk_0x6355dd6f: Vec<Unk0x6355dd6f>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xcdf661db {
    pub category: String,
    pub unk_0x2de18da: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xcf4a55da {
    pub overlays: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xd178749c {
    pub definition: Unk0x429a2180,
    pub name: u32,
    pub transform: Mat4,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xd5c9eb1 {
    pub event_name: u32,
    pub unk_0x1004c9c8: HashMap<u32, Unk0x56bb851>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xd91a223 {
    pub unk_0x68309c0b: bool,
    pub unk_0xe2e5b6dd: Vec<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xdd661aab {
    pub override_params: Option<Unk0xc7e628b9>,
    pub trigger_spells: Vec<u32>,
    pub unk_0x6cd45762: bool,
    pub unk_0x77cff90e: Unk0xd91a223,
    pub unk_0x8f7842e4: Vec<Unk0x55f6bf86>,
    pub unk_0x96e77860: u32,
    pub unk_0xda1ee5bc: bool,
    pub unk_0xe4ecb00c: Unk0xfb16e4be,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xe1555e0a {
    pub augment_group: Vec<u32>,
    pub unk_0x9a676645: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xe2ef74d0 {
    pub slot: u8,
    pub unk_0xc073c624: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xe6147387 {
    pub default_on: bool,
    pub joints: Vec<u32>,
    pub orientation_source: Unk0x19da44b2,
    pub orientation_type: u8,
    pub unk_0x1a30a486: bool,
    pub unk_0x420b233d: f32,
    pub unk_0xa57f0269: u8,
    pub unk_0xab2e032a: u8,
    pub unk_0xae1cbd5f: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xe7b61183 {
    pub unk_0x44146c9d: Vec<u32>,
    pub unk_0x8f149e18: f32,
    pub unk_0xe1795243: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xe7ee4f28 {
    pub unk_0x7dd33afb: u32,
    pub unk_0xa2cb8e03: Option<HashMap<String, u32>>,
    pub unk_0xc19c58be: Option<HashMap<String, String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xe90af953 {
    pub buff: u32,
    pub unk_0xbe161d6e: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xeb997689 {
    pub definition: Unk0xfcb92181,
    pub name: u32,
    pub transform: Mat4,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xee18a47b {
    pub unk_0x589a59c: u32,
    pub unk_0xb65bc23: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xee39916f {
    pub emit_offset: Option<Vec3>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xf090d2e7 {
    pub unk_0x9567c2a: Option<u8>,
    pub unk_0xa567dbd: Option<u8>,
    pub unk_0xbe0de52: Option<u8>,
    pub unk_0xce0dfe5: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xf3cbe7b2 {
    pub m_spell_calculation_key: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xf775806c {
    pub character_record: String,
    pub skin: String,
    pub team: Option<u32>,
    pub unk_0x651de225: f32,
    pub unk_0xd1318f26: f32,
    pub unk_0xf908963: Vec3,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xfb16e4be {
    pub order_types: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xfbbe5989 {
    pub unit_tags: ObjectTags,
    pub unk_0x998d15e9: bool,
    pub unk_0x9a6a9339: u32,
    pub unk_0xf43f2e26: Unk0x92024c11,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xfcb92181 {
    pub tags: Option<Vec<u32>>,
    pub team: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xfde6a2d7 {
    pub barracks_config: u32,
    pub team: Option<u32>,
    pub unk_0xdb6ea1a7: Option<u32>,
    pub unk_0xdbde2288: Vec<Unk0x82cab1b3>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unk0xfe70e9c4 {
    pub unk_0x3ef62dce: u8,
    pub unk_0x4e748038: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpdaterData {
    pub input: EnumParametricUpdater,
    pub m_output_type: u32,
    pub m_value_processor_data_list: Option<Vec<LinearTransformProcessorData>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpdaterResourceData {
    pub m_updater_data_list: Option<Vec<UpdaterData>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UseAutoattackCastTimeData {
    pub m_autoattack_cast_time_calculation: Option<GameCalculation>,
    pub m_use_cast_time_as_total_time: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UseableData {
    pub flags: Option<u32>,
    pub use_cooldown_spell_slot: Option<i32>,
    pub use_hero_spell_name: Option<String>,
    pub use_spell_name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UvScaleBiasFromAnimationDynamicMaterialDriver {
    pub m_sub_mesh_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ValueColor {
    pub constant_value: Option<Vec4>,
    pub dynamics: Option<VfxAnimatedColorVariableData>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ValueFloat {
    pub constant_value: Option<f32>,
    pub dynamics: Option<VfxAnimatedFloatVariableData>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ValueVector2 {
    pub constant_value: Option<Vec2>,
    pub dynamics: Option<VfxAnimatedVector2fVariableData>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ValueVector3 {
    pub constant_value: Option<Vec3>,
    pub dynamics: Option<VfxAnimatedVector3fVariableData>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxAlphaErosionDefinitionData {
    pub erosion_drive_curve: Option<ValueFloat>,
    pub erosion_drive_source: Option<u8>,
    pub erosion_feather_in: Option<f32>,
    pub erosion_feather_out: Option<f32>,
    pub erosion_map_address_mode: Option<u8>,
    pub erosion_map_channel_mixer: Option<ValueColor>,
    pub erosion_map_name: Option<String>,
    pub erosion_slice_width: Option<f32>,
    pub linger_erosion_drive_curve: Option<ValueFloat>,
    pub use_linger_erosion_drive_curve: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxAnimatedColorVariableData {
    pub probability_tables: Option<Vec<VfxProbabilityTableData>>,
    pub times: Option<Vec<f32>>,
    pub values: Option<Vec<Vec4>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxAnimatedFloatVariableData {
    pub probability_tables: Option<Vec<VfxProbabilityTableData>>,
    pub times: Vec<f32>,
    pub values: Vec<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxAnimatedVector2fVariableData {
    pub probability_tables: Option<Vec<VfxProbabilityTableData>>,
    pub times: Vec<f32>,
    pub values: Vec<Vec2>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxAnimatedVector3fVariableData {
    pub probability_tables: Option<Vec<VfxProbabilityTableData>>,
    pub times: Vec<f32>,
    pub values: Vec<Vec3>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxAssetRemap {
    pub new_asset: Option<String>,
    pub old_asset: Option<u32>,
    pub r#type: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxBeamDefinitionData {
    pub m_animated_color_with_distance: Option<ValueColor>,
    pub m_birth_tiling_size: Option<ValueVector3>,
    pub m_is_color_binded_with_distance: Option<bool>,
    pub m_local_space_source_offset: Option<Vec3>,
    pub m_local_space_target_offset: Option<Vec3>,
    pub m_mode: Option<u8>,
    pub m_segments: Option<i32>,
    pub m_trail_mode: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxChildIdentifier {
    pub effect: Option<u32>,
    pub effect_key: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxChildParticleSetDefinitionData {
    pub bone_to_spawn_at: Option<Vec<String>>,
    pub child_emit_on_death: Option<bool>,
    pub children_identifiers: Option<Vec<VfxChildIdentifier>>,
    pub children_probability: Option<ValueFloat>,
    pub parent_inheritance_definition: Option<VfxParentInheritanceParams>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxColorOverLifeMaterialDriver {
    pub colors: VfxAnimatedColorVariableData,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxDistortionDefinitionData {
    pub distortion: Option<f32>,
    pub distortion_mode: Option<u8>,
    pub normal_map_texture: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxEmissionSurfaceData {
    pub unk_0x2808fffd: Option<Unk0xcd5a34f5>,
    pub unk_0xf8b81c77: Option<Unk0x671b7351>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxEmitterAudio {
    pub sound_on_create: Option<String>,
    pub sound_persistent: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxEmitterDefinitionData {
    pub acceleration: Option<ValueVector3>,
    pub alpha_erosion_definition: Option<VfxAlphaErosionDefinitionData>,
    pub alpha_ref: Option<u8>,
    pub audio: Option<VfxEmitterAudio>,
    pub bind_weight: Option<ValueFloat>,
    pub birth_acceleration: Option<ValueVector3>,
    pub birth_color: Option<ValueColor>,
    pub birth_drag: Option<ValueVector3>,
    pub birth_frame_rate: Option<ValueFloat>,
    pub birth_orbital_velocity: Option<ValueVector3>,
    pub birth_rotation0: Option<ValueVector3>,
    pub birth_rotational_acceleration: Option<ValueVector3>,
    pub birth_rotational_velocity0: Option<ValueVector3>,
    pub birth_scale0: Option<ValueVector3>,
    pub birth_uv_offset: Option<ValueVector2>,
    pub birth_uv_rotate_rate: Option<ValueFloat>,
    pub birth_uv_scroll_rate: Option<ValueVector2>,
    pub birth_velocity: Option<ValueVector3>,
    pub blend_mode: Option<u8>,
    pub censor_modulate_value: Option<Vec4>,
    pub chance_to_not_exist: Option<f32>,
    pub child_particle_set_definition: Option<VfxChildParticleSetDefinitionData>,
    pub color: Option<ValueColor>,
    pub color_look_up_offsets: Option<Vec2>,
    pub color_look_up_scales: Option<Vec2>,
    pub color_look_up_type_x: Option<u8>,
    pub color_look_up_type_y: Option<u8>,
    pub color_render_flags: Option<u8>,
    pub colorblind_visibility: Option<u8>,
    pub custom_material: Option<VfxMaterialDefinitionData>,
    pub depth_bias_factors: Option<Vec2>,
    pub direction_velocity_min_scale: Option<f32>,
    pub direction_velocity_scale: Option<f32>,
    pub disable_backface_cull: Option<bool>,
    pub disabled: Option<bool>,
    pub distortion_definition: Option<VfxDistortionDefinitionData>,
    pub does_cast_shadow: Option<bool>,
    pub does_lifetime_scale: Option<bool>,
    pub drag: Option<ValueVector3>,
    pub emission_mesh_name: Option<String>,
    pub emission_mesh_scale: Option<f32>,
    pub emission_surface_definition: Option<VfxEmissionSurfaceData>,
    pub emitter_linger: Option<f32>,
    pub emitter_name: Option<String>,
    pub emitter_position: Option<ValueVector3>,
    pub emitter_uv_scroll_rate: Option<Vec2>,
    pub falloff_texture: Option<String>,
    pub field_collection_definition: Option<VfxFieldCollectionDefinitionData>,
    pub filtering: Option<VfxEmitterFiltering>,
    pub flex_birth_rotational_velocity0: Option<FlexValueVector3>,
    pub flex_birth_uv_offset: Option<FlexValueVector2>,
    pub flex_birth_uv_scroll_rate: Option<FlexValueVector2>,
    pub flex_birth_velocity: Option<FlexValueVector3>,
    pub flex_instance_scale: Option<FlexTypeFloat>,
    pub flex_particle_lifetime: Option<FlexValueFloat>,
    pub flex_rate: Option<FlexValueFloat>,
    pub flex_scale_birth_scale: Option<FlexTypeFloat>,
    pub flex_shape_definition: Option<VfxFlexShapeDefinitionData>,
    pub frame_rate: Option<f32>,
    pub has_post_rotate_orientation: Option<bool>,
    pub has_variable_start_time: Option<bool>,
    pub importance: Option<u8>,
    pub is_direction_oriented: Option<bool>,
    pub is_emitter_space: Option<bool>,
    pub is_following_terrain: Option<bool>,
    pub is_ground_layer: Option<bool>,
    pub is_local_orientation: Option<bool>,
    pub is_random_start_frame: Option<bool>,
    pub is_rotation_enabled: Option<bool>,
    pub is_single_particle: Option<bool>,
    pub is_texture_pixelated: Option<bool>,
    pub is_uniform_scale: Option<bool>,
    pub legacy_simple: Option<VfxEmitterLegacySimple>,
    pub lifetime: Option<f32>,
    pub linger: Option<VfxLingerDefinitionData>,
    pub material_override_definitions: Option<Vec<VfxMaterialOverrideDefinitionData>>,
    pub maximum_rate_by_velocity: Option<f32>,
    pub mesh_render_flags: Option<u8>,
    pub misc_render_flags: Option<u8>,
    pub modulation_factor: Option<Vec4>,
    pub num_frames: Option<u16>,
    pub offset_life_scaling_symmetry_mode: Option<u8>,
    pub offset_lifetime_scaling: Option<Vec3>,
    pub palette_definition: Option<VfxPaletteDefinitionData>,
    pub particle_color_texture: Option<String>,
    pub particle_is_local_orientation: Option<bool>,
    pub particle_lifetime: Option<ValueFloat>,
    pub particle_linger: Option<f32>,
    pub particle_linger_type: Option<u8>,
    pub particle_uv_rotate_rate: Option<IntegratedValueFloat>,
    pub particle_uv_scroll_rate: Option<IntegratedValueVector2>,
    pub particles_share_random_value: Option<bool>,
    pub pass: Option<i16>,
    pub period: Option<f32>,
    pub post_rotate_orientation_axis: Option<Vec3>,
    pub primitive: Option<EnumVfxPrimitive>,
    pub rate: Option<ValueFloat>,
    pub rate_by_velocity_function: Option<ValueVector2>,
    pub reflection_definition: Option<VfxReflectionDefinitionData>,
    pub render_phase_override: Option<u8>,
    pub rotation0: Option<IntegratedValueVector3>,
    pub rotation_override: Option<Vec3>,
    pub scale0: Option<ValueVector3>,
    pub scale_override: Option<Vec3>,
    pub slice_technique_range: Option<f32>,
    pub soft_particle_params: Option<VfxSoftParticleDefinitionData>,
    pub sort_emitters_by_pos: Option<bool>,
    pub spawn_shape: Option<EnumVfxShape>,
    pub start_frame: Option<u16>,
    pub stencil_mode: Option<u8>,
    pub stencil_ref: Option<u8>,
    pub stencil_reference_id: Option<u32>,
    pub tex_address_mode_base: Option<u8>,
    pub tex_div: Option<Vec2>,
    pub texture: Option<String>,
    pub texture_flip_u: Option<bool>,
    pub texture_flip_v: Option<bool>,
    pub texture_mult: Option<VfxTextureMultDefinitionData>,
    pub time_active_during_period: Option<f32>,
    pub time_before_first_emission: Option<f32>,
    pub translation_override: Option<Vec3>,
    pub unk_0xcb13aff1: Option<f32>,
    pub unk_0xd1ee8634: Option<bool>,
    pub use_emission_mesh_normal_for_birth: Option<bool>,
    pub use_navmesh_mask: Option<bool>,
    pub uv_mode: Option<u8>,
    pub uv_parallax_scale: Option<f32>,
    pub uv_rotation: Option<ValueFloat>,
    pub uv_scale: Option<ValueVector2>,
    pub uv_scroll_clamp: Option<bool>,
    pub uv_transform_center: Option<Vec2>,
    pub velocity: Option<ValueVector3>,
    pub world_acceleration: Option<IntegratedValueVector3>,
    pub write_alpha_only: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxEmitterFiltering {
    pub censor_policy: Option<u8>,
    pub keywords_excluded: Option<Vec<String>>,
    pub keywords_required: Option<Vec<String>>,
    pub spectator_policy: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxEmitterLegacySimple {
    pub birth_rotation: Option<ValueFloat>,
    pub birth_rotational_velocity: Option<ValueFloat>,
    pub birth_scale: Option<ValueFloat>,
    pub has_fixed_orbit: Option<bool>,
    pub orientation: Option<u8>,
    pub particle_bind: Option<Vec2>,
    pub rotation: Option<ValueFloat>,
    pub scale: Option<ValueFloat>,
    pub scale_bias: Option<Vec2>,
    pub uv_scroll_rate: Option<Vec2>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxFieldAccelerationDefinitionData {
    pub acceleration: Option<ValueVector3>,
    pub is_local_space: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxFieldAttractionDefinitionData {
    pub acceleration: Option<ValueFloat>,
    pub position: Option<ValueVector3>,
    pub radius: Option<ValueFloat>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxFieldCollectionDefinitionData {
    pub field_acceleration_definitions: Option<Vec<VfxFieldAccelerationDefinitionData>>,
    pub field_attraction_definitions: Option<Vec<VfxFieldAttractionDefinitionData>>,
    pub field_drag_definitions: Option<Vec<VfxFieldDragDefinitionData>>,
    pub field_noise_definitions: Option<Vec<VfxFieldNoiseDefinitionData>>,
    pub field_orbital_definitions: Option<Vec<VfxFieldOrbitalDefinitionData>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxFieldDragDefinitionData {
    pub position: Option<ValueVector3>,
    pub radius: Option<ValueFloat>,
    pub strength: Option<ValueFloat>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxFieldNoiseDefinitionData {
    pub axis_fraction: Option<Vec3>,
    pub frequency: Option<ValueFloat>,
    pub position: Option<ValueVector3>,
    pub radius: Option<ValueFloat>,
    pub velocity_delta: Option<ValueFloat>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxFieldOrbitalDefinitionData {
    pub direction: Option<ValueVector3>,
    pub is_local_space: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxFlexShapeDefinitionData {
    pub flex_birth_translation: Option<FlexValueVector3>,
    pub flex_scale_emit_offset: Option<FlexTypeFloat>,
    pub scale_birth_scale_by_bound_object_height: Option<f32>,
    pub scale_birth_scale_by_bound_object_radius: Option<f32>,
    pub scale_birth_scale_by_bound_object_size: Option<f32>,
    pub scale_birth_translation_by_bound_object_size: Option<f32>,
    pub scale_emit_offset_by_bound_object_height: Option<f32>,
    pub scale_emit_offset_by_bound_object_radius: Option<f32>,
    pub scale_emit_offset_by_bound_object_size: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxFloatOverLifeMaterialDriver {
    pub frequency: Option<u8>,
    pub graph: VfxAnimatedFloatVariableData,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxLingerDefinitionData {
    pub keyed_linger_acceleration: Option<ValueVector3>,
    pub keyed_linger_drag: Option<ValueVector3>,
    pub keyed_linger_velocity: Option<ValueVector3>,
    pub linger_rotation: Option<ValueVector3>,
    pub linger_scale: Option<ValueVector3>,
    pub separate_linger_color: Option<ValueColor>,
    pub use_keyed_linger_acceleration: Option<bool>,
    pub use_keyed_linger_drag: Option<bool>,
    pub use_keyed_linger_velocity: Option<bool>,
    pub use_linger_rotation: Option<bool>,
    pub use_linger_scale: Option<bool>,
    pub use_separate_linger_color: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxMaterialDefinitionData {
    pub material: u32,
    pub material_drivers: Option<HashMap<String, EnumOverLifeMaterialDriver>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxMaterialOverrideDefinitionData {
    pub base_texture: Option<String>,
    pub gloss_texture: Option<String>,
    pub material: Option<u32>,
    pub override_blend_mode: Option<u32>,
    pub priority: Option<i32>,
    pub sub_mesh_name: Option<String>,
    pub transition_sample: Option<f32>,
    pub transition_source: Option<u32>,
    pub transition_texture: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxMeshDefinitionData {
    pub m_animation_name: Option<String>,
    pub m_animation_variants: Option<Vec<String>>,
    pub m_lock_mesh_to_attachment: Option<bool>,
    pub m_mesh_name: Option<String>,
    pub m_mesh_skeleton_name: Option<String>,
    pub m_simple_mesh_name: Option<String>,
    pub m_submeshes_to_draw: Option<Vec<u32>>,
    pub m_submeshes_to_draw_always: Option<Vec<u32>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxPaletteDefinitionData {
    pub palette_count: Option<i32>,
    pub palette_selector: Option<ValueVector3>,
    pub palette_texture: Option<String>,
    pub palette_texture_address_mode: Option<u8>,
    pub palette_u_animation_curve: Option<ValueFloat>,
    pub palette_v_animation_curve: Option<ValueFloat>,
    pub pallete_src_mix_color: Option<ValueColor>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxParentInheritanceParams {
    pub mode: Option<u8>,
    pub relative_offset: Option<ValueVector3>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxPrimitiveArbitraryTrail {
    pub m_trail: Option<VfxTrailDefinitionData>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxPrimitiveAttachedMesh {
    pub align_pitch_to_camera: Option<bool>,
    pub align_yaw_to_camera: Option<bool>,
    pub m_mesh: Option<VfxMeshDefinitionData>,
    pub unk_0x6aec9e7a: Option<bool>,
    pub use_avatar_specific_submesh_mask: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxPrimitiveBeam {
    pub m_beam: Option<VfxBeamDefinitionData>,
    pub m_mesh: Option<VfxMeshDefinitionData>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxPrimitiveCameraSegmentBeam {
    pub m_beam: Option<VfxBeamDefinitionData>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxPrimitiveCameraTrail {
    pub m_trail: Option<VfxTrailDefinitionData>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxPrimitiveMesh {
    pub align_pitch_to_camera: Option<bool>,
    pub align_yaw_to_camera: Option<bool>,
    pub m_mesh: Option<VfxMeshDefinitionData>,
    pub unk_0x6aec9e7a: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxPrimitivePlanarProjection {
    pub m_projection: Option<VfxProjectionDefinitionData>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxProbabilityTableData {
    pub key_times: Option<Vec<f32>>,
    pub key_values: Option<Vec<f32>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxProjectionDefinitionData {
    pub color_modulate: Option<ValueColor>,
    pub m_fading: Option<f32>,
    pub m_y_range: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxReflectionDefinitionData {
    pub fresnel: Option<f32>,
    pub fresnel_color: Option<Vec4>,
    pub reflection_fresnel: Option<f32>,
    pub reflection_fresnel_color: Option<Vec4>,
    pub reflection_map_texture: Option<String>,
    pub reflection_opacity_direct: Option<f32>,
    pub reflection_opacity_glancing: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxShapeBox {
    pub flags: Option<u8>,
    pub size: Option<Vec3>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxShapeCylinder {
    pub flags: Option<u8>,
    pub height: Option<f32>,
    pub radius: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxShapeLegacy {
    pub emit_offset: Option<ValueVector3>,
    pub emit_rotation_angles: Option<Vec<ValueFloat>>,
    pub emit_rotation_axes: Option<Vec<Vec3>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxShapeSphere {
    pub flags: Option<u8>,
    pub radius: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxSoftParticleDefinitionData {
    pub begin_in: Option<f32>,
    pub begin_out: Option<f32>,
    pub delta_in: Option<f32>,
    pub delta_out: Option<f32>,
    pub unk_0x3bf176bc: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Asset, TypePath)]
#[serde(rename_all = "camelCase")]
pub struct VfxSystemDefinitionData {
    pub asset_remapping_table: Option<Vec<VfxAssetRemap>>,
    pub audio_parameter_flex_id: Option<i32>,
    pub audio_parameter_time_scaled_duration: Option<f32>,
    pub build_up_time: Option<f32>,
    pub clock_to_use: Option<u8>,
    pub complex_emitter_definition_data: Option<Vec<VfxEmitterDefinitionData>>,
    pub drawing_layer: Option<u8>,
    pub flags: Option<u16>,
    pub hud_anchor_position_from_world_projection: Option<bool>,
    pub hud_layer_dimension: Option<f32>,
    pub m_eye_candy: Option<bool>,
    pub m_is_pose_afterimage: Option<bool>,
    pub material_override_definitions: Option<Vec<VfxMaterialOverrideDefinitionData>>,
    pub override_scale_cap: Option<f32>,
    pub particle_name: String,
    pub particle_path: String,
    pub scale_dynamically_with_attached_bone: Option<bool>,
    pub self_illumination: Option<f32>,
    pub simple_emitter_definition_data: Option<Vec<VfxEmitterDefinitionData>>,
    pub sound_on_create_default: Option<String>,
    pub sound_persistent_default: Option<String>,
    pub transform: Option<Mat4>,
    pub unk_0x8b301739: Option<Unk0x75e34c40>,
    pub unk_0x9836cd87: Option<u8>,
    pub unk_0xf97b1289: Option<Unk0x7fb92f53>,
    pub visibility_radius: Option<f32>,
    pub voice_over_on_create_default: Option<String>,
    pub voice_over_persistent_default: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxTextureMultDefinitionData {
    pub birth_uv_offset_mult: Option<ValueVector2>,
    pub birth_uv_rotate_rate_mult: Option<ValueFloat>,
    pub birth_uv_scroll_rate_mult: Option<ValueVector2>,
    pub emitter_uv_scroll_rate_mult: Option<Vec2>,
    pub flex_birth_uv_scroll_rate_mult: Option<FlexValueVector2>,
    pub is_random_start_frame_mult: Option<bool>,
    pub particle_integrated_uv_rotate_mult: Option<IntegratedValueFloat>,
    pub particle_integrated_uv_scroll_mult: Option<IntegratedValueVector2>,
    pub tex_address_mode_mult: Option<u8>,
    pub tex_div_mult: Option<Vec2>,
    pub texture_mult: Option<String>,
    pub texture_mult_filp_u: Option<bool>,
    pub texture_mult_filp_v: Option<bool>,
    pub uv_rotation_mult: Option<ValueFloat>,
    pub uv_scale_mult: Option<ValueVector2>,
    pub uv_scroll_alpha_mult: Option<bool>,
    pub uv_scroll_clamp_mult: Option<bool>,
    pub uv_transform_center_mult: Option<Vec2>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VfxTrailDefinitionData {
    pub m_birth_tiling_size: Option<ValueVector3>,
    pub m_cutoff: Option<f32>,
    pub m_max_added_per_frame: Option<i32>,
    pub m_mode: Option<u8>,
    pub m_smoothing_mode: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WallDetection {
    pub detection_range: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WallFollowMovement {
    pub m_counter_clockwise: Option<bool>,
    pub m_infer_direction_from_facing_if_needed: bool,
    pub m_speed: f32,
    pub m_start_bone_name: String,
    pub m_stop_halfway_around: bool,
    pub m_tracks_target: bool,
    pub m_use_ground_height_at_target: bool,
    pub m_wall_length: f32,
    pub m_wall_offset: f32,
    pub m_wall_search_radius: f32,
    pub use_point_smoothing: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WidthPerSecond {
    pub m_width_per_second: f32,
}
