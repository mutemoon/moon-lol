use league_core::extract;
use lol_base::barrack::{
    ConfigBarracks, ConfigBarracksMinion, ConfigMinionUpgrade, ConstantWaveBehavior,
    EnumWaveBehavior, InhibitorWaveBehavior, RotatingWaveBehavior, TimedVariableWaveBehavior,
    TimedWaveBehaviorInfo,
};

pub fn barracks_config_to_barracks(value: extract::BarracksConfig) -> ConfigBarracks {
    ConfigBarracks {
        exp_radius: value.exp_radius,
        gold_radius: value.gold_radius,
        initial_spawn_time_secs: value.initial_spawn_time_secs,
        minion_spawn_interval_secs: value.minion_spawn_interval_secs,
        move_speed_increase_increment: value.move_speed_increase_increment,
        move_speed_increase_initial_delay_secs: value.move_speed_increase_initial_delay_secs,
        move_speed_increase_interval_secs: value.move_speed_increase_interval_secs,
        move_speed_increase_max_times: value.move_speed_increase_max_times,
        units: value
            .units
            .into_iter()
            .map(barracks_minion_config_to_barracks_minion)
            .collect(),
        upgrade_interval_secs: value.upgrade_interval_secs,
        upgrades_before_late_game_scaling: value.upgrades_before_late_game_scaling,
        wave_spawn_interval_secs: value.wave_spawn_interval_secs,
    }
}

pub fn barracks_minion_config_to_barracks_minion(
    value: extract::BarracksMinionConfig,
) -> ConfigBarracksMinion {
    ConfigBarracksMinion {
        minion_type: value.minion_type,
        minion_upgrade_stats: minion_upgrade_config_to_minion_upgrade(value.minion_upgrade_stats),
        unk_0xfee040bc: value.unk_0xfee040bc,
        wave_behavior: enum_wave_behavior_to_enum_wave_behavior(value.wave_behavior),
    }
}

pub fn minion_upgrade_config_to_minion_upgrade(
    value: extract::MinionUpgradeConfig,
) -> ConfigMinionUpgrade {
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

pub fn enum_wave_behavior_to_enum_wave_behavior(
    value: extract::EnumWaveBehavior,
) -> EnumWaveBehavior {
    match value {
        extract::EnumWaveBehavior::ConstantWaveBehavior(behavior) => {
            EnumWaveBehavior::ConstantWaveBehavior(ConstantWaveBehavior {
                spawn_count: behavior.spawn_count,
            })
        }
        extract::EnumWaveBehavior::InhibitorWaveBehavior(behavior) => {
            EnumWaveBehavior::InhibitorWaveBehavior(InhibitorWaveBehavior {
                spawn_count_per_inhibitor_down: behavior.spawn_count_per_inhibitor_down,
            })
        }
        extract::EnumWaveBehavior::RotatingWaveBehavior(behavior) => {
            EnumWaveBehavior::RotatingWaveBehavior(RotatingWaveBehavior {
                spawn_counts_by_wave: behavior.spawn_counts_by_wave,
            })
        }
        extract::EnumWaveBehavior::TimedVariableWaveBehavior(behavior) => {
            EnumWaveBehavior::TimedVariableWaveBehavior(TimedVariableWaveBehavior {
                behaviors: behavior
                    .behaviors
                    .into_iter()
                    .map(|b| timed_wave_behavior_info_to_timed_wave_behavior_info(*b))
                    .collect(),
                default_spawn_count: behavior.default_spawn_count,
            })
        }
    }
}

pub fn timed_wave_behavior_info_to_timed_wave_behavior_info(
    value: extract::TimedWaveBehaviorInfo,
) -> TimedWaveBehaviorInfo {
    TimedWaveBehaviorInfo {
        behavior: enum_wave_behavior_to_enum_wave_behavior(*value.behavior),
        start_time_secs: value.start_time_secs,
    }
}
