use std::collections::HashMap;
use std::fs;
use std::time::Duration;

use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use league_core::{
    CharacterRecord, JungleQuadrantFlags, MainRegionFlags, NearestLaneFlags, POIFlags, RingFlags,
    RiverRegionFlags, SpellObject, UnknownSRXFlags, VisionPathingFlags,
};
use league_property::{from_entry, PropFile};
use league_utils::{hash_bin, hash_wad, type_name_to_hash};
use lol_config::{ConfigNavigationGrid, ConfigNavigationGridCell, LoadHashKeyTrait};
use lol_core::Team;
use moon_lol::{
    get_skill_value, skill_slot_from_index, AbilityResource, AbilityResourceType, Action,
    BuffShieldWhite, Buffs, CommandAction, CommandDamageCreate, CoolDown, Damage, DamageType,
    Health, Level, Movement, NavigationDebug, NavigationStats, PluginAction, PluginCooldown,
    PluginDamage, PluginLife, PluginMovement, PluginRiven, PluginRivenQ, PluginRotate,
    PluginShieldWhite, PluginSkill, ResourceGrid, Riven, Skill, SkillCooldownMode, SkillPoints,
    SkillRecastWindow, SkillSlot, Skills,
};

const TEST_FPS: f32 = 30.0;
const EPSILON: f32 = 1e-3;

const RIVEN_Q_KEY: &str = "Characters/Riven/Spells/RivenTriCleaveAbility/RivenTriCleave";
const RIVEN_W_KEY: &str = "Characters/Riven/Spells/RivenMartyrAbility/RivenMartyr";
const RIVEN_E_KEY: &str = "Characters/Riven/Spells/RivenFeintAbility/RivenFeint";
const RIVEN_R_KEY: &str = "Characters/Riven/Spells/RivenFengShuiEngineAbility/RivenFengShuiEngine";
const RIVEN_BIN_PATH: &str = "DATA/Characters/Riven/Riven.bin";

struct RivenHarness {
    app: App,
    riven: Entity,
    enemy_near: Entity,
    enemy_far: Entity,
    ally_near: Entity,
}

