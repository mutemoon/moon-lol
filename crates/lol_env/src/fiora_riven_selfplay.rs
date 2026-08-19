use std::collections::HashMap;
use std::path::PathBuf;

use bevy::app::ScheduleRunnerPlugin;
use bevy::ecs::schedule::SingleThreadedExecutor;
use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use bevy::world_serialization::DynamicWorld;
use lol_base::character::Skin;
use lol_base_render::camera::CameraState;
use lol_champions::fiora::PluginFiora;
use lol_champions::fiora::passive::Vital;
use lol_champions::fiora::r::BuffFioraR;
use lol_champions::riven::PluginRiven;
use lol_core::action::{Action, CommandAction};
use lol_core::attack::{AttackState, AttackStatus};
use lol_core::base::buff::Buffs;
use lol_core::base::direction::Direction;
use lol_core::character::CharacterReady;
use lol_core::life::Health;
use lol_core::navigation::navigation::NavigationDebug;
use lol_core::skill::{CoolDown, SkillRecastWindow, Skills, is_skill_ready};
use lol_rl_protocol::{ActionSpace, ObsFeaturePayload, RewardFormulaSpec, RewardTermSpec};

use crate::fiora_riven_common::{
    ATTACK_MASK_DISTANCE, FioraRivenEntities, VitalBreakTracker, add_common_observers,
    reset_episode_world, setup_skill_levels_world, spawn_champions_world, unpause_virtual_time,
};
pub use crate::traits::{
    EnvConfig, EnvMeta, RenderMode, RewardBreakdownItem, RlEnvironment, StepResult,
    VisualEnvironment,
};

/// 连续位移与技能偏移缩放：offset_x/offset_z ∈ [-1,1] 映射到相对对手的 ±OFFSET_SCALE 偏移。
pub const SELFPLAY_OFFSET_SCALE: f32 = 100.0;

/// 闪现瞬移距离（单位）：沿 offset 连续值方向瞬移 300 单位。
pub const SELFPLAY_FLASH_DISTANCE: f32 = 300.0;

/// 闪现冷却时长（秒），与英雄联盟召唤师技能一致。
pub const SELFPLAY_FLASH_COOLDOWN_SECS: f32 = 300.0;

/// 观测向量维度：自我中心化 36 维。
pub const SELFPLAY_OBS_DIM: usize = 36;
pub const SELFPLAY_OBS_DISTANCE_SCALE: f32 = 100.0;

/// 自博弈双方标准初始血量（对称竞技血量）
pub const SELFPLAY_CHAMPION_HP: f32 = 1000.0;

/// 初始化/重置双方生命值与闪现状态
pub fn setup_selfplay_health_world(world: &mut World, fiora: Entity, riven: Entity) {
    for entity in [fiora, riven] {
        if let Some(mut health) = world.get_mut::<Health>(entity) {
            health.value = SELFPLAY_CHAMPION_HP;
            health.max = SELFPLAY_CHAMPION_HP;
        } else {
            world
                .entity_mut(entity)
                .insert(Health::new(SELFPLAY_CHAMPION_HP));
        }

        if let Some(mut flash) = world.get_mut::<SelfPlayFlashCooldown>(entity) {
            flash.0 = None;
        } else {
            world
                .entity_mut(entity)
                .insert(SelfPlayFlashCooldown::default());
        }
    }
}

// ── 离散动作与混合动作定义 ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SelfPlayDiscreteAction {
    /// 0: 不做任何新动作（保持当前执行状态，关键用于不打断普攻前摇/后摇/技能位移）
    NoOp = 0,
    /// 1: 移动到 target_pos + [offset_x, offset_z] * OFFSET_SCALE
    Move = 1,
    /// 2: 普通攻击对手
    Attack = 2,
    /// 3: Q 技能（剑姬：Q 突刺；瑞雯：折翼之舞 Q1/Q2/Q3 突进斩击）
    CastQ = 3,
    /// 4: W 技能（剑姬：W 心眼刀；瑞雯：W 镇魂怒吼）
    CastW = 4,
    /// 5: E 技能（剑姬：E 重置普攻；瑞雯：E 勇往直前）
    CastE = 5,
    /// 6: R 技能（剑姬：R 挂 4 破绽；瑞雯：R 放逐之锋/疾风斩）
    CastR = 6,
    /// 7: 闪现：沿 offset 方向瞬移 300 单位
    CastFlash = 7,
}

impl SelfPlayDiscreteAction {
    pub fn from_u8(val: u8) -> Self {
        match val {
            0 => Self::NoOp,
            1 => Self::Move,
            2 => Self::Attack,
            3 => Self::CastQ,
            4 => Self::CastW,
            5 => Self::CastE,
            6 => Self::CastR,
            7 => Self::CastFlash,
            _ => Self::NoOp,
        }
    }

