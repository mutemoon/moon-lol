use std::env::var;
use std::path::PathBuf;

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
use lol_champions::fiora::{Fiora, PluginFiora};
use lol_champions::riven::{PluginRiven, Riven};
use lol_core::action::{Action, CommandAction};
use lol_core::character::{CharacterReady, SpawnTransform};
use lol_core::damage::Armor;
use lol_core::game::{GameState, WaitCharacterReady};
use lol_core::life::Health;
use lol_core::movement::Movement;
use lol_core::navigation::navigation::NavigationDebug;
use lol_core::skill::{Skill, Skills};
use lol_core::team::Team;
use lol_render::controller::SelfPlayer;

use crate::traits::{EnvConfig, RenderMode};

// ── 英雄插件与生成规格 ───────────────────────────────────────────────────────

pub type ChampionSpawner = Box<dyn Fn(&mut World, &AssetServer, bool) -> Entity + Send + Sync>;

pub struct ChampionPluginSpec {
    pub name: &'static str,
    pub team: Team,
    pub initial_pos: Vec3,
    pub initial_skill_levels: [usize; 4],
    pub is_player_focus: bool,
    pub plugin_fn: fn(&mut App),
    pub spawner: ChampionSpawner,
}

/// 预设 Fiora 英雄规格
pub fn fiora_champion_spec(
    team: Team,
    initial_pos: Vec3,
    initial_skill_levels: [usize; 4],
    is_player_focus: bool,
) -> ChampionPluginSpec {
    ChampionPluginSpec {
        name: "fiora",
        team,
        initial_pos,
        initial_skill_levels,
        is_player_focus,
        plugin_fn: |app| {
            app.add_plugins(PluginFiora);
        },
        spawner: Box::new(move |world, asset_server, render| {
            let config_handle = asset_server.load::<DynamicWorld>("characters/fiora/config.ron");
            let skin_handle = if render && is_player_focus {
                Some(asset_server.load::<DynamicWorld>("characters/fiora/skins/skin0.ron"))
            } else {
                None
            };

            let mut builder = world.spawn((
                Fiora::default(),
                Transform::from_translation(initial_pos),
                SpawnTransform(Transform::from_translation(initial_pos)),
                WaitCharacterReady,
                team,
                ConfigCharacterRecord {
                    character_record: config_handle,
                },
                Health::new(500.0),
                Armor(35.0),
                Movement { speed: 345.0 },
            ));

            if render && is_player_focus {
                if let Some(skin) = skin_handle {
                    builder.insert((SelfPlayer, Focus, ConfigSkin { skin }));
                }
            }

            builder.id()
        }),
    }
}

/// 预设 Riven 英雄规格
pub fn riven_champion_spec(
    team: Team,
    initial_pos: Vec3,
    initial_skill_levels: [usize; 4],
    is_player_focus: bool,
) -> ChampionPluginSpec {
    ChampionPluginSpec {
        name: "riven",
        team,
        initial_pos,
        initial_skill_levels,
        is_player_focus,
        plugin_fn: |app| {
            app.add_plugins(PluginRiven);
        },
        spawner: Box::new(move |world, asset_server, render| {
            let config_handle = asset_server.load::<DynamicWorld>("characters/Riven/config.ron");
            let skin_handle = if render {
                Some(asset_server.load::<DynamicWorld>("characters/Riven/skins/skin0.ron"))
            } else {
                None
            };

            let mut builder = world.spawn((
                Riven::default(),
                Transform::from_translation(initial_pos),
                SpawnTransform(Transform::from_translation(initial_pos)),
                WaitCharacterReady,
                team,
                ConfigCharacterRecord {
                    character_record: config_handle,
                },
                Health::new(500.0),
                Armor(33.0),
                Movement { speed: 340.0 },
            ));

            if render {
                if is_player_focus {
                    if let Some(skin) = skin_handle {
                        builder.insert((SelfPlayer, Focus, ConfigSkin { skin }));
                    }
                } else if let Some(skin) = skin_handle {
                    builder.insert(ConfigSkin { skin });
                }
            }

            builder.id()
        }),
    }
}

