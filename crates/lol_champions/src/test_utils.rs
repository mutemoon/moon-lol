#![cfg(test)]
//! Shared champion test harness.
//!
//! Uses real exported champion assets (`ConfigCharacterRecord`, `ConfigSkin`) so skill data
//! comes from the game files — no manual mock-spell construction needed.
//!
//! # Usage
//!
//! ```ignore
//! use crate::test_utils::*;
//!
//! fn my_config() -> ChampionHarnessConfig { ... }
//!
//! #[test]
//! fn my_skill_test() {
//!     let mut h = ChampionTestHarness::build::<MyChamp>(
//!         "test_name", HarnessMode::Headless, &my_config(),
//!     );
//!     h.cast_skill(0, Vec2::new(100.0, 0.0));
//!     h.advance(0.2);
//! }
//! ```

use std::fs;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::PathBuf;
use std::time::Duration;

use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use bevy::winit::WinitPlugin;
use league_utils::hash_key::HashKey;
use lol_base::character::{ConfigCharacterRecord, ConfigSkin};
use lol_base::grid::{
    ConfigNavigationGrid, ConfigNavigationGridCell, GridFlagsJungleQuadrant, GridFlagsMainRegion,
    GridFlagsNearestLane, GridFlagsPOI, GridFlagsRing, GridFlagsRiverRegion, GridFlagsSRX,
    GridFlagsVisionPathing,
};
use lol_base::prop::LoadHashKeyTrait;
use lol_base::spell::Spell;
use lol_core::action::{Action, CommandAction};
use lol_core::base::ability_resource::{AbilityResource, AbilityResourceType};
use lol_core::base::buff::Buffs;
use lol_core::base::level::Level;
use lol_core::buffs::shield_white::BuffShieldWhite;
use lol_core::damage::{CommandDamageCreate, Damage, DamageType};
use lol_core::entities::champion::Champion;
use lol_core::life::Health;
use lol_core::navigation::grid::ResourceGrid;
use lol_core::skill::{CoolDown, Skill, SkillCooldownMode, SkillPoints, SkillSlot, Skills};
use lol_core::team::Team;
use lol_render::test_render::{
    PluginSkillTestRender, SkillTestRenderConfig, SkillTestVideoFormat, SkillTestVideoOutput,
};

// ── Harness mode ──

#[derive(Clone, Copy, Debug)]
pub enum HarnessMode {
    Headless,
    Render { max_frames: u32 },
}

impl HarnessMode {
    fn max_frames(&self) -> Option<u32> {
        match self {
            Self::Render { max_frames } => Some(*max_frames),
            Self::Headless => None,
        }
    }

    fn is_render(&self) -> bool {
        matches!(self, Self::Render { .. })
    }
}

// ── Champion-specific config ──

pub struct ChampionHarnessConfig {
    pub champion_dir: &'static str,
    /// Path to the champion's config scene, e.g. `"characters/riven/config.ron"` (render mode).
    pub config_path: &'static str,
    /// Path to the skin scene (render mode).
    pub skin_path: &'static str,
    /// Add the champion's plugin (e.g. `PluginRiven`) to the `App`.
    pub add_champion_plugin: fn(&mut App),
    /// Mock spell factory (headless mode — fast, synchronous, no asset IO needed).
    pub make_mock_spell: fn() -> Spell,
    /// Cooldown mode per skill slot (headless mode).
    pub cooldown_mode_for: fn(SkillSlot) -> SkillCooldownMode,
    /// Spell keys for mock registration (headless mode).
    pub spell_keys: SpellKeySet,
}

/// Path-string config for a champion's spell slots (used in headless mock-spell mode).
pub struct SpellKeySet {
    pub q: &'static str,
    pub w: &'static str,
    pub e: &'static str,
    pub r: &'static str,
    pub passive: &'static str,
}

// ── Shared harness ──

pub struct ChampionTestHarness {
    pub app: App,
    pub champion: Entity,
    pub enemy_near: Entity,
    pub enemy_far: Entity,
    pub ally_near: Entity,
    pub current_frame: u32,
    champion_dir: &'static str,
    test_name: String,
    mode: HarnessMode,
}