    pub fn to_u8(self) -> u8 {
        self as u8
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SelfPlayAction {
    /// 连续偏移 X 分量 ∈ [-1, 1]，相对对手的 X 偏移（Move / CastQ / CastFlash 共用）
    pub offset_x: f32,
    /// 连续偏移 Z 分量 ∈ [-1, 1]，相对对手的 Z 偏移（Move / CastQ / CastFlash 共用）
    pub offset_z: f32,
    /// 离散动作类别 (0: NoOp, 1: Move, 2: Attack, 3: CastQ, 4: CastW, 5: CastE, 6: CastR, 7: CastFlash)
    pub discrete: SelfPlayDiscreteAction,
}

impl SelfPlayAction {
    pub const fn new(offset_x: f32, offset_z: f32, discrete: SelfPlayDiscreteAction) -> Self {
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
            SelfPlayDiscreteAction::from_u8(discrete_idx),
        )
    }

    pub fn to_encoding(&self) -> Vec<f32> {
        vec![self.offset_x, self.offset_z, self.discrete.to_u8() as f32]
    }

    pub fn preset_from_index(index: usize) -> Self {
        match index {
            0 => Self::new(0.0, 0.0, SelfPlayDiscreteAction::NoOp),
            1 => Self::new(1.0, 0.0, SelfPlayDiscreteAction::Move),
            2 => Self::new(-1.0, 0.0, SelfPlayDiscreteAction::Move),
            3 => Self::new(0.0, 1.0, SelfPlayDiscreteAction::Move),
            4 => Self::new(0.0, -1.0, SelfPlayDiscreteAction::Move),
            5 => Self::new(0.0, 0.0, SelfPlayDiscreteAction::Move),
            6 => Self::new(0.0, 0.0, SelfPlayDiscreteAction::Attack),
            7 => Self::new(1.0, 0.0, SelfPlayDiscreteAction::CastQ),
            8 => Self::new(0.0, 0.0, SelfPlayDiscreteAction::CastW),
            9 => Self::new(0.0, 0.0, SelfPlayDiscreteAction::CastE),
            10 => Self::new(0.0, 0.0, SelfPlayDiscreteAction::CastR),
            11 => Self::new(1.0, 0.0, SelfPlayDiscreteAction::CastFlash),
            _ => Self::new(0.0, 0.0, SelfPlayDiscreteAction::NoOp),
        }
    }

    pub fn preset_index(&self) -> usize {
        match self.discrete {
            SelfPlayDiscreteAction::NoOp => 0,
            SelfPlayDiscreteAction::Move => {
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
            SelfPlayDiscreteAction::Attack => 6,
            SelfPlayDiscreteAction::CastQ => 7,
            SelfPlayDiscreteAction::CastW => 8,
            SelfPlayDiscreteAction::CastE => 9,
            SelfPlayDiscreteAction::CastR => 10,
            SelfPlayDiscreteAction::CastFlash => 11,
        }
    }

    pub fn desc(&self) -> &'static str {
        match self.discrete {
            SelfPlayDiscreteAction::NoOp => "NoOp (保持/空动作)",
            SelfPlayDiscreteAction::Move => "Move (移动)",
            SelfPlayDiscreteAction::Attack => "Attack (普攻)",
            SelfPlayDiscreteAction::CastQ => "CastQ (Q技能)",
            SelfPlayDiscreteAction::CastW => "CastW (W技能)",
            SelfPlayDiscreteAction::CastE => "CastE (E技能)",
            SelfPlayDiscreteAction::CastR => "CastR (R技能)",
            SelfPlayDiscreteAction::CastFlash => "Flash (闪现)",
        }
    }
}

// ── 闪现组件 ─────────────────────────────────────────────────────────────────

#[derive(Component, Default)]
pub struct SelfPlayFlashCooldown(pub Option<Timer>);

impl SelfPlayFlashCooldown {
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

