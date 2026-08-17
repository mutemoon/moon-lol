use std::path::PathBuf;

use bevy::app::ScheduleRunnerPlugin;
use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use lol_base::character::{ConfigCharacterRecord, ConfigSkin, Skin};
use lol_champions::fiora::passive::Vital;
use lol_champions::fiora::{Fiora, PluginFiora};
use lol_champions::riven::{PluginRiven, Riven};
use lol_core::action::{Action, CommandAction};
use lol_core::character::CharacterReady;
use lol_core::life::Health;
use lol_core::navigation::navigation::NavigationDebug;
use lol_core::team::Team;
use lol_rl_protocol::ActionSpace;

// 供 `lib.rs` 与其他调用方直接复用的公共实现（收敛到 `fiora_riven_common`）。
pub use crate::fiora_riven_common::{
    ATTACK_MASK_DISTANCE, FioraVsRivenObs, compute_step_reward, get_obs_from_world,
    is_position_aligned_with_vital, pause_virtual_time, reset_episode_world,
    setup_skill_levels_world,
};
use crate::fiora_riven_common::{
    AttackEventTracker, FioraRivenEntities, VitalBreakTracker, add_common_observers,
    unpause_virtual_time,
};
use crate::reward::{FioraVsRivenRewardModel, RewardModel};
pub use crate::traits::{
    EnvConfig, EnvMeta, RenderMode, RewardBreakdownItem, RlEnvironment, StepResult,
    VisualEnvironment,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FioraVsRivenAction {
    MoveEast50 = 0,  // Stand 50u East (+X relative to Riven)
    MoveWest50 = 1,  // Stand 50u West (-X relative to Riven)
    MoveNorth50 = 2, // Stand 50u North (+Z relative to Riven)
    MoveSouth50 = 3, // Stand 50u South (-Z relative to Riven)
    AttackRiven = 4, // Basic attack Riven
}

impl FioraVsRivenAction {
    pub fn from_index(index: usize) -> Self {
        match index {
            0 => Self::MoveEast50,
            1 => Self::MoveWest50,
            2 => Self::MoveNorth50,
            3 => Self::MoveSouth50,
            4 => Self::AttackRiven,
            _ => Self::MoveEast50,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::MoveEast50 => "MoveEast50 (东侧 50u 站位)",
            Self::MoveWest50 => "MoveWest50 (西侧 50u 站位)",
            Self::MoveNorth50 => "MoveNorth50 (北侧 50u 站位)",
            Self::MoveSouth50 => "MoveSouth50 (南侧 50u 站位)",
            Self::AttackRiven => "AttackRiven (普通攻击 瑞雯)",
        }
    }
}

pub struct FioraVsRivenEnv {
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

impl FioraVsRivenEnv {
    pub const DEFAULT_MAX_STEPS: usize = 40;

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

    /// Construct with full configuration.
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

        // High CPU throughput configuration per docs/game/facts/bevy.md:
        // FixedTimesteps(1) with app.update() for exact stepping
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
                // Disable WinitPlugin — the caller (visual_runner) drives its own winit event loop
                app.add_plugins(
                    DefaultPlugins
                        .build()
                        .disable::<bevy::winit::WinitPlugin>()
                        .set(asset_plugin)
                        .set(WindowPlugin {
                            primary_window: Some(Window {
                                title: "Fiora vs Riven - RL Visual Viewer".to_string(),
                                resolution: (1280, 720).into(),
                                ..Default::default()
                            }),
                            ..Default::default()
                        }),
                );
            } else {
                app.add_plugins(DefaultPlugins.set(asset_plugin).set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Fiora vs Riven RL Evaluation (Render Mode)".to_string(),
                        resolution: (1280, 720).into(),
                        ..Default::default()
                    }),
                    ..Default::default()
                }));
            }
            app.add_plugins(lol_render::PluginRender);
            app.add_plugins(lol_core::PluginCore);
            app.add_plugins(lol_particle::PluginParticle);
        } else {
            app.add_plugins((
                MinimalPlugins.set(ScheduleRunnerPlugin::run_once()),
                asset_plugin,
                bevy::world_serialization::WorldSerializationPlugin,
            ));
            app.add_plugins(lol_core::PluginCore);
        }

        app.add_plugins(PluginFiora);
        app.add_plugins(PluginRiven);

        app.insert_resource(lol_base::map::MapPaths::new("test"));
        app.insert_resource(NavigationDebug);

        app.finish();
        app.cleanup();

        let asset_server = app.world().resource::<AssetServer>();
        let fiora_config_handle = asset_server.load::<DynamicWorld>("characters/fiora/config.ron");
        let riven_config_handle = asset_server.load::<DynamicWorld>("characters/Riven/config.ron");

        let fiora_skin_handle = if render {
            Some(asset_server.load::<DynamicWorld>("characters/fiora/skins/skin0.ron"))
        } else {
            None
        };
        let riven_skin_handle = if render {
            Some(asset_server.load::<DynamicWorld>("characters/Riven/skins/skin0.ron"))
        } else {
            None
        };

        let initial_fiora_pos = Vec3::ZERO;
        let initial_riven_pos = Vec3::new(50.0, 0.0, 0.0);

        // Spawn Level 6 Fiora (Order team)
        let mut fiora_builder = app.world_mut().spawn((
            Fiora::default(),
            Transform::from_translation(initial_fiora_pos),
            Team::Order,
            ConfigCharacterRecord {
                character_record: fiora_config_handle.clone(),
            },
            Health::new(500.0),
            lol_core::damage::Armor(35.0),
            lol_core::movement::Movement { speed: 345.0 },
        ));

        if render {
            fiora_builder.insert((
                lol_render::controller::SelfPlayer,
                lol_base_render::camera::Focus,
                ConfigSkin {
                    skin: fiora_skin_handle.clone().unwrap(),
                },
            ));
        }

        let fiora = fiora_builder.id();

        // Spawn Level 6 Riven (Chaos team)
        let mut riven_builder = app.world_mut().spawn((
            Riven::default(),
            Transform::from_translation(initial_riven_pos),
            Team::Chaos,
            ConfigCharacterRecord {
                character_record: riven_config_handle.clone(),
            },
            Health::new(500.0),
            lol_core::damage::Armor(33.0),
            lol_core::movement::Movement { speed: 340.0 },
        ));

        if render {
            riven_builder.insert(ConfigSkin {
                skin: riven_skin_handle.clone().unwrap(),
            });
        }

        let riven = riven_builder.id();

        // 注入实体引用与事件追踪（必须在资产加载循环的 `app.update()` 之前就绪）
        app.world_mut()
            .insert_resource(FioraRivenEntities { fiora, riven });
        add_common_observers(&mut app);

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

    // ── Accessors ──────────────────────────────────────────────

    pub fn render_mode(&self) -> RenderMode {
        self.render_mode
    }

    pub fn app(&self) -> &App {
        &self.app
    }

    pub fn app_mut(&mut self) -> &mut App {
        &mut self.app
    }

    pub fn fiora(&self) -> Entity {
        self.fiora
    }

    pub fn riven(&self) -> Entity {
        self.riven
    }

    pub fn initial_fiora_pos(&self) -> Vec3 {
        self.initial_fiora_pos
    }

    pub fn initial_riven_pos(&self) -> Vec3 {
        self.initial_riven_pos
    }

    pub fn max_steps(&self) -> usize {
        self.max_steps
    }

    pub fn step_count(&self) -> usize {
        self.step_count
    }

    /// Sets Fiora and Riven to Level 6 with Q3, W1, E1, R1
    fn setup_champion_skill_levels(&mut self) {
        setup_skill_levels_world(self.app.world_mut(), self.fiora, self.riven);
    }

    pub fn reset(&mut self) -> FioraVsRivenObs {
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
        self.app.update();
        self.setup_champion_skill_levels();
        self.get_obs()
    }

    pub fn get_obs(&self) -> FioraVsRivenObs {
        get_obs_from_world(self.app.world(), self.fiora, self.riven)
    }

    /// Dispatch a single action into the Bevy ECS world **without** advancing ticks.
    /// This is the shared implementation used by both `step()` and `visual_runner`.
    pub fn dispatch_action(&mut self, action: FioraVsRivenAction) {
        dispatch_action_world(self.app.world_mut(), self.fiora, self.riven, action);
    }

    /// Advances the environment by 1 timestep with given action
    pub fn step(&mut self, action: FioraVsRivenAction) -> StepResult<FioraVsRivenObs> {
        self.step_count += 1;
        step_world(
            &mut self.app,
            self.fiora,
            self.riven,
            action,
            self.step_count,
            self.max_steps,
        )
    }
}

