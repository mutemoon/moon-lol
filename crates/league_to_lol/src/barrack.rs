use std::collections::HashMap;

use bevy::ecs::archetype;
use bevy::prelude::*;
use bevy::world_serialization::WorldSerializationPlugin;
use league_core::extract;
use league_loader::game::{Data, PropGroup};
use lol_base::barrack::{
    ConfigBarracks, ConfigBarracksMinion, ConfigMinionUpgrade, ConstantWaveBehavior,
    EnumWaveBehavior, InhibitorWaveBehavior, RotatingWaveBehavior, TimedVariableWaveBehavior,
    TimedWaveBehaviorInfo,
};
use lol_base::map::MapPaths;
use lol_core::entities::minion::Minion;
use lol_core::team::Team;

use crate::extract::champion::ChampionRecordData;
use crate::extract::map::spawn_character_record;
use crate::extract::utils::write_to_file;

pub fn barracks_config_to_barracks(
    value: extract::BarracksConfig,
    prop_group: &PropGroup,
    map_paths: &MapPaths,
    map_character_records: &mut HashMap<String, Vec<ChampionRecordData>>,
    team: Team,
) -> ConfigBarracks {
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
            .map(|u| {
                barracks_minion_config_to_barracks_minion(
                    u,
                    prop_group,
                    map_paths,
                    map_character_records,
                    team.clone(),
                )
            })
            .collect(),
        upgrade_interval_secs: value.upgrade_interval_secs,
        upgrades_before_late_game_scaling: value.upgrades_before_late_game_scaling,
        wave_spawn_interval_secs: value.wave_spawn_interval_secs,
    }
}

pub fn barracks_minion_config_to_barracks_minion(
    value: extract::BarracksMinionConfig,
    prop_group: &PropGroup,
    map_paths: &MapPaths,
    map_character_records: &mut HashMap<String, Vec<ChampionRecordData>>,
    team: Team,
) -> ConfigBarracksMinion {
    let unk0xad65d8c4 = prop_group.get_data::<extract::Unk0xad65d8c4>(value.unk_0xfee040bc);

    // 创建一个临时的 World 来 spawn 小兵并序列化
    let mut app = App::new();
    app.add_plugins(AssetPlugin::default());
    app.add_plugins(WorldSerializationPlugin);
    app.finish();
    app.cleanup();

    let world = app.world_mut();
    spawn_character_record(
        world,
        &unk0xad65d8c4,
        map_character_records,
        (Minion::from(value.minion_type), team),
    );

    let type_registry = world.resource::<AppTypeRegistry>();
    let type_registry = type_registry.read();
    let scene = DynamicWorldBuilder::from_world(&world, &type_registry)
        .deny_component::<GlobalTransform>()
        .deny_component::<TransformTreeChanged>()
        .extract_entities(
            // we do this instead of a query, in order to completely sidestep default query filters.
            // while we could use `Allow<_>`, this wouldn't account for custom disabled components
            world
                .archetypes()
                .iter()
                .flat_map(archetype::Archetype::entities)
                .map(archetype::ArchetypeEntity::id),
        )
        .extract_resources()
        .build();
    let serialized_scene = scene.serialize(&type_registry).unwrap();

    let minion_template = format!(
        "maps/{}/minions/{:x}.ron",
        map_paths.name, value.unk_0xfee040bc
    );
    write_to_file(&minion_template, serialized_scene);

    ConfigBarracksMinion {
        minion_type: value.minion_type,
        minion_upgrade_stats: minion_upgrade_config_to_minion_upgrade(value.minion_upgrade_stats),
        minion_template,
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