    pub fn start(&mut self) {
        self.0 = Some(Timer::from_seconds(
            SELFPLAY_FLASH_COOLDOWN_SECS,
            TimerMode::Once,
        ));
    }
}

pub fn tick_selfplay_flash_cooldown(
    time: Res<Time<Fixed>>,
    mut q: Query<&mut SelfPlayFlashCooldown>,
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

// ── 自我中心化观测数据结构 ─────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct SelfPlayObs {
    /// 角色标识：0.0 代表剑姬 (Fiora), 1.0 代表瑞雯 (Riven)
    pub role_id: f32,

    pub self_pos: Vec3,
    pub self_hp: f32,
    pub self_max_hp: f32,
    pub target_pos: Vec3,
    pub target_hp: f32,
    pub target_max_hp: f32,
    pub distance: f32,

    // 破绽方位与状态（以被挂载者为中心）
    pub has_vital: bool,
    pub vital_is_active: bool,
    pub vital_active_timer_remaining: f32,
    pub vital_remove_timer_remaining: f32,
    pub vital_dir_x: f32,
    pub vital_dir_neg_x: f32,
    pub vital_dir_z: f32,
    pub vital_dir_neg_z: f32,

    // 大招破绽
    pub has_r_vital: bool,
    pub r_is_active: bool,
    pub r_active_timer_remaining: f32,
    pub r_remove_timer_remaining: f32,
    pub r_vital_east: bool,
    pub r_vital_west: bool,
    pub r_vital_north: bool,
    pub r_vital_south: bool,

    // 自身普攻状态
    pub attack_state: u8,
    pub attack_is_windup: bool,
    pub attack_is_cooldown: bool,
    pub attack_timer_remaining: f32,

    // 自身技能冷却状态
    pub q_ready: bool,
    pub q_cd_remaining: f32,
    pub w_ready: bool,
    pub w_cd_remaining: f32,
    pub e_ready: bool,
    pub e_cd_remaining: f32,
    pub r_ready: bool,
    pub r_cd_remaining: f32,

    // 自身闪现状态
    pub flash_ready: bool,
    pub flash_cd_remaining: f32,
}

impl SelfPlayObs {
    /// 转换为 36 维自我中心化特征向量
    pub fn to_vector(&self) -> Vec<f32> {
        let rel_x = self.target_pos.x - self.self_pos.x;
        let rel_z = self.target_pos.z - self.self_pos.z;

        vec![
            // 0: 角色标识 (0.0: Fiora, 1.0: Riven)
            self.role_id,
            // 1..5: 被动破绽四方位 (4维)
            self.vital_dir_x,
            self.vital_dir_neg_x,
            self.vital_dir_z,
            self.vital_dir_neg_z,
            // 5..7: 被动破绽状态 (2维)
            if self.has_vital { 1.0 } else { 0.0 },
            if self.vital_is_active { 1.0 } else { 0.0 },
            // 7..9: 被动破绽倒计时 (2维)
            self.vital_active_timer_remaining / 1.7,
            self.vital_remove_timer_remaining / 4.0,
            // 9..13: 大招破绽四方位 (4维)
            if self.r_vital_east { 1.0 } else { 0.0 },
            if self.r_vital_west { 1.0 } else { 0.0 },
            if self.r_vital_north { 1.0 } else { 0.0 },
            if self.r_vital_south { 1.0 } else { 0.0 },
            // 13..15: 大招破绽状态 (2维)
            if self.has_r_vital { 1.0 } else { 0.0 },
            if self.r_is_active { 1.0 } else { 0.0 },
            // 15..17: 大招破绽倒计时 (2维)
            self.r_active_timer_remaining / 0.5,
            self.r_remove_timer_remaining / 8.0,
            // 17..20: 自我中心空间几何：距离与相对向量 (3维)
            self.distance / SELFPLAY_OBS_DISTANCE_SCALE,
            rel_x / SELFPLAY_OFFSET_SCALE,
            rel_z / SELFPLAY_OFFSET_SCALE,
            // 20..24: 自身普攻状态与计时器 (4维)
            if self.attack_state == 0 { 1.0 } else { 0.0 },
            if self.attack_is_windup { 1.0 } else { 0.0 },
            if self.attack_is_cooldown { 1.0 } else { 0.0 },
            self.attack_timer_remaining / 1.0,
            // 24..32: 自身技能就绪与冷却 (8维)
            if self.q_ready { 1.0 } else { 0.0 },
            self.q_cd_remaining / 10.0,
            if self.w_ready { 1.0 } else { 0.0 },
            self.w_cd_remaining / 10.0,
            if self.e_ready { 1.0 } else { 0.0 },
            self.e_cd_remaining / 10.0,
            if self.r_ready { 1.0 } else { 0.0 },
            self.r_cd_remaining / 60.0,
            // 32..34: 自身与对手血量百分比 (2维)
            if self.self_max_hp > 0.0 {
                self.self_hp / self.self_max_hp
            } else {
                0.0
            },
            if self.target_max_hp > 0.0 {
                self.target_hp / self.target_max_hp
            } else {
                0.0
            },
            // 34..36: 自身闪现就绪与冷却 (2维)
            if self.flash_ready { 1.0 } else { 0.0 },
            self.flash_cd_remaining / SELFPLAY_FLASH_COOLDOWN_SECS,
        ]
    }

    pub fn dim() -> usize {
        SELFPLAY_OBS_DIM
    }

