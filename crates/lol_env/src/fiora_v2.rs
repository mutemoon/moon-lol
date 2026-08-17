use std::collections::HashMap;
use std::path::PathBuf;

use bevy::app::ScheduleRunnerPlugin;
use bevy::ecs::schedule::SingleThreadedExecutor;
use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use lol_base::character::Skin;
use lol_base_render::camera::CameraState;
use lol_champions::fiora::e::BuffFioraE;
use lol_champions::fiora::passive::Vital;
use lol_champions::fiora::r::BuffFioraR;
use lol_champions::fiora::{Fiora, PluginFiora};
use lol_champions::riven::PluginRiven;
use lol_core::action::{Action, CommandAction};
use lol_core::attack::{Attack, AttackState, AttackStatus};
use lol_core::base::buff::Buffs;
use lol_core::base::direction::Direction;
use lol_core::character::CharacterReady;
use lol_core::life::Health;
use lol_core::navigation::navigation::NavigationDebug;
use lol_core::skill::{CoolDown, SkillRecastWindow, Skills, is_skill_ready};
use lol_rl_protocol::{
    ActionSpace, ObsFeaturePayload, RewardExpr, RewardFormulaSpec, RewardTermSpec,
};

use crate::fiora_riven_common::{
    ATTACK_MASK_DISTANCE, FioraRivenEntities, VitalBreakTracker, add_common_observers,
    reset_episode_world, setup_skill_levels_world, spawn_champions_world, unpause_virtual_time,
};
use crate::reward::RewardModel;
pub use crate::traits::{
    EnvConfig, EnvMeta, RenderMode, RewardBreakdownItem, RlEnvironment, StepResult,
    VisualEnvironment,
};

/// 连续位移与 Q 突刺偏移缩放：offset_x/offset_z ∈ [-1,1] 映射到相对 riven 的 ±OFFSET_SCALE 偏移。
pub const OFFSET_SCALE: f32 = 100.0;

/// 闪现瞬移距离（单位）：沿 offset 连续值方向瞬移 300 单位。
pub const FLASH_DISTANCE: f32 = 300.0;

/// 闪现冷却时长（秒），与英雄联盟召唤师技能一致。
pub const FLASH_COOLDOWN_SECS: f32 = 300.0;

/// 观测向量中「相对距离归一化列」的下标，与 [`FioraV2Obs::to_vector`] 的布局一致。
pub const V2_OBS_DISTANCE_IDX: usize = 16;
pub const V2_OBS_DISTANCE_SCALE: f32 = 100.0;

/// 靶子瑞雯在 V2 中的生命值上限与初始值（避免被角色配置默认血量覆盖）
pub const RIVEN_V2_HP: f32 = 10000.0;

/// 设置 V2 靶子瑞雯的生命值为 10000.0
pub fn setup_v2_riven_health_world(world: &mut World, riven: Entity) {
    if let Some(mut health) = world.get_mut::<Health>(riven) {
        health.value = RIVEN_V2_HP;
        health.max = RIVEN_V2_HP;
    } else {
        world.entity_mut(riven).insert(Health::new(RIVEN_V2_HP));
    }
}

/// 角色配置写入完成后（`CharacterReady` 插入时，覆盖了 DynamicWorld 的默认血量），自动将 V2 瑞雯血量设为 10000.0
pub fn on_v2_character_ready_setup_riven_health(
    trigger: On<Add, CharacterReady>,
    mut q_health: Query<&mut Health, With<lol_champions::riven::Riven>>,
) {
    let entity = trigger.entity;
    if let Ok(mut health) = q_health.get_mut(entity) {
        health.value = RIVEN_V2_HP;
        health.max = RIVEN_V2_HP;
    }
}

// ── 离散动作与混合动作定义 ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FioraV2DiscreteAction {
    /// 0: 不做任何新动作（保持当前执行状态，关键用于不打断普攻前摇/后摇/技能位移）
    NoOp = 0,
    /// 1: 移动到 riven_pos + [offset_x, offset_z] * OFFSET_SCALE
    Move = 1,
    /// 2: 普通攻击瑞雯
    Attack = 2,
    /// 3: 向 riven_pos + [offset_x, offset_z] * OFFSET_SCALE 施放 Q 技能突刺
    CastQ = 3,
    /// 4: 施放 E 技能（重置普攻并强化下两次攻击）
    CastE = 4,
    /// 5: 对瑞雯施放 R 技能（套上 4 破绽）
    CastR = 5,
    /// 6: 闪现：沿 offset 方向瞬移 300 单位（手动实现，无游戏侧技能）
    CastFlash = 6,
}

impl FioraV2DiscreteAction {
    pub fn from_u8(val: u8) -> Self {
        match val {
            0 => Self::NoOp,
            1 => Self::Move,
            2 => Self::Attack,
            3 => Self::CastQ,
            4 => Self::CastE,
            5 => Self::CastR,
            6 => Self::CastFlash,
            _ => Self::NoOp,
        }
    }