impl RlEnvironment for FioraVsRivenEnv {
    type Action = FioraVsRivenAction;
    type Obs = FioraVsRivenObs;

    fn env_name() -> &'static str {
        "FioraV0"
    }

    fn display_name() -> &'static str {
        "剑姬 vs 瑞雯 (瞬移基准环境-5动作)"
    }

    fn description() -> &'static str {
        "剑姬与瑞雯单挑对决，移动为瞬移四个方位（东/西/南/北 50u）"
    }

    fn action_dim() -> usize {
        5
    }

    fn action_space() -> ActionSpace {
        ActionSpace::Discrete(5)
    }

    fn state_dim() -> usize {
        FioraVsRivenObs::dim()
    }

    fn action_labels() -> &'static [&'static str] {
        &[
            "MoveEast50 (东侧50u)",
            "MoveWest50 (西侧50u)",
            "MoveNorth50 (北侧50u)",
            "MoveSouth50 (南侧50u)",
            "AttackRiven (攻击瑞雯)",
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
            "相对 X 偏移 (rel_x, 归一化)",
            "相对 Z 偏移 (rel_z, 归一化)",
            "英雄间距 (distance, 归一化)",
        ]
    }

    fn action_from_index(idx: usize) -> Self::Action {
        FioraVsRivenAction::from_index(idx)
    }

    fn action_to_index(action: Self::Action) -> usize {
        action as usize
    }

    fn action_name(action: Self::Action) -> &'static str {
        action.label()
    }

    fn default_max_steps() -> usize {
        Self::DEFAULT_MAX_STEPS
    }

    fn max_steps(&self) -> usize {
        self.max_steps
    }

    fn new() -> Self {
        Self::new()
    }

    fn with_config(config: EnvConfig) -> Self {
        Self::with_config(config)
    }

    fn reset(&mut self) -> Vec<Self::Obs> {
        vec![self.reset()]
    }

    fn step(&mut self, actions: &[Self::Action]) -> Vec<StepResult<Self::Obs>> {
        let action = actions.first().copied().unwrap_or(FioraVsRivenAction::MoveEast50);
        vec![self.step(action)]
    }

    fn obs_to_vector(obs: &Self::Obs) -> Vec<f32> {
        obs.to_vector()
    }

    fn obs_to_payload(obs: &Self::Obs) -> Option<lol_rl_protocol::ObsFeaturePayload> {
        Some(obs.to_payload())
    }

    fn is_action_masked(obs: &Self::Obs, action_idx: usize) -> bool {
        obs.distance > ATTACK_MASK_DISTANCE && action_idx == 4
    }

    fn action_mask(obs: &Self::Obs) -> Option<Vec<bool>> {
        let mut mask = vec![true; 5];
        if obs.distance > ATTACK_MASK_DISTANCE {
            mask[4] = false;
        }
        Some(mask)
    }

    fn reward_formula_spec() -> Option<lol_rl_protocol::RewardFormulaSpec> {
        Some(FioraVsRivenRewardModel.formula_spec())
    }
}