    pub fn to_payload(&self) -> ObsFeaturePayload {
        let is_fiora = self.role_id < 0.5;
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

        let s_hp = if self.self_max_hp > 0.0 {
            self.self_hp / self.self_max_hp
        } else {
            0.0
        };
        let t_hp = if self.target_max_hp > 0.0 {
            self.target_hp / self.target_max_hp
        } else {
            0.0
        };

        let (f_hp, r_hp) = if is_fiora { (s_hp, t_hp) } else { (t_hp, s_hp) };

        ObsFeaturePayload {
            self_hp_pct: s_hp,
            target_hp_pct: t_hp,
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

// ── 自博弈环境实体 ─────────────────────────────────────────────────────────────

pub struct FioraRivenSelfPlayEnv {
    app: App,
    fiora: Entity,
    riven: Entity,
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

impl FioraRivenSelfPlayEnv {
    pub const DEFAULT_MAX_STEPS: usize = 160;

    pub fn new() -> Self {
        Self::with_config(EnvConfig::default())
    }

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
            .and_then(|p| p.parent())
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
                                title: "Fiora vs Riven (Self-Play RL Viewer)".to_string(),
                                resolution: (1280, 720).into(),
                                ..Default::default()
                            }),
                            ..Default::default()
                        }),
                );
            } else {
                app.add_plugins(DefaultPlugins.set(asset_plugin).set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Fiora vs Riven Self-Play RL Evaluation".to_string(),
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
        app.add_systems(FixedUpdate, tick_selfplay_flash_cooldown);

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

        let initial_fiora_pos = Vec3::new(0.0, 0.0, 0.0);
        let initial_riven_pos = Vec3::new(0.0, 0.0, 300.0);

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
        app.init_resource::<VitalBreakTracker>();
        add_common_observers(&mut app);

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

        env.reset_wait_for_ready();
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

    fn reset_wait_for_ready(&mut self) {
        loop {
            self.app.update();
            let world = self.app.world();
            if world.get::<CharacterReady>(self.fiora).is_some()
                && world.get::<CharacterReady>(self.riven).is_some()
            {
                break;
            }
        }
        setup_skill_levels_world(self.app.world_mut(), self.fiora, self.riven);
        setup_selfplay_health_world(self.app.world_mut(), self.fiora, self.riven);
    }

    pub fn get_fiora_obs(&self) -> SelfPlayObs {
        get_ego_obs_from_world(self.app.world(), self.fiora, self.riven, 0.0)
    }

    pub fn get_riven_obs(&self) -> SelfPlayObs {
        get_ego_obs_from_world(self.app.world(), self.riven, self.fiora, 1.0)
    }

    /// 双智能体同时步进（自博弈标准调用接口）
    pub fn step_both(
        &mut self,
        act_fiora: SelfPlayAction,
        act_riven: SelfPlayAction,
    ) -> (StepResult<SelfPlayObs>, StepResult<SelfPlayObs>) {
        self.step_count += 1;
        step_selfplay_world(
            &mut self.app,
            self.fiora,
            self.riven,
            act_fiora,
            act_riven,
            self.step_count,
            self.max_steps,
        )
    }

    pub fn reset(&mut self) -> Vec<SelfPlayObs> {
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
        self.reset_wait_for_ready();
        vec![self.get_fiora_obs(), self.get_riven_obs()]
    }

    pub fn step(&mut self, actions: &[SelfPlayAction]) -> Vec<StepResult<SelfPlayObs>> {
        let act_fiora = actions.get(0).copied().unwrap_or(SelfPlayAction::new(
            0.0,
            0.0,
            SelfPlayDiscreteAction::NoOp,
        ));
        let act_riven = actions.get(1).copied().unwrap_or(SelfPlayAction::new(
            0.0,
            0.0,
            SelfPlayDiscreteAction::NoOp,
        ));
        let (res_fiora, res_riven) = self.step_both(act_fiora, act_riven);
        vec![res_fiora, res_riven]
    }
}

// ── RlEnvironment 实现 ────────────────────────────────────────────────────────

impl RlEnvironment for FioraRivenSelfPlayEnv {
    type Action = SelfPlayAction;
    type Obs = SelfPlayObs;

    fn num_agents() -> usize {
        2
    }

    fn agent_names() -> &'static [&'static str] {
        &["Fiora", "Riven"]
    }

    fn env_name() -> &'static str {
        "FioraRivenSelfPlay"
    }

    fn display_name() -> &'static str {
        "剑姬 vs 瑞雯 (双Agent自博弈)"
    }

    fn description() -> &'static str {
        "单神经网络自博弈控制双方，自我中心化 36 维观测与混合动作空间，10 帧物理对抗与零和博弈"
    }

    fn action_space() -> ActionSpace {
        ActionSpace::Hybrid {
            continuous_dims: 2,
            discrete_classes: 8,
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
        SELFPLAY_OBS_DIM
    }

    fn action_labels() -> &'static [&'static str] {
        &[
            "NoOp (保持/空动作)",
            "Move (相对寻路)",
            "Attack (普攻)",
            "CastQ (Q技能)",
            "CastW (W技能)",
            "CastE (E技能)",
            "CastR (R技能)",
            "Flash (闪现)",
        ]
    }

    fn obs_dim_labels() -> &'static [&'static str] {
        &[
            "角色标识 (role_id: 0=剑姬, 1=瑞雯)",
            "破绽 +X 方向 (vital_dir_x)",
            "破绽 -X 方向 (vital_dir_neg_x)",
            "破绽 +Z 方向 (vital_dir_z)",
            "破绽 -Z 方向 (vital_dir_neg_z)",
            "存在被动破绽 (has_vital)",
            "被动破绽已激活 (vital_is_active)",
            "被动破绽激活剩余倒计时 (/1.7s)",
            "被动破绽移除剩余倒计时 (/4.0s)",
            "大招破绽 东 (r_vital_east)",
            "大招破绽 西 (r_vital_west)",
            "大招破绽 北 (r_vital_north)",
            "大招破绽 南 (r_vital_south)",
            "存在大招破绽 (has_r_vital)",
            "大招破绽已激活 (r_is_active)",
            "大招破绽激活剩余倒计时 (/0.5s)",
            "大招破绽移除剩余倒计时 (/8.0s)",
            "欧氏距离 (/100)",
            "自我中心相对 X 偏移 (rel_x /100)",
            "自我中心相对 Z 偏移 (rel_z /100)",
            "自身普攻就绪 (attack_ready)",
            "自身普攻前摇中 (attack_windup)",
            "自身普攻后摇中 (attack_cooldown)",
            "自身普攻计时器剩余 (/1.0s)",
            "自身 Q 技能就绪 (q_ready)",
            "自身 Q 技能 CD 剩余 (/10s)",
            "自身 W 技能就绪 (w_ready)",
            "自身 W 技能 CD 剩余 (/10s)",
            "自身 E 技能就绪 (e_ready)",
            "自身 E 技能 CD 剩余 (/10s)",
            "自身 R 技能就绪 (r_ready)",
            "自身 R 技能 CD 剩余 (/60s)",
            "自身血量百分比 (self_hp / max_hp)",
            "对手血量百分比 (target_hp / max_hp)",
            "自身闪现就绪 (flash_ready)",
            "自身闪现 CD 剩余 (/300s)",
        ]
    }

    fn action_to_index(action: Self::Action) -> usize {
        action.preset_index()
    }

    fn action_from_index(idx: usize) -> Self::Action {
        SelfPlayAction::preset_from_index(idx)
    }

    fn action_from_encoding(encoded: &[f32]) -> Self::Action {
        SelfPlayAction::from_encoding(encoded)
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

    fn reset(&mut self) -> Vec<Self::Obs> {
        self.reset();
        vec![self.get_fiora_obs(), self.get_riven_obs()]
    }

    fn step(&mut self, actions: &[Self::Action]) -> Vec<StepResult<Self::Obs>> {
        let act_fiora = actions.get(0).copied().unwrap_or(SelfPlayAction::new(
            0.0,
            0.0,
            SelfPlayDiscreteAction::NoOp,
        ));
        let act_riven = actions.get(1).copied().unwrap_or(SelfPlayAction::new(
            0.0,
            0.0,
            SelfPlayDiscreteAction::NoOp,
        ));
        let (res_fiora, res_riven) = self.step_both(act_fiora, act_riven);
        vec![res_fiora, res_riven]
    }

    fn obs_to_vector(obs: &Self::Obs) -> Vec<f32> {
        obs.to_vector()
    }

    fn obs_to_payload(obs: &Self::Obs) -> Option<ObsFeaturePayload> {
        Some(obs.to_payload())
    }

    fn is_action_masked(obs: &Self::Obs, action_idx: usize) -> bool {
        match action_idx {
            6 => obs.distance > ATTACK_MASK_DISTANCE,
            7 => !obs.q_ready,
            8 => !obs.w_ready,
            9 => !obs.e_ready,
            10 => !obs.r_ready,
            11 => !obs.flash_ready,
            _ => false,
        }
    }

    fn action_mask(obs: &Self::Obs) -> Option<Vec<bool>> {
        let is_windup = obs.attack_is_windup;
        let dist_ok = obs.distance <= ATTACK_MASK_DISTANCE;

        Some(vec![
            true,                      // 0: NoOp
            true,                      // 1: Move
            dist_ok && !is_windup,     // 2: Attack
            obs.q_ready && !is_windup, // 3: CastQ
            obs.w_ready && !is_windup, // 4: CastW
            obs.e_ready && !is_windup, // 5: CastE
            obs.r_ready && !is_windup, // 6: CastR
            obs.flash_ready,           // 7: CastFlash
        ])
    }

    fn reward_formula_spec() -> Option<RewardFormulaSpec> {
        Some(RewardFormulaSpec {
            name: "Fiora vs Riven Self-Play Zero-Sum Reward".to_string(),
            terms: vec![
                RewardTermSpec::new(
                    "damage_dealt",
                    "造成伤害收益",
                    lol_rl_protocol::RewardExpr::Variable("self_damage_dealt_reward".to_string()),
                ),
                RewardTermSpec::new(
                    "damage_taken",
                    "承受伤害惩罚",
                    lol_rl_protocol::RewardExpr::Variable("self_damage_taken_penalty".to_string()),
                ),
                RewardTermSpec::new(
                    "vital_break",
                    "破绽攻防转移",
                    lol_rl_protocol::RewardExpr::Variable("vital_reward".to_string()),
                ),
                RewardTermSpec::new(
                    "kill_win",
                    "击杀胜负判定",
                    lol_rl_protocol::RewardExpr::Variable("kill_reward".to_string()),
                ),
            ],
        })
    }

    fn reward_formula(&self) -> Option<RewardFormulaSpec> {
        Self::reward_formula_spec()
    }
}