impl ChampionTestHarness {
    pub fn build<C: Component + Default + Send + Sync + 'static>(
        test_name: &str,
        mode: HarnessMode,
        config: &ChampionHarnessConfig,
    ) -> Self {
        if mode.is_render() {
            Self::build_with_real_assets::<C>(test_name, mode, config)
        } else {
            Self::build_with_mocks::<C>(test_name, mode, config)
        }
    }

    /// Headless: fast synchronous setup with mock spells.
    fn build_with_mocks<C: Component + Default + Send + Sync + 'static>(
        test_name: &str,
        mode: HarnessMode,
        config: &ChampionHarnessConfig,
    ) -> Self {
        let mut app = App::new();
        app.insert_resource(Time::<Fixed>::from_duration(Duration::from_millis(16)));
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            16,
        )));

        setup_app_plugins(&mut app, &mode, test_name, config.champion_dir);
        add_common_plugins_and_init(&mut app, config.add_champion_plugin);

        // Mock spells
        {
            let spell = (config.make_mock_spell)();
            let mut objects = app.world_mut().resource_mut::<Assets<Spell>>();
            objects.add_hash(config.spell_keys.q, spell.clone());
            objects.add_hash(config.spell_keys.w, spell.clone());
            objects.add_hash(config.spell_keys.e, spell.clone());
            objects.add_hash(config.spell_keys.r, spell.clone());
            objects.add_hash(config.spell_keys.passive, spell);
        }

        let q = config.spell_keys.q;
        let w = config.spell_keys.w;
        let e = config.spell_keys.e;
        let r = config.spell_keys.r;
        let cooldown_for = config.cooldown_mode_for;
        app.add_systems(Startup, move |mut commands: Commands| {
            let champion = commands
                .spawn((
                    C::default(),
                    Team::Order,
                    Transform::default(),
                    Health::new(1000.0),
                    AbilityResource {
                        ar_type: AbilityResourceType::Mana,
                        value: 1000.0,
                        max: 1000.0,
                        base: 1000.0,
                        per_level: 0.0,
                        base_static_regen: 0.0,
                        regen_per_level: 0.0,
                    },
                    Level {
                        value: 18,
                        ..default()
                    },
                    SkillPoints(4),
                    Damage(100.0),
                    lol_core::damage::Armor(0.0),
                    lol_core::movement::Movement { speed: 340.0 },
                    lol_core::movement::MovementState::default(),
                ))
                .id();
            for (slot, key) in [
                (SkillSlot::Q, q),
                (SkillSlot::W, w),
                (SkillSlot::E, e),
                (SkillSlot::R, r),
            ] {
                let mut skill = Skill::new(
                    slot,
                    Handle::from(league_utils::hash_key::HashKey::<Spell>::from(key)),
                )
                .with_level(1);
                let m = cooldown_for(slot);
                if m != SkillCooldownMode::AfterCast {
                    skill = skill.with_cooldown_mode(m);
                }
                commands
                    .entity(champion)
                    .with_related::<lol_core::skill::SkillOf>((
                        skill,
                        CoolDown {
                            duration: 10.0,
                            timer: Some({
                                let mut t = Timer::from_seconds(10.0, TimerMode::Once);
                                t.set_elapsed(Duration::from_secs(10));
                                t
                            }),
                        },
                    ));
            }
            commands.insert_resource(HarnessEntities {
                champion,
                enemy_near: Entity::PLACEHOLDER,
                enemy_far: Entity::PLACEHOLDER,
                ally_near: Entity::PLACEHOLDER,
            });
        });

        app.finish();
        app.cleanup();
        app.update();
        let champion = app
            .world()
            .get_resource::<HarnessEntities>()
            .unwrap()
            .champion;
        app.world_mut().remove_resource::<HarnessEntities>();

        // Enemies + ally
        let enemy_near = app
            .world_mut()
            .spawn((
                Champion,
                Team::Chaos,
                Transform::from_xyz(100.0, 0.0, 0.0),
                Health::new(6000.0),
                lol_core::damage::Armor(0.0),
            ))
            .id();
        let enemy_far = app
            .world_mut()
            .spawn((
                Champion,
                Team::Chaos,
                Transform::from_xyz(420.0, 0.0, 0.0),
                Health::new(6000.0),
                lol_core::damage::Armor(0.0),
            ))
            .id();
        let ally_near = app
            .world_mut()
            .spawn((
                Team::Order,
                Transform::from_xyz(60.0, 0.0, 0.0),
                Health::new(6000.0),
                lol_core::damage::Armor(0.0),
            ))
            .id();

        if let Some(mut t) = app.world_mut().get_mut::<Transform>(champion) {
            t.look_to(Vec3::X, Vec3::Y);
        }
        for _ in 0..15 {
            app.update();
        }

        Self {
            app,
            champion_dir: config.champion_dir,
            test_name: test_name.into(),
            mode,
            champion,
            enemy_near,
            enemy_far,
            ally_near,
            current_frame: 15,
        }
    }

    /// Render: real ConfigCharacterRecord + ConfigSkin via AssetServer.
    fn build_with_real_assets<C: Component + Default + Send + Sync + 'static>(
        test_name: &str,
        mode: HarnessMode,
        config: &ChampionHarnessConfig,
    ) -> Self {
        let mut app = App::new();
        app.insert_resource(Time::<Fixed>::from_duration(Duration::from_millis(16)));
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            16,
        )));

        setup_app_plugins(&mut app, &mode, test_name, config.champion_dir);
        add_common_plugins_and_init(&mut app, config.add_champion_plugin);

        let config_handle = app
            .world()
            .resource::<AssetServer>()
            .load(config.config_path);
        let skin_handle = app.world().resource::<AssetServer>().load(config.skin_path);

        let champion = app
            .world_mut()
            .spawn((
                C::default(),
                ConfigCharacterRecord {
                    character_record: config_handle,
                },
                ConfigSkin { skin: skin_handle },
                Team::Order,
                Transform::default(),
            ))
            .id();

        // Poll until ConfigCharacterRecord is processed
        for _ in 0..1000 {
            app.update();
            if !app
                .world()
                .entity(champion)
                .contains::<ConfigCharacterRecord>()
            {
                break;
            }
        }
        assert!(
            !app.world()
                .entity(champion)
                .contains::<ConfigCharacterRecord>(),
            "config load failed: {}",
            config.config_path
        );
        for _ in 0..500 {
            app.update();
            if !app.world().entity(champion).contains::<ConfigSkin>() {
                break;
            }
        }
        assert!(
            !app.world().entity(champion).contains::<ConfigSkin>(),
            "skin load failed: {}",
            config.skin_path
        );

        // Override stats
        let lvl = app.world().entity(champion).get::<Level>().cloned();
        app.world_mut().entity_mut(champion).insert((
            Level {
                value: 18,
                ..lvl.unwrap_or_default()
            },
            Health::new(1000.0),
            AbilityResource {
                ar_type: AbilityResourceType::Mana,
                value: 1000.0,
                max: 1000.0,
                base: 1000.0,
                per_level: 0.0,
                base_static_regen: 0.0,
                regen_per_level: 0.0,
            },
            Damage(100.0),
            lol_core::damage::Armor(0.0),
            SkillPoints(4),
        ));

        // Set initial cooldowns
        let skill_entities: Vec<Entity> = {
            let skills = app
                .world()
                .get::<Skills>(champion)
                .expect("Skills missing after config load");
            (0..skills.len()).map(|i| skills[i]).collect()
        };
        for se in skill_entities {
            app.world_mut().entity_mut(se).insert(CoolDown {
                duration: 10.0,
                timer: Some({
                    let mut t = Timer::from_seconds(10.0, TimerMode::Once);
                    t.set_elapsed(Duration::from_secs(10));
                    t
                }),
            });
        }

        app.world_mut().entity_mut(champion).insert((
            lol_core::movement::Movement { speed: 340.0 },
            lol_core::movement::MovementState::default(),
        ));

        // Enemies + ally with render meshes
        let is_render = mode.is_render();

        // Pre-create mesh handles via temporary WorldMut calls
        let capsule = is_render.then(|| {
            app.world_mut()
                .resource_mut::<Assets<Mesh>>()
                .add(Capsule3d::new(0.3, 1.8))
        });
        let mat_red = is_render.then(|| {
            app.world_mut()
                .resource_mut::<Assets<StandardMaterial>>()
                .add(StandardMaterial {
                    base_color: Color::srgb(0.9, 0.2, 0.2),
                    ..default()
                })
        });
        let mat_green = is_render.then(|| {
            app.world_mut()
                .resource_mut::<Assets<StandardMaterial>>()
                .add(StandardMaterial {
                    base_color: Color::srgb(0.2, 0.9, 0.3),
                    ..default()
                })
        });

        let enemy_near = {
            let mut e = app.world_mut().spawn((
                Champion,
                Team::Chaos,
                Transform::from_xyz(100.0, 0.0, 0.0),
                Health::new(6000.0),
                lol_core::damage::Armor(0.0),
            ));
            if let (Some(c), Some(r)) = (&capsule, &mat_red) {
                e.insert((Mesh3d(c.clone()), MeshMaterial3d(r.clone())));
            }
            e.id()
        };
        let enemy_far = {
            let mut e = app.world_mut().spawn((
                Champion,
                Team::Chaos,
                Transform::from_xyz(420.0, 0.0, 0.0),
                Health::new(6000.0),
                lol_core::damage::Armor(0.0),
            ));
            if let (Some(c), Some(r)) = (&capsule, &mat_red) {
                e.insert((Mesh3d(c.clone()), MeshMaterial3d(r.clone())));
            }
            e.id()
        };
        let ally_near = {
            let mut a = app.world_mut().spawn((
                Team::Order,
                Transform::from_xyz(60.0, 0.0, 0.0),
                Health::new(6000.0),
                lol_core::damage::Armor(0.0),
            ));
            if let (Some(c), Some(g)) = (&capsule, &mat_green) {
                a.insert((Mesh3d(c.clone()), MeshMaterial3d(g.clone())));
            }
            a.id()
        };

        // Render-only: light + platform
        if is_render {
            let platform_mesh = app
                .world_mut()
                .resource_mut::<Assets<Mesh>>()
                .add(Plane3d::new(Vec3::Y, Vec2::splat(12.0)));
            let platform_mat = app
                .world_mut()
                .resource_mut::<Assets<StandardMaterial>>()
                .add(StandardMaterial {
                    base_color: Color::srgb(0.16, 0.18, 0.22),
                    perceptual_roughness: 0.95,
                    ..default()
                });

            app.world_mut().spawn((
                DirectionalLight {
                    illuminance: 20_000.0,
                    shadow_maps_enabled: true,
                    ..default()
                },
                Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.8, -0.7, 0.0)),
            ));
            app.world_mut().spawn((
                Mesh3d(platform_mesh),
                MeshMaterial3d(platform_mat),
                Name::new("RenderTestPlatform"),
            ));
        }

        // Face X+
        if let Some(mut transform) = app.world_mut().get_mut::<Transform>(champion) {
            transform.look_to(Vec3::new(1.0, 0.0, 0.0), Vec3::Y);
        }

        // Settle frames
        for _ in 0..15 {
            app.update();
        }

        Self {
            app,
            champion_dir: config.champion_dir,
            test_name: test_name.to_string(),
            mode,
            champion,
            enemy_near,
            enemy_far,
            ally_near,
            current_frame: 15,
        }
    }

    // ── time ──

    pub fn advance_frame(&mut self) -> &mut Self {
        self.app.update();
        self.current_frame += 1;
        self
    }

    pub fn advance(&mut self, seconds: f32) -> &mut Self {
        let ticks = (seconds / 0.016).ceil() as u32;
        for _ in 0..ticks {
            self.advance_frame();
        }
        self
    }

    // ── input ──

    pub fn cast_skill(&mut self, index: usize, point: Vec2) -> &mut Self {
        self.app.world_mut().trigger(CommandAction {
            entity: self.champion,
            action: Action::Skill { index, point },
        });
        self.app.update();
        self
    }

    pub fn apply_damage(&mut self, amount: f32) -> &mut Self {
        self.app.world_mut().trigger(CommandDamageCreate {
            entity: self.champion,
            source: self.enemy_near,
            damage_type: DamageType::Physical,
            amount,
        });
        self.app.update();
        self
    }

    // ── queries ──

    pub fn shield_value(&self) -> Option<f32> {
        let buffs = self.app.world().get::<Buffs>(self.champion)?;
        for buff in buffs.iter() {
            if let Some(shield) = self.app.world().get::<BuffShieldWhite>(buff) {
                return Some(shield.current);
            }
        }
        None
    }

    pub fn position(&self, entity: Entity) -> Vec3 {
        self.app
            .world()
            .get::<Transform>(entity)
            .expect("transform should exist")
            .translation
    }

    pub fn health(&self, entity: Entity) -> f32 {
        self.app
            .world()
            .get::<Health>(entity)
            .expect("health should exist")
            .value
    }

    pub fn mana(&self) -> f32 {
        self.app
            .world()
            .get::<AbilityResource>(self.champion)
            .map(|r| r.value)
            .unwrap_or(0.0)
    }

    pub fn cooldown_finished(&self, index: usize) -> bool {
        let skills = self
            .app
            .world()
            .get::<Skills>(self.champion)
            .expect("skills should exist");
        let skill_entity = skills[index];
        self.app
            .world()
            .get::<CoolDown>(skill_entity)
            .expect("cooldown state should exist")
            .timer
            .as_ref()
            .map_or(true, |t| t.is_finished())
    }

    pub fn spell(&self, index: usize) -> Option<&Spell> {
        let skills = self.app.world().get::<Skills>(self.champion)?;
        let skill_entity = if index < skills.len() {
            Some(skills[index])
        } else {
            None
        }?;
        let skill = self.app.world().get::<Skill>(skill_entity)?;
        self.app
            .world()
            .resource::<Assets<Spell>>()
            .load_hash(skill.spell.id())
    }

    pub fn print_skill_logs(&self) {
        use lol_core::skill::SkillCastLog;
        if let Some(log) = self.app.world().get_resource::<SkillCastLog>() {
            for record in &log.0 {
                println!("Skill Cast Record: {:?}", record);
            }
        }
    }

    /// Pad frames and produce the video file.  No-op in headless mode.
    pub fn finish(&mut self) {
        let Some(max_frames) = self.mode.max_frames() else {
            return;
        };

        while self.current_frame < max_frames {
            self.advance_frame();
        }

        for _ in 0..40 {
            self.advance_frame();
        }

        let video = render_output_dir(self.champion_dir).join(format!("{}.mp4", self.test_name));
        assert!(
            video.is_file(),
            "expected capture video at {}",
            video.display()
        );
    }
}

