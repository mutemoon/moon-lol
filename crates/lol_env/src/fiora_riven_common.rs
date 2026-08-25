use std::collections::HashMap;
use std::env::var;
use std::path::PathBuf;
use std::time::Duration;

use bevy::app::ScheduleRunnerPlugin;
use bevy::asset::AssetPlugin;
use bevy::ecs::schedule::SingleThreadedExecutor;
use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use bevy::winit::WinitPlugin;
use bevy::world_serialization::DynamicWorld;
use lol_base::character::{ConfigCharacterRecord, ConfigSkin};
use lol_base::map::MapPaths;
use lol_base_render::camera::Focus;
use lol_champions::fiora::passive::Vital;
use lol_champions::fiora::{Fiora, PluginFiora};
use lol_champions::riven::{PluginRiven, Riven};
use lol_core::action::{Action, CommandAction};
use lol_core::base::direction::Direction;
use lol_core::character::{CharacterReady, SpawnTransform};
use lol_core::damage::{Armor, DamageType, EventDamageCreate};
use lol_core::game::{GameState, WaitCharacterReady};
use lol_core::life::Health;
use lol_core::movement::Movement;
use lol_core::navigation::navigation::NavigationDebug;
use lol_core::skill::{CoolDown, Skill, SkillRecastWindow, Skills, is_skill_ready};
use lol_core::team::Team;
use lol_render::controller::SelfPlayer;
use rand::random;

use crate::reward::{FioraRewardContext, FioraVsRivenRewardModel, RewardModel};
use crate::traits::{EnvConfig, RenderMode, RewardBreakdownItem};

/// 攻击类动作的掩码距离阈值：超过该距离不允许攻击（单一事实来源）。
pub const ATTACK_MASK_DISTANCE: f32 = 220.0;

/// obs 向量中「相对距离归一化列」的下标，与 [`FioraVsRivenObs::to_vector`] 的布局一致。
pub const OBS_DISTANCE_IDX: usize = 8;
/// obs 向量中距离的归一化缩放：`to_vector` 写入 `distance / OBS_DISTANCE_SCALE`。
pub const OBS_DISTANCE_SCALE: f32 = 100.0;

// ── 基础环境宿主与 Builder (插件化架构) ──────────────────────────────────────

/// 英雄初始技能等级配置（Q, W, E, R）
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChampionInitialSkillLevels(pub [usize; 4]);

impl Default for ChampionInitialSkillLevels {
    fn default() -> Self {
        Self([3, 1, 1, 1])
    }
}

/// 剑姬 vs 瑞雯对战环境的公共 ECS 引擎基底。
/// 封装完整的 Bevy App 实例、实体句柄与生命周期管理。
pub struct FioraRivenBaseEnv {
    pub app: App,
    pub fiora: Entity,
    pub riven: Entity,
    pub fiora_config_handle: Handle<DynamicWorld>,
    pub riven_config_handle: Handle<DynamicWorld>,
    pub fiora_skin_handle: Option<Handle<DynamicWorld>>,
    pub riven_skin_handle: Option<Handle<DynamicWorld>>,
    pub step_count: usize,
    pub max_steps: usize,
    pub initial_fiora_pos: Vec3,
    pub initial_riven_pos: Vec3,
    pub map_name: String,
    pub enable_barrack: bool,
    pub initial_skill_levels: [usize; 4],
    pub warmup_secs: f32,
    pub render_mode: RenderMode,
    pub on_ready_hooks: Vec<fn(Entity, Entity, &mut World)>,
    pub on_reset_hooks: Vec<fn(Entity, Entity, &mut World)>,
}

impl FioraRivenBaseEnv {
    /// 获取环境 Builder
    pub fn builder(config: EnvConfig, default_max_steps: usize) -> FioraRivenEnvBuilder {
        FioraRivenEnvBuilder::new(config, default_max_steps)
    }

    pub fn app(&self) -> &App {
        &self.app
    }