    pub fn to_u8(self) -> u8 {
        self as u8
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FioraV2Action {
    /// 连续偏移 X 分量 ∈ [-1, 1]，相对 riven 的 X 偏移（Move / CastQ / CastFlash 共用）
    pub offset_x: f32,
    /// 连续偏移 Z 分量 ∈ [-1, 1]，相对 riven 的 Z 偏移（Move / CastQ / CastFlash 共用）
    pub offset_z: f32,
    /// 离散动作类别 (0: NoOp, 1: Move, 2: Attack, 3: CastQ, 4: CastE, 5: CastR, 6: CastFlash)
    pub discrete: FioraV2DiscreteAction,
}

impl FioraV2Action {
    pub const fn new(offset_x: f32, offset_z: f32, discrete: FioraV2DiscreteAction) -> Self {
        Self {
            offset_x,
            offset_z,
            discrete,
        }
    }

    /// 扁平编码转动作：[offset_x, offset_z, discrete_idx]
    pub fn from_encoding(encoded: &[f32]) -> Self {
        let offset_x = encoded.first().copied().unwrap_or(0.0);
        let offset_z = encoded.get(1).copied().unwrap_or(0.0);
        let discrete_idx = encoded.get(2).copied().unwrap_or(0.0).round().max(0.0) as u8;
        Self::new(
            offset_x,
            offset_z,
            FioraV2DiscreteAction::from_u8(discrete_idx),
        )
    }

    pub fn to_encoding(&self) -> Vec<f32> {
        vec![self.offset_x, self.offset_z, self.discrete.to_u8() as f32]
    }

    /// UI 手动面板预设（11 按钮）
    pub fn preset_from_index(index: usize) -> Self {
        match index {
            0 => Self::new(0.0, 0.0, FioraV2DiscreteAction::NoOp),
            1 => Self::new(1.0, 0.0, FioraV2DiscreteAction::Move), // Move East
            2 => Self::new(-1.0, 0.0, FioraV2DiscreteAction::Move), // Move West
            3 => Self::new(0.0, 1.0, FioraV2DiscreteAction::Move), // Move North
            4 => Self::new(0.0, -1.0, FioraV2DiscreteAction::Move), // Move South
            5 => Self::new(0.0, 0.0, FioraV2DiscreteAction::Move), // Chase Riven
            6 => Self::new(0.0, 0.0, FioraV2DiscreteAction::Attack), // Attack
            7 => Self::new(1.0, 0.0, FioraV2DiscreteAction::CastQ), // Q East
            8 => Self::new(0.0, 0.0, FioraV2DiscreteAction::CastE), // E
            9 => Self::new(0.0, 0.0, FioraV2DiscreteAction::CastR), // R
            10 => Self::new(1.0, 0.0, FioraV2DiscreteAction::CastFlash), // Flash East
            _ => Self::new(0.0, 0.0, FioraV2DiscreteAction::NoOp),
        }
    }

    pub fn preset_index(&self) -> usize {
        match self.discrete {
            FioraV2DiscreteAction::NoOp => 0,
            FioraV2DiscreteAction::Move => {
                if self.offset_x > 0.5 {
                    1
                } else if self.offset_x < -0.5 {
                    2
                } else if self.offset_z > 0.5 {
                    3
                } else if self.offset_z < -0.5 {
                    4
                } else {
                    5
                }
            }
            FioraV2DiscreteAction::Attack => 6,
            FioraV2DiscreteAction::CastQ => 7,
            FioraV2DiscreteAction::CastE => 8,
            FioraV2DiscreteAction::CastR => 9,
            FioraV2DiscreteAction::CastFlash => 10,
        }
    }

    pub fn desc(&self) -> &'static str {
        match self.discrete {
            FioraV2DiscreteAction::NoOp => "NoOp (保持/空动作)",
            FioraV2DiscreteAction::Move => "Move (移动)",
            FioraV2DiscreteAction::Attack => "Attack (普攻)",
            FioraV2DiscreteAction::CastQ => "CastQ (Q突刺)",
            FioraV2DiscreteAction::CastE => "CastE (E剑术)",
            FioraV2DiscreteAction::CastR => "CastR (R无双挑战)",
            FioraV2DiscreteAction::CastFlash => "Flash (闪现)",
        }
    }
}

// ── 闪现（Flash） ──────────────────────────────────────────────────────────────

/// 闪现冷却状态（挂在剑姬实体上）。`None` = 已就绪；`Some(timer)` = 冷却中。
///
/// 游戏中没有召唤师技能/闪现机制，闪现由本环境手动实现：
/// `dispatch_action_world` 直接瞬移剑姬 `Transform`，冷却由本组件 + FixedUpdate 系统推进。
#[derive(Component, Default)]
pub struct FlashCooldown(pub Option<Timer>);

impl FlashCooldown {
    pub fn is_ready(&self) -> bool {
        self.0.as_ref().map_or(true, |t| t.is_finished())
    }

    pub fn remaining_secs(&self) -> f32 {
        self.0
            .as_ref()
            .map(|t| {
                if t.is_finished() {
                    0.0
                } else {
                    t.remaining_secs()
                }
            })
            .unwrap_or(0.0)
    }

    /// 施放闪现后开始 300s 冷却。
    pub fn start(&mut self) {
        self.0 = Some(Timer::from_seconds(FLASH_COOLDOWN_SECS, TimerMode::Once));
    }
}

/// FixedUpdate 中推进剑姬的闪现冷却。
pub fn tick_flash_cooldown(
    time: Res<Time<Fixed>>,
    mut q: Query<&mut FlashCooldown, With<Fiora>>,
) {
    for mut flash in q.iter_mut() {
        if let Some(timer) = flash.0.as_mut() {
            timer.tick(time.delta());
            if timer.is_finished() {
                flash.0 = None;
            }
        }
    }
}

// ── 观测数据结构 ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct FioraV2Obs {
    pub fiora_pos: Vec3,
    pub fiora_hp: f32,
    pub fiora_max_hp: f32,
    pub riven_pos: Vec3,
    pub riven_hp: f32,
    pub riven_max_hp: f32,
    pub distance: f32,

    // 被动破绽
    pub has_vital: bool,
    pub vital_is_active: bool,
    pub vital_active_timer_remaining: f32,
    pub vital_remove_timer_remaining: f32,
    pub vital_dir_x: f32,
    pub vital_dir_neg_x: f32,
    pub vital_dir_z: f32,
    pub vital_dir_neg_z: f32,

    // 大招破绽 (BuffFioraR)
    pub has_r_vital: bool,
    pub r_is_active: bool,
    pub r_active_timer_remaining: f32,
    pub r_remove_timer_remaining: f32,
    pub r_vital_east: bool,
    pub r_vital_west: bool,
    pub r_vital_north: bool,
    pub r_vital_south: bool,

    // 普攻状态与计时器
    pub attack_state: u8, // 0: Ready, 1: Windup, 2: Cooldown
    pub attack_is_windup: bool,
    pub attack_is_cooldown: bool,
    pub attack_timer_remaining: f32,
    pub attack_windup_duration: f32,
    pub attack_total_duration: f32,

    // 技能冷却状态
    pub q_ready: bool,
    pub q_cd_remaining: f32,
    pub w_ready: bool,
    pub w_cd_remaining: f32,
    pub e_ready: bool,
    pub e_cd_remaining: f32,
    pub r_ready: bool,
    pub r_cd_remaining: f32,

    // 闪现状态
    pub flash_ready: bool,
    pub flash_cd_remaining: f32,

    // E Buff 状态
    pub has_buff_e: bool,
    pub buff_e_left: i32,
}

