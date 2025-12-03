use std::{collections::VecDeque, time::Duration};

use bevy::prelude::*;
use league_core::{
    BarracksMinionConfigWaveBehavior, ConstantWaveBehavior, InhibitorWaveBehavior,
    RotatingWaveBehavior, TimedVariableWaveBehavior,
};
use lol_config::ConfigMap;
use lol_core::{Lane, Team};
use serde::{Deserialize, Serialize};

use crate::{
    core::{Armor, CommandSpawnCharacter, Damage, Health, Movement, ResourceCache},
    entities::Minion,
};

/// 兵营的动态状态，用于跟踪计时器和生成队列
#[derive(Component, Serialize, Deserialize)]
pub struct Barrack {
    pub id: u32,
    /// 下一波兵的生成计时器
    pub wave_timer: Timer,
    /// 属性升级计时器
    pub upgrade_timer: Timer,
    /// 移动速度升级计时器
    pub move_speed_upgrade_timer: Timer,
    /// 同一波兵中，每个小兵之间的生成间隔计时器
    pub intra_spawn_timer: Timer,
    /// 当前的生成队列
    pub spawn_queue: VecDeque<(usize, i32)>,
    /// 已应用的属性升级次数
    pub upgrade_count: i32,
    /// 已应用的移速升级次数
    pub move_speed_upgrade_count: i32,
    /// 已生成的波数
    pub wave_count: u32,
}

#[derive(Resource, Default)]
pub struct InhibitorState {
    pub inhibitors_down: usize,
}

#[derive(Default)]
pub struct PluginBarrack;

impl Plugin for PluginBarrack {
    fn build(&self, app: &mut App) {
        app.init_resource::<InhibitorState>();
        app.add_systems(Startup, setup);
        app.add_systems(FixedUpdate, barracks_spawning_system);
    }
}

fn setup(mut commands: Commands, res_game_config: Res<ConfigMap>) {
    for (hash, barrack) in res_game_config.barracks.iter() {
        let barrack_config = res_game_config
            .barrack_configs
            .get(&barrack.definition.barracks_config)
            .unwrap();

        // let initial_delay = barrack_config.initial_spawn_time_secs;
        let initial_delay = 0.0;

        commands.spawn((
            Transform::from_matrix(barrack.transform),
            Team::from(barrack.definition.team),
            Lane::from(barrack.definition.unk_0xdbde2288[0].lane),
            Barrack {
                id: *hash,
                // 第一波兵有初始延迟
                wave_timer: Timer::from_seconds(initial_delay, TimerMode::Repeating),
                // 属性升级从第一波兵生成后开始计算
                upgrade_timer: Timer::new(
                    Duration::from_secs_f32(barrack_config.upgrade_interval_secs),
                    TimerMode::Repeating,
                ),
                // 移速升级有自己的独立延迟
                move_speed_upgrade_timer: Timer::new(
                    Duration::from_secs_f32(barrack_config.move_speed_increase_initial_delay_secs),
                    TimerMode::Repeating,
                ),
                // 小兵间的生成间隔计时器
                intra_spawn_timer: Timer::from_seconds(
                    barrack_config.minion_spawn_interval_secs,
                    TimerMode::Repeating,
                ),
                spawn_queue: VecDeque::new(),
                upgrade_count: 0,
                move_speed_upgrade_count: 0,
                wave_count: 0,
            },
        ));
    }
}

