use crate::{
    core::{Armor, Bounding, Damage, Health, Lane, Movement, Team},
    entities::{
        Barrack, BarracksMinionConfig, Minion, MinionUpgradeConfig, TimedWaveBehaviorInfo,
        WaveBehavior,
    },
    league::{u16_to_lane, u32_option_to_team, LeagueLoader, LeagueLoaderError},
};
use bevy::{math::Mat4, transform::components::Transform};
use cdragon_prop::{
    BinEmbed, BinFloat, BinLink, BinList, BinMatrix, BinS32, BinString, BinStruct, BinU16, BinU32,
    BinU8,
};

impl LeagueLoader {
    pub fn save_barrack(
        &self,
        value: &BinStruct,
    ) -> Result<(Transform, Team, Lane, Barrack), LeagueLoaderError> {
        let transform = value
            .getv::<BinMatrix>(LeagueLoader::hash_bin("transform").into())
            .unwrap();

        let mut transform = Mat4::from_cols_array_2d(&transform.0);
        transform.w_axis.z = -transform.w_axis.z;

        let definition = value
            .getv::<BinStruct>(LeagueLoader::hash_bin("definition").into())
            .unwrap();

        let unknown_list = definition.getv::<BinList>(0xdbde2288.into()).unwrap();

        let first_item = unknown_list
            .downcast::<BinStruct>()
            .unwrap()
            .first()
            .unwrap();

        let lane = first_item
            .getv::<BinU16>(LeagueLoader::hash_bin("lane").into())
            .map(|v| v.0)
            .unwrap();

        let team = definition
            .getv::<BinU32>(LeagueLoader::hash_bin("Team").into())
            .map(|v| v.0);

        let barracks_config = definition
            .getv::<BinLink>(LeagueLoader::hash_bin("BarracksConfig").into())
            .map(|v| v.0.hash)
            .unwrap();

        let value = self
            .materials_bin
            .entries
            .iter()
            .find(|v| v.path.hash == barracks_config)
            .unwrap();

        // let initial_spawn_time_secs = value
        //     .getv::<BinFloat>(LeagueLoader::hash_bin("InitialSpawnTimeSecs").into())
        //     .map(|f| f.0)
        //     .unwrap();
        let initial_spawn_time_secs = 0.0;

        let wave_spawn_interval_secs = value
            .getv::<BinFloat>(LeagueLoader::hash_bin("WaveSpawnIntervalSecs").into())
            .map(|f| f.0)
            .unwrap();

        let minion_spawn_interval_secs = value
            .getv::<BinFloat>(LeagueLoader::hash_bin("MinionSpawnIntervalSecs").into())
            .map(|f| f.0)
            .unwrap();

        let upgrade_interval_secs = value
            .getv::<BinFloat>(LeagueLoader::hash_bin("UpgradeIntervalSecs").into())
            .map(|f| f.0)
            .unwrap();

        let upgrades_before_late_game_scaling = value
            .getv::<BinS32>(LeagueLoader::hash_bin("UpgradesBeforeLateGameScaling").into())
            .map(|i| i.0)
            .unwrap();

        let move_speed_increase_initial_delay_secs = value
            .getv::<BinFloat>(LeagueLoader::hash_bin("MoveSpeedIncreaseInitialDelaySecs").into())
            .map(|f| f.0)
            .unwrap();

        let move_speed_increase_interval_secs = value
            .getv::<BinFloat>(LeagueLoader::hash_bin("MoveSpeedIncreaseIntervalSecs").into())
            .map(|f| f.0)
            .unwrap();

        let move_speed_increase_increment = value
            .getv::<BinS32>(LeagueLoader::hash_bin("MoveSpeedIncreaseIncrement").into())
            .map(|i| i.0)
            .unwrap();

        let move_speed_increase_max_times = value
            .getv::<BinS32>(LeagueLoader::hash_bin("MoveSpeedIncreaseMaxTimes").into())
            .map(|i| i.0)
            .unwrap();

        let exp_radius = value
            .getv::<BinFloat>(LeagueLoader::hash_bin("ExpRadius").into())
            .map(|f| f.0)
            .unwrap();

        let gold_radius = value
            .getv::<BinFloat>(LeagueLoader::hash_bin("goldRadius").into())
            .map(|f| f.0)
            .unwrap();

        let units = value
            .getv::<BinList>(LeagueLoader::hash_bin("units").into())
            .map(|list| {
                list.downcast::<BinEmbed>()
                    .unwrap()
                    .iter()
                    .map(|embed| self.load_barrack_minion_config(embed).unwrap())
                    .collect()
            })
            .unwrap();

        Ok((
            Transform::from_matrix(transform),
            u32_option_to_team(team),
            u16_to_lane(lane),
            Barrack {
                initial_spawn_time_secs,
                wave_spawn_interval_secs,
                minion_spawn_interval_secs,
                upgrade_interval_secs,
                upgrades_before_late_game_scaling,
                move_speed_increase_initial_delay_secs,
                move_speed_increase_interval_secs,
                move_speed_increase_increment,
                move_speed_increase_max_times,
                exp_radius,
                gold_radius,
                units,
            },
        ))
    }

