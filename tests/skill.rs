use std::collections::BTreeMap;
use std::time::Duration;

use bevy::prelude::*;
use league_utils::hash_key::HashKey;
use lol_base::prop::LoadHashKeyTrait;
use lol_base::spell::{DataSpell, Spell, ValuesEffect};
use lol_base::spell_calc::{CalculationPartEffectValue, CalculationSpell, CalculationType};
use lol_core::action::PluginAction;
use lol_core::action::damage::{DamageShape, TargetDamage, TargetFilter};
use lol_core::base::ability_resource::{AbilityResource, AbilityResourceType};
use lol_core::base::level::Level;
use lol_core::cooldown::PluginCooldown;
use lol_core::damage::{DamageType, PluginDamage};
use lol_core::life::{Health, PluginLife};
use lol_core::movement::PluginMovement;
use lol_core::skill::{
    CommandSkillLevelUp, CommandSkillStart, CoolDown, EventSkillCast, PluginSkill, Skill,
    SkillCastFailureReason, SkillCastLog, SkillCastResult, SkillCooldownMode, SkillOf, SkillPoints,
    SkillRecastWindow, SkillSlot, Skills, skill_damage,
};
use lol_core::team::Team;

const TEST_FPS: f32 = 30.0;
const SPELL_KEY: u32 = 0x1001;
const DAMAGE_AMOUNT_KEY: u32 = 0x3001;
const EPSILON: f32 = 1e-4;

fn spell_handle(key: u32) -> Handle<Spell> {
    Handle::from(HashKey::<Spell>::from(key))
}

#[derive(Component)]
struct TestObserverSkill;

#[derive(Component)]
struct TestDamageSkill;

#[derive(Resource, Default, Debug)]
struct ObserverStages(Vec<u8>);

#[derive(Default)]
struct PluginTestObserverSkill;

impl Plugin for PluginTestObserverSkill {
    fn build(&self, app: &mut App) {
        app.init_resource::<ObserverStages>();
        app.add_observer(on_test_observer_skill_cast);
        app.add_observer(on_test_damage_skill_cast);
    }
}

fn on_test_observer_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    mut stages: ResMut<ObserverStages>,
    q_skill: Query<(&Skill, Option<&SkillRecastWindow>, &CoolDown), With<TestObserverSkill>>,
) {
    let Ok((skill, recast, cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if skill.slot != SkillSlot::Q {
        return;
    }

    let stage = recast.map(|value| value.stage).unwrap_or(1);
    stages.0.push(stage);

    if stage >= 3 {
        commands
            .entity(trigger.skill_entity)
            .remove::<SkillRecastWindow>();
        commands.entity(trigger.skill_entity).insert((CoolDown {
            duration: cooldown.duration,
            timer: Some(Timer::from_seconds(cooldown.duration, TimerMode::Once)),
        },));
    } else {
        commands
            .entity(trigger.skill_entity)
            .insert(SkillRecastWindow::new(stage + 1, 3, 4.0));
    }
}

fn on_test_damage_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_skill: Query<&Skill, With<TestDamageSkill>>,
) {
    if q_skill.get(trigger.skill_entity).is_err() {
        return;
    }

    skill_damage(
        &mut commands,
        trigger.entity,
        spell_handle(SPELL_KEY),
        DamageShape::Circle { radius: 100.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: DAMAGE_AMOUNT_KEY,
            damage_type: DamageType::Physical,
        }],
        None,
    );
}

struct SkillHarness {
    app: App,
    caster: Entity,
    enemy: Entity,
}

impl SkillHarness {
    fn new() -> Self {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(AssetPlugin::default());
        app.add_plugins(PluginAction);
        app.add_plugins(PluginCooldown);
        app.add_plugins(PluginDamage);
        app.add_plugins(PluginLife);
        app.add_plugins(PluginMovement);
        app.add_plugins(PluginSkill);
        app.add_plugins(PluginTestObserverSkill);
        app.init_asset::<Spell>();
        app.insert_resource(Time::<Fixed>::from_hz(TEST_FPS as f64));

        let caster = app
            .world_mut()
            .spawn((
                Team::Order,
                Transform::default(),
                Level::default(),
                SkillPoints(1),
                Skills::default(),
                make_mana(100.0),
            ))
            .id();
        let enemy = app
            .world_mut()
            .spawn((
                Team::Chaos,
                Transform::from_xyz(50.0, 0.0, 0.0),
                Health::new(1000.0),
            ))
            .id();

        Self { app, caster, enemy }
    }