impl FioraV2Obs {
    /// 转换为强化学习策略网络输入向量（33 维）
    pub fn to_vector(&self) -> Vec<f32> {
        let rel_x = self.fiora_pos.x - self.riven_pos.x;
        let rel_z = self.fiora_pos.z - self.riven_pos.z;

        vec![
            // 0..4: 被动破绽四方位
            self.vital_dir_x,
            self.vital_dir_neg_x,
            self.vital_dir_z,
            self.vital_dir_neg_z,
            // 4..8: 被动破绽状态与计时器
            if self.has_vital { 1.0 } else { 0.0 },
            if self.vital_is_active { 1.0 } else { 0.0 },
            self.vital_active_timer_remaining / 1.7,
            self.vital_remove_timer_remaining / 4.0,
            // 8..12: 大招破绽四方向存在状态
            if self.r_vital_east { 1.0 } else { 0.0 },
            if self.r_vital_west { 1.0 } else { 0.0 },
            if self.r_vital_north { 1.0 } else { 0.0 },
            if self.r_vital_south { 1.0 } else { 0.0 },
            // 12..16: 大招破绽状态与计时器
            if self.has_r_vital { 1.0 } else { 0.0 },
            if self.r_is_active { 1.0 } else { 0.0 },
            self.r_active_timer_remaining / 0.5,
            self.r_remove_timer_remaining / 8.0,
            // 16..19: 空间相对位置与距离 (idx 16 对应 V2_OBS_DISTANCE_IDX)
            self.distance / V2_OBS_DISTANCE_SCALE,
            rel_x / V2_OBS_DISTANCE_SCALE,
            rel_z / V2_OBS_DISTANCE_SCALE,
            // 19..23: 普攻状态与计时器
            if self.attack_state == 0 { 1.0 } else { 0.0 }, // Is Ready
            if self.attack_is_windup { 1.0 } else { 0.0 },
            if self.attack_is_cooldown { 1.0 } else { 0.0 },
            self.attack_timer_remaining / 1.0,
            // 23..29: 技能就绪与冷却
            if self.q_ready { 1.0 } else { 0.0 },
            self.q_cd_remaining / 10.0,
            if self.e_ready { 1.0 } else { 0.0 },
            self.e_cd_remaining / 10.0,
            if self.r_ready { 1.0 } else { 0.0 },
            self.r_cd_remaining / 60.0,
            // 29..31: 血量百分比
            if self.fiora_max_hp > 0.0 {
                self.fiora_hp / self.fiora_max_hp
            } else {
                1.0
            },
            if self.riven_max_hp > 0.0 {
                self.riven_hp / self.riven_max_hp
            } else {
                1.0
            },
            // 31..33: 闪现就绪与冷却
            if self.flash_ready { 1.0 } else { 0.0 },
            self.flash_cd_remaining / FLASH_COOLDOWN_SECS,
        ]
    }

    pub fn dim() -> usize {
        33
    }

    pub fn to_payload(&self) -> ObsFeaturePayload {
        let vital_dir = if self.vital_dir_x > 0.5 {
            "+X (东)".to_string()
        } else if self.vital_dir_neg_x > 0.5 {
            "-X (西)".to_string()
        } else if self.vital_dir_z > 0.5 {
            "+Z (北)".to_string()
        } else if self.vital_dir_neg_z > 0.5 {
            "-Z (南)".to_string()
        } else {
            "无".to_string()
        };

        let atk_str = match self.attack_state {
            0 => "就绪 (Ready)".to_string(),
            1 => format!("前摇中 ({:.2}s)", self.attack_timer_remaining),
            2 => format!("后摇冷却 ({:.2}s)", self.attack_timer_remaining),
            _ => "未知".to_string(),
        };

        let f_hp = if self.fiora_max_hp > 0.0 {
            self.fiora_hp / self.fiora_max_hp
        } else {
            1.0
        };
        let r_hp = if self.riven_max_hp > 0.0 {
            self.riven_hp / self.riven_max_hp
        } else {
            1.0
        };

        ObsFeaturePayload {
            self_hp_pct: f_hp,
            target_hp_pct: r_hp,
            fiora_hp_pct: f_hp,
            riven_hp_pct: r_hp,
            distance: self.distance,
            q_ready: self.q_ready,
            w_ready: self.w_ready,
            e_ready: self.e_ready,
            r_ready: self.r_ready,
            has_vital: self.has_vital,
            vital_is_active: self.vital_is_active,
            vital_direction: vital_dir,
            vital_active_time: self.vital_active_timer_remaining,
            has_r_vital: self.has_r_vital,
            r_is_active: self.r_is_active,
            attack_state: atk_str,
            attack_timer: self.attack_timer_remaining,
            ..Default::default()
        }
    }
}

// ── 奖励模型 ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default)]
pub struct FioraV2RewardContext {
    pub prev_aligned: bool,
    pub curr_aligned: bool,
    pub is_vital_break: bool,
    pub prev_riven_hp: f32,
    pub curr_riven_hp: f32,
    pub riven_max_hp: f32,
    pub elapsed_secs: f32,
}

pub struct FioraV2RewardModel;

impl RewardModel for FioraV2RewardModel {
    type Context = FioraV2RewardContext;

    fn formula_spec(&self) -> RewardFormulaSpec {
        RewardFormulaSpec {
            name: "FioraV2 实战对决奖励公式".to_string(),
            terms: vec![
                RewardTermSpec::new(
                    "time_penalty",
                    "时间步惩罚 (Step Penalty)",
                    RewardExpr::Constant(-0.001),
                ),
                RewardTermSpec::new(
                    "damage_dealt",
                    "伤害收益 (Damage Dealt, 归一化)",
                    RewardExpr::Mul(
                        Box::new(RewardExpr::Constant(2.5)),
                        Box::new(RewardExpr::Variable("damage_ratio".into())),
                    ),
                ),
                RewardTermSpec::new(
                    "kill_reward",
                    "击杀基础奖励 (Kill Reward)",
                    RewardExpr::Mul(
                        Box::new(RewardExpr::Constant(2.0)),
                        Box::new(RewardExpr::Variable("is_kill".into())),
                    ),
                ),
            ],
        }
    }

    fn extract_variables(&self, ctx: &FioraV2RewardContext) -> HashMap<String, f32> {
        let mut vars = HashMap::new();
        let max_hp = if ctx.riven_max_hp > 0.0 {
            ctx.riven_max_hp
        } else {
            1000.0
        };
        let hp_diff = (ctx.prev_riven_hp - ctx.curr_riven_hp).max(0.0);
        let damage_ratio = hp_diff / max_hp;
        let is_kill = if ctx.curr_riven_hp <= 0.0 && ctx.prev_riven_hp > 0.0 {
            1.0
        } else {
            0.0
        };

        vars.insert("damage_ratio".into(), damage_ratio);
        vars.insert("hp_diff".into(), hp_diff);
        vars.insert("is_kill".into(), is_kill);
        vars.insert("elapsed_secs".into(), ctx.elapsed_secs);
        vars.insert("step_tick".into(), 1.0);
        vars
    }
}

// ── 环境主体定义 ─────────────────────────────────────────────────────────────

pub struct FioraV2Env {
    pub app: App,
    pub fiora: Entity,
    pub riven: Entity,
    fiora_config_handle: Handle<DynamicWorld>,
    riven_config_handle: Handle<DynamicWorld>,
    fiora_skin_handle: Option<Handle<DynamicWorld>>,
    riven_skin_handle: Option<Handle<DynamicWorld>>,
    step_count: usize,
    max_steps: usize,
    initial_fiora_pos: Vec3,
    initial_riven_pos: Vec3,
    render_mode: RenderMode,
}