    pub fn app_mut(&mut self) -> &mut App {
        &mut self.app
    }

    pub fn world(&self) -> &World {
        self.app.world()
    }

    pub fn world_mut(&mut self) -> &mut World {
        self.app.world_mut()
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

    pub fn render_mode(&self) -> RenderMode {
        self.render_mode
    }

    pub fn is_render(&self) -> bool {
        matches!(
            self.render_mode,
            RenderMode::Window | RenderMode::WindowCustomLoop
        )
    }

    pub fn max_steps(&self) -> usize {
        self.max_steps
    }

    pub fn step_count(&self) -> usize {
        self.step_count
    }

    pub fn increment_step(&mut self) {
        self.step_count += 1;
    }

    pub fn setup_champion_skill_levels(&mut self) {
        setup_skill_levels_world(self.app.world_mut(), self.fiora, self.riven);
    }

    /// 资产加载完成后的统一初次就绪流程（设置技能等级、执行 on_ready 钩子并进行预热）
    /// 无头模式在 build 结束时调用，有头模式在 VisualEnvironment::on_assets_loaded 中调用。
    pub fn on_assets_ready(&mut self, app: &mut App) {
        setup_skill_levels_world(app.world_mut(), self.fiora, self.riven);
        for hook in &self.on_ready_hooks {
            hook(self.fiora, self.riven, app.world_mut());
        }
        run_warmup_on_app(
            app,
            self.fiora,
            self.riven,
            self.warmup_secs,
            &self.on_ready_hooks,
        );
    }

    /// 执行基础环境重置（通过 Action::Reset 进行就地状态重置，执行 on_reset 钩子与物理预热）
    pub fn reset_base(&mut self) {
        let mut app = std::mem::replace(&mut self.app, App::new());
        self.reset_app(&mut app);
        self.app = app;
    }

    /// 在传入的 App 中执行对局重置（通过 Action::Reset 就地重置所有组件，保留 Entity ID 与已加载资源）
    pub fn reset_app(&mut self, app: &mut App) -> (Entity, Entity) {
        let fiora = self.fiora;
        let riven = self.riven;
        reset_app_internal(
            app,
            fiora,
            riven,
            self.warmup_secs,
            &self.on_reset_hooks,
        );
        self.step_count = 0;
        (fiora, riven)
    }

    /// 检查资产是否加载就绪（通过 GameState::Playing 判断）
    pub fn is_assets_loaded(&self, world: &World) -> bool {
        world
            .get_resource::<State<GameState>>()
            .is_some_and(|s| *s.get() == GameState::Playing)
    }
}

/// 执行统一预热（推进指定秒数的 App update，并重新执行状态钩子，无头与有头完全共用）
pub fn run_warmup_on_app(
    app: &mut App,
    fiora: Entity,
    riven: Entity,
    warmup_secs: f32,
    hooks: &[fn(Entity, Entity, &mut World)],
) {
    if warmup_secs <= 0.0 {
        return;
    }
    let warmup_ticks = (warmup_secs * 64.0).round() as usize;
    for _ in 0..warmup_ticks {
        app.update();
    }
    for hook in hooks {
        hook(fiora, riven, app.world_mut());
    }
}

/// 在 App 中执行就地重置的核心逻辑（不销毁实体）
fn reset_app_internal(
    app: &mut App,
    fiora: Entity,
    riven: Entity,
    warmup_secs: f32,
    on_reset_hooks: &[fn(Entity, Entity, &mut World)],
) {
    // 确保虚拟时间恢复，避免预热期间因暂停导致 delta 为 0 无法生成小兵与推进物理
    crate::visual_runner::unpause_virtual_time(app.world_mut());

    // 1. 触发核心 Action::Reset -> 全局 EventReset
    app.world_mut().trigger(CommandAction {
        entity: fiora,
        action: Action::Reset,
    });

    // 2. 推进一帧 schedule 使所有系统响应 EventReset 并清理缓冲区
    app.update();

    // 3. 重置环境专用 Tracker
    if let Some(mut tracker) = app.world_mut().get_resource_mut::<AttackEventTracker>() {
        tracker.attack_hit = false;
        tracker.attack_ready = false;
    }
    if let Some(mut tracker) = app.world_mut().get_resource_mut::<VitalBreakTracker>() {
        tracker.hit = false;
    }

    // 4. 重置剑姬初始要害
    let random_dir = match random::<u8>() % 4 {
        0 => Direction::X,
        1 => Direction::NegX,
        2 => Direction::Z,
        _ => Direction::NegZ,
    };
    let mut initial_vital = Vital::new(random_dir, 0.0, 10.0);
    initial_vital.active_timer.tick(Duration::from_millis(1));
    app.world_mut().entity_mut(riven).insert(initial_vital);

    // 5. 设置技能等级与执行重置钩子
    setup_skill_levels_world(app.world_mut(), fiora, riven);

    for hook in on_reset_hooks {
        hook(fiora, riven, app.world_mut());
    }

    // 6. 统一执行物理预热（无头与有头完全共用）
    run_warmup_on_app(app, fiora, riven, warmup_secs, on_reset_hooks);
}

/// 环境构造器：支持通过注册插件与钩子按需组装具体环境。
pub struct FioraRivenEnvBuilder {
    pub config: EnvConfig,
    pub default_max_steps: usize,
    pub window_title: String,
    pub map_name: String,
    pub enable_barrack: bool,
    pub initial_skill_levels: [usize; 4],
    pub warmup_secs: f32,
    pub initial_fiora_pos: Vec3,
    pub initial_riven_pos: Vec3,
    pub app_plugins: Vec<fn(&mut App)>,
    pub extra_observers: Vec<fn(&mut App)>,
    pub on_ready_hooks: Vec<fn(Entity, Entity, &mut World)>,
    pub on_reset_hooks: Vec<fn(Entity, Entity, &mut World)>,
}

impl FioraRivenEnvBuilder {
    pub fn new(config: EnvConfig, default_max_steps: usize) -> Self {
        Self {
            config,
            default_max_steps,
            window_title: "Fiora vs Riven RL".to_string(),
            map_name: "test".to_string(),
            enable_barrack: false,
            initial_skill_levels: [3, 1, 1, 1],
            warmup_secs: 0.0,
            initial_fiora_pos: Vec3::ZERO,
            initial_riven_pos: Vec3::new(50.0, 0.0, 0.0),
            app_plugins: Vec::new(),
            extra_observers: Vec::new(),
            on_ready_hooks: Vec::new(),
            on_reset_hooks: Vec::new(),
        }
    }

