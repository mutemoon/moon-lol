use crate::render::LeagueLoader;
use bevy::math::{Mat4, Vec3};
use cdragon_prop::{
    BinEmbed, BinEntry, BinFloat, BinList, BinMatrix, BinS32, BinString, BinStruct, BinU8, BinVec3,
};

#[derive(Debug)]
pub struct LeagueMinionPath {
    pub transform: Mat4,
    pub name: String,
    pub segments: Vec<Vec3>,
}

impl From<&BinStruct> for LeagueMinionPath {
    fn from(value: &BinStruct) -> Self {
        let transform = value
            .getv::<BinMatrix>(LeagueLoader::hash_bin("transform").into())
            .map(|v| Mat4::from_cols_array_2d(&v.0))
            .unwrap();

        let name = value
            .getv::<BinString>(LeagueLoader::hash_bin("name").into())
            .map(|v| v.0.clone())
            .unwrap();

        let segments = value
            .getv::<BinList>(LeagueLoader::hash_bin("Segments").into())
            .iter()
            .filter_map(|v| v.downcast::<BinVec3>())
            .flat_map(|v| v.iter().map(|v| Vec3::new(v.0, v.1, v.2)))
            .collect();

        Self {
            transform,
            name,
            segments,
        }
    }
}

#[derive(Debug)]
pub struct LeagueBarracksConfig {
    pub units: Vec<BarracksMinionConfig>,
}

impl From<&BinEntry> for LeagueBarracksConfig {
    fn from(value: &BinEntry) -> Self {
        let units = value
            .getv::<BinList>(LeagueLoader::hash_bin("units").into())
            .map(|list| {
                list.downcast::<BinEmbed>()
                    .unwrap()
                    .iter()
                    .map(|embed| BarracksMinionConfig::from(embed))
                    .collect()
            })
            .unwrap();

        Self { units }
    }
}

#[derive(Debug)]
pub struct BarracksMinionConfig {
    pub minion_type: u8,
    pub wave_behavior: WaveBehavior,
    pub minion_upgrade_stats: MinionUpgradeConfig,
}

impl From<&BinEmbed> for BarracksMinionConfig {
    fn from(value: &BinEmbed) -> Self {
        let minion_type = value
            .getv::<BinU8>(LeagueLoader::hash_bin("MinionType").into())
            .map(|u| u.0)
            .unwrap();

        let wave_behavior = value
            .getv::<BinStruct>(LeagueLoader::hash_bin("WaveBehavior").into())
            .map(|s| WaveBehavior::from(s))
            .unwrap();

        let minion_upgrade_stats = value
            .getv::<BinEmbed>(LeagueLoader::hash_bin("MinionUpgradeStats").into())
            .map(|e| MinionUpgradeConfig::from(e))
            .unwrap();

        Self {
            minion_type,
            wave_behavior,
            minion_upgrade_stats,
        }
    }
}

#[derive(Debug)]
pub enum WaveBehavior {
    InhibitorWaveBehavior {
        spawn_count_per_inhibitor_down: Vec<i32>,
    },
    ConstantWaveBehavior {
        spawn_count: i32,
    },
    TimedVariableWaveBehavior {
        behaviors: Vec<TimedWaveBehaviorInfo>,
    },
    RotatingWaveBehavior {
        spawn_counts_by_wave: Vec<i32>,
    },
    Unknown,
}

