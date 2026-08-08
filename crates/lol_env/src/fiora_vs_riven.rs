use std::path::PathBuf;

use bevy::app::ScheduleRunnerPlugin;
use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use lol_base::character::{ConfigCharacterRecord, ConfigSkin};
use lol_champions::fiora::passive::Vital;
use lol_champions::fiora::{Fiora, PluginFiora};
use lol_champions::riven::{PluginRiven, Riven};
use lol_core::action::{Action, CommandAction};
use lol_core::attack::AttackState;
use lol_core::attack_auto::AttackAuto;
use lol_core::base::direction::Direction;
use lol_core::life::{Death, Health, RespawnTimer};
use lol_core::rotate::Rotate;
use lol_core::run::Run;
use lol_core::skill::{Skill, Skills};
use lol_core::team::Team;

/// Controls whether the Env runs headless (for training) or with a window (for visualization).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderMode {
    /// No window, MinimalPlugins, maximum throughput.
    Headless,
    /// With window + render/particle plugins (standard Bevy WinitPlugin).
    Window,
    /// With window + render/particle plugins, but **without** WinitPlugin.
    /// Used by `visual_runner` which drives its own custom winit event loop.
    WindowCustomLoop,
}

/// Configuration for constructing a `FioraVsRivenEnv`.
pub struct EnvConfig {
    pub max_steps: usize,
    pub render_mode: RenderMode,
}

impl Default for EnvConfig {
    fn default() -> Self {
        Self {
            max_steps: 0,
            render_mode: RenderMode::Headless,
        }
    }
}

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

#[derive(Debug, Clone)]
pub struct FioraVsRivenObs {
    pub fiora_pos: Vec3,
    pub fiora_hp: f32,
    pub fiora_max_hp: f32,
    pub riven_pos: Vec3,
    pub riven_hp: f32,
    pub riven_max_hp: f32,
    pub distance: f32,
    pub q_ready: bool,
    pub w_ready: bool,
    pub e_ready: bool,
    pub r_ready: bool,
    // Vital observation
    pub has_vital: bool,
    pub vital_is_active: bool,
    pub vital_dir_x: f32,
    pub vital_dir_neg_x: f32,
    pub vital_dir_z: f32,
    pub vital_dir_neg_z: f32,
}

impl FioraVsRivenObs {
    pub fn to_vector(&self) -> Vec<f32> {
        let rel_x = self.fiora_pos.x - self.riven_pos.x;
        let rel_z = self.fiora_pos.z - self.riven_pos.z;

        vec![
            // 破绽四方位 (4维)
            self.vital_dir_x,
            self.vital_dir_neg_x,
            self.vital_dir_z,
            self.vital_dir_neg_z,
            // 剑姬相对于瑞雯的相对位置与距离 (3维，归一化/100.0)
            rel_x / 100.0,
            rel_z / 100.0,
            self.distance / 100.0,
        ]
    }

    pub fn dim() -> usize {
        7
    }
}

#[derive(Debug, Clone)]
pub struct StepResult {
    pub obs: FioraVsRivenObs,
    pub reward: f32,
    pub terminated: bool,
    pub truncated: bool,
    pub step: usize,
    pub reward_breakdown: Vec<RewardBreakdownItem>,
}

#[derive(Debug, Clone)]
pub struct RewardBreakdownItem {
    pub name: String,
    pub value: f32,
}

#[derive(Resource, Default, Debug, Clone)]
pub struct AttackEventTracker {
    pub attack_hit: bool,
    pub attack_ready: bool,
}

fn on_attack_end(
    _trigger: On<lol_core::attack::EventAttackEnd>,
    mut tracker: ResMut<AttackEventTracker>,
) {
    tracker.attack_hit = true;
}

fn on_attack_ready(
    _trigger: On<lol_core::attack::EventAttackReady>,
    mut tracker: ResMut<AttackEventTracker>,
) {
    tracker.attack_ready = true;
}

pub struct FioraVsRivenEnv {
    app: App,
    fiora: Entity,
    riven: Entity,
    step_count: usize,
    max_steps: usize,
    initial_fiora_pos: Vec3,
    initial_riven_pos: Vec3,
    render_mode: RenderMode,
}

