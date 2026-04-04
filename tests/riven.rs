use std::collections::HashMap;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::time::Duration;

use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use bevy::winit::WinitPlugin;
use league_core::extract::{SpellDataResource, SpellDataValue, SpellObject};
use league_core::grid::{
    JungleQuadrantFlags, MainRegionFlags, NearestLaneFlags, POIFlags, RingFlags, RiverRegionFlags,
    UnknownSRXFlags, VisionPathingFlags,
};
use league_utils::hash_bin;
use lol_config::grid::{ConfigNavigationGrid, ConfigNavigationGridCell};
use lol_config::prop::LoadHashKeyTrait;
use lol_core::team::Team;
use lol_core_render::camera::PluginCamera;
use lol_core_render::particle::PluginParticle;
use moon_lol::buffs::shield_white::BuffShieldWhite;
use moon_lol::core::action::{Action, CommandAction};
use moon_lol::core::animation::PluginAnimation;
use moon_lol::core::base::ability_resource::{AbilityResource, AbilityResourceType};
use moon_lol::core::base::buff::Buffs;
use moon_lol::core::base::level::Level;
use moon_lol::core::damage::{Armor, CommandDamageCreate, Damage, DamageType};
use moon_lol::core::life::Health;
use moon_lol::core::movement::{Movement, MovementState};
use moon_lol::core::navigation::grid::ResourceGrid;
use moon_lol::core::navigation::navigation::{NavigationDebug, NavigationStats, PluginNavigaton};
use moon_lol::core::resource::PluginResource;
use moon_lol::core::skill::{
    get_skill_value, CoolDown, Skill, SkillCooldownMode, SkillOf, SkillPoints, SkillRecastWindow,
    SkillSlot, Skills,
};
use moon_lol::core::skin::PluginSkin;
use moon_lol::core::test_render::{
    attach_skill_test_actor, PluginSkillTestRender, SkillTestActor, SkillTestRenderConfig,
    SkillTestVideoFormat, SkillTestVideoOutput,
};
use moon_lol::entities::barrack::PluginBarrack;
use moon_lol::entities::champion::Champion;
use moon_lol::entities::champions::riven::Riven;
use moon_lol::entities::minion::PluginMinion;
use moon_lol::entities::shpere::PluginDebugSphere;
use moon_lol::entities::turret::PluginTurret;
use moon_lol::ui::PluginUI;
use moon_lol::PluginCore;

const RIVEN_Q_KEY: &str = "Characters/Riven/Spells/RivenTriCleaveAbility/RivenTriCleave";
const RIVEN_W_KEY: &str = "Characters/Riven/Spells/RivenMartyrAbility/RivenMartyr";
const RIVEN_E_KEY: &str = "Characters/Riven/Spells/RivenFeintAbility/RivenFeint";
const RIVEN_R_KEY: &str = "Characters/Riven/Spells/RivenFengShuiEngineAbility/RivenFengShuiEngine";

const EPSILON: f32 = 1e-3;

#[derive(Resource)]
struct RivenHarnessEntities {
    riven: Entity,
    enemy_near: Entity,
    enemy_far: Entity,
    ally_near: Entity,
}

// ── headless 逻辑测试基础设施 ──

struct RivenHarness {
    app: App,
    riven: Entity,
    enemy_near: Entity,
    enemy_far: Entity,
    ally_near: Entity,
    current_frame: u32,
}

impl RivenHarness {
    pub fn run(test_name: &str, max_frames: u32, test_fn: fn(&mut RivenHarness)) {
        if std::env::var("MOON_LOL_RUN_RENDER_TESTS").as_deref() == Ok("1") {
            let skipped = skip_due_to_missing_gpu(|| {
                let mut harness = RivenHarness::new_internal(test_name, true);
                test_fn(&mut harness);
                while harness.current_frame < max_frames + 40 {
                    harness.advance_frame();
                }
            });
            if !skipped {
                return;
            }
        }

        let mut harness = RivenHarness::new_internal(test_name, false);
        test_fn(&mut harness);
    }