    fn load_barrack_minion_config(
        &self,
        value: &BinEmbed,
    ) -> Result<BarracksMinionConfig, LeagueLoaderError> {
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

        let character_record = value
            .getv::<BinLink>(0x8a3fc6eb.into())
            .map(|v| v.0.hash)
            .unwrap();

        let character_map_record = self
            .materials_bin
            .entries
            .iter()
            .find(|v| v.path.hash == character_record)
            .unwrap();

        // let team = character_map_record
        //     .getv::<BinU32>(LeagueLoader::hash_bin("Team").into())
        //     .map(|i| i.0);

        let character_record_path = character_map_record
            .getv::<BinString>(LeagueLoader::hash_bin("CharacterRecord").into())
            .map(|i| i.0.clone())
            .unwrap();

        let skin = character_map_record
            .getv::<BinString>(LeagueLoader::hash_bin("Skin").into())
            .map(|i| i.0.clone())
            .unwrap();

        let character_record = self.load_character_record(&character_record_path);

        let health = Health {
            value: character_record.base_hp.unwrap(),
            max: character_record.base_hp.unwrap(),
        };

        let movement = Movement {
            speed: character_record.base_move_speed.unwrap(),
        };

        let bounding = Bounding {
            radius: character_record.pathfinding_collision_radius.unwrap(),
            sides: 10,
            height: 10.0,
        };

        let damage = Damage(character_record.base_damage.unwrap());

        let armor = Armor(character_record.base_armor.unwrap());

        let environment_object = self.save_environment_object(&skin)?;

        Ok(BarracksMinionConfig {
            minion_type: match minion_type {
                4 => Minion::Melee,
                6 => Minion::Siege,
                5 => Minion::Ranged,
                7 => Minion::Super,
                _ => panic!("unknown minion type"),
            },
            minion_object: (
                health,
                movement,
                bounding,
                damage,
                armor,
                environment_object,
            ),
            wave_behavior,
            minion_upgrade_stats,
        })
    }
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

impl From<&BinEmbed> for MinionUpgradeConfig {
    fn from(value: &BinEmbed) -> Self {
        let armor_max = value
            .getv::<BinFloat>(LeagueLoader::hash_bin("ArmorMax").into())
            .map(|f| f.0)
            .unwrap_or(f32::MAX);

        let armor_upgrade_growth = value
            .getv::<BinFloat>(LeagueLoader::hash_bin("ArmorUpgradeGrowth").into())
            .map(|f| f.0)
            .unwrap_or(0.0);

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
            .unwrap_or(hp_upgrade);

        let damage_max = value
            .getv::<BinFloat>(LeagueLoader::hash_bin("DamageMax").into())
            .map(|f| f.0)
            .unwrap();

        let damage_upgrade = value
            .getv::<BinFloat>(LeagueLoader::hash_bin("DamageUpgrade").into())
            .map(|f| f.0)
            .unwrap_or(0.0);

        let damage_upgrade_late = value
            .getv::<BinFloat>(LeagueLoader::hash_bin("DamageUpgradeLate").into())
            .map(|f| f.0)
            .unwrap_or(damage_upgrade);

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