impl FioraVsRivenEnv {
    /// Shorthand for headless training: `FioraVsRivenEnv::new(max_steps)`.
    pub fn new(max_steps: usize) -> Self {
        Self::with_config(EnvConfig {
            max_steps,
            render_mode: RenderMode::Headless,
        })
    }

    /// Construct with full configuration.
    pub fn with_config(config: EnvConfig) -> Self {
        let max_steps = config.max_steps;
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

        app.init_resource::<AttackEventTracker>();
        app.add_observer(on_attack_end);
        app.add_observer(on_attack_ready);

        app.insert_resource(lol_base::map::MapPaths::new("test"));

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
        let initial_riven_pos = Vec3::new(250.0, 0.0, 0.0);

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

        // Wait for config and skin assets to load completely
        for _ in 0..500 {
            let asset_server = app.world().resource::<AssetServer>();
            let fiora_ready = if render {
                asset_server
                    .get_recursive_dependency_load_state(&fiora_skin_handle.clone().unwrap())
                    .is_some_and(|s| s.is_loaded())
            } else {
                asset_server
                    .get_recursive_dependency_load_state(&fiora_config_handle)
                    .is_some_and(|s| s.is_loaded())
            };
            let riven_ready = if render {
                asset_server
                    .get_recursive_dependency_load_state(&riven_skin_handle.clone().unwrap())
                    .is_some_and(|s| s.is_loaded())
            } else {
                asset_server
                    .get_recursive_dependency_load_state(&riven_config_handle)
                    .is_some_and(|s| s.is_loaded())
            };

            if fiora_ready && riven_ready {
                break;
            }
            app.update();
        }

        let mut env = Self {
            app,
            fiora,
            riven,
            step_count: 0,
            max_steps,
            initial_fiora_pos,
            initial_riven_pos,
            render_mode: config.render_mode,
        };

        env.setup_champion_skill_levels();
        env
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
        reset_episode_world(
            self.app.world_mut(),
            self.fiora,
            self.riven,
            self.initial_fiora_pos,
            self.initial_riven_pos,
        );
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
    pub fn step(&mut self, action: FioraVsRivenAction) -> StepResult {
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

// ── Shared World/App Level Helper Functions ─────────────────────────────────

/// Extract observation from the Bevy ECS world.
pub fn get_obs_from_world(world: &World, fiora: Entity, riven: Entity) -> FioraVsRivenObs {
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

    let vital = world.get::<Vital>(riven);
    let (has_vital, vital_is_active, v_x, v_neg_x, v_z, v_neg_z) = match vital {
        Some(v) => {
            let (vx, vnx, vz, vnz) = match v.direction {
                Direction::X => (1.0, 0.0, 0.0, 0.0),
                Direction::NegX => (0.0, 1.0, 0.0, 0.0),
                Direction::Z => (0.0, 0.0, 1.0, 0.0),
                Direction::NegZ => (0.0, 0.0, 0.0, 1.0),
            };
            (true, v.is_active(), vx, vnx, vz, vnz)
        }
        None => (false, false, 0.0, 0.0, 0.0, 0.0),
    };

    let (q_ready, w_ready, e_ready, r_ready) = {
        let mut ready = (true, true, true, true);
        if let Some(skills) = world.get::<Skills>(fiora) {
            let skill_entities = skills.to_vec();
            let check_ready = |idx: usize| -> bool {
                if idx < skill_entities.len() {
                    let s_entity = skill_entities[idx];
                    let cd = world.get::<lol_core::skill::CoolDown>(s_entity);
                    let recast = world.get::<lol_core::skill::SkillRecastWindow>(s_entity);
                    match cd {
                        Some(c) => lol_core::skill::is_skill_ready(c, recast),
                        None => true,
                    }
                } else {
                    true
                }
            };
            ready = (
                check_ready(0),
                check_ready(1),
                check_ready(2),
                check_ready(3),
            );
        }
        ready
    };

    FioraVsRivenObs {
        fiora_pos: fpos,
        fiora_hp: fhp.map(|h| h.value).unwrap_or(0.0),
        fiora_max_hp: fhp.map(|h| h.max).unwrap_or(500.0),
        riven_pos: rpos,
        riven_hp: rhp.map(|h| h.value).unwrap_or(0.0),
        riven_max_hp: rhp.map(|h| h.max).unwrap_or(500.0),
        distance: dist,
        q_ready,
        w_ready,
        e_ready,
        r_ready,
        has_vital,
        vital_is_active,
        vital_dir_x: v_x,
        vital_dir_neg_x: v_neg_x,
        vital_dir_z: v_z,
        vital_dir_neg_z: v_neg_z,
    }
}

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

/// Reset entities in the Bevy ECS world for a new episode.
pub fn reset_episode_world(
    world: &mut World,
    fiora: Entity,
    riven: Entity,
    initial_fiora_pos: Vec3,
    initial_riven_pos: Vec3,
) {
    for champion in [fiora, riven] {
        let mut entity_mut = world.entity_mut(champion);
        entity_mut.remove::<Death>();
        entity_mut.remove::<RespawnTimer>();
        entity_mut.remove::<AttackAuto>();
        entity_mut.remove::<AttackState>();
        entity_mut.remove::<Run>();
        entity_mut.remove::<Rotate>();
    }

    if let Some(mut t) = world.get_mut::<Transform>(fiora) {
        t.translation = initial_fiora_pos;
    }
    if let Some(mut t) = world.get_mut::<Transform>(riven) {
        t.translation = initial_riven_pos;
    }
    if let Some(mut h) = world.get_mut::<Health>(fiora) {
        h.value = if h.max > 0.0 { h.max } else { 500.0 };
    }
    if let Some(mut h) = world.get_mut::<Health>(riven) {
        h.value = if h.max > 0.0 { h.max } else { 500.0 };
    }

    if let Some(skills) = world.get::<Skills>(fiora) {
        let skill_entities = skills.to_vec();
        for s_entity in skill_entities {
            if let Some(mut cd) = world.get_mut::<lol_core::skill::CoolDown>(s_entity) {
                cd.timer = None;
            }
        }
    }

    // 随机为目标生成一个初始已激活的破绽 (Active Vital)
    let random_dir = match rand::random::<u8>() % 4 {
        0 => Direction::X,
        1 => Direction::NegX,
        2 => Direction::Z,
        _ => Direction::NegZ,
    };
    let mut initial_vital = Vital::new(random_dir, 0.0, 10.0);
    initial_vital
        .active_timer
        .tick(std::time::Duration::from_millis(1));
    world.entity_mut(riven).insert(initial_vital);

    if let Some(mut tracker) = world.get_resource_mut::<AttackEventTracker>() {
        tracker.attack_hit = false;
        tracker.attack_ready = false;
    }
}

/// Set skill levels for Fiora and Riven in the Bevy ECS world.
pub fn setup_skill_levels_world(world: &mut World, fiora: Entity, riven: Entity) {
    for champion in [fiora, riven] {
        if let Some(skills) = world.get::<Skills>(champion) {
            let skill_entities = skills.to_vec();
            if skill_entities.len() >= 4 {
                if let Some(mut q) = world.get_mut::<Skill>(skill_entities[0]) {
                    q.level = 3;
                }
                if let Some(mut w) = world.get_mut::<Skill>(skill_entities[1]) {
                    w.level = 1;
                }
                if let Some(mut e) = world.get_mut::<Skill>(skill_entities[2]) {
                    e.level = 1;
                }
                if let Some(mut r) = world.get_mut::<Skill>(skill_entities[3]) {
                    r.level = 1;
                }
            }
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

            if let Some(mut tracker) = app.world_mut().get_resource_mut::<AttackEventTracker>() {
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

/// Helper function to check if a 3D position is aligned with the vital's direction quadrant relative to target.
pub fn is_position_aligned_with_vital(fpos: Vec3, rpos: Vec3, obs: &FioraVsRivenObs) -> bool {
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

/// Compute step reward and its breakdown items.
/// Extracted as a standalone function to be shared across Env modes and visual runners.
pub fn compute_step_reward(
    prev_riven_hp: f32,
    _prev_fiora_hp: f32,
    curr_riven_hp: f32,
    _curr_fiora_hp: f32,
    prev_fpos: Vec3,
    curr_fpos: Vec3,
    riven_pos: Vec3,
    action: FioraVsRivenAction,
    prev_obs: &FioraVsRivenObs,
) -> (f32, Vec<RewardBreakdownItem>) {
    let _damage_dealt = (prev_riven_hp - curr_riven_hp).max(0.0);

    // 1. 时间惩罚 (Step/Time penalty)
    let time_penalty = -0.5;

    // 2. 判断攻击时（prev_fpos）或移动后（curr_fpos）是否与破绽方位对齐
    let is_attack_pos_aligned =
        prev_obs.has_vital && is_position_aligned_with_vital(prev_fpos, riven_pos, prev_obs);
    let is_move_pos_aligned =
        prev_obs.has_vital && is_position_aligned_with_vital(curr_fpos, riven_pos, prev_obs);

    // 3. 严格打破绽判定：必须满足破绽已激活、攻击动作、且站位处于正确破绽象限
    let is_vital_break = prev_obs.has_vital
        && prev_obs.vital_is_active
        && action == FioraVsRivenAction::AttackRiven
        && is_attack_pos_aligned;

    let mut attack_miss_penalty = 0.0;

    let (vital_break_reward, shaping_reward) = match action {
        FioraVsRivenAction::AttackRiven => {
            if is_vital_break {
                (100.0, 0.0)
            } else {
                // 攻击时没有打到破绽：给予强烈的失误扣分惩罚，且不给任何伤害奖励
                attack_miss_penalty = -10.0;
                (0.0, 0.0)
            }
        }
        FioraVsRivenAction::MoveEast50
        | FioraVsRivenAction::MoveWest50
        | FioraVsRivenAction::MoveNorth50
        | FioraVsRivenAction::MoveSouth50 => {
            // 移动动作：如果移动后的新位置对齐了当前破绽方位，给予站位对齐正反馈；否则给予微小惩罚
            if is_move_pos_aligned {
                (0.0, 2.0)
            } else {
                (0.0, -0.5)
            }
        }
    };

    let breakdown = vec![
        RewardBreakdownItem {
            name: "时间惩罚 (Time Penalty)".to_string(),
            value: time_penalty,
        },
        RewardBreakdownItem {
            name: "站位引导 (Alignment Shaping)".to_string(),
            value: shaping_reward,
        },
        RewardBreakdownItem {
            name: "未打破绽失误扣分 (Missed Vital Penalty)".to_string(),
            value: attack_miss_penalty,
        },
        RewardBreakdownItem {
            name: "打破绽成功 (Vital Break)".to_string(),
            value: vital_break_reward,
        },
    ];

    let reward = time_penalty + shaping_reward + attack_miss_penalty + vital_break_reward;

    (reward, breakdown)
}

/// Helper functions for controlling Virtual time during visual stepping.
pub fn pause_virtual_time(world: &mut World) {
    if let Some(mut time) = world.get_resource_mut::<Time<Virtual>>() {
        time.pause();
    }
}

pub fn unpause_virtual_time(world: &mut World) {
    if let Some(mut time) = world.get_resource_mut::<Time<Virtual>>() {
        time.unpause();
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
) -> StepResult {
    let prev_obs = get_obs_from_world(app.world(), fiora, riven);
    let prev_riven_hp = prev_obs.riven_hp;
    let prev_fiora_hp = prev_obs.fiora_hp;

    // 1. Dispatch action to Bevy ECS World
    dispatch_action_world(app.world_mut(), fiora, riven, action);

    // 2. Ensure virtual time is active so step simulation advances
    unpause_virtual_time(app.world_mut());

    // 3. Advance simulation for the action
    let attack_obs = advance_action_simulation(app, fiora, riven, action);
    let eval_obs = attack_obs.as_ref().unwrap_or(&prev_obs);

    // 4. Sample updated obs AFTER step completes
    let obs = get_obs_from_world(app.world(), fiora, riven);
    let curr_riven_hp = obs.riven_hp;
    let curr_fiora_hp = obs.fiora_hp;

    // 5. Compute step reward and breakdown
    let (reward, reward_breakdown) = compute_step_reward(
        prev_riven_hp,
        prev_fiora_hp,
        curr_riven_hp,
        curr_fiora_hp,
        eval_obs.fiora_pos,
        obs.fiora_pos,
        eval_obs.riven_pos,
        action,
        eval_obs,
    );

    let terminated = curr_riven_hp <= 0.0 || curr_fiora_hp <= 0.0;
    let truncated = max_steps > 0 && step_count >= max_steps;

    StepResult {
        obs,
        reward,
        terminated,
        truncated,
        step: step_count,
        reward_breakdown,
    }
}