// ── VisualEnvironment 实现 ────────────────────────────────────────────────────

impl VisualEnvironment for FioraRivenSelfPlayEnv {
    fn take_app(&mut self) -> App {
        std::mem::replace(&mut self.app, App::new())
    }

    fn window_title(&self) -> &'static str {
        "Fiora vs Riven (Self-Play RL Viewer)"
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
        setup_selfplay_health_world(world, self.fiora, self.riven);
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
        setup_selfplay_health_world(world, self.fiora, self.riven);
        vec![
            get_ego_obs_from_world(world, self.fiora, self.riven, 0.0),
            get_ego_obs_from_world(world, self.riven, self.fiora, 1.0),
        ]
    }

    fn get_current_obs_all(&self, world: &World) -> Vec<Self::Obs> {
        vec![
            get_ego_obs_from_world(world, self.fiora, self.riven, 0.0),
            get_ego_obs_from_world(world, self.riven, self.fiora, 1.0),
        ]
    }

    fn action_from_screen_click(
        &mut self,
        world: &mut World,
        screen_pos: Vec2,
    ) -> Option<SelfPlayAction> {
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
        let dx = (p.x - rpos.x) / SELFPLAY_OFFSET_SCALE;
        let dz = (p.z - rpos.z) / SELFPLAY_OFFSET_SCALE;
        if Vec2::new(dx, dz).length() * SELFPLAY_OFFSET_SCALE < 60.0 {
            Some(SelfPlayAction::new(
                0.0,
                0.0,
                SelfPlayDiscreteAction::Attack,
            ))
        } else {
            Some(SelfPlayAction::new(
                dx.clamp(-1.0, 1.0),
                dz.clamp(-1.0, 1.0),
                SelfPlayDiscreteAction::Move,
            ))
        }
    }

    fn step_world(
        &mut self,
        app: &mut App,
        actions: &[Self::Action],
    ) -> Vec<StepResult<Self::Obs>> {
        self.step_count += 1;
        let act_fiora = actions.get(0).copied().unwrap_or(SelfPlayAction::new(
            0.0,
            0.0,
            SelfPlayDiscreteAction::NoOp,
        ));
        let act_riven = actions.get(1).copied().unwrap_or(SelfPlayAction::new(
            0.0,
            0.0,
            SelfPlayDiscreteAction::NoOp,
        ));
        let (res_fiora, res_riven) = step_selfplay_world(
            app,
            self.fiora,
            self.riven,
            act_fiora,
            act_riven,
            self.step_count,
            self.max_steps,
        );
        vec![res_fiora, res_riven]
    }
}