    fn register_spell(&mut self, mana_cost: f32, effect_amounts: Vec<f32>) -> &mut Self {
        use lol_base::spell_calc::CalculationPart;

        let mut calculations = BTreeMap::new();
        calculations.insert(
            DAMAGE_AMOUNT_KEY,
            CalculationType::CalculationSpell(CalculationSpell {
                formula_parts: Some(vec![CalculationPart::CalculationPartEffectValue(
                    CalculationPartEffectValue {
                        effect_index: Some(1),
                    },
                )]),
                multiplier: None,
                precision: None,
            }),
        );

        let effect_values = ValuesEffect {
            values: Some(effect_amounts),
        };

        self.app
            .world_mut()
            .resource_mut::<Assets<Spell>>()
            .add_hash(
                SPELL_KEY,
                Spell {
                    spell_data: Some(DataSpell {
                        calculations: Some(calculations),
                        effect_amounts: Some(vec![effect_values]),
                        data_values: None,
                        mana: Some(vec![mana_cost; 6]),
                        missile_spec: None,
                        hit_bone_name: None,
                        missile_speed: None,
                        missile_effect_key: None,
                        cast_type: None,
                    }),
                },
            );
        self
    }

    fn add_skill(&mut self, skill: Skill, cooldown_duration: f32, extra: impl Bundle) -> Entity {
        let mut timer = Timer::from_seconds(cooldown_duration, TimerMode::Once);
        timer.tick(Duration::from_secs_f32(cooldown_duration));

        let skill_entity = self
            .app
            .world_mut()
            .spawn((
                SkillOf(self.caster),
                skill,
                CoolDown {
                    duration: cooldown_duration,
                    timer: Some(timer),
                },
                extra,
            ))
            .id();

        self.app
            .world_mut()
            .get_mut::<Skills>(self.caster)
            .expect("caster should have skills")
            .push(skill_entity);

        skill_entity
    }

    fn cast_skill(&mut self, index: usize, point: Vec2) -> &mut Self {
        self.app.world_mut().trigger(CommandSkillStart {
            entity: self.caster,
            index,
            point,
        });
        self.app.update();
        self
    }

    fn level_up_skill(&mut self, index: usize) -> &mut Self {
        self.app.world_mut().trigger(CommandSkillLevelUp {
            entity: self.caster,
            index,
        });
        self.app.update();
        self
    }

    fn advance_time(&mut self, seconds: f32) -> &mut Self {
        let ticks = (seconds * TEST_FPS).ceil() as u32;
        for _ in 0..ticks {
            let mut time = self.app.world_mut().resource_mut::<Time<Fixed>>();
            time.advance_by(Duration::from_secs_f64(1.0 / TEST_FPS as f64));
            drop(time);
            self.app.world_mut().run_schedule(FixedUpdate);
        }
        self
    }

    fn mana(&self) -> f32 {
        self.app
            .world()
            .get::<AbilityResource>(self.caster)
            .expect("mana should exist")
            .value
    }

    fn health(&self, entity: Entity) -> f32 {
        self.app
            .world()
            .get::<Health>(entity)
            .expect("health should exist")
            .value
    }