// ── 通用环境宿主 ─────────────────────────────────────────────────────────────

const WARMUP_TICKS_PER_UPDATE: u32 = 16;

/// 执行通用 App 预热（推进指定秒数的 App update，并执行状态钩子）
pub fn run_warmup_on_app(
    app: &mut App,
    champions: &[Entity],
    warmup_secs: f32,
    hooks: &[fn(&[Entity], &mut World)],
) {
    if warmup_secs <= 0.0 {
        return;
    }
    let mut remaining = (warmup_secs * 64.0).round() as u32;
    while remaining > 0 {
        let ticks = remaining.min(WARMUP_TICKS_PER_UPDATE);
        remaining -= ticks;
        app.world_mut()
            .insert_resource(TimeUpdateStrategy::FixedTimesteps(ticks));
        app.update();
    }
    app.world_mut()
        .insert_resource(TimeUpdateStrategy::FixedTimesteps(1));
    for hook in hooks {
        hook(champions, app.world_mut());
    }
}

/// 设置英雄实体的技能等级
pub fn setup_champion_skill_levels(world: &mut World, champion: Entity, levels: [usize; 4]) {
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

/// 通用强化学习环境 ECS 引擎基底
pub struct LolBaseEnv {
    pub app: App,
    pub champions: Vec<Entity>,
    pub step_count: usize,
    pub max_steps: usize,
    pub map_name: String,
    pub enable_barrack: bool,
    pub warmup_secs: f32,
    pub render_mode: RenderMode,
    pub skill_levels: Vec<[usize; 4]>,
    pub on_ready_hooks: Vec<fn(&[Entity], &mut World)>,
    pub on_reset_hooks: Vec<fn(&[Entity], &mut World)>,
}

impl LolBaseEnv {
    pub fn builder(config: EnvConfig, default_max_steps: usize) -> LolBaseEnvBuilder {
        LolBaseEnvBuilder::new(config, default_max_steps)
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

    pub fn champions(&self) -> &[Entity] {
        &self.champions
    }

    pub fn champion(&self, idx: usize) -> Entity {
        self.champions[idx]
    }

    pub fn fiora(&self) -> Entity {
        self.champions[0]
    }

    pub fn riven(&self) -> Entity {
        self.champions[1]
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

    pub fn on_assets_ready(&mut self, app: &mut App) {
        for (i, &entity) in self.champions.iter().enumerate() {
            if let Some(&levels) = self.skill_levels.get(i) {
                setup_champion_skill_levels(app.world_mut(), entity, levels);
            }
        }
        for hook in &self.on_ready_hooks {
            hook(&self.champions, app.world_mut());
        }
        run_warmup_on_app(app, &self.champions, self.warmup_secs, &self.on_ready_hooks);
    }

    pub fn reset_base(&mut self) {
        let mut app = std::mem::replace(&mut self.app, App::new());
        self.reset_app(&mut app);
        self.app = app;
    }

    pub fn reset_app(&mut self, app: &mut App) -> &[Entity] {
        crate::visual_runner::unpause_virtual_time(app.world_mut());

        if let Some(&primary) = self.champions.first() {
            app.world_mut().trigger(CommandAction {
                entity: primary,
                action: Action::Reset,
            });
        }
        app.update();

        if let Some(mut tracker) = app.world_mut().get_resource_mut::<crate::fiora_riven_common::AttackEventTracker>() {
            tracker.attack_hit = false;
            tracker.attack_ready = false;
        }

        for (i, &entity) in self.champions.iter().enumerate() {
            if let Some(&levels) = self.skill_levels.get(i) {
                setup_champion_skill_levels(app.world_mut(), entity, levels);
            }
        }

        for hook in &self.on_reset_hooks {
            hook(&self.champions, app.world_mut());
        }

        run_warmup_on_app(app, &self.champions, self.warmup_secs, &self.on_reset_hooks);
        self.step_count = 0;
        &self.champions
    }

    pub fn is_assets_loaded(&self, world: &World) -> bool {
        world
            .get_resource::<State<GameState>>()
            .is_some_and(|s| *s.get() == GameState::Playing)
    }
}

// ── 环境构造器 ───────────────────────────────────────────────────────────────

pub struct LolBaseEnvBuilder {
    pub config: EnvConfig,
    pub default_max_steps: usize,
    pub window_title: String,
    pub map_name: String,
    pub enable_barrack: bool,
    pub enable_log: bool,
    pub enable_navigation: bool,
    pub warmup_secs: f32,
    pub champions: Vec<ChampionPluginSpec>,
    pub app_plugins: Vec<fn(&mut App)>,
    pub extra_observers: Vec<fn(&mut App)>,
    pub on_ready_hooks: Vec<fn(&[Entity], &mut World)>,
    pub on_reset_hooks: Vec<fn(&[Entity], &mut World)>,
}

impl LolBaseEnvBuilder {
    pub fn new(config: EnvConfig, default_max_steps: usize) -> Self {
        Self {
            config,
            default_max_steps,
            window_title: "LOL RL Environment".to_string(),
            map_name: "solo".to_string(),
            enable_barrack: false,
            enable_log: true,
            enable_navigation: true,
            warmup_secs: 0.0,
            champions: Vec::new(),
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

    pub fn enable_log(mut self, enable: bool) -> Self {
        self.enable_log = enable;
        self
    }

    pub fn enable_navigation(mut self, enable: bool) -> Self {
        self.enable_navigation = enable;
        self
    }

    pub fn warmup_secs(mut self, secs: f32) -> Self {
        self.warmup_secs = secs;
        self
    }

    pub fn add_champion(mut self, champion_spec: ChampionPluginSpec) -> Self {
        self.champions.push(champion_spec);
        self
    }

    pub fn with_plugin(mut self, plugin_fn: fn(&mut App)) -> Self {
        self.app_plugins.push(plugin_fn);
        self
    }

    pub fn with_observer(mut self, observer_fn: fn(&mut App)) -> Self {
        self.extra_observers.push(observer_fn);
        self
    }

    pub fn on_ready(mut self, hook: fn(&[Entity], &mut World)) -> Self {
        self.on_ready_hooks.push(hook);
        self
    }

    pub fn on_reset(mut self, hook: fn(&[Entity], &mut World)) -> Self {
        self.on_reset_hooks.push(hook);
        self
    }

    pub fn build(self) -> LolBaseEnv {
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
            let mut core_plugins = lol_core::PluginCore.build();
            if !self.enable_barrack {
                core_plugins = core_plugins.disable::<lol_core::PluginBarrack>();
            }
            if !self.enable_log {
                core_plugins = core_plugins.disable::<lol_core::log::PluginLog>();
            }
            if !self.enable_navigation {
                core_plugins =
                    core_plugins.disable::<lol_core::navigation::navigation::PluginNavigaton>();
            }
            app.add_plugins(core_plugins);
            app.add_plugins(lol_particle::PluginParticle);
        } else {
            app.add_plugins((
                MinimalPlugins.set(ScheduleRunnerPlugin::run_once()),
                asset_plugin,
                bevy::world_serialization::WorldSerializationPlugin,
                bevy::mesh::MeshPlugin,
                bevy::image::ImagePlugin::default(),
                bevy::scene::ScenePlugin,
            ));
            league_core::register::init_league_asset(&mut app);
            app.init_asset::<StandardMaterial>();
            app.init_asset::<Shader>();
            app.init_asset::<bevy::animation::AnimationClip>();
            app.init_asset::<bevy::animation::graph::AnimationGraph>();
            app.init_asset::<lol_base_render::animation::LOLAnimationGraph>();
            app.init_asset::<lol_base_render::particle::ConfigVfx>();
            app.init_asset::<bevy::prelude::WorldAsset>();
            app.init_asset::<lol_base::audio::ConfigAudio>();
            app.init_asset::<lol_base::spell::Spell>();
            app.init_asset::<lol_base::grid::ConfigNavigationGrid>();
            app.init_asset::<lol_base::item::ConfigItem>();

            let mut core_plugins = lol_core::PluginCore.build();
            if !self.enable_barrack {
                core_plugins = core_plugins.disable::<lol_core::PluginBarrack>();
            }
            if !self.enable_log {
                core_plugins = core_plugins.disable::<lol_core::log::PluginLog>();
            }
            if !self.enable_navigation {
                core_plugins =
                    core_plugins.disable::<lol_core::navigation::navigation::PluginNavigaton>();
            }
            app.add_plugins(core_plugins);
        }

        // 注册各个英雄插件
        for champ_spec in &self.champions {
            (champ_spec.plugin_fn)(&mut app);
        }

        // 注册额外插件
        for plugin in &self.app_plugins {
            plugin(&mut app);
        }

        app.insert_resource(MapPaths::new(&self.map_name));
        app.insert_resource(NavigationDebug);

        app.finish();
        app.cleanup();

        if !render {
            let mut schedules = app.world_mut().resource_mut::<Schedules>();
            for (_, schedule) in schedules.iter_mut() {
                schedule.set_executor(SingleThreadedExecutor::new());
            }
        }

        // 生成英雄实体
        let mut champion_entities = Vec::with_capacity(self.champions.len());
        let mut skill_levels = Vec::with_capacity(self.champions.len());

        let asset_server = app.world().resource::<AssetServer>().clone();
        for champ_spec in &self.champions {
            let entity = (champ_spec.spawner)(app.world_mut(), &asset_server, render);
            champion_entities.push(entity);
            skill_levels.push(champ_spec.initial_skill_levels);
        }

        if champion_entities.len() >= 2 {
            app.world_mut()
                .insert_resource(crate::fiora_riven_common::FioraRivenEntities {
                    fiora: champion_entities[0],
                    riven: champion_entities[1],
                });
        }

        app.init_resource::<crate::fiora_riven_common::AttackEventTracker>();
        app.init_resource::<crate::fiora_riven_common::VitalBreakTracker>();
        app.add_observer(crate::fiora_riven_common::on_attack_end);
        app.add_observer(crate::fiora_riven_common::on_attack_ready);
        app.add_observer(crate::fiora_riven_common::on_vital_break_damage);

        let levels_snapshot = skill_levels.clone();
        let entities_snapshot = champion_entities.clone();
        app.add_observer(
            move |trigger: On<Add, CharacterReady>,
                  q_skills: Query<&Skills>,
                  mut q_skill: Query<&mut Skill>| {
                let entity = trigger.entity;
                if let Some(pos) = entities_snapshot.iter().position(|&e| e == entity) {
                    if let Some(&levels) = levels_snapshot.get(pos) {
                        if let Ok(skills) = q_skills.get(entity) {
                            let skill_entities = skills.to_vec();
                            for (idx, &level) in levels.iter().enumerate() {
                                if idx < skill_entities.len() {
                                    if let Ok(mut s) = q_skill.get_mut(skill_entities[idx]) {
                                        s.level = level;
                                    }
                                }
                            }
                        }
                    }
                }
            },
        );

        for observer in &self.extra_observers {
            observer(&mut app);
        }

        if !render {
            for _ in 0..500 {
                app.update();
                let is_playing = app
                    .world()
                    .get_resource::<State<GameState>>()
                    .is_some_and(|s| *s.get() == GameState::Playing);

                if is_playing {
                    break;
                }
            }
        }

        let mut base = LolBaseEnv {
            app,
            champions: champion_entities,
            step_count: 0,
            max_steps,
            map_name: self.map_name,
            enable_barrack: self.enable_barrack,
            warmup_secs: self.warmup_secs,
            render_mode: self.config.render_mode,
            skill_levels,
            on_ready_hooks: self.on_ready_hooks,
            on_reset_hooks: self.on_reset_hooks,
        };

        if !render {
            let mut app = std::mem::replace(&mut base.app, App::new());
            base.on_assets_ready(&mut app);
            base.app = app;
        }

        base
    }
}