    fn new_internal(_test_name: &str, is_render: bool) -> Self {
        let mut app = App::new();

        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            16,
        )));

        let mut plugin_group = PluginCore.build().set(PluginResource {
            game_config_path: "games/riven_render.ron".to_string(),
        });

        plugin_group = plugin_group
            .disable::<PluginBarrack>()
            .disable::<PluginMinion>()
            .disable::<PluginTurret>()
            .disable::<PluginUI>();

        if is_render {
            let output_dir = std::path::PathBuf::from(format!("artifacts/tests/{}", _test_name));
            let _ = std::fs::remove_dir_all(&output_dir);
            app.insert_resource(SkillTestRenderConfig {
                output_dir,
                width: 1280,
                height: 720,
                capture_every_nth_frame: 1,
                max_frames: None,
                spawn_default_scene: false,
                video_output: Some(SkillTestVideoOutput {
                    format: SkillTestVideoFormat::Mp4,
                    fps: 60,
                    file_name: format!("{}.mp4", _test_name),
                }),
                keep_frame_images: false,
            });

            app.add_plugins(DefaultPlugins.build().disable::<WinitPlugin>());
            app.add_plugins(PluginSkillTestRender);
            app.add_plugins(plugin_group);

            app.add_systems(Startup, setup_render_stage);
            app.add_systems(Update, attach_skill_test_actor::<Riven>);
        } else {
            app.add_plugins(MinimalPlugins);
            app.add_plugins(AssetPlugin::default());
            app.add_plugins(bevy::input::InputPlugin);
            app.add_plugins(bevy::state::app::StatesPlugin);
            app.add_plugins(bevy::picking::PickingPlugin);
            app.insert_resource(Time::<Fixed>::from_duration(Duration::from_millis(16)));

            // Disable graphics/audio plugins in logic test to avoid parallel GPU panics
            plugin_group = plugin_group
                .disable::<PluginAnimation>()
                .disable::<PluginSkin>()
                .disable::<PluginParticle>()
                .disable::<PluginNavigaton>()
                .disable::<PluginDebugSphere>()
                .disable::<PluginCamera>();

            app.add_plugins(plugin_group);
        }

        app.init_resource::<NavigationStats>();
        app.init_resource::<NavigationDebug>();
        app.init_asset::<ConfigNavigationGrid>();
        app.init_asset::<SpellObject>();
        app.init_asset::<Image>();
        app.init_asset::<Mesh>();
        app.init_asset::<Shader>();
        app.init_asset::<StandardMaterial>();

        // Grid Mock
        let grid_handle = app
            .world_mut()
            .resource_mut::<Assets<ConfigNavigationGrid>>()
            .add(make_test_grid());
        app.insert_resource(ResourceGrid(grid_handle));

        // Mock SpellObject assets
        {
            let mut spell_objects = app.world_mut().resource_mut::<Assets<SpellObject>>();
            let make_spell_object = || {
                use league_core::extract::{
                    EnumAbilityResourceByCoefficientCalculationPart, EnumGameCalculation,
                    GameCalculation, NamedDataValueCalculationPart,
                };
                let mut calculations = HashMap::new();

                let make_calc = |name: &str| {
                    EnumGameCalculation::GameCalculation(GameCalculation {
                        m_formula_parts: Some(vec![
                            EnumAbilityResourceByCoefficientCalculationPart::NamedDataValueCalculationPart(
                                NamedDataValueCalculationPart {
                                    m_data_value: hash_bin(name),
                                }
                            )
                        ]),
                        m_display_as_percent: None,
                        m_expanded_tooltip_calculation_display: None,
                        m_multiplier: None,
                        m_precision: None,
                        m_simple_tooltip_calculation_display: None,
                        result_modifier: None,
                        tooltip_only: None,
                    })
                };

                calculations.insert(hash_bin("mDamage"), make_calc("mDamage"));
                calculations.insert(hash_bin("TotalDamage"), make_calc("TotalDamage"));
                calculations.insert(hash_bin("FirstSlashDamage"), make_calc("FirstSlashDamage"));
                calculations.insert(hash_bin("ShieldStrength"), make_calc("ShieldStrength"));

                SpellObject {
                    m_spell: Some(SpellDataResource {
                        m_spell_calculations: Some(calculations),
                        data_values: Some(vec![
                            SpellDataValue {
                                m_name: "mDamage".to_string(),
                                m_values: Some(vec![130.0f32; 6]),
                            },
                            SpellDataValue {
                                m_name: "TotalDamage".to_string(),
                                m_values: Some(vec![130.0f32; 6]),
                            },
                            SpellDataValue {
                                m_name: "FirstSlashDamage".to_string(),
                                m_values: Some(vec![130.0f32; 6]),
                            },
                            SpellDataValue {
                                m_name: "ShieldStrength".to_string(),
                                m_values: Some(vec![100.0f32; 6]),
                            },
                        ]),
                        ..default()
                    }),
                    bot_data: None,
                    cc_behavior_data: None,
                    m_buff: None,
                    m_script_name: "".to_string(),
                    object_name: "".to_string(),
                    script: None,
                }
            };
            spell_objects.add_hash(RIVEN_Q_KEY, make_spell_object());
            spell_objects.add_hash(RIVEN_W_KEY, make_spell_object());
            spell_objects.add_hash(RIVEN_E_KEY, make_spell_object());
            spell_objects.add_hash(RIVEN_R_KEY, make_spell_object());
            spell_objects.add_hash(
                "Characters/Riven/Spells/RivenPassiveAbility/RivenPassive",
                make_spell_object(),
            );
        }

        // Initialization system
        app.add_systems(Startup, |mut commands: Commands| {
            let riven = commands
                .spawn((
                    Riven,
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
                    Armor(0.0),
                    Movement { speed: 340.0 },
                    MovementState::default(),
                    SkillTestActor,
                ))
                .id();

            // Link skills
            let add_sk = |commands: &mut Commands, slot: SkillSlot, key: &str, manual: bool| {
                let mut skill = Skill::new(slot, key).with_level(1);
                if manual {
                    skill = skill.with_cooldown_mode(SkillCooldownMode::Manual);
                }
                commands.entity(riven).with_related::<SkillOf>((
                    skill,
                    CoolDown {
                        timer: {
                            let mut t = Timer::from_seconds(10.0, TimerMode::Once);
                            t.set_elapsed(Duration::from_secs(10));
                            t
                        },
                        duration: 10.0,
                    },
                ));
            };
            add_sk(&mut commands, SkillSlot::Q, RIVEN_Q_KEY, true);
            add_sk(&mut commands, SkillSlot::W, RIVEN_W_KEY, false);
            add_sk(&mut commands, SkillSlot::E, RIVEN_E_KEY, false);
            add_sk(&mut commands, SkillSlot::R, RIVEN_R_KEY, false);

            let enemy_near = commands
                .spawn((
                    Champion,
                    Team::Chaos,
                    Transform::from_xyz(100.0, 0.0, 0.0),
                    Health::new(6000.0),
                    Armor(0.0),
                ))
                .id();

            let enemy_far = commands
                .spawn((
                    Champion,
                    Team::Chaos,
                    Transform::from_xyz(420.0, 0.0, 0.0),
                    Health::new(6000.0),
                    Armor(0.0),
                ))
                .id();

            let ally_near = commands
                .spawn((
                    Team::Order,
                    Transform::from_xyz(60.0, 0.0, 0.0),
                    Health::new(6000.0),
                    Armor(0.0),
                ))
                .id();

            commands.insert_resource(RivenHarnessEntities {
                riven,
                enemy_near,
                enemy_far,
                ally_near,
            });
        });

        app.finish();
        app.cleanup();

        // Run startup systems
        app.update();

        let ents = app
            .world()
            .get_resource::<RivenHarnessEntities>()
            .expect("Harness entities failed to initialize");
        let riven = ents.riven;
        let enemy_near = ents.enemy_near;
        let enemy_far = ents.enemy_far;
        let ally_near = ents.ally_near;

        // Setup Riven facing X+
        if let Some(mut transform) = app.world_mut().get_mut::<Transform>(riven) {
            transform.look_to(Vec3::new(1.0, 0.0, 0.0), Vec3::Y);
        }

        // Run settle frames
        for _ in 0..15 {
            app.update();
        }

        Self {
            app,
            riven,
            enemy_near,
            enemy_far,
            ally_near,
            current_frame: 15,
        }
    }

    fn w_damage_key() -> u32 {
        hash_bin("TotalDamage")
    }

    fn advance_frame(&mut self) -> &mut Self {
        self.app.update();
        self.current_frame += 1;
        self
    }

    fn advance(&mut self, seconds: f32) -> &mut Self {
        let ticks = (seconds / 0.016).ceil() as u32;
        for _ in 0..ticks {
            self.advance_frame();
        }
        self
    }

    fn cast_skill(&mut self, index: usize, point: Vec2) -> &mut Self {
        self.app.world_mut().trigger(CommandAction {
            entity: self.riven,
            action: Action::Skill { index, point },
        });
        self.app.update();
        self
    }

    fn apply_damage_to_riven(&mut self, amount: f32) -> &mut Self {
        self.app.world_mut().trigger(CommandDamageCreate {
            entity: self.riven,
            source: self.enemy_near,
            damage_type: DamageType::Physical,
            amount,
        });
        self.app.update();
        self
    }

    fn shield_value(&self) -> Option<f32> {
        let buffs = self.app.world().get::<Buffs>(self.riven)?;
        for buff in buffs.iter() {
            if let Some(shield) = self.app.world().get::<BuffShieldWhite>(buff) {
                return Some(shield.current);
            }
        }
        None
    }

    fn position(&self, entity: Entity) -> Vec3 {
        self.app
            .world()
            .get::<Transform>(entity)
            .expect("transform should exist")
            .translation
    }

    fn health(&self, entity: Entity) -> f32 {
        self.app
            .world()
            .get::<Health>(entity)
            .expect("health should exist")
            .value
    }

    fn mana(&self) -> f32 {
        self.app
            .world()
            .get::<AbilityResource>(self.riven)
            .map(|r| r.value)
            .unwrap_or(0.0)
    }

    fn cooldown_finished(&self, index: usize) -> bool {
        let skills = self
            .app
            .world()
            .get::<Skills>(self.riven)
            .expect("skills should exist");
        let skill_entity = skills[index];
        self.app
            .world()
            .get::<CoolDown>(skill_entity)
            .expect("cooldown should exist")
            .timer
            .is_finished()
    }

    fn spell(&self, index: usize) -> Option<SpellObject> {
        let skills = self.app.world().get::<Skills>(self.riven)?;
        let skill_entity = if index < skills.len() {
            Some(skills[index])
        } else {
            None
        }?;
        let skill = self.app.world().get::<Skill>(skill_entity)?;
        self.app
            .world()
            .resource::<Assets<SpellObject>>()
            .load_hash(&skill.key_spell_object)
            .cloned()
    }

    fn print_skill_logs(&self) {
        use moon_lol::core::skill::SkillCastLog;
        if let Some(log) = self.app.world().get_resource::<SkillCastLog>() {
            for record in &log.0 {
                println!("Skill Cast Record: {:?}", record);
            }
        }
    }
}

