use bevy::prelude::*;

#[derive(Component, Reflect)]
#[reflect(Component)]
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

#[derive(Reflect)]
pub struct ConfigBarracksMinion {
    pub minion_type: u8,
    pub minion_upgrade_stats: ConfigMinionUpgrade,
    pub unk_0xfee040bc: u32,
    pub wave_behavior: EnumWaveBehavior,
}

#[derive(Reflect)]
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

#[derive(Reflect)]
pub enum EnumWaveBehavior {
    ConstantWaveBehavior(ConstantWaveBehavior),
    InhibitorWaveBehavior(InhibitorWaveBehavior),
    RotatingWaveBehavior(RotatingWaveBehavior),
    TimedVariableWaveBehavior(TimedVariableWaveBehavior),
}

#[derive(Reflect)]
pub struct ConstantWaveBehavior {
    pub spawn_count: i32,
}

#[derive(Reflect)]
pub struct InhibitorWaveBehavior {
    pub spawn_count_per_inhibitor_down: Vec<i32>,
}
#[derive(Reflect)]
pub struct RotatingWaveBehavior {
    pub spawn_counts_by_wave: Vec<i32>,
}

#[derive(Reflect)]
pub struct TimedVariableWaveBehavior {
    pub behaviors: Vec<TimedWaveBehaviorInfo>,
    pub default_spawn_count: Option<i32>,
}

#[derive(Reflect)]
pub struct TimedWaveBehaviorInfo {
    pub behavior: EnumWaveBehavior,
    pub start_time_secs: Option<i32>,
}

impl From<league_core::extract::BarracksConfig> for ConfigBarracks {
    fn from(value: league_core::extract::BarracksConfig) -> Self {
        ConfigBarracks {
            exp_radius: value.exp_radius,
            gold_radius: value.gold_radius,
            initial_spawn_time_secs: value.initial_spawn_time_secs,
            minion_spawn_interval_secs: value.minion_spawn_interval_secs,
            move_speed_increase_increment: value.move_speed_increase_increment,
            move_speed_increase_initial_delay_secs: value.move_speed_increase_initial_delay_secs,
            move_speed_increase_interval_secs: value.move_speed_increase_interval_secs,
            move_speed_increase_max_times: value.move_speed_increase_max_times,
            units: value.units.into_iter().map(|u| u.into()).collect(),
            upgrade_interval_secs: value.upgrade_interval_secs,
            upgrades_before_late_game_scaling: value.upgrades_before_late_game_scaling,
            wave_spawn_interval_secs: value.wave_spawn_interval_secs,
        }
    }
}

impl From<league_core::extract::BarracksMinionConfig> for ConfigBarracksMinion {
    fn from(value: league_core::extract::BarracksMinionConfig) -> Self {
        ConfigBarracksMinion {
            minion_type: value.minion_type,
            minion_upgrade_stats: value.minion_upgrade_stats.into(),
            unk_0xfee040bc: value.unk_0xfee040bc,
            wave_behavior: value.wave_behavior.into(),
        }
    }
}

impl From<league_core::extract::MinionUpgradeConfig> for ConfigMinionUpgrade {
    fn from(value: league_core::extract::MinionUpgradeConfig) -> Self {
        ConfigMinionUpgrade {
            armor_max: value.armor_max,
            armor_upgrade: value.armor_upgrade,
            armor_upgrade_growth: value.armor_upgrade_growth,
            damage_max: value.damage_max,
            damage_upgrade: value.damage_upgrade,
            damage_upgrade_late: value.damage_upgrade_late,
            gold_max: value.gold_max,
            hp_max_bonus: value.hp_max_bonus,
            hp_upgrade: value.hp_upgrade,
            hp_upgrade_late: value.hp_upgrade_late,
            magic_resistance_upgrade: value.magic_resistance_upgrade,
            unk_0x726ae049: value.unk_0x726ae049,
        }
    }
}

impl From<league_core::extract::EnumWaveBehavior> for EnumWaveBehavior {
    fn from(value: league_core::extract::EnumWaveBehavior) -> Self {
        match value {
            league_core::extract::EnumWaveBehavior::ConstantWaveBehavior(behavior) => {
                EnumWaveBehavior::ConstantWaveBehavior(ConstantWaveBehavior {
                    spawn_count: behavior.spawn_count,
                })
            }
            league_core::extract::EnumWaveBehavior::InhibitorWaveBehavior(behavior) => {
                EnumWaveBehavior::InhibitorWaveBehavior(InhibitorWaveBehavior {
                    spawn_count_per_inhibitor_down: behavior.spawn_count_per_inhibitor_down,
                })
            }
            league_core::extract::EnumWaveBehavior::RotatingWaveBehavior(behavior) => {
                EnumWaveBehavior::RotatingWaveBehavior(RotatingWaveBehavior {
                    spawn_counts_by_wave: behavior.spawn_counts_by_wave,
                })
            }
            league_core::extract::EnumWaveBehavior::TimedVariableWaveBehavior(behavior) => {
                EnumWaveBehavior::TimedVariableWaveBehavior(TimedVariableWaveBehavior {
                    behaviors: behavior
                        .behaviors
                        .into_iter()
                        .map(|b| (*b).into())
                        .collect(),
                    default_spawn_count: behavior.default_spawn_count,
                })
            }
        }
    }
}

impl From<league_core::extract::TimedWaveBehaviorInfo> for TimedWaveBehaviorInfo {
    fn from(value: league_core::extract::TimedWaveBehaviorInfo) -> Self {
        TimedWaveBehaviorInfo {
            behavior: (*value.behavior).into(),
            start_time_secs: value.start_time_secs,
        }
    }
}