impl FioraV2Env {
    pub const DEFAULT_MAX_STEPS: usize = 160;

    /// 使用环境固有默认最大步数构造 Headless 训练实例。
    pub fn new() -> Self {
        Self::with_config(EnvConfig::default())
    }

    /// 使用显式最大步数构造 Headless 训练实例。
    pub fn new_with_max_steps(max_steps: usize) -> Self {
        Self::with_config(EnvConfig {
            max_steps,
            render_mode: RenderMode::Headless,
        })
    }

    pub fn max_steps(&self) -> usize {
        self.max_steps
    }

    pub fn with_config(config: EnvConfig) -> Self {
        let max_steps = if config.max_steps > 0 {
            config.max_steps
        } else {
            Self::DEFAULT_MAX_STEPS
        };
        let render = matches!(
            config.render_mode,
            RenderMode::Window | RenderMode::WindowCustomLoop
        );
        let mut app = App::new();
        app.insert_resource(TimeUpdateStrategy::FixedTimesteps(1));

        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
            .unwrap_or_else(|_| env!("CARGO_MANIFEST_DIR").to_string());
        let workspace_root = PathBuf::from(&manifest_dir)
            .parent()
            .map(|p| p.parent())
            .flatten()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from(&manifest_dir));

        let asset_plugin = bevy::asset::AssetPlugin {
            file_path: workspace_root.join("assets").to_string_lossy().to_string(),
            ..Default::default()
        };

        if render {
            if config.render_mode == RenderMode::WindowCustomLoop {
                app.add_plugins(
                    DefaultPlugins
                        .build()
                        .disable::<bevy::winit::WinitPlugin>()
                        .set(asset_plugin)
                        .set(WindowPlugin {
                            primary_window: Some(Window {
                                title: "Fiora vs Riven (V2 Full Skills 10f) - RL Visual Viewer"
                                    .to_string(),
                                resolution: (1280, 720).into(),
                                ..Default::default()
                            }),
                            ..Default::default()
                        }),
                );
            } else {
                app.add_plugins(DefaultPlugins.set(asset_plugin).set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Fiora vs Riven RL V2 Evaluation".to_string(),
                        resolution: (1280, 720).into(),
                        ..Default::default()
                    }),
                    ..Default::default()
                }));
            }
            app.add_plugins(lol_render::PluginRender);
            app.add_plugins(
                lol_core::PluginCore
                    .build()
                    .disable::<lol_core::PluginBarrack>(),
            );
            app.add_plugins(lol_particle::PluginParticle);
        } else {
            app.add_plugins((
                MinimalPlugins.set(ScheduleRunnerPlugin::run_once()),
                asset_plugin,
                bevy::world_serialization::WorldSerializationPlugin,
            ));
            app.add_plugins(
                lol_core::PluginCore
                    .build()
                    .disable::<lol_core::PluginBarrack>(),
            );
        }

        app.add_plugins(PluginFiora);
        app.add_plugins(PluginRiven);

        // 闪现冷却推进（手动实现，无游戏侧技能）
        app.add_systems(FixedUpdate, tick_flash_cooldown);

        app.insert_resource(lol_base::map::MapPaths::new("test"));
        app.insert_resource(NavigationDebug);

        app.finish();
        app.cleanup();

        if !render {
            let mut schedules = app.world_mut().resource_mut::<Schedules>();
            for (_, schedule) in schedules.iter_mut() {
                schedule.set_executor(SingleThreadedExecutor::new());
            }
        }

        let (fiora_config_handle, riven_config_handle, fiora_skin_handle, riven_skin_handle) = {
            let asset_server = app.world().resource::<AssetServer>();
            let fc = asset_server.load::<DynamicWorld>("characters/fiora/config.ron");
            let rc = asset_server.load::<DynamicWorld>("characters/Riven/config.ron");
            let fs = if render {
                Some(asset_server.load::<DynamicWorld>("characters/fiora/skins/skin0.ron"))
            } else {
                None
            };
            let rs = if render {
                Some(asset_server.load::<DynamicWorld>("characters/Riven/skins/skin0.ron"))
            } else {
                None
            };
            (fc, rc, fs, rs)
        };

        let initial_fiora_pos = Vec3::ZERO;
        let initial_riven_pos = Vec3::new(50.0, 0.0, 0.0);

        let (fiora, riven) = spawn_champions_world(
            app.world_mut(),
            fiora_config_handle.clone(),
            riven_config_handle.clone(),
            fiora_skin_handle.clone(),
            riven_skin_handle.clone(),
            initial_fiora_pos,
            initial_riven_pos,
            render,
        );

        app.world_mut()
            .insert_resource(FioraRivenEntities { fiora, riven });
        add_common_observers(&mut app);
        app.add_observer(on_v2_character_ready_setup_riven_health);

        // 等待 DynamicWorld 资产加载并完成向实体写入 (以 CharacterReady 为准)
        for _ in 0..500 {
            app.update();
            let world = app.world();
            let fiora_ready = world.get::<CharacterReady>(fiora).is_some()
                && (!render || world.get::<Skin>(fiora).is_some());
            let riven_ready = world.get::<CharacterReady>(riven).is_some()
                && (!render || world.get::<Skin>(riven).is_some());

            if fiora_ready && riven_ready {
                break;
            }
        }

        // 避免被加载后的血量覆盖，加载完成后把 riven 血量设置为 10000.0
        setup_v2_riven_health_world(app.world_mut(), riven);

        let mut env = Self {
            app,
            fiora,
            riven,
            fiora_config_handle,
            riven_config_handle,
            fiora_skin_handle,
            riven_skin_handle,
            step_count: 0,
            max_steps,
            initial_fiora_pos,
            initial_riven_pos,
            render_mode: config.render_mode,
        };

        env.setup_champion_skill_levels();
        env
    }

    pub fn meta() -> EnvMeta {
        EnvMeta {
            name: <Self as RlEnvironment>::env_name().to_string(),
            display_name: <Self as RlEnvironment>::display_name().to_string(),
            description: <Self as RlEnvironment>::description().to_string(),
            action_dim: <Self as RlEnvironment>::action_dim(),
            state_dim: <Self as RlEnvironment>::state_dim(),
            action_labels: <Self as RlEnvironment>::action_labels()
                .iter()
                .map(|s| s.to_string())
                .collect(),
        }
    }

    fn setup_champion_skill_levels(&mut self) {
        setup_skill_levels_world(self.app.world_mut(), self.fiora, self.riven);
        let world = self.app.world_mut();
        if let Some(mut flash) = world.get_mut::<FlashCooldown>(self.fiora) {
            flash.0 = None;
        } else {
            world.entity_mut(self.fiora).insert(FlashCooldown::default());
        }
    }

    pub fn reset(&mut self) -> FioraV2Obs {
        self.step_count = 0;
        let render = matches!(
            self.render_mode,
            RenderMode::Window | RenderMode::WindowCustomLoop
        );
        let (new_fiora, new_riven) = reset_episode_world(
            self.app.world_mut(),
            self.fiora,
            self.riven,
            &self.fiora_config_handle,
            &self.riven_config_handle,
            &self.fiora_skin_handle,
            &self.riven_skin_handle,
            self.initial_fiora_pos,
            self.initial_riven_pos,
            render,
        );
        self.fiora = new_fiora;
        self.riven = new_riven;
        for _ in 0..500 {
            self.app.update();
            let world = self.app.world();
            if world.get::<CharacterReady>(self.fiora).is_some()
                && world.get::<CharacterReady>(self.riven).is_some()
            {
                break;
            }
        }
        self.setup_champion_skill_levels();
        // 避免被加载后的血量覆盖，重置并完成世界更新后将 riven 血量设置为 10000.0
        setup_v2_riven_health_world(self.app.world_mut(), self.riven);
        self.get_obs()
    }

    pub fn get_obs(&self) -> FioraV2Obs {
        get_v2_obs_from_world(self.app.world(), self.fiora, self.riven)
    }

    pub fn step(&mut self, action: FioraV2Action) -> StepResult<FioraV2Obs> {
        self.step_count += 1;
        step_v2_world(
            &mut self.app,
            self.fiora,
            self.riven,
            action,
            self.step_count,
            self.max_steps,
        )
    }
}