fn skip_due_to_missing_gpu(run: impl FnOnce()) -> bool {
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

fn make_test_grid() -> ConfigNavigationGrid {
    let cell = ConfigNavigationGridCell {
        heuristic: 1.0,
        vision_pathing_flags: VisionPathingFlags::Walkable,
        river_region_flags: RiverRegionFlags::NonJungle,
        jungle_quadrant_flags: JungleQuadrantFlags::None,
        main_region_flags: MainRegionFlags::Spawn,
        nearest_lane_flags: NearestLaneFlags::BlueSideTopLane,
        poi_flags: POIFlags::None,
        ring_flags: RingFlags::BlueSpawnToNexus,
        srx_flags: UnknownSRXFlags::Walkable,
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

fn setup_render_stage(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        DirectionalLight {
            illuminance: 20_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.8, -0.7, 0.0)),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(12.0)))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.16, 0.18, 0.22),
            perceptual_roughness: 0.95,
            ..default()
        })),
        Name::new("RenderTestPlatform"),
    ));
}

// ── 测试用例 ──

#[test]
fn riven_q_cycles_through_three_real_stages() {
    RivenHarness::run("riven_q", 180, |harness| {
        harness.cast_skill(0, Vec2::new(140.0, 0.0)).advance(0.4);
        let q_entity = (harness
            .app
            .world()
            .get::<Skills>(harness.riven)
            .expect("Skills missing on Riven"))[0];
        let q_stage = harness
            .app
            .world()
            .get::<SkillRecastWindow>(q_entity)
            .map(|window| window.stage);
        if q_stage != Some(2) {
            harness.print_skill_logs();
        }
        assert_eq!(q_stage, Some(2));
        assert!(harness.cooldown_finished(0));

        harness.cast_skill(0, Vec2::new(140.0, 0.0)).advance(0.4);
        assert_eq!(
            harness
                .app
                .world()
                .get::<SkillRecastWindow>(q_entity)
                .map(|window| window.stage),
            Some(3)
        );
        assert!(harness.cooldown_finished(0));

        harness.cast_skill(0, Vec2::new(140.0, 0.0)).advance(0.1);
        assert!(harness
            .app
            .world()
            .get::<SkillRecastWindow>(q_entity)
            .is_none());
        assert!(!harness.cooldown_finished(0));
        harness.advance(10.1);
        assert!(harness.cooldown_finished(0));

        let q_pos = harness.position(harness.riven);
        assert!(
            q_pos.length() > 5.0,
            "expected q combo movement, got {q_pos:?}"
        );
    });
}