// ── Internal helpers ──

fn add_common_plugins_and_init(app: &mut App, add_champion_plugin: fn(&mut App)) {
    app.add_plugins(lol_core::action::PluginAction);
    app.add_plugins(lol_core::cooldown::PluginCooldown);
    app.add_plugins(lol_core::damage::PluginDamage);
    app.add_plugins(lol_core::life::PluginLife);
    app.add_plugins(lol_core::movement::PluginMovement);
    app.add_plugins(lol_core::skill::PluginSkill);
    app.add_plugins(lol_core::character::PluginCharacter);
    add_champion_plugin(app);
    app.init_resource::<lol_core::navigation::navigation::NavigationStats>();
    app.init_resource::<lol_core::navigation::navigation::NavigationDebugState>();
    app.init_asset::<bevy::prelude::DynamicWorld>();
    app.init_asset::<ConfigNavigationGrid>();
    app.init_asset::<Spell>();
    app.init_asset::<Image>();
    app.init_asset::<Mesh>();
    app.init_asset::<Shader>();
    app.init_asset::<StandardMaterial>();
    app.finish();
    app.cleanup();
    let grid_handle = app
        .world_mut()
        .resource_mut::<Assets<ConfigNavigationGrid>>()
        .add(make_test_grid());
    app.insert_resource(ResourceGrid(grid_handle));
}

