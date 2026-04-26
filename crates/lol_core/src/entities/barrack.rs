use std::collections::VecDeque;
use std::time::Duration;

use bevy::prelude::*;
use lol_base::barrack::{
    ConfigBarracks, ConstantWaveBehavior, EnumWaveBehavior, InhibitorWaveBehavior,
    RotatingWaveBehavior, TimedVariableWaveBehavior,
};
use lol_loader::barrack::ConfigBarracksLoader;

use crate::entities::minion::Minion;
use crate::lane::Lane;
use crate::team::Team;

#[derive(Default)]
pub struct PluginBarrack;

impl Plugin for PluginBarrack {
    fn build(&self, app: &mut App) {
        app.init_asset::<ConfigBarracks>();
        app.init_asset_loader::<ConfigBarracksLoader>();
        app.init_resource::<InhibitorState>();
        app.add_systems(FixedUpdate, init_barrack_state_system);
        app.add_systems(FixedUpdate, barracks_spawning_system);
    }
}

/// 兵营配置句柄，仅持有配置资源的引用
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct BarrackConfigHandler {
    /// 兵营配置的句柄
    pub config_handle: Handle<ConfigBarracks>,
}

impl BarrackConfigHandler {
    pub fn new(config_handle: Handle<ConfigBarracks>) -> Self {
        Self { config_handle }
    }
}

/// 兵营的动态状态，用于跟踪计时器和生成队列
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
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
    pub spawn_queue: VecDeque<(usize, i32)>,
    /// 已应用的属性升级次数
    pub upgrade_count: i32,
    /// 已应用的移速升级次数
    pub move_speed_upgrade_count: i32,
    /// 已生成的波数
    pub wave_count: u32,
}

/// 系统：为拥有 BarrackConfigHandler 但没有 BarrackState 的实体添加 BarrackState
/// 根据 ConfigBarracks 初始化计时器
fn init_barrack_state_system(
    mut commands: Commands,
    query: Query<(Entity, &BarrackConfigHandler), Without<BarrackState>>,
    res_barracks_config: Res<Assets<ConfigBarracks>>,
) {
    for (entity, config_handler) in query.iter() {
        let Some(config) = res_barracks_config.get(&config_handler.config_handle) else {
            continue;
        };

        commands.entity(entity).insert(BarrackState {
            // 第一波兵有初始延迟
            wave_timer: Timer::from_seconds(0.0, TimerMode::Repeating),
            // 属性升级从第一波兵生成后开始计算
            upgrade_timer: Timer::new(
                Duration::from_secs_f32(config.upgrade_interval_secs),
                TimerMode::Repeating,
            ),
            // 移速升级有自己的独立延迟
            move_speed_upgrade_timer: Timer::new(
                Duration::from_secs_f32(config.move_speed_increase_initial_delay_secs),
                TimerMode::Repeating,
            ),
            // 小兵间的生成间隔计时器
            intra_spawn_timer: Timer::from_seconds(
                config.minion_spawn_interval_secs,
                TimerMode::Repeating,
            ),
            ..Default::default()
        });
    }
}

#[derive(Resource, Default)]
pub struct InhibitorState {
    pub inhibitors_down: usize,
}

