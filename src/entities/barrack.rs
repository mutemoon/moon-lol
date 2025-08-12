use crate::{
    core::{
        spawn_environment_object, Armor, Bounding, ConfigEnvironmentObject, Configs, Damage,
        Health, Lane, Movement, Team,
    },
    entities::{Minion, MinionPath},
};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::{collections::VecDeque, time::Duration};

#[derive(Component, Clone, Serialize, Deserialize)]
pub struct Barrack {
    pub initial_spawn_time_secs: f32,
    pub wave_spawn_interval_secs: f32,
    pub minion_spawn_interval_secs: f32,
    pub upgrade_interval_secs: f32,
    pub upgrades_before_late_game_scaling: i32,
    pub move_speed_increase_initial_delay_secs: f32,
    pub move_speed_increase_interval_secs: f32,
    pub move_speed_increase_increment: i32,
    pub move_speed_increase_max_times: i32,
    pub exp_radius: f32,
    pub gold_radius: f32,
    pub units: Vec<BarracksMinionConfig>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MinionRecord {
    pub team: Team,
    pub character_record: String,
    pub skin: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct BarracksMinionConfig {
    pub minion_type: Minion,
    pub minion_object: (
        Health,
        Movement,
        Bounding,
        Damage,
        Armor,
        ConfigEnvironmentObject,
    ),
    pub wave_behavior: WaveBehavior,
    pub minion_upgrade_stats: MinionUpgradeConfig,
}

#[derive(Clone, Serialize, Deserialize)]
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

#[derive(Clone, Serialize, Deserialize)]
pub struct TimedWaveBehaviorInfo {
    pub start_time_secs: i32,
    pub behavior: WaveBehavior,
}

#[derive(Clone, Serialize, Deserialize)]
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

/// 兵营的动态状态，用于跟踪计时器和生成队列
#[derive(Component)]
pub struct BarrackState {
    /// 下一波兵的生成计时器
    pub wave_timer: Timer,
    /// 属性升级计时器
    pub upgrade_timer: Timer,
    /// 移动速度升级计时器
    pub move_speed_upgrade_timer: Timer,
    /// 同一波兵中，每个小兵之间的生成间隔计时器
    pub intra_spawn_timer: Timer,
    /// 当前的生成队列
    pub spawn_queue: VecDeque<MinionSpawnInfo>,
    /// 已应用的属性升级次数
    pub upgrade_count: i32,
    /// 已应用的移速升级次数
    pub move_speed_upgrade_count: i32,
    /// 已生成的波数
    pub wave_count: u32,
}

/// 用于在生成队列中存储信息
#[derive(Clone, Serialize, Deserialize)]
pub struct MinionSpawnInfo {
    /// 对应兵营 `units` Vec 中的索引
    pub config_index: usize,
    /// 剩余要生成的数量
    pub count: i32,
}

/// 一个全局资源，用于跟踪被摧毁的水晶数量
/// `InhibitorWaveBehavior` 需要这个状态
#[derive(Resource, Default)]
pub struct InhibitorState {
    pub inhibitors_down: usize,
}

// --- Bevy 插件和系统 ---

pub struct PluginBarrack;

impl Plugin for PluginBarrack {
    fn build(&self, app: &mut App) {
        app.init_resource::<InhibitorState>();
        app.add_systems(Startup, setup);
        app.add_systems(Update, barracks_spawning_system);
    }
}

fn setup(mut commands: Commands, res_game_config: Res<Configs>) {
    for (transform, team, lane, barrack) in res_game_config.barracks.clone().into_iter() {
        let initial_delay = barrack.initial_spawn_time_secs;

        commands.spawn((
            transform,
            team,
            lane,
            BarrackState {
                // 第一波兵有初始延迟
                wave_timer: Timer::from_seconds(initial_delay, TimerMode::Repeating),
                // 属性升级从第一波兵生成后开始计算
                upgrade_timer: Timer::new(
                    Duration::from_secs_f32(barrack.upgrade_interval_secs),
                    TimerMode::Repeating,
                ),
                // 移速升级有自己的独立延迟
                move_speed_upgrade_timer: Timer::new(
                    Duration::from_secs_f32(barrack.move_speed_increase_initial_delay_secs),
                    TimerMode::Repeating,
                ),
                // 小兵间的生成间隔计时器
                intra_spawn_timer: Timer::from_seconds(
                    barrack.minion_spawn_interval_secs,
                    TimerMode::Repeating,
                ),
                spawn_queue: VecDeque::new(),
                upgrade_count: 0,
                move_speed_upgrade_count: 0,
                wave_count: 0,
            },
            barrack,
        ));
    }
}

/// 核心系统：处理兵营的计时、升级和生成逻辑
fn barracks_spawning_system(
    asset_server: Res<AssetServer>,
    game_time: Res<Time<Virtual>>,
    inhibitor_state: Res<InhibitorState>,
    mut commands: Commands,
    mut query: Query<(&GlobalTransform, &Barrack, &mut BarrackState, &Team, &Lane)>,
    res_game_config: Res<Configs>,
    time: Res<Time>,
) {
    for (transform, barrack, mut state, team, lane) in query.iter_mut() {
        // --- 1. 更新所有计时器 ---
        state.wave_timer.tick(time.delta());

        // 只有在第一波之后才开始计时升级
        if state.wave_count > 0 {
            state.upgrade_timer.tick(time.delta());
            state.move_speed_upgrade_timer.tick(time.delta());
        }

        // --- 2. 处理属性和移速升级 ---
        if state.upgrade_timer.just_finished() {
            state.upgrade_count += 1;
            println!("Barrack upgraded! New count: {}", state.upgrade_count);
        }

        if state.move_speed_upgrade_timer.just_finished() {
            if state.move_speed_upgrade_count < barrack.move_speed_increase_max_times {
                state.move_speed_upgrade_count += 1;
                println!(
                    "Minion move speed upgraded! New count: {}",
                    state.move_speed_upgrade_count
                );
            }
        }

        // --- 3. 检查是否需要生成新一波小兵 ---
        // 只有当上一波完全生成完后（队列为空），才开始准备新一波
        if state.wave_timer.just_finished() && state.spawn_queue.is_empty() {
            state.wave_count += 1;
            state
                .wave_timer
                .set_duration(Duration::from_secs_f32(barrack.wave_spawn_interval_secs));

            // 遍历兵营配置中的所有小兵类型
            for (index, minion_config) in barrack.units.iter().enumerate() {
                let spawn_count = calculate_spawn_count(
                    &minion_config.wave_behavior,
                    game_time.elapsed_secs(),
                    state.wave_count,
                    &inhibitor_state,
                );

                if spawn_count > 0 {
                    state.spawn_queue.push_back(MinionSpawnInfo {
                        config_index: index,
                        count: spawn_count,
                    });
                }
            }
        }

        // --- 4. 处理生成队列，逐个生成小兵 ---
        if !state.spawn_queue.is_empty() {
            state.intra_spawn_timer.tick(time.delta());

            if state.intra_spawn_timer.just_finished() {
                let upgrade_count = state.upgrade_count;
                let move_speed_upgrade_count = state.move_speed_upgrade_count;

                // 获取队列头部的待生成小兵信息
                if let Some(current_spawn) = state.spawn_queue.front_mut() {
                    let config_index = current_spawn.config_index;
                    let minion_config = &barrack.units[config_index];
                    let upgrade_config = &minion_config.minion_upgrade_stats;

                    // --- 计算小兵最终属性 ---
                    let is_late_game = upgrade_count >= barrack.upgrades_before_late_game_scaling;

                    let (
                        mut health,
                        mut movement,
                        bounding,
                        mut damage,
                        mut armor,
                        config_environment_object,
                    ) = minion_config.minion_object.clone();

                    let hp_upgrade = if is_late_game {
                        upgrade_config.hp_upgrade_late
                    } else {
                        upgrade_config.hp_upgrade
                    };

                    health.max += upgrade_config.hp_max_bonus + hp_upgrade * upgrade_count as f32;
                    health.value = health.max;

                    let damage_upgrade = if is_late_game {
                        upgrade_config.damage_upgrade_late
                    } else {
                        upgrade_config.damage_upgrade
                    };

                    damage.0 += upgrade_config.damage_max + damage_upgrade * upgrade_count as f32;

                    armor.0 += upgrade_config.armor_max
                        + upgrade_config.armor_upgrade_growth * upgrade_count as f32;

                    movement.speed +=
                        (barrack.move_speed_increase_increment * move_speed_upgrade_count) as f32;

                    let entity = spawn_environment_object(
                        &mut commands,
                        &asset_server,
                        transform.compute_transform(),
                        &config_environment_object,
                    );

                    let mut path = res_game_config.minion_paths.get(lane).unwrap().clone();

                    if *team == Team::Chaos {
                        path.reverse();
                    }

                    commands.entity(entity).insert((
                        minion_config.minion_type,
                        MinionPath(path),
                        health,
                        movement,
                        damage,
                        armor,
                        bounding,
                        team.clone(),
                    ));

                    // 更新队列
                    current_spawn.count -= 1;
                    if current_spawn.count <= 0 {
                        state.spawn_queue.pop_front();
                    }
                }
            }
        }
    }
}

/// 辅助函数：根据不同的 WaveBehavior 计算应生成的数量
fn calculate_spawn_count(
    behavior: &WaveBehavior,
    game_time_secs: f32,
    wave_count: u32,
    inhibitor_state: &InhibitorState,
) -> i32 {
    match behavior {
        WaveBehavior::ConstantWaveBehavior { spawn_count } => *spawn_count,
        WaveBehavior::InhibitorWaveBehavior {
            spawn_count_per_inhibitor_down,
        } => spawn_count_per_inhibitor_down
            .get(inhibitor_state.inhibitors_down)
            .copied()
            .unwrap_or(0),
        WaveBehavior::RotatingWaveBehavior {
            spawn_counts_by_wave,
        } => {
            if spawn_counts_by_wave.is_empty() {
                0
            } else {
                spawn_counts_by_wave
                    [((wave_count - 1) % spawn_counts_by_wave.len() as u32) as usize]
            }
        }
        WaveBehavior::TimedVariableWaveBehavior { behaviors } => {
            // 寻找当前时间点最合适的行为
            let mut active_behavior = &WaveBehavior::Unknown;
            for timed_behavior in behaviors.iter().rev() {
                if game_time_secs >= timed_behavior.start_time_secs as f32 {
                    active_behavior = &timed_behavior.behavior;
                    break;
                }
            }
            // 递归调用
            calculate_spawn_count(active_behavior, game_time_secs, wave_count, inhibitor_state)
        }
        WaveBehavior::Unknown => 0,
    }
}