// ── RlEnvironment 实现 ────────────────────────────────────────────────────────

impl RlEnvironment for FioraV2Env {
    type Action = FioraV2Action;
    type Obs = FioraV2Obs;

    fn env_name() -> &'static str {
        "FioraV2"
    }

    fn display_name() -> &'static str {
        "剑姬 vs 瑞雯 (全技能实战-10f)"
    }

    fn description() -> &'static str {
        "剑姬与瑞雯实战对决，支持 Q/E/R 技能、普攻、闪现（300 单位瞬移）与 NoOp 空动作，无辅助 10 帧物理推演"
    }

    fn action_space() -> ActionSpace {
        ActionSpace::Hybrid {
            continuous_dims: 2,
            discrete_classes: 7,
        }
    }

    fn default_max_steps() -> usize {
        Self::DEFAULT_MAX_STEPS
    }

    fn max_steps(&self) -> usize {
        self.max_steps
    }

    fn action_dim() -> usize {
        Self::action_space().actor_head_dim()
    }

    fn state_dim() -> usize {
        FioraV2Obs::dim()
    }

    fn reset(&mut self) -> Vec<Self::Obs> {
        vec![self.reset()]
    }

    fn step(&mut self, actions: &[Self::Action]) -> Vec<StepResult<Self::Obs>> {
        let action = actions
            .first()
            .copied()
            .unwrap_or(FioraV2Action::new(0.0, 0.0, FioraV2DiscreteAction::NoOp));
        vec![self.step(action)]
    }

    fn action_labels() -> &'static [&'static str] {
        &[
            "NoOp (保持/空动作)",
            "Move East (+100, 0)",
            "Move West (-100, 0)",
            "Move North (0, +100)",
            "Move South (0, -100)",
            "Chase Riven (追敌)",
            "Attack (普攻)",
            "CastQ East (Q突刺)",
            "CastE (E剑术)",
            "CastR (R无双挑战)",
            "Flash (闪现)",
        ]
    }

    fn obs_dim_labels() -> &'static [&'static str] {
        &[
            "被动破绽 +X 方向 (vital_dir_x)",
            "被动破绽 -X 方向 (vital_dir_neg_x)",
            "被动破绽 +Z 方向 (vital_dir_z)",
            "被动破绽 -Z 方向 (vital_dir_neg_z)",
            "存在被动破绽 (has_vital)",
            "被动破绽已激活 (vital_is_active)",
            "被动破绽激活剩余 (归一化 /1.7s)",
            "被动破绽移除剩余 (归一化 /4s)",
            "大招破绽 东 (r_vital_east)",
            "大招破绽 西 (r_vital_west)",
            "大招破绽 北 (r_vital_north)",
            "大招破绽 南 (r_vital_south)",
            "存在大招破绽 (has_r_vital)",
            "大招已激活 (r_is_active)",
            "大招激活剩余 (归一化 /0.5s)",
            "大招移除剩余 (归一化 /8s)",
            "英雄间距 (distance, 归一化)",
            "相对 X 偏移 (rel_x, 归一化)",
            "相对 Z 偏移 (rel_z, 归一化)",
            "普攻就绪 (attack_ready)",
            "普攻前摇中 (attack_windup)",
            "普攻后摇冷却中 (attack_cooldown)",
            "普攻计时剩余 (归一化 /1s)",
            "Q 就绪 (q_ready)",
            "Q 冷却剩余 (归一化 /10s)",
            "E 就绪 (e_ready)",
            "E 冷却剩余 (归一化 /10s)",
            "R 就绪 (r_ready)",
            "R 冷却剩余 (归一化 /60s)",
            "菲奥娜血量百分比",
            "瑞雯血量百分比",
            "闪现就绪 (flash_ready)",
            "闪现冷却剩余 (归一化 /300s)",
        ]
    }

    fn action_from_index(idx: usize) -> Self::Action {
        FioraV2Action::preset_from_index(idx)
    }

    fn action_to_index(action: Self::Action) -> usize {
        action.preset_index()
    }

    fn action_from_encoding(encoded: &[f32]) -> Self::Action {
        FioraV2Action::from_encoding(encoded)
    }

    fn action_to_encoding(action: Self::Action) -> Vec<f32> {
        action.to_encoding()
    }

    fn action_name(action: Self::Action) -> &'static str {
        action.desc()
    }

    fn new() -> Self {
        Self::new()
    }

    fn with_config(config: EnvConfig) -> Self {
        Self::with_config(config)
    }

    fn obs_to_vector(obs: &Self::Obs) -> Vec<f32> {
        obs.to_vector()
    }

    fn obs_to_payload(obs: &Self::Obs) -> Option<ObsFeaturePayload> {
        Some(obs.to_payload())
    }

    fn is_action_masked(obs: &Self::Obs, action_idx: usize) -> bool {
        match action_idx {
            6 => obs.distance > ATTACK_MASK_DISTANCE, // 普攻：超出攻击距离掩码
            7 => !obs.q_ready,                        // Q 突刺：冷却中掩码
            8 => !obs.e_ready,                        // E 剑术：冷却中掩码
            9 => !obs.r_ready,                        // R 大招：冷却中掩码
            10 => !obs.flash_ready,                   // 闪现：冷却中掩码
            _ => false,
        }
    }

    fn action_mask(obs: &Self::Obs) -> Option<Vec<bool>> {
        let mut mask = vec![true; 7];
        if obs.distance > ATTACK_MASK_DISTANCE {
            mask[2] = false; // 2: Attack
        }
        if !obs.q_ready {
            mask[3] = false; // 3: CastQ
        }
        if !obs.e_ready {
            mask[4] = false; // 4: CastE
        }
        if !obs.r_ready {
            mask[5] = false; // 5: CastR
        }
        if !obs.flash_ready {
            mask[6] = false; // 6: CastFlash
        }
        Some(mask)
    }

    fn reward_formula_spec() -> Option<RewardFormulaSpec> {
        Some(FioraV2RewardModel.formula_spec())
    }

    fn reward_formula(&self) -> Option<RewardFormulaSpec> {
        Self::reward_formula_spec()
    }
}