impl VisualEnvironment for FioraVsRivenEnv {
    fn take_app(&mut self) -> App {
        std::mem::replace(&mut self.app, App::new())
    }

    fn window_title(&self) -> &'static str {
        "Fiora vs Riven (Teleport) - RL Visual Viewer"
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
        vec![self.get_current_obs(world)]
    }

    fn get_current_obs_all(&self, world: &World) -> Vec<Self::Obs> {
        vec![get_obs_from_world(world, self.fiora, self.riven)]
    }

    fn step_world(
        &mut self,
        app: &mut App,
        actions: &[Self::Action],
    ) -> Vec<StepResult<Self::Obs>> {
        self.step_count += 1;
        let action = actions.first().copied().unwrap_or(FioraVsRivenAction::MoveEast50);
        let res = step_world(
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

// ── Shared World/App Level Helper Functions ─────────────────────────────────

/// Dispatch an action to the Bevy ECS world without advancing frames.
pub fn dispatch_action_world(
    world: &mut World,
    fiora: Entity,
    riven: Entity,
    action: FioraVsRivenAction,
) {
    let riven_pos = world
        .get::<Transform>(riven)
        .map(|t| t.translation)
        .unwrap_or(Vec3::new(250.0, 0.0, 0.0));

    match action {
        FioraVsRivenAction::MoveEast50 => {
            if let Some(mut t) = world.get_mut::<Transform>(fiora) {
                t.translation = Vec3::new(riven_pos.x + 50.0, riven_pos.y, riven_pos.z);
            }
        }
        FioraVsRivenAction::MoveWest50 => {
            if let Some(mut t) = world.get_mut::<Transform>(fiora) {
                t.translation = Vec3::new(riven_pos.x - 50.0, riven_pos.y, riven_pos.z);
            }
        }
        FioraVsRivenAction::MoveNorth50 => {
            if let Some(mut t) = world.get_mut::<Transform>(fiora) {
                t.translation = Vec3::new(riven_pos.x, riven_pos.y, riven_pos.z + 50.0);
            }
        }
        FioraVsRivenAction::MoveSouth50 => {
            if let Some(mut t) = world.get_mut::<Transform>(fiora) {
                t.translation = Vec3::new(riven_pos.x, riven_pos.y, riven_pos.z - 50.0);
            }
        }
        FioraVsRivenAction::AttackRiven => {
            // CommandAction::Attack will be triggered in advance_action_simulation after waiting for vital to actually spawn/activate.
        }
    }
}

/// Advance simulation for a given action until completion (including attack event cycle).
/// Returns the observation captured right before the attack command is released (if action is AttackRiven),
/// ensuring vital activation is correctly reflected for reward evaluation.
pub fn advance_action_simulation(
    app: &mut App,
    fiora: Entity,
    riven: Entity,
    action: FioraVsRivenAction,
) -> Option<FioraVsRivenObs> {
    match action {
        FioraVsRivenAction::MoveEast50
        | FioraVsRivenAction::MoveWest50
        | FioraVsRivenAction::MoveNorth50
        | FioraVsRivenAction::MoveSouth50 => {
            // Instant movement requires only 1 frame / 1 tick update
            app.update();
            None
        }
        FioraVsRivenAction::AttackRiven => {
            // 攻击动作前：等待破绽真的生成并激活再释放攻击命令
            let mut wait_frames = 0;
            while wait_frames < 300 {
                let vital_active = app
                    .world()
                    .get::<Vital>(riven)
                    .map(|v| v.is_active())
                    .unwrap_or(false);
                if vital_active {
                    break;
                }
                app.update();
                wait_frames += 1;
            }

            // 采样攻击命令释放时刻的 observation (破绽已激活)
            let attack_obs = get_obs_from_world(app.world(), fiora, riven);

            // 破绽生成并激活后，释放攻击命令
            app.world_mut().trigger(CommandAction {
                entity: fiora,
                action: Action::Attack(riven),
            });

            {
                let mut tracker = app
                    .world_mut()
                    .get_resource_mut::<AttackEventTracker>()
                    .unwrap();
                tracker.attack_hit = false;
                tracker.attack_ready = false;
            }
            // Remove redundant tracker clear since we clear it at the start of step_world
            {
                let mut tracker = app
                    .world_mut()
                    .get_resource_mut::<AttackEventTracker>()
                    .unwrap();
                tracker.attack_hit = false;
                tracker.attack_ready = false;
            }

            // Run initial frame to trigger CommandAttackAutoStart and CommandAttackStart
            app.update();

            // Advance world until both attack hit (EventAttackEnd) and attack ready (EventAttackReady) events are received
            let mut attack_frames = 0;
            while attack_frames < 100 {
                let (hit, ready) = app
                    .world()
                    .get_resource::<AttackEventTracker>()
                    .map(|t| (t.attack_hit, t.attack_ready))
                    .unwrap_or((true, true));
                if hit && ready {
                    break;
                }
                app.update();
                attack_frames += 1;
            }

            // Stop auto attack repeat so Fiora does not start another attack after this one
            app.world_mut()
                .trigger(lol_core::attack_auto::CommandAttackAutoStop { entity: fiora });

            Some(attack_obs)
        }
    }
}

/// Execute a complete single timestep simulation in the Bevy ECS World/App.
/// Shared across headless training (`FioraVsRivenEnv::step`) and GUI (`visual_runner`).
pub fn step_world(
    app: &mut App,
    fiora: Entity,
    riven: Entity,
    action: FioraVsRivenAction,
    step_count: usize,
    max_steps: usize,
) -> StepResult<FioraVsRivenObs> {
    let prev_obs = get_obs_from_world(app.world(), fiora, riven);
    let prev_riven_hp = prev_obs.riven_hp;

    // 清理上一帧的追踪器状态，防止非攻击动作（如移动）持续吃到上一帧的破绽击破奖励
    if let Some(mut tracker) = app.world_mut().get_resource_mut::<VitalBreakTracker>() {
        tracker.hit = false;
    }

    // 1. Dispatch action to Bevy ECS World
    dispatch_action_world(app.world_mut(), fiora, riven, action);

    // 2. Ensure virtual time is active so step simulation advances
    unpause_virtual_time(app.world_mut());

    // 3. Advance simulation for the action
    let _attack_obs = advance_action_simulation(app, fiora, riven, action);

    // 4. Sample updated obs AFTER step completes
    let obs = get_obs_from_world(app.world(), fiora, riven);
    let curr_riven_hp = obs.riven_hp;

    // 5. 真实破绽击破信号：来自菲奥娜被动的真实伤害事件
    let mut is_vital_break = app
        .world()
        .get_resource::<VitalBreakTracker>()
        .map(|t| t.hit)
        .unwrap_or(false);

    // 必须要求在执行攻击指令前，破绽已经存在且激活，防止智能体利用环境的“等待”机制盲目攻击
    is_vital_break = is_vital_break && prev_obs.has_vital && prev_obs.vital_is_active;

    // 6. Compute step reward and breakdown using structured AST formula
    let elapsed_secs = step_count as f32 * (10.0 / 60.0);
    let (reward, reward_breakdown, reward_variables) = compute_step_reward(
        prev_riven_hp,
        curr_riven_hp,
        prev_obs.fiora_pos,
        obs.fiora_pos,
        prev_obs.riven_pos,
        action == FioraVsRivenAction::AttackRiven,
        is_vital_break,
        &prev_obs,
        elapsed_secs,
    );

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
