use crate::{
    combat::{Lane, Team},
    entities::Barrack,
    render::{u16_to_lane, u32_option_to_team, LeagueLoader},
};
use bevy::{math::Mat4, transform::components::Transform};
use cdragon_prop::{
    BinFloat, BinLink, BinList, BinMatrix, BinS32, BinStruct, BinU16, BinU32, PropFile,
};

pub fn get_barrack_by_bin(bin: &PropFile, value: &BinStruct) -> (Barrack, Transform, Team, Lane) {
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

    let value = bin
        .entries
        .iter()
        .find(|v| v.path.hash == barracks_config)
        .unwrap();

    let initial_spawn_time_secs = value
        .getv::<BinFloat>(LeagueLoader::hash_bin("InitialSpawnTimeSecs").into())
        .map(|f| f.0)
        .unwrap();

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

    (
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
        },
        Transform::from_matrix(transform),
        u32_option_to_team(team),
        u16_to_lane(lane),
    )
}