#[test]
fn riven_w_hits_only_enemies_in_range() {
    RivenHarness::run("riven_w", 120, |harness| {
        let expected_damage = get_skill_value(
            &harness.spell(1).expect("W spell missing"),
            RivenHarness::w_damage_key(),
            1,
            |stat| if stat == 2 { 100.0 } else { 0.0 }, // Base AD
        )
        .expect("riven w damage should exist");

        let initial_near = harness.health(harness.enemy_near);
        let initial_far = harness.health(harness.enemy_far);
        let initial_ally = harness.health(harness.ally_near);

        harness.cast_skill(1, Vec2::new(140.0, 0.0));
        harness.advance(0.2); // wait for damage frame

        let near = harness.health(harness.enemy_near);
        let far = harness.health(harness.enemy_far);
        let ally = harness.health(harness.ally_near);
        assert!(
            (initial_near - near - expected_damage).abs() < EPSILON,
            "expected near enemy damage {}, got {}",
            expected_damage,
            initial_near - near
        );
        assert!(
            (far - initial_far).abs() < EPSILON,
            "expected far enemy health unchanged"
        );
        assert!(
            (ally - initial_ally).abs() < EPSILON,
            "expected ally health unchanged"
        );
    });
}

#[test]
fn riven_e_spawns_shield_and_dash_absorbs_damage() {
    RivenHarness::run("riven_e", 120, |harness| {
        harness.cast_skill(2, Vec2::new(140.0, 0.0)).advance(0.4);

        let e_pos = harness.position(harness.riven);
        assert!(
            e_pos.length() > 2.0,
            "expected e dash position, got {e_pos:?}"
        );
        let initial_health = harness.health(harness.riven);
        let shield_val = harness.shield_value().unwrap_or(0.0);
        assert!(
            shield_val > 80.0 && shield_val <= 100.0,
            "Expected shield value near 100, got {}",
            shield_val
        );

        harness.apply_damage_to_riven(60.0);
        assert!((harness.health(harness.riven) - initial_health).abs() < EPSILON);
        let remaining_shield = harness.shield_value().unwrap_or(0.0);
        assert!(remaining_shield > 20.0 && remaining_shield < shield_val); // absorbed damage

        harness.apply_damage_to_riven(50.0);
        assert!(harness.health(harness.riven) < initial_health);
    });
}

#[test]
fn riven_r_starts_cooldown_without_moving_or_damaging() {
    RivenHarness::run("riven_r", 140, |harness| {
        let expected_mana_cost = harness
            .spell(3)
            .expect("R spell missing")
            .m_spell
            .as_ref()
            .and_then(|spell| spell.mana.as_ref())
            .and_then(|mana| mana.first().copied())
            .unwrap_or(0.0);

        let initial_mana = harness.mana();
        let initial_enemy_hp = harness.health(harness.enemy_near);
        let _initial_pos = harness.position(harness.riven).x;

        harness.cast_skill(3, Vec2::new(140.0, 0.0)).advance(0.2);

        assert!((harness.mana() - (initial_mana - expected_mana_cost)).abs() < EPSILON);
        assert!(!harness.cooldown_finished(3));
        assert!(
            harness
                .position(harness.riven)
                .distance(Vec3::new(0.0, 0.0, 0.0))
                < EPSILON
        );
        assert!((harness.health(harness.enemy_near) - initial_enemy_hp).abs() < EPSILON);
    });
}