// ── VisualEnvironment 实现 ────────────────────────────────────────────────────

impl VisualEnvironment for FioraV2Env {
    fn take_app(&mut self) -> App {
        std::mem::replace(&mut self.app, App::new())
    }

    fn window_title(&self) -> &'static str {
        "Fiora vs Riven (V2 Full Skills)"
    }

    fn is_assets_loaded(&self, world: &World) -> bool {
        let fiora_ready = world.get::<CharacterReady>(self.fiora).is_some()
            && (self.fiora_skin_handle.is_none() || world.get::<Skin>(self.fiora).is_some());
        let riven_ready = world.get::<CharacterReady>(self.riven).is_some()
            && (self.riven_skin_handle.is_none() || world.get::<Skin>(self.riven).is_some());
        fiora_ready && riven_ready
    }

    fn on_assets_loaded(&mut self, world: &mut World) {
        setup_skill_levels_world(world, self.fiora, self.riven);
        setup_v2_riven_health_world(world, self.riven);
    }

    fn reset_world(&mut self, world: &mut World) -> Vec<Self::Obs> {
        self.step_count = 0;
        let render = matches!(
            self.render_mode,
            RenderMode::Window | RenderMode::WindowCustomLoop
        );
        let (new_fiora, new_riven) = reset_episode_world(
            world,
            self.fiora,
            self.riven,
            &self.fiora_config_handle,
            &self.riven_config_handle,
            &self.fiora_skin_handle,
            &self.riven_skin_handle,
            self.initial_fiora_pos,
            self.initial_riven_pos,
            render,
        );
        self.fiora = new_fiora;
        self.riven = new_riven;
        setup_skill_levels_world(world, self.fiora, self.riven);
        setup_v2_riven_health_world(world, self.riven);
        vec![self.get_current_obs(world)]
    }

    fn get_current_obs_all(&self, world: &World) -> Vec<Self::Obs> {
        vec![get_v2_obs_from_world(world, self.fiora, self.riven)]
    }

    fn action_from_screen_click(
        &mut self,
        world: &mut World,
        screen_pos: Vec2,
    ) -> Option<FioraV2Action> {
        let ray = {
            let mut q = world.query_filtered::<(&Camera, &GlobalTransform), With<CameraState>>();
            let Ok((cam, cam_tf)) = q.single(world) else {
                return None;
            };
            cam.viewport_to_world(cam_tf, screen_pos).ok()?
        };
        let rpos = world.get::<Transform>(self.riven)?.translation;
        let t = (rpos.y - ray.origin.y) / ray.direction.y;
        if !t.is_finite() || t <= 0.0 {
            return None;
        }
        let p = ray.origin + ray.direction * t;
        let dx = (p.x - rpos.x) / OFFSET_SCALE;
        let dz = (p.z - rpos.z) / OFFSET_SCALE;
        if Vec2::new(dx, dz).length() * OFFSET_SCALE < 60.0 {
            Some(FioraV2Action::new(0.0, 0.0, FioraV2DiscreteAction::Attack))
        } else {
            Some(FioraV2Action::new(
                dx.clamp(-1.0, 1.0),
                dz.clamp(-1.0, 1.0),
                FioraV2DiscreteAction::Move,
            ))
        }
    }

    fn step_world(
        &mut self,
        app: &mut App,
        actions: &[Self::Action],
    ) -> Vec<StepResult<Self::Obs>> {
        self.step_count += 1;
        let action = actions
            .first()
            .copied()
            .unwrap_or(FioraV2Action::new(0.0, 0.0, FioraV2DiscreteAction::NoOp));
        let res = step_v2_world(
            app,
            self.fiora,
            self.riven,
            action,
            self.step_count,
            self.max_steps,
        );
        vec![res]
    }
}

// ── 底层 ECS 交互与单步推演 ──────────────────────────────────────────────────