/// 核心系统：处理兵营的计时、升级和生成逻辑
fn barracks_spawning_system(
    game_time: Res<Time<Virtual>>,
    inhibitor_state: Res<InhibitorState>,
    mut commands: Commands,
    mut query: Query<(&GlobalTransform, &mut Barrack, &Team, &Lane)>,
    res_game_config: Res<ConfigMap>,
    resource_cache: Res<ResourceCache>,
    time: Res<Time>,
) {
    for (transform, mut barrack_state, team, lane) in query.iter_mut() {
        let barrack_place = res_game_config.barracks.get(&barrack_state.id).unwrap();

        let barrack_config = res_game_config
            .barrack_configs
            .get(&barrack_place.definition.barracks_config)
            .unwrap();

        // --- 1. 更新所有计时器 ---
        barrack_state.wave_timer.tick(time.delta());

        // 只有在第一波之后才开始计时升级
        if barrack_state.wave_count > 0 {
            barrack_state.upgrade_timer.tick(time.delta());
            barrack_state.move_speed_upgrade_timer.tick(time.delta());
        }

        // --- 2. 处理属性和移速升级 ---
        if barrack_state.upgrade_timer.just_finished() {
            barrack_state.upgrade_count += 1;
            debug!(
                "Barrack upgraded! New count: {}",
                barrack_state.upgrade_count
            );
        }

        if barrack_state.move_speed_upgrade_timer.just_finished() {
            if barrack_state.move_speed_upgrade_count < barrack_config.move_speed_increase_max_times
            {
                barrack_state.move_speed_upgrade_count += 1;
                debug!(
                    "Minion move speed upgraded! New count: {}",
                    barrack_state.move_speed_upgrade_count
                );
            }
        }

        // --- 3. 检查是否需要生成新一波小兵 ---
        // 只有当上一波完全生成完后（队列为空），才开始准备新一波
        if barrack_state.wave_timer.just_finished() && barrack_state.spawn_queue.is_empty() {
            barrack_state.wave_count += 1;
            barrack_state
                .wave_timer
                .set_duration(Duration::from_secs_f32(
                    barrack_config.wave_spawn_interval_secs,
                ));

            // 遍历兵营配置中的所有小兵类型
            for (index, minion_config) in barrack_config.units.iter().enumerate() {
                let spawn_count = calculate_spawn_count(
                    &minion_config.wave_behavior,
                    game_time.elapsed_secs(),
                    barrack_state.wave_count,
                    &inhibitor_state,
                );

                if spawn_count > 0 {
                    barrack_state.spawn_queue.push_back((index, spawn_count));
                }
            }
        }

        // --- 4. 处理生成队列，逐个生成小兵 ---
        if barrack_state.spawn_queue.is_empty() {
            continue;
        }

        barrack_state.intra_spawn_timer.tick(time.delta());

        if !barrack_state.intra_spawn_timer.just_finished() {
            continue;
        }

        let upgrade_count = barrack_state.upgrade_count;
        let move_speed_upgrade_count = barrack_state.move_speed_upgrade_count;

        // 获取队列头部的待生成小兵信息
        let Some(current_spawn) = barrack_state.spawn_queue.front_mut() else {
            continue;
        };

        let config_index = current_spawn.0;
        let minion_config = &barrack_config.units[config_index];
        let upgrade_config = &minion_config.minion_upgrade_stats;

        // --- 计算小兵最终属性 ---
        let is_late_game = upgrade_count >= barrack_config.upgrades_before_late_game_scaling;

        let link = minion_config.unk_0x8a3fc6eb;

        let character = res_game_config.characters.get(&link).unwrap();

        let character_record = resource_cache
            .character_records
            .get(&character.character_record)
            .unwrap();

        let mut health = Health::new(character_record.base_hp.unwrap_or(0.0));
        let hp_upgrade = if is_late_game {
            upgrade_config.hp_upgrade_late.unwrap_or(0.0)
        } else {
            upgrade_config.hp_upgrade
        };
        health.max += upgrade_config.hp_max_bonus + hp_upgrade * upgrade_count as f32;
        health.value = health.max;

        let mut damage = Damage(character_record.base_damage.unwrap_or(0.0));
        let damage_upgrade = if is_late_game {
            upgrade_config.damage_upgrade_late
        } else {
            upgrade_config.damage_upgrade
        };
        damage.0 +=
            upgrade_config.damage_max + damage_upgrade.unwrap_or(0.0) * upgrade_count as f32;

        let mut armor = Armor(character_record.base_armor.unwrap_or(0.0));
        armor.0 += upgrade_config.armor_max.unwrap_or(0.0)
            + upgrade_config.armor_upgrade_growth.unwrap_or(0.0) * upgrade_count as f32;

        let mut movement = Movement {
            speed: character_record.base_move_speed.unwrap_or(0.0),
        };
        movement.speed +=
            (barrack_config.move_speed_increase_increment * move_speed_upgrade_count) as f32;

        let entity = commands
            .spawn((
                Transform::from_matrix(transform.to_matrix()),
                Minion::from(minion_config.minion_type),
                lane.clone(),
                team.clone(),
            ))
            .id();

        // 触发角色生成命令（创建基础组件并加载皮肤）
        commands.trigger(CommandSpawnCharacter {
            entity,
            character_record_key: character.character_record.clone(),
            skin_path: character.skin.clone(),
        });

        commands
            .entity(entity)
            .insert((health, damage, armor, movement));

        let mut path = res_game_config.minion_paths.get(lane).unwrap().clone();

        if *team == Team::Chaos {
            path.reverse();
        }

        // 更新队列
        current_spawn.1 -= 1;
        if current_spawn.1 <= 0 {
            barrack_state.spawn_queue.pop_front();
        }
    }
}

/// 辅助函数：根据不同的 WaveBehavior 计算应生成的数量
fn calculate_spawn_count(
    behavior: &BarracksMinionConfigWaveBehavior,
    game_time_secs: f32,
    wave_count: u32,
    inhibitor_state: &InhibitorState,
) -> i32 {
    match behavior {
        BarracksMinionConfigWaveBehavior::ConstantWaveBehavior(ConstantWaveBehavior {
            spawn_count,
        }) => *spawn_count,
        BarracksMinionConfigWaveBehavior::InhibitorWaveBehavior(InhibitorWaveBehavior {
            spawn_count_per_inhibitor_down,
        }) => {
            if inhibitor_state.inhibitors_down == 0 {
                return 0;
            }

            spawn_count_per_inhibitor_down
                .get(inhibitor_state.inhibitors_down - 1)
                .copied()
                .unwrap_or(0)
        }
        BarracksMinionConfigWaveBehavior::TimedVariableWaveBehavior(
            TimedVariableWaveBehavior { behaviors },
        ) => {
            // 寻找当前时间点最合适的行为
            let mut active_behavior = None;
            for timed_behavior in behaviors.iter().rev() {
                if game_time_secs >= timed_behavior.start_time_secs as f32 {
                    active_behavior = Some(&timed_behavior.behavior);
                    break;
                }
            }

            if let Some(active_behavior) = active_behavior {
                // 递归调用
                calculate_spawn_count(active_behavior, game_time_secs, wave_count, inhibitor_state)
            } else {
                0
            }
        }
        BarracksMinionConfigWaveBehavior::RotatingWaveBehavior(RotatingWaveBehavior {
            spawn_counts_by_wave,
        }) => {
            if spawn_counts_by_wave.is_empty() {
                0
            } else {
                spawn_counts_by_wave
                    [((wave_count - 1) % spawn_counts_by_wave.len() as u32) as usize]
            }
        }
    }
}