impl From<&BinStruct> for WaveBehavior {
    fn from(value: &BinStruct) -> Self {
        let inhibitor_hash = LeagueLoader::hash_bin("InhibitorWaveBehavior");
        let constant_hash = LeagueLoader::hash_bin("ConstantWaveBehavior");
        let timed_variable_hash = LeagueLoader::hash_bin("TimedVariableWaveBehavior");
        let rotating_hash = LeagueLoader::hash_bin("RotatingWaveBehavior");

        match value.ctype.hash {
            hash if hash == inhibitor_hash => {
                let spawn_count_per_inhibitor_down = value
                    .getv::<BinList>(LeagueLoader::hash_bin("SpawnCountPerInhibitorDown").into())
                    .map(|list| {
                        list.downcast::<BinS32>()
                            .unwrap()
                            .iter()
                            .map(|i| i.0)
                            .collect()
                    })
                    .unwrap();

                WaveBehavior::InhibitorWaveBehavior {
                    spawn_count_per_inhibitor_down,
                }
            }
            hash if hash == constant_hash => {
                let spawn_count = value
                    .getv::<BinS32>(LeagueLoader::hash_bin("SpawnCount").into())
                    .map(|i| i.0)
                    .unwrap();

                WaveBehavior::ConstantWaveBehavior { spawn_count }
            }
            hash if hash == timed_variable_hash => {
                let behaviors = value
                    .getv::<BinList>(LeagueLoader::hash_bin("behaviors").into())
                    .map(|list| {
                        list.downcast::<BinEmbed>()
                            .unwrap()
                            .iter()
                            .map(|embed| TimedWaveBehaviorInfo::from(embed))
                            .collect()
                    })
                    .unwrap();

                WaveBehavior::TimedVariableWaveBehavior { behaviors }
            }
            hash if hash == rotating_hash => {
                let spawn_counts_by_wave = value
                    .getv::<BinList>(LeagueLoader::hash_bin("SpawnCountsByWave").into())
                    .map(|list| {
                        list.downcast::<BinS32>()
                            .unwrap()
                            .iter()
                            .map(|i| i.0)
                            .collect()
                    })
                    .unwrap();

                WaveBehavior::RotatingWaveBehavior {
                    spawn_counts_by_wave,
                }
            }
            _ => WaveBehavior::Unknown,
        }
    }
}

#[derive(Debug)]
pub struct TimedWaveBehaviorInfo {
    pub start_time_secs: i32,
    pub behavior: WaveBehavior,
}

impl From<&BinEmbed> for TimedWaveBehaviorInfo {
    fn from(value: &BinEmbed) -> Self {
        let start_time_secs = value
            .getv::<BinS32>(LeagueLoader::hash_bin("StartTimeSecs").into())
            .map(|i| i.0)
            .unwrap();

        let behavior = value
            .getv::<BinStruct>(LeagueLoader::hash_bin("Behavior").into())
            .map(|s| WaveBehavior::from(s))
            .unwrap();

        Self {
            start_time_secs,
            behavior,
        }
    }
}

#[derive(Debug, Default)]
pub struct MinionUpgradeConfig {
    pub armor_max: f32,
    pub armor_upgrade_growth: f32,
    pub hp_max_bonus: f32,
    pub hp_upgrade: f32,
    pub hp_upgrade_late: f32,
    pub damage_max: f32,
    pub damage_upgrade: f32,
    pub damage_upgrade_late: f32,
}

impl From<&BinEmbed> for MinionUpgradeConfig {
    fn from(value: &BinEmbed) -> Self {
        let armor_max = value
            .getv::<BinFloat>(LeagueLoader::hash_bin("ArmorMax").into())
            .map(|f| f.0)
            .unwrap();

        let armor_upgrade_growth = value
            .getv::<BinFloat>(LeagueLoader::hash_bin("ArmorUpgradeGrowth").into())
            .map(|f| f.0)
            .unwrap();

        let hp_max_bonus = value
            .getv::<BinFloat>(LeagueLoader::hash_bin("HpMaxBonus").into())
            .map(|f| f.0)
            .unwrap();

        let hp_upgrade = value
            .getv::<BinFloat>(LeagueLoader::hash_bin("HPUpgrade").into())
            .map(|f| f.0)
            .unwrap();

        let hp_upgrade_late = value
            .getv::<BinFloat>(LeagueLoader::hash_bin("HPUpgradeLate").into())
            .map(|f| f.0)
            .unwrap();

        let damage_max = value
            .getv::<BinFloat>(LeagueLoader::hash_bin("DamageMax").into())
            .map(|f| f.0)
            .unwrap();

        let damage_upgrade = value
            .getv::<BinFloat>(LeagueLoader::hash_bin("DamageUpgrade").into())
            .map(|f| f.0)
            .unwrap();

        let damage_upgrade_late = value
            .getv::<BinFloat>(LeagueLoader::hash_bin("DamageUpgradeLate").into())
            .map(|f| f.0)
            .unwrap();

        Self {
            armor_max,
            armor_upgrade_growth,
            hp_max_bonus,
            hp_upgrade,
            hp_upgrade_late,
            damage_max,
            damage_upgrade,
            damage_upgrade_late,
        }
    }
}