#[derive(Resource)]
struct HarnessEntities {
    champion: Entity,
    enemy_near: Entity,
    enemy_far: Entity,
    ally_near: Entity,
}

fn setup_app_plugins(app: &mut App, mode: &HarnessMode, test_name: &str, champion_dir: &str) {
    match mode {
        HarnessMode::Headless => {
            app.add_plugins(MinimalPlugins);
            app.add_plugins(AssetPlugin::default());
            app.add_plugins(bevy::input::InputPlugin);
            app.add_plugins(bevy::state::app::StatesPlugin);
            app.add_plugins(bevy::picking::PickingPlugin);
            // ScenePlugin is needed for loading .ron files as DynamicWorld assets
            app.add_plugins(bevy::scene::ScenePlugin);
        }
        HarnessMode::Render { max_frames } => {
            let output_dir = render_output_dir(champion_dir);
            let _ = fs::create_dir_all(&output_dir);
            app.insert_resource(SkillTestRenderConfig {
                output_dir,
                width: 1280,
                height: 720,
                capture_every_nth_frame: 1,
                max_frames: Some(*max_frames),
                spawn_default_scene: false,
                video_output: Some(SkillTestVideoOutput {
                    format: SkillTestVideoFormat::Mp4,
                    fps: 60,
                    file_name: format!("{test_name}.mp4"),
                }),
                keep_frame_images: false,
            });
            app.insert_resource(lol_base::map::MapPaths::default());
            app.add_plugins(DefaultPlugins.build().disable::<WinitPlugin>());
            app.add_plugins(PluginSkillTestRender);
            app.add_plugins(bevy::input::InputPlugin);
            app.add_plugins(bevy::state::app::StatesPlugin);
            app.add_plugins(bevy::picking::PickingPlugin);
        }
    }
}