    pub fn window_title(mut self, title: impl Into<String>) -> Self {
        self.window_title = title.into();
        self
    }

    pub fn map_name(mut self, map_name: impl Into<String>) -> Self {
        self.map_name = map_name.into();
        self
    }

    pub fn enable_barrack(mut self, enable: bool) -> Self {
        self.enable_barrack = enable;
        self
    }

    pub fn initial_skill_levels(mut self, levels: [usize; 4]) -> Self {
        self.initial_skill_levels = levels;
        self
    }

    pub fn warmup_secs(mut self, secs: f32) -> Self {
        self.warmup_secs = secs;
        self
    }

    pub fn initial_positions(mut self, fiora_pos: Vec3, riven_pos: Vec3) -> Self {
        self.initial_fiora_pos = fiora_pos;
        self.initial_riven_pos = riven_pos;
        self
    }

    /// 注册一个插件函数（在 App::finish 之前向 App 注册系统/资源）
    pub fn with_plugin(mut self, plugin_fn: fn(&mut App)) -> Self {
        self.app_plugins.push(plugin_fn);
        self
    }

    /// 注册额外的 ECS 观察者
    pub fn with_observer(mut self, observer_fn: fn(&mut App)) -> Self {
        self.extra_observers.push(observer_fn);
        self
    }