/// 瑞雯自博弈默认战斗对抗策略（在单动作接口推演中自动执行智能攻防）
pub fn get_default_riven_combat_action(
    world: &World,
    riven: Entity,
    fiora: Entity,
) -> SelfPlayAction {
    let obs = get_ego_obs_from_world(world, riven, fiora, 1.0);
    if obs.attack_is_windup {
        return SelfPlayAction::new(0.0, 0.0, SelfPlayDiscreteAction::NoOp);
    }

    // 破绽防守偏移（当破绽朝向存在时，微调走位方向）
    let offset_x = if obs.vital_dir_x > 0.5 {
        -0.5
    } else if obs.vital_dir_neg_x > 0.5 {
        0.5
    } else {
        0.0
    };
    let offset_z = if obs.vital_dir_z > 0.5 {
        -0.5
    } else if obs.vital_dir_neg_z > 0.5 {
        0.5
    } else {
        0.0
    };

    if obs.distance <= ATTACK_MASK_DISTANCE {
        // 近战范围：连招对抗（W控制 > Q斩击 > 普攻 > E护盾拉扯 > R强化）
        if obs.w_ready {
            SelfPlayAction::new(offset_x, offset_z, SelfPlayDiscreteAction::CastW)
        } else if obs.q_ready {
            SelfPlayAction::new(offset_x, offset_z, SelfPlayDiscreteAction::CastQ)
        } else if obs.attack_state == 0 {
            SelfPlayAction::new(0.0, 0.0, SelfPlayDiscreteAction::Attack)
        } else if obs.e_ready {
            SelfPlayAction::new(offset_x, offset_z, SelfPlayDiscreteAction::CastE)
        } else if obs.r_ready {
            SelfPlayAction::new(0.0, 0.0, SelfPlayDiscreteAction::CastR)
        } else {
            SelfPlayAction::new(offset_x, offset_z, SelfPlayDiscreteAction::Move)
        }
    } else {
        // 远距离：突进切入或追逐（E突进 > Q突进 > Move追赶）
        if obs.e_ready {
            SelfPlayAction::new(0.0, 0.0, SelfPlayDiscreteAction::CastE)
        } else if obs.q_ready {
            SelfPlayAction::new(0.0, 0.0, SelfPlayDiscreteAction::CastQ)
        } else {
            SelfPlayAction::new(0.0, 0.0, SelfPlayDiscreteAction::Move)
        }
    }
}

// ── 底层 ECS 交互与自博弈单步推演 ─────────────────────────────────────────────