// ── Shared free functions ──

pub fn render_output_dir(champion: &str) -> PathBuf {
    PathBuf::from("assets").join("test_videos").join(champion)
}

pub fn make_test_grid() -> ConfigNavigationGrid {
    let cell = ConfigNavigationGridCell {
        heuristic: 1.0,
        vision_pathing_flags: GridFlagsVisionPathing::Walkable,
        river_region_flags: GridFlagsRiverRegion::NonJungle,
        jungle_quadrant_flags: GridFlagsJungleQuadrant::None,
        main_region_flags: GridFlagsMainRegion::Spawn,
        nearest_lane_flags: GridFlagsNearestLane::BlueSideTopLane,
        poi_flags: GridFlagsPOI::None,
        ring_flags: GridFlagsRing::BlueSpawnToNexus,
        srx_flags: GridFlagsSRX::Walkable,
    };

    ConfigNavigationGrid {
        min_position: Vec2::new(-2000.0, -2000.0),
        cell_size: 50.0,
        x_len: 100,
        y_len: 100,
        cells: vec![vec![cell; 100]; 100],
        height_x_len: 2,
        height_y_len: 2,
        height_samples: vec![vec![0.0; 2]; 2],
        occupied_cells: Default::default(),
        exclude_cells: Default::default(),
    }
}