    /// 注册资产加载就绪后的一次性初始化钩子
    pub fn on_ready(mut self, hook: fn(Entity, Entity, &mut World)) -> Self {
        self.on_ready_hooks.push(hook);
        self
    }

    /// 注册重置钩子（每次 reset_base 与 reset_world_base 时都会被自动调用）
    pub fn on_reset(mut self, hook: fn(Entity, Entity, &mut World)) -> Self {
        self.on_reset_hooks.push(hook);
        self
    }

    /// 组装并初始化 `FioraRivenBaseEnv`
    pub fn build(self) -> FioraRivenBaseEnv {
        let max_steps = if self.config.max_steps > 0 {
            self.config.max_steps
        } else {
            self.default_max_steps
        };
        let render = matches!(
            self.config.render_mode,
            RenderMode::Window | RenderMode::WindowCustomLoop
        );
        let mut app = App::new();

        app.insert_resource(TimeUpdateStrategy::FixedTimesteps(1));

        let manifest_dir =
            var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| env!("CARGO_MANIFEST_DIR").to_string());
        let workspace_root = PathBuf::from(&manifest_dir)
            .parent()
            .and_then(|p| p.parent())
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from(&manifest_dir));

        let asset_plugin = AssetPlugin {
            file_path: workspace_root.join("assets").to_string_lossy().to_string(),
            ..Default::default()
        };

        if render {
            if self.config.render_mode == RenderMode::WindowCustomLoop {
                app.add_plugins(
                    DefaultPlugins
                        .build()
                        .disable::<WinitPlugin>()
                        .set(asset_plugin)
                        .set(WindowPlugin {
                            primary_window: Some(Window {
                                title: self.window_title.clone(),
                                resolution: (1280, 720).into(),
                                ..Default::default()
                            }),
                            ..Default::default()
                        }),
                );
            } else {
                app.add_plugins(DefaultPlugins.set(asset_plugin).set(WindowPlugin {
                    primary_window: Some(Window {
                        title: self.window_title.clone(),
                        resolution: (1280, 720).into(),
                        ..Default::default()
                    }),
                    ..Default::default()
                }));
            }
            app.add_plugins(lol_render::PluginRender);
            if self.enable_barrack {
                app.add_plugins(lol_core::PluginCore);
            } else {
                app.add_plugins(
                    lol_core::PluginCore
                        .build()
                        .disable::<lol_core::PluginBarrack>(),
                );
            }
            app.add_plugins(lol_particle::PluginParticle);
        } else {
            app.add_plugins((
                MinimalPlugins.set(ScheduleRunnerPlugin::run_once()),
                asset_plugin,
                bevy::world_serialization::WorldSerializationPlugin,
                bevy::mesh::MeshPlugin,
                bevy::image::ImagePlugin::default(),
                bevy::animation::AnimationPlugin,
                bevy::audio::AudioPlugin::default(),
                bevy::scene::ScenePlugin,
            ));
            league_core::register::init_league_asset(&mut app);
            app.init_asset::<StandardMaterial>();
            app.init_asset::<Shader>();
            app.init_asset::<lol_base_render::animation::LOLAnimationGraph>();
            app.init_asset::<lol_base_render::particle::ConfigVfx>();
            app.init_asset::<bevy::prelude::WorldAsset>();
            app.init_asset::<lol_base::audio::ConfigAudio>();
            app.init_asset::<lol_base::spell::Spell>();
            app.init_asset::<lol_base::grid::ConfigNavigationGrid>();
            app.init_asset::<lol_base::item::ConfigItem>();

            if self.enable_barrack {
                app.add_plugins(lol_core::PluginCore);
            } else {
                app.add_plugins(
                    lol_core::PluginCore
                        .build()
                        .disable::<lol_core::PluginBarrack>(),
                );
            }
        }