/// 核心系统：处理兵营的计时、升级和生成逻辑
fn barracks_spawning_system(
    game_time: Res<Time<Virtual>>,
    inhibitor_state: Res<InhibitorState>,
    mut commands: Commands,
    mut query: Query<(
        &GlobalTransform,
        &BarrackConfigHandler,
        &mut BarrackState,
        &Team,
        &Lane,
    )>,
    res_barracks_config: Res<Assets<ConfigBarracks>>,
    time: Res<Time>,
    // res_assets_unk_ad65d8c4: Res<Assets<Unk0xad65d8c4>>,
) {
    for (transform, config_handler, mut barrack_state, team, lane) in query.iter_mut() {
        let Some(barracks_config) = res_barracks_config.get(&config_handler.config_handle) else {
            continue;
        };

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
            if barrack_state.move_speed_upgrade_count
                < barracks_config.move_speed_increase_max_times
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
                    barracks_config.wave_spawn_interval_secs,
                ));

            // 遍历兵营配置中的所有小兵类型
            for (index, minion_config) in barracks_config.units.iter().enumerate() {
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

        // 获取队列头部的待生成小兵信息
        let Some(current_spawn) = barrack_state.spawn_queue.front_mut() else {
            continue;
        };

        let config_index = current_spawn.0;
        let minion_config = &barracks_config.units[config_index];
        let upgrade_config = &minion_config.minion_upgrade_stats;

        // --- 计算小兵最终属性 ---
        let is_late_game = upgrade_count >= barracks_config.upgrades_before_late_game_scaling;

        // let character = res_assets_unk_ad65d8c4
        //     .load_hash(minion_config.unk_0xfee040bc)
        //     .unwrap();

        // let _character_record = res_assets_character_record
        //     .load_hash(&character.character.character_record)
        //     .unwrap();

        // let mut health = Health::new(character_record.base_hp.unwrap_or(0.0));
        let _hp_upgrade = if is_late_game {
            upgrade_config.hp_upgrade_late.unwrap_or(0.0)
        } else {
            upgrade_config.hp_upgrade
        };
        // health.max += (hp_upgrade * upgrade_count as f32).min(upgrade_config.hp_max_bonus);

        // let mut damage = Damage(character_record.base_damage.unwrap_or(0.0));
        let _damage_upgrade = if is_late_game {
            upgrade_config.damage_upgrade_late
        } else {
            upgrade_config.damage_upgrade
        };
        // damage.0 += damage_upgrade.unwrap_or(0.0) * upgrade_count as f32;
        // damage.0 = damage.0.min(upgrade_config.damage_max);

        // let mut armor = Armor(character_record.base_armor.unwrap_or(0.0));
        // armor.0 += upgrade_config.armor_upgrade_growth.unwrap_or(0.0) * upgrade_count as f32;
        // if let Some(max) = upgrade_config.armor_max {
        //     armor.0 = armor.0.min(max);
        // }

        // let mut movement = Movement {
        //     speed: character_record.base_move_speed.unwrap_or(0.0),
        // };
        // movement.speed +=
        //     (barracks_config.move_speed_increase_increment * move_speed_upgrade_count) as f32;

        debug!("生成一个小兵");

        let _entity = commands
            .spawn((
                Transform::from_matrix(transform.to_matrix()),
                Minion::from(minion_config.minion_type),
                lane.clone(),
                team.clone(),
            ))
            .id();

        // 触发角色生成命令（创建基础组件并加载皮肤）
        // commands.trigger(CommandCharacterSpawn {
        //     entity,
        //     character_record: (&character.character.character_record).into(),
        //     skin: (&character.character.skin).into(),
        // });

        // commands
        //     .entity(entity)
        //     .insert((health, damage, armor, movement));

        // 更新队列
        current_spawn.1 -= 1;
        if current_spawn.1 <= 0 {
            barrack_state.spawn_queue.pop_front();
        }
    }
}

/// 辅助函数：根据不同的 WaveBehavior 计算应生成的数量
fn calculate_spawn_count(
    behavior: &EnumWaveBehavior,
    game_time_secs: f32,
    wave_count: u32,
    inhibitor_state: &InhibitorState,
) -> i32 {
    match behavior {
        EnumWaveBehavior::ConstantWaveBehavior(ConstantWaveBehavior { spawn_count }) => {
            *spawn_count
        }
        EnumWaveBehavior::InhibitorWaveBehavior(InhibitorWaveBehavior {
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
        EnumWaveBehavior::TimedVariableWaveBehavior(TimedVariableWaveBehavior {
            behaviors,
            ..
        }) => {
            // 寻找当前时间点最合适的行为
            let mut active_behavior = None;
            for timed_behavior in behaviors.iter().rev() {
                let start_time: f32 = timed_behavior.start_time_secs.unwrap_or(0) as f32;
                if game_time_secs >= start_time {
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
        EnumWaveBehavior::RotatingWaveBehavior(RotatingWaveBehavior {
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