    fn skill_entity(&self, index: usize) -> Entity {
        self.app.world().get::<Skills>(self.caster).unwrap()[index]
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

#[test]
fn skill_level_up_still_respects_basic_rules() {
    let mut harness = SkillHarness::new();
    harness
        .app
        .world_mut()
        .entity_mut(harness.caster)
        .insert(Level {
            value: 5,
            experience: 0,
            experience_to_next_level: 100,
        });
    harness.add_skill(Skill::new(SkillSlot::Q, spell_handle(SPELL_KEY)), 5.0, ());
    harness.add_skill(Skill::new(SkillSlot::W, spell_handle(SPELL_KEY)), 5.0, ());
    harness.add_skill(Skill::new(SkillSlot::E, spell_handle(SPELL_KEY)), 5.0, ());
    harness.add_skill(Skill::new(SkillSlot::R, spell_handle(SPELL_KEY)), 5.0, ());

    harness.level_up_skill(3);

    let skill_entity = harness.skill_entity(3);
    assert_eq!(
        harness
            .app
            .world()
            .get::<Skill>(skill_entity)
            .unwrap()
            .level,
        0
    );
    assert_eq!(
        harness
            .app
            .world()
            .get::<SkillPoints>(harness.caster)
            .unwrap()
            .0,
        1
    );
}

#[test]
fn observer_skill_cast_spends_mana_starts_cooldown_and_applies_damage() {
    let mut harness = SkillHarness::new();
    harness.register_spell(30.0, vec![40.0; 6]);

    harness.add_skill(
        Skill::new(SkillSlot::Q, spell_handle(SPELL_KEY)).with_level(1),
        3.0,
        TestDamageSkill,
    );

    harness.cast_skill(0, Vec2::new(50.0, 0.0));

    assert!((harness.mana() - 70.0).abs() < EPSILON);
    assert!((harness.health(harness.enemy) - 960.0).abs() < EPSILON);

    let cooldown_state = harness
        .app
        .world()
        .get::<CoolDown>(harness.skill_entity(0))
        .unwrap();
    assert!(
        !cooldown_state
            .timer
            .as_ref()
            .map_or(true, |t| t.is_finished())
    );

    let log = harness.app.world().resource::<SkillCastLog>();
    assert!(matches!(
        log.0.last().unwrap().result,
        SkillCastResult::Started
    ));
}

#[test]
fn observer_skill_can_drive_recast_state_and_manual_cooldown() {
    let mut harness = SkillHarness::new();
    harness.register_spell(20.0, vec![0.0; 6]);
    harness.add_skill(
        Skill::new(SkillSlot::Q, spell_handle(SPELL_KEY))
            .with_level(1)
            .with_cooldown_mode(SkillCooldownMode::Manual),
        6.0,
        TestObserverSkill,
    );

    harness
        .cast_skill(0, Vec2::new(50.0, 0.0))
        .cast_skill(0, Vec2::new(50.0, 0.0))
        .cast_skill(0, Vec2::new(50.0, 0.0));

    let stages = &harness.app.world().resource::<ObserverStages>().0;
    assert_eq!(stages.as_slice(), &[1, 2, 3]);

    let skill_entity = harness.skill_entity(0);
    assert!(
        harness
            .app
            .world()
            .get::<SkillRecastWindow>(skill_entity)
            .is_none()
    );
    assert!(
        !harness
            .app
            .world()
            .get::<CoolDown>(skill_entity)
            .unwrap()
            .timer
            .as_ref()
            .map_or(true, |t| t.is_finished())
    );
}

#[test]
fn skill_recast_window_expires_in_fixed_update() {
    let mut harness = SkillHarness::new();
    harness.register_spell(20.0, vec![0.0; 6]);
    harness.add_skill(
        Skill::new(SkillSlot::Q, spell_handle(SPELL_KEY))
            .with_level(1)
            .with_cooldown_mode(SkillCooldownMode::Manual),
        6.0,
        TestObserverSkill,
    );

    harness
        .cast_skill(0, Vec2::new(50.0, 0.0))
        .advance_time(4.1);

    let skill_entity = harness.skill_entity(0);
    assert!(
        harness
            .app
            .world()
            .get::<SkillRecastWindow>(skill_entity)
            .is_none()
    );

    harness.cast_skill(0, Vec2::new(50.0, 0.0));
    let stages = &harness.app.world().resource::<ObserverStages>().0;
    assert_eq!(stages.as_slice(), &[1, 1]);
}

#[test]
fn insufficient_mana_is_recorded_without_starting_cooldown() {
    let mut harness = SkillHarness::new();
    harness
        .app
        .world_mut()
        .entity_mut(harness.caster)
        .insert(make_mana(10.0));
    harness.register_spell(30.0, vec![40.0; 6]);
    harness.add_skill(
        Skill::new(SkillSlot::Q, spell_handle(SPELL_KEY)).with_level(1),
        3.0,
        TestDamageSkill,
    );

    harness.cast_skill(0, Vec2::new(50.0, 0.0));

    assert!((harness.mana() - 10.0).abs() < EPSILON);
    assert!(
        harness
            .app
            .world()
            .get::<CoolDown>(harness.skill_entity(0))
            .unwrap()
            .timer
            .as_ref()
            .map_or(true, |t| t.is_finished())
    );
    assert!(matches!(
        harness
            .app
            .world()
            .resource::<SkillCastLog>()
            .0
            .last()
            .unwrap()
            .result,
        SkillCastResult::Failed(SkillCastFailureReason::InsufficientAbilityResource)
    ));
}