        app.add_plugins(PluginFiora);
        app.add_plugins(PluginRiven);

        // 注册用户扩展插件
        for plugin in &self.app_plugins {
            plugin(&mut app);
        }

        app.insert_resource(MapPaths::new(&self.map_name));
        app.insert_resource(NavigationDebug);
        app.insert_resource(ChampionInitialSkillLevels(self.initial_skill_levels));

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

        // 生成英雄实体
        let (fiora, riven) = spawn_champions_world(
            app.world_mut(),
            fiora_config_handle.clone(),
            riven_config_handle.clone(),
            fiora_skin_handle.clone(),
            riven_skin_handle.clone(),
            self.initial_fiora_pos,
            self.initial_riven_pos,
            render,
        );

        app.world_mut()
            .insert_resource(FioraRivenEntities { fiora, riven });
        add_common_observers(&mut app);

        for observer in &self.extra_observers {
            observer(&mut app);
        }

        // 等待游戏资源就绪并进入 GameState::Playing（仅无头模式在 build 中同步自旋等待；渲染模式由 visual_runner 事件循环异步等待，避免阻塞主线程窗口创建）
        if !render {
            for _ in 0..500 {
                app.update();
                let world = app.world();
                let is_playing = world
                    .get_resource::<State<GameState>>()
                    .is_some_and(|s| *s.get() == GameState::Playing);

                if is_playing {
                    break;
                }
            }
        }

        let mut base = FioraRivenBaseEnv {
            app,
            fiora,
            riven,
            fiora_config_handle,
            riven_config_handle,
            fiora_skin_handle,
            riven_skin_handle,
            step_count: 0,
            max_steps,
            initial_fiora_pos: self.initial_fiora_pos,
            initial_riven_pos: self.initial_riven_pos,
            map_name: self.map_name,
            enable_barrack: self.enable_barrack,
            initial_skill_levels: self.initial_skill_levels,
            warmup_secs: self.warmup_secs,
            render_mode: self.config.render_mode,
            on_ready_hooks: self.on_ready_hooks,
            on_reset_hooks: self.on_reset_hooks,
        };

        // 仅在无头模式下由 build 执行初次就绪与物理预热
        if !render {
            let mut app = std::mem::replace(&mut base.app, App::new());
            base.on_assets_ready(&mut app);
            base.app = app;
        }

        base
    }
}

// ── 观测 ────────────────────────────────────────────────────────────────────

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
    pub has_vital: bool,
    pub vital_is_active: bool,
    pub vital_dir_x: f32,
    pub vital_dir_neg_x: f32,
    pub vital_dir_z: f32,
    pub vital_dir_neg_z: f32,
}

impl FioraVsRivenObs {
    /// 转换为强化学习策略网络输入向量。
    pub fn to_vector(&self) -> Vec<f32> {
        let rel_x = self.fiora_pos.x - self.riven_pos.x;
        let rel_z = self.fiora_pos.z - self.riven_pos.z;

        vec![
            // 破绽四方位 (4维)
            self.vital_dir_x,
            self.vital_dir_neg_x,
            self.vital_dir_z,
            self.vital_dir_neg_z,
            // 破绽状态 (2维：是否存在、是否已激活)
            if self.has_vital { 1.0 } else { 0.0 },
            if self.vital_is_active { 1.0 } else { 0.0 },
            // 剑姬相对于瑞雯的相对位置与距离 (3维，归一化/OBS_DISTANCE_SCALE)
            rel_x / OBS_DISTANCE_SCALE,
            rel_z / OBS_DISTANCE_SCALE,
            self.distance / OBS_DISTANCE_SCALE,
        ]
    }

    pub fn dim() -> usize {
        9
    }