impl RivenHarness {
    fn new() -> Self {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(AssetPlugin::default());
        app.add_plugins(PluginAction);
        app.add_plugins(PluginCooldown);
        app.add_plugins(PluginDamage);
        app.add_plugins(PluginLife);
        app.add_plugins(PluginMovement);
        app.add_plugins(PluginRotate);
        app.add_plugins(PluginShieldWhite);
        app.add_plugins(PluginSkill);
        app.add_plugins(PluginRiven);
        app.add_plugins(PluginRivenQ);
        app.init_asset::<CharacterRecord>();
        app.init_asset::<ConfigNavigationGrid>();
        app.init_asset::<SpellObject>();
        app.init_resource::<NavigationStats>();
        app.init_resource::<NavigationDebug>();
        app.insert_resource(Time::<Fixed>::from_hz(TEST_FPS as f64));
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
            1.0 / TEST_FPS as f64,
        )));

        let grid = app
            .world_mut()
            .resource_mut::<Assets<ConfigNavigationGrid>>()
            .add(make_test_grid());
        app.insert_resource(ResourceGrid(grid));

        let riven = app
            .world_mut()
            .spawn((
                Riven,
                Team::Order,
                Transform::default(),
                Movement { speed: 350.0 },
                Damage(100.0),
                Health::new(1000.0),
                Level::default(),
                SkillPoints(4),
                Skills::default(),
                make_mana(100.0),
            ))
            .id();
        let enemy_near = app
            .world_mut()
            .spawn((
                Team::Chaos,
                Transform::from_xyz(80.0, 0.0, 0.0),
                Health::new(1000.0),
            ))
            .id();
        let enemy_far = app
            .world_mut()
            .spawn((
                Team::Chaos,
                Transform::from_xyz(420.0, 0.0, 0.0),
                Health::new(1000.0),
            ))
            .id();
        let ally_near = app
            .world_mut()
            .spawn((
                Team::Order,
                Transform::from_xyz(60.0, 0.0, 0.0),
                Health::new(1000.0),
            ))
            .id();

        app.update();

        Self {
            app,
            riven,
            enemy_near,
            enemy_far,
            ally_near,
        }
    }

    fn w_damage_key() -> u32 {
        hash_bin("TotalDamage")
    }

    fn load_real_spell(&mut self, spell_key: &str) -> &mut Self {
        let path = format!(
            "assets/data/{:x}.lol",
            hash_wad(&RIVEN_BIN_PATH.to_lowercase())
        );
        let bytes = fs::read(&path).unwrap_or_else(|e| panic!("failed to read {path}: {e}"));
        let (_, prop) = PropFile::parse(&bytes)
            .unwrap_or_else(|e| panic!("failed to parse prop file {path}: {e:?}"));
        let spell_class_hash = type_name_to_hash("SpellObject");
        let spell_name = spell_key
            .rsplit('/')
            .next()
            .unwrap_or(spell_key)
            .to_ascii_lowercase();

        let mut candidates = Vec::new();
        let mut selected = None;

        for (class_hash, entry) in prop.iter_class_hash_and_entry() {
            if class_hash != spell_class_hash {
                continue;
            }

            let spell: SpellObject = from_entry(entry)
                .unwrap_or_else(|e| panic!("failed to deserialize SpellObject from {path}: {e}"));

            let script = spell.m_script_name.to_ascii_lowercase();
            let object = spell.object_name.to_ascii_lowercase();
            candidates.push(format!(
                "script={}, object={}",
                spell.m_script_name, spell.object_name
            ));

            if script == spell_name || object == spell_name || object.contains(&spell_name) {
                selected = Some(spell);
                break;
            }
        }

        let spell = selected.unwrap_or_else(|| {
            panic!(
                "failed to find spell {spell_key} in {path}. candidates: {}",
                candidates.join(" | ")
            )
        });

        self.app
            .world_mut()
            .resource_mut::<Assets<SpellObject>>()
            .add_hash(spell_key, spell);

        self
    }

    fn add_skill(&mut self, spell_key: &str, level: usize, cooldown: f32) -> &mut Self {
        let mut timer = Timer::from_seconds(cooldown, TimerMode::Once);
        timer.tick(Duration::from_secs_f32(cooldown));
        let slot = match spell_key {
            RIVEN_Q_KEY => SkillSlot::Q,
            RIVEN_W_KEY => SkillSlot::W,
            RIVEN_E_KEY => SkillSlot::E,
            RIVEN_R_KEY => SkillSlot::R,
            _ => skill_slot_from_index(
                self.app
                    .world()
                    .get::<Skills>(self.riven)
                    .expect("riven should always have a skill list")
                    .len(),
            ),
        };
        let mut skill = Skill::new(slot, spell_key).with_level(level);
        if slot == SkillSlot::Q {
            skill = skill.with_cooldown_mode(SkillCooldownMode::Manual);
        }

        let skill_entity = self
            .app
            .world_mut()
            .spawn((
                skill,
                CoolDown {
                    timer,
                    duration: cooldown,
                },
            ))
            .id();

        self.app
            .world_mut()
            .get_mut::<Skills>(self.riven)
            .expect("riven should always have a skill list")
            .push(skill_entity);

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

    fn advance(&mut self, seconds: f32) -> &mut Self {
        let ticks = (seconds * TEST_FPS).ceil() as u32;
        for _ in 0..ticks {
            self.app.update();
        }
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
            .expect("riven mana should exist")
            .value
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

    fn spell(&self, spell_key: &str) -> &SpellObject {
        self.app
            .world()
            .resource::<Assets<SpellObject>>()
            .load_hash(spell_key)
            .unwrap_or_else(|| panic!("spell not loaded: {spell_key}"))
    }
}

fn make_mana(value: f32) -> AbilityResource {
    AbilityResource {
        ar_type: AbilityResourceType::Mana,
        value,
        max: 100.0,
        base: 0.0,
        per_level: 0.0,
        base_static_regen: 0.0,
        regen_per_level: 0.0,
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
        occupied_cells: HashMap::default(),
        exclude_cells: Default::default(),
    }
}

#[test]
fn riven_q_cycles_through_three_real_stages() {
    let mut harness = RivenHarness::new();
    harness
        .load_real_spell(RIVEN_Q_KEY)
        .add_skill(RIVEN_Q_KEY, 1, 0.2);

    harness.cast_skill(0, Vec2::new(1000.0, 0.0)).advance(0.4);
    let q_entity = harness.app.world().get::<Skills>(harness.riven).unwrap()[0];
    assert_eq!(
        harness
            .app
            .world()
            .get::<SkillRecastWindow>(q_entity)
            .map(|window| window.stage),
        Some(2)
    );
    assert!(harness.cooldown_finished(0));

    harness.cast_skill(0, Vec2::new(1000.0, 0.0)).advance(0.4);
    assert_eq!(
        harness
            .app
            .world()
            .get::<SkillRecastWindow>(q_entity)
            .map(|window| window.stage),
        Some(3)
    );
    assert!(harness.cooldown_finished(0));

    harness.cast_skill(0, Vec2::new(1000.0, 0.0)).advance(0.1);
    assert!(harness
        .app
        .world()
        .get::<SkillRecastWindow>(q_entity)
        .is_none());
    assert!(!harness.cooldown_finished(0));
    harness.advance(0.3);
    let q_pos = harness.position(harness.riven).x;
    assert!(
        (q_pos - 750.0).abs() < 5.0,
        "expected q combo position near 750, got {q_pos}"
    );
}

#[test]
fn riven_w_hits_only_enemies_in_range() {
    let mut harness = RivenHarness::new();
    harness
        .load_real_spell(RIVEN_W_KEY)
        .load_real_spell(RIVEN_Q_KEY)
        .add_skill(RIVEN_Q_KEY, 1, 0.2)
        .add_skill(RIVEN_W_KEY, 1, 1.0);

    let expected_damage = get_skill_value(
        harness.spell(RIVEN_W_KEY),
        RivenHarness::w_damage_key(),
        1,
        |stat| if stat == 2 { 100.0 } else { 0.0 },
    )
    .expect("riven w damage should exist");

    harness.cast_skill(1, Vec2::new(0.0, 0.0));

    let near = harness.health(harness.enemy_near);
    let far = harness.health(harness.enemy_far);
    let ally = harness.health(harness.ally_near);
    assert!(
        (near - (1000.0 - expected_damage)).abs() < EPSILON,
        "expected near enemy {}, got {near}",
        1000.0 - expected_damage
    );
    assert!(
        (far - 1000.0).abs() < EPSILON,
        "expected far enemy 1000, got {far}"
    );
    assert!(
        (ally - 1000.0).abs() < EPSILON,
        "expected ally 1000, got {ally}"
    );
}

#[test]
fn riven_e_spawns_shield_and_dash_absorbs_damage() {
    let mut harness = RivenHarness::new();
    harness
        .load_real_spell(RIVEN_E_KEY)
        .load_real_spell(RIVEN_Q_KEY)
        .add_skill(RIVEN_Q_KEY, 1, 0.2)
        .add_skill(RIVEN_E_KEY, 1, 1.0);

    harness.cast_skill(1, Vec2::new(1000.0, 0.0)).advance(0.4);

    let e_pos = harness.position(harness.riven).x;
    assert!(
        (e_pos - 250.0).abs() < 5.0,
        "expected e dash position near 250, got {e_pos}"
    );
    assert_eq!(harness.shield_value(), Some(100.0));

    harness.apply_damage_to_riven(60.0);
    assert!((harness.health(harness.riven) - 1000.0).abs() < EPSILON);
    assert_eq!(harness.shield_value(), Some(40.0));

    harness.apply_damage_to_riven(50.0);
    assert!((harness.health(harness.riven) - 990.0).abs() < EPSILON);
}

#[test]
fn riven_r_starts_cooldown_without_moving_or_damaging() {
    let mut harness = RivenHarness::new();
    harness
        .load_real_spell(RIVEN_R_KEY)
        .load_real_spell(RIVEN_Q_KEY)
        .add_skill(RIVEN_Q_KEY, 1, 0.2)
        .add_skill(RIVEN_R_KEY, 1, 4.0);

    let expected_mana_cost = harness
        .spell(RIVEN_R_KEY)
        .m_spell
        .as_ref()
        .and_then(|spell| spell.mana.as_ref())
        .and_then(|mana| mana.first().copied())
        .unwrap_or(0.0);

    harness.cast_skill(1, Vec2::new(1000.0, 0.0));

    assert!((harness.mana() - (100.0 - expected_mana_cost)).abs() < EPSILON);
    assert!(!harness.cooldown_finished(1));
    assert!((harness.position(harness.riven).x - 0.0).abs() < EPSILON);
    assert!((harness.health(harness.enemy_near) - 1000.0).abs() < EPSILON);
}