pub fn get_v2_obs_from_world(world: &World, fiora: Entity, riven: Entity) -> FioraV2Obs {
    let fpos = world
        .get::<Transform>(fiora)
        .map(|t| t.translation)
        .unwrap_or_default();
    let rpos = world
        .get::<Transform>(riven)
        .map(|t| t.translation)
        .unwrap_or_default();
    let dist = fpos.distance(rpos);

    let fhp = world.get::<Health>(fiora);
    let rhp = world.get::<Health>(riven);

    // 1. 被动破绽信息
    let vital = world.get::<Vital>(riven);
    let (has_vital, vital_is_active, vital_active_rem, vital_remove_rem, vx, vnx, vz, vnz) =
        match vital {
            Some(v) => {
                let (vx, vnx, vz, vnz) = match v.direction {
                    Direction::X => (1.0, 0.0, 0.0, 0.0),
                    Direction::NegX => (0.0, 1.0, 0.0, 0.0),
                    Direction::Z => (0.0, 0.0, 1.0, 0.0),
                    Direction::NegZ => (0.0, 0.0, 0.0, 1.0),
                };
                let active_rem = if v.is_active() {
                    0.0
                } else {
                    v.active_timer.remaining_secs()
                };
                let remove_rem = v.remove_timer.remaining_secs();
                (
                    true,
                    v.is_active(),
                    active_rem,
                    remove_rem,
                    vx,
                    vnx,
                    vz,
                    vnz,
                )
            }
            None => (false, false, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
        };

    // 2. 大招破绽 (BuffFioraR)
    let mut r_info = (false, false, 0.0, 0.0, false, false, false, false);
    if let Some(buffs) = world.get::<Buffs>(riven) {
        for buff_entity in buffs.iter() {
            if let Some(buff_r) = world.get::<BuffFioraR>(buff_entity) {
                let is_active = buff_r.is_active();
                let active_rem = if is_active {
                    0.0
                } else {
                    buff_r.active_timer.remaining_secs()
                };
                let remove_rem = buff_r.remove_timer.remaining_secs();
                let has_e = buff_r.vitals.contains(&Direction::X);
                let has_w = buff_r.vitals.contains(&Direction::NegX);
                let has_n = buff_r.vitals.contains(&Direction::Z);
                let has_s = buff_r.vitals.contains(&Direction::NegZ);
                r_info = (
                    true, is_active, active_rem, remove_rem, has_e, has_w, has_n, has_s,
                );
                break;
            }
        }
    }

    // 3. 普攻状态与计时器
    let now = world
        .get_resource::<Time<Fixed>>()
        .map(|t| t.elapsed_secs())
        .unwrap_or(0.0);
    let attack_state = world.get::<AttackState>(fiora);
    let attack_prop = world.get::<Attack>(fiora);
    let (
        atk_status_code,
        atk_is_windup,
        atk_is_cooldown,
        atk_rem_timer,
        atk_windup_dur,
        atk_total_dur,
    ) = {
        let windup_dur = attack_prop
            .map(|a| a.windup_duration_secs())
            .unwrap_or(0.25);
        let total_dur = attack_prop.map(|a| a.total_duration_secs()).unwrap_or(0.8);
        if let Some(state) = attack_state {
            match &state.status {
                AttackStatus::Windup { end_time, .. } => {
                    let rem = (*end_time - now).max(0.0);
                    (1u8, true, false, rem, windup_dur, total_dur)
                }
                AttackStatus::Cooldown { end_time } => {
                    let rem = (*end_time - now).max(0.0);
                    (2u8, false, true, rem, windup_dur, total_dur)
                }
            }
        } else {
            (0u8, false, false, 0.0, windup_dur, total_dur)
        }
    };

    // 4. 技能冷却与状态
    let mut q_info = (true, 0.0);
    let mut w_info = (true, 0.0);
    let mut e_info = (true, 0.0);
    let mut r_info_cd = (true, 0.0);
    if let Some(skills) = world.get::<Skills>(fiora) {
        let skill_entities = skills.to_vec();
        let get_cd_info = |idx: usize| -> (bool, f32) {
            if idx < skill_entities.len() {
                let s_entity = skill_entities[idx];
                let cd = world.get::<CoolDown>(s_entity);
                let recast = world.get::<SkillRecastWindow>(s_entity);
                let ready = match cd {
                    Some(c) => is_skill_ready(c, recast),
                    None => true,
                };
                let rem = cd
                    .and_then(|c| c.timer.as_ref())
                    .map(|t| {
                        if t.is_finished() {
                            0.0
                        } else {
                            t.remaining_secs()
                        }
                    })
                    .unwrap_or(0.0);
                (ready, rem)
            } else {
                (true, 0.0)
            }
        };
        q_info = get_cd_info(0);
        w_info = get_cd_info(1);
        e_info = get_cd_info(2);
        r_info_cd = get_cd_info(3);
    }

    // 5. E Buff 状态
    let mut buff_e_info = (false, 0);
    if let Some(buffs) = world.get::<Buffs>(fiora) {
        for buff_entity in buffs.iter() {
            if let Some(buff_e) = world.get::<BuffFioraE>(buff_entity) {
                buff_e_info = (true, buff_e.left);
                break;
            }
        }
    }

    // 6. 闪现状态
    let flash_info = world
        .get::<FlashCooldown>(fiora)
        .map(|f| (f.is_ready(), f.remaining_secs()))
        .unwrap_or((true, 0.0));

    FioraV2Obs {
        fiora_pos: fpos,
        fiora_hp: fhp.map(|h| h.value).unwrap_or(0.0),
        fiora_max_hp: fhp.map(|h| h.max).unwrap_or(500.0),
        riven_pos: rpos,
        riven_hp: rhp.map(|h| h.value).unwrap_or(0.0),
        riven_max_hp: rhp.map(|h| h.max).unwrap_or(500.0),
        distance: dist,
        has_vital,
        vital_is_active,
        vital_active_timer_remaining: vital_active_rem,
        vital_remove_timer_remaining: vital_remove_rem,
        vital_dir_x: vx,
        vital_dir_neg_x: vnx,
        vital_dir_z: vz,
        vital_dir_neg_z: vnz,
        has_r_vital: r_info.0,
        r_is_active: r_info.1,
        r_active_timer_remaining: r_info.2,
        r_remove_timer_remaining: r_info.3,
        r_vital_east: r_info.4,
        r_vital_west: r_info.5,
        r_vital_north: r_info.6,
        r_vital_south: r_info.7,
        attack_state: atk_status_code,
        attack_is_windup: atk_is_windup,
        attack_is_cooldown: atk_is_cooldown,
        attack_timer_remaining: atk_rem_timer,
        attack_windup_duration: atk_windup_dur,
        attack_total_duration: atk_total_dur,
        q_ready: q_info.0,
        q_cd_remaining: q_info.1,
        w_ready: w_info.0,
        w_cd_remaining: w_info.1,
        e_ready: e_info.0,
        e_cd_remaining: e_info.1,
        r_ready: r_info_cd.0,
        r_cd_remaining: r_info_cd.1,
        flash_ready: flash_info.0,
        flash_cd_remaining: flash_info.1,
        has_buff_e: buff_e_info.0,
        buff_e_left: buff_e_info.1,
    }
}

pub fn dispatch_action_world(
    world: &mut World,
    fiora: Entity,
    riven: Entity,
    action: FioraV2Action,
) {
    let rpos = world
        .get::<Transform>(riven)
        .map(|t| t.translation)
        .unwrap_or_default();
    let fpos = world
        .get::<Transform>(fiora)
        .map(|t| t.translation)
        .unwrap_or_default();

    let target_offset = Vec2::new(
        action.offset_x.clamp(-1.0, 1.0) * OFFSET_SCALE,
        action.offset_z.clamp(-1.0, 1.0) * OFFSET_SCALE,
    );
    let target_pos = Vec2::new(rpos.x + target_offset.x, rpos.z + target_offset.y);

    match action.discrete {
        FioraV2DiscreteAction::NoOp => {
            // 空动作：不做任何新派发，保持当前正在进行的行为（例如让普攻前摇或技能位移顺利推演）
        }
        FioraV2DiscreteAction::Move => {
            world.trigger(CommandAction {
                entity: fiora,
                action: Action::Move(target_pos),
            });
        }
        FioraV2DiscreteAction::Attack => {
            world.trigger(CommandAction {
                entity: fiora,
                action: Action::Attack(riven),
            });
        }
        FioraV2DiscreteAction::CastQ => {
            world.trigger(CommandAction {
                entity: fiora,
                action: Action::Skill {
                    index: 0,
                    point: target_pos,
                },
            });
        }
        FioraV2DiscreteAction::CastE => {
            world.trigger(CommandAction {
                entity: fiora,
                action: Action::Skill {
                    index: 2,
                    point: Vec2::new(fpos.x, fpos.z),
                },
            });
        }
        FioraV2DiscreteAction::CastR => {
            world.trigger(CommandAction {
                entity: fiora,
                action: Action::Skill {
                    index: 3,
                    point: Vec2::new(rpos.x, rpos.z),
                },
            });
        }
        FioraV2DiscreteAction::CastFlash => {
            // 闪现：沿 offset 连续值方向瞬移 300 单位（手动实现，无游戏侧技能）。
            // 冷却中或死亡时不瞬移。
            let ready = world
                .get::<FlashCooldown>(fiora)
                .map(|f| f.is_ready())
                .unwrap_or(true);
            let alive = world
                .get::<Health>(fiora)
                .map(|h| h.value > 0.0)
                .unwrap_or(true);
            if !ready || !alive {
                return;
            }
            let mut dir = Vec2::new(
                action.offset_x.clamp(-1.0, 1.0),
                action.offset_z.clamp(-1.0, 1.0),
            );
            if dir.length_squared() < 1e-4 {
                // offset 为零时回退为指向瑞雯的方向（与 Move 追敌语义一致）
                dir = Vec2::new(rpos.x - fpos.x, rpos.z - fpos.z);
            }
            let dir = dir.normalize_or_zero();
            if dir.length_squared() < 1e-4 {
                return;
            }
            let target = Vec2::new(fpos.x, fpos.z) + dir * FLASH_DISTANCE;
            if let Some(mut tf) = world.get_mut::<Transform>(fiora) {
                tf.translation.x = target.x;
                tf.translation.z = target.y;
            }
            if let Some(mut flash) = world.get_mut::<FlashCooldown>(fiora) {
                flash.start();
            }
        }
    }

    // 派发 riven 的随机移动策略（主要 0.5 概率往远离 fiora 的相反方向移动）
    dispatch_v2_riven_action(world, fiora, riven);
}

/// 瑞雯在 V2 中的 AI 行为：随机移动，主要（0.5 概率）向远离剑姬（Fiora）的相反方向移动
pub fn dispatch_v2_riven_action(world: &mut World, fiora: Entity, riven: Entity) {
    let riven_alive = world
        .get::<Health>(riven)
        .map(|h| h.value > 0.0)
        .unwrap_or(true);
    if !riven_alive {
        return;
    }

    let rpos = world
        .get::<Transform>(riven)
        .map(|t| t.translation)
        .unwrap_or_default();
    let fpos = world
        .get::<Transform>(fiora)
        .map(|t| t.translation)
        .unwrap_or_default();

    let rpos2 = Vec2::new(rpos.x, rpos.z);
    let fpos2 = Vec2::new(fpos.x, fpos.z);

    // 0.5 概率往远离剑姬的相反方向移动，其余 0.5 概率往随机方向移动
    let move_dir = if rand::random::<f32>() < 0.5 {
        let away = rpos2 - fpos2;
        if away.length_squared() > 1e-4 {
            away.normalize()
        } else {
            let angle = rand::random::<f32>() * std::f32::consts::TAU;
            Vec2::new(angle.cos(), angle.sin())
        }
    } else {
        let angle = rand::random::<f32>() * std::f32::consts::TAU;
        Vec2::new(angle.cos(), angle.sin())
    };

    let target_pos = rpos2 + move_dir * 300.0;
    world.trigger(CommandAction {
        entity: riven,
        action: Action::Move(target_pos),
    });
}

pub fn is_v2_aligned_with_vital(fpos: Vec3, rpos: Vec3, obs: &FioraV2Obs) -> bool {
    let delta_x = fpos.x - rpos.x;
    let delta_z = fpos.z - rpos.z;
    let abs_delta_x = delta_x.abs();
    let abs_delta_z = delta_z.abs();

    if obs.vital_dir_x > 0.5 {
        delta_x > 0.0 && abs_delta_x > abs_delta_z
    } else if obs.vital_dir_neg_x > 0.5 {
        delta_x < 0.0 && abs_delta_x > abs_delta_z
    } else if obs.vital_dir_z > 0.5 {
        delta_z > 0.0 && abs_delta_z > abs_delta_x
    } else if obs.vital_dir_neg_z > 0.5 {
        delta_z < 0.0 && abs_delta_z > abs_delta_x
    } else {
        false
    }
}

pub fn step_v2_world(
    app: &mut App,
    fiora: Entity,
    riven: Entity,
    action: FioraV2Action,
    step_count: usize,
    max_steps: usize,
) -> StepResult<FioraV2Obs> {
    let prev_obs = get_v2_obs_from_world(app.world(), fiora, riven);
    let prev_riven_hp = prev_obs.riven_hp;

    // 清理上一帧追踪器标记
    if let Some(mut tracker) = app.world_mut().get_resource_mut::<VitalBreakTracker>() {
        tracker.hit = false;
    }

    // 1. 派发动作指令
    dispatch_action_world(app.world_mut(), fiora, riven, action);

    // 2. 确保虚拟时间流动
    unpause_virtual_time(app.world_mut());

    // 3. 固定推演 10 帧（无任何辅助等待循环）
    for _ in 0..10 {
        app.update();
    }

    // 4. 获取推演后的最新观测
    let obs = get_v2_obs_from_world(app.world(), fiora, riven);
    let curr_riven_hp = obs.riven_hp;

    // 5. 真实破绽击破信号
    let is_vital_break = app
        .world()
        .get_resource::<VitalBreakTracker>()
        .map(|t| t.hit)
        .unwrap_or(false);

    // 6. 奖励计算
    let prev_aligned = prev_obs.has_vital
        && is_v2_aligned_with_vital(prev_obs.fiora_pos, prev_obs.riven_pos, &prev_obs);
    let curr_aligned =
        obs.has_vital && is_v2_aligned_with_vital(obs.fiora_pos, obs.riven_pos, &obs);

    let elapsed_secs = step_count as f32 * (10.0 / 60.0);

    let ctx = FioraV2RewardContext {
        prev_aligned,
        curr_aligned,
        is_vital_break,
        prev_riven_hp,
        curr_riven_hp,
        riven_max_hp: obs.riven_max_hp,
        elapsed_secs,
    };

    let (reward, reward_breakdown_items, reward_variables) = FioraV2RewardModel.evaluate(&ctx);
    let reward_breakdown = reward_breakdown_items
        .into_iter()
        .map(|item| RewardBreakdownItem {
            name: item.name,
            value: item.value,
        })
        .collect();

    let terminated = curr_riven_hp <= 0.0 || obs.fiora_hp <= 0.0;
    let truncated = max_steps > 0 && step_count >= max_steps;

    StepResult {
        obs,
        reward,
        terminated,
        truncated,
        step: step_count,
        reward_breakdown,
        reward_variables,
    }
}