    pub fn to_payload(&self) -> lol_rl_protocol::ObsFeaturePayload {
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

        lol_rl_protocol::ObsFeaturePayload {
            fiora_hp_pct: if self.fiora_max_hp > 0.0 {
                self.fiora_hp / self.fiora_max_hp
            } else {
                1.0
            },
            riven_hp_pct: if self.riven_max_hp > 0.0 {
                self.riven_hp / self.riven_max_hp
            } else {
                1.0
            },
            distance: self.distance,
            q_ready: self.q_ready,
            w_ready: self.w_ready,
            e_ready: self.e_ready,
            r_ready: self.r_ready,
            has_vital: self.has_vital,
            vital_is_active: self.vital_is_active,
            vital_direction: vital_dir,
            ..Default::default()
        }
    }
}

// ── 事件追踪 ────────────────────────────────────────────────────────────────

/// 普攻事件追踪：攻击命中（EventAttackEnd）与攻击就绪（EventAttackReady）。
#[derive(Resource, Default, Debug, Clone)]
pub struct AttackEventTracker {
    pub attack_hit: bool,
    pub attack_ready: bool,
}

pub fn on_attack_end(
    _trigger: On<lol_core::attack::EventAttackEnd>,
    mut tracker: ResMut<AttackEventTracker>,
) {
    tracker.attack_hit = true;
}

pub fn on_attack_ready(
    _trigger: On<lol_core::attack::EventAttackReady>,
    mut tracker: ResMut<AttackEventTracker>,
) {
    tracker.attack_ready = true;
}

/// 环境中的 Fiora / Riven 实体，供观察者过滤事件来源。
#[derive(Resource)]
pub struct FioraRivenEntities {
    pub fiora: Entity,
    pub riven: Entity,
}

/// 真实破绽击破信号：菲奥娜被动击破要害会对目标造成一次真实伤害
#[derive(Resource, Default, Debug, Clone)]
pub struct VitalBreakTracker {
    pub hit: bool,
}

pub fn on_vital_break_damage(
    trigger: On<EventDamageCreate>,
    entities: Res<FioraRivenEntities>,
    mut tracker: ResMut<VitalBreakTracker>,
) {
    if trigger.source == entities.fiora && trigger.damage_type == DamageType::True {
        tracker.hit = true;
    }
}

/// 注册两个环境共用的资源与观察者（在 `App::finish()` 之前调用）。
pub fn add_common_observers(app: &mut App) {
    app.init_resource::<AttackEventTracker>();
    app.init_resource::<VitalBreakTracker>();
    app.add_observer(on_attack_end);
    app.add_observer(on_attack_ready);
    app.add_observer(on_vital_break_damage);
    app.add_observer(on_character_ready_set_skill_levels);
}

/// 角色配置写入完成后同步设置 Q/W/E/R 技能等级。
pub fn on_character_ready_set_skill_levels(
    trigger: On<Add, CharacterReady>,
    q_skills: Query<&Skills>,
    mut q_skill: Query<&mut Skill>,
    levels_res: Option<Res<ChampionInitialSkillLevels>>,
) {
    let entity = trigger.entity;
    let Ok(skills) = q_skills.get(entity) else {
        return;
    };
    let skill_entities = skills.to_vec();
    if skill_entities.len() < 4 {
        return;
    }
    let levels = levels_res.map(|r| r.0).unwrap_or([3, 1, 1, 1]);
    for (idx, level) in levels.into_iter().enumerate() {
        if let Ok(mut skill) = q_skill.get_mut(skill_entities[idx]) {
            skill.level = level;
        }
    }
}

