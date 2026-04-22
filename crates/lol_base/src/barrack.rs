use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Reflect, Asset, Serialize, Deserialize)]
pub struct ConfigBarracks {
    pub exp_radius: f32,
    pub gold_radius: f32,
    pub initial_spawn_time_secs: f32,
    pub minion_spawn_interval_secs: f32,
    pub move_speed_increase_increment: i32,
    pub move_speed_increase_initial_delay_secs: f32,
    pub move_speed_increase_interval_secs: f32,
    pub move_speed_increase_max_times: i32,
    pub units: Vec<ConfigBarracksMinion>,
    pub upgrade_interval_secs: f32,
    pub upgrades_before_late_game_scaling: i32,
    pub wave_spawn_interval_secs: f32,
}

#[derive(Reflect, Serialize, Deserialize)]
pub struct ConfigBarracksMinion {
    pub minion_type: u8,
    pub minion_upgrade_stats: ConfigMinionUpgrade,
    pub unk_0xfee040bc: u32,
    pub wave_behavior: EnumWaveBehavior,
}

#[derive(Reflect, Serialize, Deserialize)]
pub struct ConfigMinionUpgrade {
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

#[derive(Reflect, Serialize, Deserialize)]
pub enum EnumWaveBehavior {
    ConstantWaveBehavior(ConstantWaveBehavior),
    InhibitorWaveBehavior(InhibitorWaveBehavior),
    RotatingWaveBehavior(RotatingWaveBehavior),
    TimedVariableWaveBehavior(TimedVariableWaveBehavior),
}

#[derive(Reflect, Serialize, Deserialize)]
pub struct ConstantWaveBehavior {
    pub spawn_count: i32,
}

#[derive(Reflect, Serialize, Deserialize)]
pub struct InhibitorWaveBehavior {
    pub spawn_count_per_inhibitor_down: Vec<i32>,
}
#[derive(Reflect, Serialize, Deserialize)]
pub struct RotatingWaveBehavior {
    pub spawn_counts_by_wave: Vec<i32>,
}

#[derive(Reflect, Serialize, Deserialize)]
pub struct TimedVariableWaveBehavior {
    pub behaviors: Vec<TimedWaveBehaviorInfo>,
    pub default_spawn_count: Option<i32>,
}

#[derive(Reflect, Serialize, Deserialize)]
pub struct TimedWaveBehaviorInfo {
    pub behavior: EnumWaveBehavior,
    pub start_time_secs: Option<i32>,
}
