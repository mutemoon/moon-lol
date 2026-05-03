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
use std::path::PathBuf;
use std::time::Duration;

use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use bevy::winit::WinitPlugin;
use lol_base::animation::LOLAnimationGraph;
use lol_base::character::{ConfigCharacterRecord, ConfigSkin};
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
use lol_core::navigation::navigation::NavigationDebug;
use lol_core::skill::{CoolDown, Skill, SkillPoints, Skills};
use lol_core::team::Team;
use lol_render::PluginRender;
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

    /// If Render mode but `render_test_guard()` is false, fall back to Headless.
    fn resolve(&self) -> Self {
        match self {
            Self::Render { .. } if !render_test_guard() => {
                eprintln!("falling back to headless mode: MOON_LOL_RUN_RENDER_TESTS not set");
                Self::Headless
            }
            other => *other,
        }
    }
}

// ── Champion-specific config ──

pub struct ChampionHarnessConfig {
    pub champion_dir: &'static str,
    /// Path to the champion's config scene, e.g. `"characters/riven/config.ron"`.
    pub config_path: &'static str,
    /// Path to the skin scene.
    pub skin_path: &'static str,
    /// Add the champion's plugin (e.g. `PluginRiven`) to the `App`.
    pub add_champion_plugin: fn(&mut App),
}

// ── Shared harness ──

pub struct ChampionTestHarness {
    pub app: App,
    pub champion: Entity,
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
        let mut app = App::new();
        app.insert_resource(Time::<Fixed>::from_duration(Duration::from_millis(16)));
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            16,
        )));

        let mode = mode.resolve();

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
        match mode {
            HarnessMode::Headless => {
                app.add_plugins((
                    MinimalPlugins,
                    asset_plugin,
                    bevy::world_serialization::WorldSerializationPlugin,
                ));
            }
            HarnessMode::Render { max_frames } => {
                app.add_plugins(
                    DefaultPlugins
                        .build()
                        .set(asset_plugin)
                        .disable::<WinitPlugin>(),
                );
                app.add_plugins(PluginRender);
                app.add_plugins(PluginSkillTestRender);

                let output_dir = render_output_dir(config.champion_dir);
                let _ = fs::create_dir_all(&output_dir);

                app.insert_resource(NavigationDebug);
                app.insert_resource(SkillTestRenderConfig {
                    output_dir,
                    width: 1280,
                    height: 720,
                    capture_every_nth_frame: 1,
                    max_frames: Some(max_frames),
                    spawn_default_scene: false,
                    video_output: Some(SkillTestVideoOutput {
                        format: SkillTestVideoFormat::Mp4,
                        fps: 60,
                        file_name: format!("{test_name}.mp4"),
                    }),
                    keep_frame_images: false,
                });
            }
        }
        app.add_plugins(lol_core::PluginCore);
        app.insert_resource(lol_base::map::MapPaths::new("test"));

        app.finish();
        app.cleanup();

        let asset_server = app.world().resource::<AssetServer>();
        let config_handle = asset_server.load(config.config_path);
        let skin_handle = asset_server.load(config.skin_path);

        let champion = app
            .world_mut()
            .spawn((
                C::default(),
                ConfigCharacterRecord {
                    character_record: config_handle,
                },
                Team::Order,
                Transform::default(),
            ))
            .id();

        // Only load skin in render mode
        if mode.is_render() {
            app.world_mut()
                .entity_mut(champion)
                .insert(ConfigSkin { skin: skin_handle });
        }

        // Poll until ConfigCharacterRecord is processed
        for i in 0..10 {
            app.update();
            if !app.world().entity(champion).contains::<Health>() {
                println!("第 {} 帧检测到 Health 组件", i);
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
            current_frame: 15,
        }
    }

    // ── helpers ──

    /// Add an enemy champion at the given position.
    pub fn add_enemy(&mut self, position: Vec3) -> Entity {
        let capsule;
        let mat_red;
        let is_render = self.mode.is_render();
        if is_render {
            capsule = Some(
                self.app
                    .world_mut()
                    .resource_mut::<Assets<Mesh>>()
                    .add(Capsule3d::new(0.3, 1.8)),
            );
            mat_red = Some(
                self.app
                    .world_mut()
                    .resource_mut::<Assets<StandardMaterial>>()
                    .add(StandardMaterial {
                        base_color: Color::srgb(0.9, 0.2, 0.2),
                        ..default()
                    }),
            );
        } else {
            capsule = None;
            mat_red = None;
        }
        let mut e = self.app.world_mut().spawn((
            Champion,
            Team::Chaos,
            Transform::from_translation(position),
            Health::new(6000.0),
            lol_core::damage::Armor(0.0),
        ));
        if is_render {
            e.insert((Mesh3d(capsule.unwrap()), MeshMaterial3d(mat_red.unwrap())));
        }
        e.id()
    }

    /// Add an ally at the given position.
    pub fn add_ally(&mut self, position: Vec3) -> Entity {
        let capsule;
        let mat_green;
        let is_render = self.mode.is_render();
        if is_render {
            capsule = Some(
                self.app
                    .world_mut()
                    .resource_mut::<Assets<Mesh>>()
                    .add(Capsule3d::new(0.3, 1.8)),
            );
            mat_green = Some(
                self.app
                    .world_mut()
                    .resource_mut::<Assets<StandardMaterial>>()
                    .add(StandardMaterial {
                        base_color: Color::srgb(0.2, 0.9, 0.3),
                        ..default()
                    }),
            );
        } else {
            capsule = None;
            mat_green = None;
        }
        let mut a = self.app.world_mut().spawn((
            Team::Order,
            Transform::from_translation(position),
            Health::new(6000.0),
            lol_core::damage::Armor(0.0),
        ));
        if is_render {
            a.insert((Mesh3d(capsule.unwrap()), MeshMaterial3d(mat_green.unwrap())));
        }
        a.id()
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

    pub fn apply_damage(&mut self, source: Entity, amount: f32) -> &mut Self {
        self.app.world_mut().trigger(CommandDamageCreate {
            entity: self.champion,
            source,
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

fn setup_app_plugins(app: &mut App, mode: &HarnessMode, test_name: &str, champion_dir: &str) {}

// ── Shared free functions ──

pub fn render_output_dir(champion: &str) -> PathBuf {
    PathBuf::from("assets").join("test_videos").join(champion)
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