// ── 世界读写 ────────────────────────────────────────────────────────────────

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
                    let cd = world.get::<CoolDown>(s_entity);
                    let recast = world.get::<SkillRecastWindow>(s_entity);
                    match cd {
                        Some(c) => is_skill_ready(c, recast),
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

/// 在世界中重新生成 Fiora 和 Riven 实体
pub fn spawn_champions_world(
    world: &mut World,
    fiora_config_handle: Handle<DynamicWorld>,
    riven_config_handle: Handle<DynamicWorld>,
    fiora_skin_handle: Option<Handle<DynamicWorld>>,
    riven_skin_handle: Option<Handle<DynamicWorld>>,
    initial_fiora_pos: Vec3,
    initial_riven_pos: Vec3,
    render: bool,
) -> (Entity, Entity) {
    let mut fiora_builder = world.spawn((
        Fiora::default(),
        Transform::from_translation(initial_fiora_pos),
        SpawnTransform(Transform::from_translation(initial_fiora_pos)),
        WaitCharacterReady,
        Team::Order,
        ConfigCharacterRecord {
            character_record: fiora_config_handle,
        },
        Health::new(500.0),
        Armor(35.0),
        Movement { speed: 345.0 },
    ));

    if render {
        if let Some(skin) = fiora_skin_handle {
            fiora_builder.insert((SelfPlayer, Focus, ConfigSkin { skin }));
        }
    }

    let fiora = fiora_builder.id();

    let mut riven_builder = world.spawn((
        Riven::default(),
        Transform::from_translation(initial_riven_pos),
        SpawnTransform(Transform::from_translation(initial_riven_pos)),
        WaitCharacterReady,
        Team::Chaos,
        ConfigCharacterRecord {
            character_record: riven_config_handle,
        },
        Health::new(500.0),
        Armor(33.0),
        Movement { speed: 340.0 },
    ));

    if render {
        if let Some(skin) = riven_skin_handle {
            riven_builder.insert(ConfigSkin { skin });
        }
    }

    let riven = riven_builder.id();

    (fiora, riven)
}

/// Set skill levels for Fiora and Riven in the Bevy ECS world.
pub fn setup_skill_levels_world(world: &mut World, fiora: Entity, riven: Entity) {
    let levels = world
        .get_resource::<ChampionInitialSkillLevels>()
        .map(|r| r.0)
        .unwrap_or([3, 1, 1, 1]);
    setup_custom_skill_levels_world(world, fiora, riven, levels);
}

/// 使用指定技能等级数组设置 Fiora 与 Riven 的技能等级
pub fn setup_custom_skill_levels_world(
    world: &mut World,
    fiora: Entity,
    riven: Entity,
    levels: [usize; 4],
) {
    for champion in [fiora, riven] {
        if let Some(skills) = world.get::<Skills>(champion) {
            let skill_entities = skills.to_vec();
            for (idx, &level) in levels.iter().enumerate() {
                if idx < skill_entities.len() {
                    if let Some(mut s) = world.get_mut::<Skill>(skill_entities[idx]) {
                        s.level = level;
                    }
                }
            }
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

/// Compute step reward and its breakdown items using the structured RewardModel.
pub fn compute_step_reward(
    prev_riven_hp: f32,
    curr_riven_hp: f32,
    prev_fpos: Vec3,
    curr_fpos: Vec3,
    riven_pos: Vec3,
    is_attack: bool,
    is_vital_break: bool,
    prev_obs: &FioraVsRivenObs,
    elapsed_secs: f32,
) -> (f32, Vec<RewardBreakdownItem>, HashMap<String, f32>) {
    let prev_aligned =
        prev_obs.has_vital && is_position_aligned_with_vital(prev_fpos, riven_pos, prev_obs);
    let curr_aligned =
        prev_obs.has_vital && is_position_aligned_with_vital(curr_fpos, riven_pos, prev_obs);

    let ctx = FioraRewardContext {
        prev_aligned,
        curr_aligned,
        is_vital_break,
        is_attack,
        prev_riven_hp,
        curr_riven_hp,
        elapsed_secs,
    };

    let model = FioraVsRivenRewardModel;
    let (reward, items, vars) = model.evaluate(&ctx);

    let breakdown = items
        .into_iter()
        .map(|it| RewardBreakdownItem {
            name: it.name,
            value: it.value,
        })
        .collect();

    (reward, breakdown, vars)
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