pub fn skip_due_to_missing_gpu(run: impl FnOnce()) -> bool {
    match catch_unwind(AssertUnwindSafe(run)) {
        Ok(()) => false,
        Err(payload) => {
            let message = if let Some(message) = payload.downcast_ref::<String>() {
                message.as_str()
            } else if let Some(message) = payload.downcast_ref::<&str>() {
                message
            } else {
                ""
            };
            if message.contains("Unable to find a GPU") {
                eprintln!("skipping render test: no GPU available");
                true
            } else {
                std::panic::resume_unwind(payload);
            }
        }
    }
}

pub fn should_run_render_tests() -> bool {
    std::env::var("MOON_LOL_RUN_RENDER_TESTS").as_deref() == Ok("1")
}

pub fn render_test_guard() -> bool {
    if !should_run_render_tests() {
        eprintln!("skipping render test: MOON_LOL_RUN_RENDER_TESTS not set");
        return false;
    }
    true
}

/// Run a render test case with GPU guard + env-var gate.
pub fn run_render_test(
    build: impl FnOnce() -> ChampionTestHarness,
    run: impl FnOnce(&mut ChampionTestHarness),
) {
    if !render_test_guard() {
        return;
    }
    let skipped = skip_due_to_missing_gpu(|| {
        let mut harness = build();
        run(&mut harness);
        harness.finish();
    });
    if skipped {
        // GPU not available
    }
}