pub fn get_ego_obs_from_world(
    world: &World,
    self_entity: Entity,
    target_entity: Entity,
    role_id: f32,
) -> SelfPlayObs {
    let spos = world
        .get::<Transform>(self_entity)
        .map(|t| t.translation)
        .unwrap_or_default();
    let tpos = world
        .get::<Transform>(target_entity)
        .map(|t| t.translation)
        .unwrap_or_default();
    let dist = spos.distance(tpos);

    let shp = world.get::<Health>(self_entity);
    let thp = world.get::<Health>(target_entity);

    // 破绽信息（破绽始终挂在承伤者 / 瑞雯身上，或全局破绽）
    // 如果 self 是 Fiora (role_id < 0.5)，target 是 Riven，破绽在 target 上；
    // 如果 self 是 Riven (role_id >= 0.5)，破绽在 self 上。
    let riven_entity = if role_id < 0.5 {
        target_entity
    } else {
        self_entity
    };

    let vital = world.get::<Vital>(riven_entity);
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

    // 大招破绽 (BuffFioraR)
    let mut r_info = (false, false, 0.0, 0.0, false, false, false, false);
    if let Some(buffs) = world.get::<Buffs>(riven_entity) {
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

    // 普攻状态
    let now = world
        .get_resource::<Time<Fixed>>()
        .map(|t| t.elapsed_secs())
        .unwrap_or(0.0);
    let attack_state = world.get::<AttackState>(self_entity);
    let (atk_status_code, atk_is_windup, atk_is_cooldown, atk_rem_timer) = {
        if let Some(state) = attack_state {
            match &state.status {
                AttackStatus::Windup { end_time, .. } => {
                    let rem = (*end_time - now).max(0.0);
                    (1u8, true, false, rem)
                }
                AttackStatus::Cooldown { end_time } => {
                    let rem = (*end_time - now).max(0.0);
                    (2u8, false, true, rem)
                }
            }
        } else {
            (0u8, false, false, 0.0)
        }
    };

    // 技能冷却状态
    let mut q_info = (true, 0.0);
    let mut w_info = (true, 0.0);
    let mut e_info = (true, 0.0);
    let mut r_info_cd = (true, 0.0);
    if let Some(skills) = world.get::<Skills>(self_entity) {
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

    // 闪现状态
    let flash_info = world
        .get::<SelfPlayFlashCooldown>(self_entity)
        .map(|f| (f.is_ready(), f.remaining_secs()))
        .unwrap_or((true, 0.0));

    SelfPlayObs {
        role_id,
        self_pos: spos,
        self_hp: shp.map(|h| h.value).unwrap_or(0.0),
        self_max_hp: shp.map(|h| h.max).unwrap_or(SELFPLAY_CHAMPION_HP),
        target_pos: tpos,
        target_hp: thp.map(|h| h.value).unwrap_or(0.0),
        target_max_hp: thp.map(|h| h.max).unwrap_or(SELFPLAY_CHAMPION_HP),
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
    }
}

pub fn dispatch_single_action(
    world: &mut World,
    self_entity: Entity,
    target_entity: Entity,
    action: SelfPlayAction,
) {
    let spos = world
        .get::<Transform>(self_entity)
        .map(|t| t.translation)
        .unwrap_or_default();
    let tpos = world
        .get::<Transform>(target_entity)
        .map(|t| t.translation)
        .unwrap_or_default();

    let target_offset = Vec2::new(
        action.offset_x.clamp(-1.0, 1.0) * SELFPLAY_OFFSET_SCALE,
        action.offset_z.clamp(-1.0, 1.0) * SELFPLAY_OFFSET_SCALE,
    );
    let offset_target_pos = Vec2::new(tpos.x + target_offset.x, tpos.z + target_offset.y);

    match action.discrete {
        SelfPlayDiscreteAction::NoOp => {}
        SelfPlayDiscreteAction::Move => {
            world.trigger(CommandAction {
                entity: self_entity,
                action: Action::Move(offset_target_pos),
            });
        }
        SelfPlayDiscreteAction::Attack => {
            world.trigger(CommandAction {
                entity: self_entity,
                action: Action::Attack(target_entity),
            });
        }
        SelfPlayDiscreteAction::CastQ => {
            world.trigger(CommandAction {
                entity: self_entity,
                action: Action::Skill {
                    index: 0,
                    point: offset_target_pos,
                },
            });
        }
        SelfPlayDiscreteAction::CastW => {
            world.trigger(CommandAction {
                entity: self_entity,
                action: Action::Skill {
                    index: 1,
                    point: Vec2::new(spos.x, spos.z),
                },
            });
        }
        SelfPlayDiscreteAction::CastE => {
            world.trigger(CommandAction {
                entity: self_entity,
                action: Action::Skill {
                    index: 2,
                    point: offset_target_pos,
                },
            });
        }
        SelfPlayDiscreteAction::CastR => {
            world.trigger(CommandAction {
                entity: self_entity,
                action: Action::Skill {
                    index: 3,
                    point: Vec2::new(tpos.x, tpos.z),
                },
            });
        }
        SelfPlayDiscreteAction::CastFlash => {
            let ready = world
                .get::<SelfPlayFlashCooldown>(self_entity)
                .map(|f| f.is_ready())
                .unwrap_or(true);
            let alive = world
                .get::<Health>(self_entity)
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
                dir = Vec2::new(tpos.x - spos.x, tpos.z - spos.z);
            }
            let dir = dir.normalize_or_zero();
            if dir.length_squared() < 1e-4 {
                return;
            }
            if let Some(mut tf) = world.get_mut::<Transform>(self_entity) {
                tf.translation.x += dir.x * SELFPLAY_FLASH_DISTANCE;
                tf.translation.z += dir.y * SELFPLAY_FLASH_DISTANCE;
            }
            if let Some(mut flash) = world.get_mut::<SelfPlayFlashCooldown>(self_entity) {
                flash.start();
            } else {
                let mut flash = SelfPlayFlashCooldown::default();
                flash.start();
                world.entity_mut(self_entity).insert(flash);
            }
        }
    }
}

pub fn step_selfplay_world(
    app: &mut App,
    fiora: Entity,
    riven: Entity,
    act_fiora: SelfPlayAction,
    act_riven: SelfPlayAction,
    step_count: usize,
    max_steps: usize,
) -> (StepResult<SelfPlayObs>, StepResult<SelfPlayObs>) {
    let prev_fiora_obs = get_ego_obs_from_world(app.world(), fiora, riven, 0.0);
    let prev_riven_obs = get_ego_obs_from_world(app.world(), riven, fiora, 1.0);

    let prev_f_hp = prev_fiora_obs.self_hp;
    let prev_r_hp = prev_riven_obs.self_hp;

    // 清理破绽追踪器
    if let Some(mut tracker) = app.world_mut().get_resource_mut::<VitalBreakTracker>() {
        tracker.hit = false;
    }

    // 1. 派发双方动作指令
    dispatch_single_action(app.world_mut(), fiora, riven, act_fiora);
    dispatch_single_action(app.world_mut(), riven, fiora, act_riven);

    // 2. 确保虚拟时间流动
    unpause_virtual_time(app.world_mut());

    // 3. 固定推演 10 帧
    for _ in 0..10 {
        app.update();
    }

    // 4. 获取推演后双方最新观测
    let curr_fiora_obs = get_ego_obs_from_world(app.world(), fiora, riven, 0.0);
    let curr_riven_obs = get_ego_obs_from_world(app.world(), riven, fiora, 1.0);

    let curr_f_hp = curr_fiora_obs.self_hp;
    let curr_r_hp = curr_riven_obs.self_hp;

    let is_vital_break = app
        .world()
        .get_resource::<VitalBreakTracker>()
        .map(|t| t.hit)
        .unwrap_or(false);

    // 5. 零和对抗奖励计算
    // 造成伤害百分比奖励 - 承受伤害百分比惩罚
    let fiora_dmg_dealt = ((prev_r_hp - curr_r_hp).max(0.0)) / SELFPLAY_CHAMPION_HP;
    let riven_dmg_dealt = ((prev_f_hp - curr_f_hp).max(0.0)) / SELFPLAY_CHAMPION_HP;

    let vital_bonus = if is_vital_break { 1.5 } else { 0.0 };

    let fiora_killed = curr_r_hp <= 0.0;
    let riven_killed = curr_f_hp <= 0.0;

    let kill_bonus_fiora = if fiora_killed {
        10.0
    } else if riven_killed {
        -10.0
    } else {
        0.0
    };

    let r_fiora = (fiora_dmg_dealt - riven_dmg_dealt) * 3.0 + vital_bonus + kill_bonus_fiora;
    let r_riven = -r_fiora;

    let terminated = curr_f_hp <= 0.0 || curr_r_hp <= 0.0;
    let truncated = max_steps > 0 && step_count >= max_steps;

    let mut vars_fiora = HashMap::new();
    vars_fiora.insert(
        "self_damage_dealt_reward".to_string(),
        fiora_dmg_dealt * 3.0,
    );
    vars_fiora.insert(
        "self_damage_taken_penalty".to_string(),
        -riven_dmg_dealt * 3.0,
    );
    vars_fiora.insert("vital_reward".to_string(), vital_bonus);
    vars_fiora.insert("kill_reward".to_string(), kill_bonus_fiora);

    let mut vars_riven = HashMap::new();
    vars_riven.insert(
        "self_damage_dealt_reward".to_string(),
        riven_dmg_dealt * 3.0,
    );
    vars_riven.insert(
        "self_damage_taken_penalty".to_string(),
        -fiora_dmg_dealt * 3.0,
    );
    vars_riven.insert("vital_reward".to_string(), -vital_bonus);
    vars_riven.insert("kill_reward".to_string(), -kill_bonus_fiora);

    let breakdown_fiora = vec![
        RewardBreakdownItem {
            name: "造成伤害收益".to_string(),
            value: fiora_dmg_dealt * 3.0,
        },
        RewardBreakdownItem {
            name: "承受伤害惩罚".to_string(),
            value: -riven_dmg_dealt * 3.0,
        },
        RewardBreakdownItem {
            name: "破绽攻防转移".to_string(),
            value: vital_bonus,
        },
        RewardBreakdownItem {
            name: "击杀胜负判定".to_string(),
            value: kill_bonus_fiora,
        },
    ];

    let breakdown_riven = vec![
        RewardBreakdownItem {
            name: "造成伤害收益".to_string(),
            value: riven_dmg_dealt * 3.0,
        },
        RewardBreakdownItem {
            name: "承受伤害惩罚".to_string(),
            value: -fiora_dmg_dealt * 3.0,
        },
        RewardBreakdownItem {
            name: "破绽攻防转移".to_string(),
            value: -vital_bonus,
        },
        RewardBreakdownItem {
            name: "击杀胜负判定".to_string(),
            value: -kill_bonus_fiora,
        },
    ];

    (
        StepResult {
            obs: curr_fiora_obs,
            reward: r_fiora,
            terminated,
            truncated,
            step: step_count,
            reward_breakdown: breakdown_fiora,
            reward_variables: vars_fiora,
        },
        StepResult {
            obs: curr_riven_obs,
            reward: r_riven,
            terminated,
            truncated,
            step: step_count,
            reward_breakdown: breakdown_riven,
            reward_variables: vars_riven,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selfplay_obs_dimension_and_role_indicator() {
        let env = FioraRivenSelfPlayEnv::new_with_max_steps(10);
        let obs_f = env.get_fiora_obs();
        let obs_r = env.get_riven_obs();

        assert_eq!(obs_f.role_id, 0.0, "剑姬视角的 role_id 应为 0.0");
        assert_eq!(obs_r.role_id, 1.0, "瑞雯视角的 role_id 应为 1.0");

        let vec_f = obs_f.to_vector();
        let vec_r = obs_r.to_vector();

        assert_eq!(vec_f.len(), SELFPLAY_OBS_DIM);
        assert_eq!(vec_r.len(), SELFPLAY_OBS_DIM);
        assert_eq!(vec_f[0], 0.0);
        assert_eq!(vec_r[0], 1.0);
    }

    #[test]
    fn test_selfplay_step_both_and_zero_sum_reward() {
        let mut env = FioraRivenSelfPlayEnv::new_with_max_steps(5);
        let act_f = SelfPlayAction::new(0.0, 0.0, SelfPlayDiscreteAction::Attack);
        let act_r = SelfPlayAction::new(0.0, 0.0, SelfPlayDiscreteAction::CastW);

        let (res_f, res_r) = env.step_both(act_f, act_r);

        assert_eq!(res_f.step, 1);
        assert_eq!(res_r.step, 1);
        // 验证对称零和奖励关系
        assert!(
            (res_f.reward + res_r.reward).abs() < 1e-4,
            "自博弈奖励应严格零和: res_f={}, res_r={}",
            res_f.reward,
            res_r.reward
        );
    }

    #[test]
    fn test_selfplay_action_mask() {
        let env = FioraRivenSelfPlayEnv::new_with_max_steps(10);
        let obs_f = env.get_fiora_obs();
        let mask = FioraRivenSelfPlayEnv::action_mask(&obs_f).expect("应当返回动作掩码");

        assert_eq!(mask.len(), 8);
        assert!(mask[0], "NoOp 永远有效");
        assert!(mask[1], "Move 永远有效");
        assert!(mask[7], "初始闪现应就绪有效");
    }
}
