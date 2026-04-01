use std::collections::HashMap;
use std::time::Duration;

use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use league_core::{
    EffectValueCalculationPart, EnumAbilityResourceByCoefficientCalculationPart,
    EnumGameCalculation, GameCalculation, SpellDataResource, SpellEffectAmount, SpellObject,
};
use lol_config::LoadHashKeyTrait;
use lol_core::Team;
use moon_lol::{
    AbilityResource, AbilityResourceType, Action, ActionDamage, ActionDamageEffect, CommandAction,
    CoolDown, DamageShape, DamageType, Health, Level, PluginAction, PluginCooldown, PluginDamage,
    PluginLife, PluginSkill, Skill, SkillAction, SkillEffect, SkillOf, SkillPoints, Skills,
    TargetDamage, TargetFilter,
};

const TEST_FPS: f32 = 30.0;
const EPSILON: f32 = 1e-4;
const SPELL_KEY: u32 = 0x1001;
const EFFECT_KEY: u32 = 0x2001;
const DAMAGE_AMOUNT_KEY: u32 = 0x3001;

struct SkillIntegrationHarness {
    app: App,
    caster: Entity,
    enemy: Entity,
    ally: Entity,
    far_enemy: Entity,
}

impl SkillIntegrationHarness {
    fn new() -> Self {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(AssetPlugin::default());
        app.add_plugins(PluginAction);
        app.add_plugins(PluginCooldown);
        app.add_plugins(PluginDamage);
        app.add_plugins(PluginLife);
        app.add_plugins(PluginSkill);
        app.init_asset::<SpellObject>();
        app.insert_resource(Time::<Fixed>::from_hz(TEST_FPS as f64));
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
            1.0 / TEST_FPS as f64,
        )));

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
                Transform::from_xyz(60.0, 0.0, 0.0),
                Health::new(1000.0),
            ))
            .id();
        let ally = app
            .world_mut()
            .spawn((
                Team::Order,
                Transform::from_xyz(40.0, 0.0, 0.0),
                Health::new(1000.0),
            ))
            .id();
        let far_enemy = app
            .world_mut()
            .spawn((
                Team::Chaos,
                Transform::from_xyz(400.0, 0.0, 0.0),
                Health::new(1000.0),
            ))
            .id();

        Self {
            app,
            caster,
            enemy,
            ally,
            far_enemy,
        }
    }

    fn register_spell(
        &mut self,
        spell_key: u32,
        mana_cost: f32,
        effect_amount_key: Option<u32>,
        effect_amounts: Vec<f32>,
    ) -> &mut Self {
        let calculations = effect_amount_key.map(|hash| {
            let mut calculations = HashMap::new();
            let calc =
                EnumAbilityResourceByCoefficientCalculationPart::EffectValueCalculationPart(
                    EffectValueCalculationPart {
                        m_effect_index: Some(1),
                    },
                );
            calculations.insert(
                hash,
                EnumGameCalculation::GameCalculation(GameCalculation {
                    m_formula_parts: Some(vec![calc]),
                    m_display_as_percent: None,
                    m_expanded_tooltip_calculation_display: None,
                    m_multiplier: None,
                    m_precision: None,
                    m_simple_tooltip_calculation_display: None,
                    result_modifier: None,
                    tooltip_only: None,
                }),
            );
            calculations
        });

        self.app
            .world_mut()
            .resource_mut::<Assets<SpellObject>>()
            .add_hash(
                spell_key,
                SpellObject {
                    m_spell: Some(SpellDataResource {
                        mana: Some(vec![mana_cost; 5]),
                        m_spell_calculations: calculations,
                        m_effect_amount: if effect_amount_key.is_some() {
                            Some(vec![SpellEffectAmount {
                                value: Some(effect_amounts),
                            }])
                        } else {
                            None
                        },
                        ..Default::default()
                    }),
                    bot_data: None,
                    cc_behavior_data: None,
                    m_buff: None,
                    m_script_name: String::new(),
                    object_name: String::new(),
                    script: None,
                },
            );

        self
    }

    fn register_effect(&mut self, effect_key: u32, actions: Vec<SkillAction>) -> &mut Self {
        self.app
            .world_mut()
            .resource_mut::<Assets<SkillEffect>>()
            .add_hash(effect_key, SkillEffect(actions));
        self
    }

    fn add_skill(&mut self, skill: Skill, cooldown_duration: f32) -> &mut Self {
        let mut timer = Timer::from_seconds(cooldown_duration, TimerMode::Once);
        timer.tick(Duration::from_secs_f32(cooldown_duration));

        let skill_entity = self
            .app
            .world_mut()
            .spawn((
                SkillOf(self.caster),
                skill,
                CoolDown {
                    timer,
                    duration: cooldown_duration,
                },
            ))
            .id();

        self.app
            .world_mut()
            .get_mut::<Skills>(self.caster)
            .expect("caster should always have skills")
            .push(skill_entity);

        self
    }

    fn simulate_skill_input(&mut self, index: usize, point: Vec2) -> &mut Self {
        self.app.world_mut().trigger(CommandAction {
            entity: self.caster,
            action: Action::Skill { index, point },
        });
        self.app.update();
        self
    }

    fn simulate_level_up_input(&mut self, index: usize) -> &mut Self {
        self.app.world_mut().trigger(CommandAction {
            entity: self.caster,
            action: Action::SkillLevelUp(index),
        });
        self.app.update();
        self
    }

    fn advance_time(&mut self, seconds: f32) -> &mut Self {
        let ticks = (seconds * TEST_FPS).ceil() as u32;
        for _ in 0..ticks {
            self.app.update();
        }
        self
    }

    fn mana(&self) -> f32 {
        self.app
            .world()
            .get::<AbilityResource>(self.caster)
            .expect("caster mana should exist")
            .value
    }

    fn skill_level(&self, index: usize) -> usize {
        let skills = self
            .app
            .world()
            .get::<Skills>(self.caster)
            .expect("caster skills should exist");
        let skill_entity = skills[index];
        self.app
            .world()
            .get::<Skill>(skill_entity)
            .expect("skill should exist")
            .level
    }

    fn skill_points(&self) -> u32 {
        self.app
            .world()
            .get::<SkillPoints>(self.caster)
            .expect("skill points should exist")
            .0
    }

    fn cooldown_finished(&self, index: usize) -> bool {
        let skills = self
            .app
            .world()
            .get::<Skills>(self.caster)
            .expect("caster skills should exist");
        let skill_entity = skills[index];
        self.app
            .world()
            .get::<CoolDown>(skill_entity)
            .expect("cooldown should exist")
            .timer
            .is_finished()
    }

    fn health(&self, entity: Entity) -> f32 {
        self.app
            .world()
            .get::<Health>(entity)
            .expect("health should exist")
            .value
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

fn test_skill(spell_key: u32, effect_key: u32, level: usize) -> Skill {
    Skill {
        key_spell_object: spell_key.into(),
        key_skill_effect: effect_key.into(),
        level,
    }
}

#[test]
fn skill_level_up_uses_headless_app_and_action_input() {
    let mut harness = SkillIntegrationHarness::new();
    harness.add_skill(test_skill(SPELL_KEY, EFFECT_KEY, 0), 5.0);

    harness.simulate_level_up_input(0);

    assert_eq!(harness.skill_level(0), 1);
    assert_eq!(harness.skill_points(), 0);
}

#[test]
fn skill_cast_spends_mana_and_starts_cooldown() {
    let mut harness = SkillIntegrationHarness::new();
    harness
        .register_spell(SPELL_KEY, 30.0, None, vec![])
        .register_effect(EFFECT_KEY, vec![])
        .add_skill(test_skill(SPELL_KEY, EFFECT_KEY, 1), 3.0);

    harness.simulate_skill_input(0, Vec2::new(120.0, 0.0));

    assert!((harness.mana() - 70.0).abs() < EPSILON);
    assert!(!harness.cooldown_finished(0));
}

#[test]
fn skill_cannot_recast_while_cooldown_is_active() {
    let mut harness = SkillIntegrationHarness::new();
    harness
        .register_spell(SPELL_KEY, 30.0, None, vec![])
        .register_effect(EFFECT_KEY, vec![])
        .add_skill(test_skill(SPELL_KEY, EFFECT_KEY, 1), 3.0);

    harness
        .simulate_skill_input(0, Vec2::new(120.0, 0.0))
        .simulate_skill_input(0, Vec2::new(120.0, 0.0));

    assert!((harness.mana() - 70.0).abs() < EPSILON);

    harness
        .advance_time(3.1)
        .simulate_skill_input(0, Vec2::new(120.0, 0.0));

    assert!((harness.mana() - 40.0).abs() < EPSILON);
}

#[test]
fn skill_cast_respects_mana_gate() {
    let mut harness = SkillIntegrationHarness::new();
    harness
        .app
        .world_mut()
        .entity_mut(harness.caster)
        .insert(make_mana(20.0));
    harness
        .register_spell(SPELL_KEY, 30.0, None, vec![])
        .register_effect(EFFECT_KEY, vec![])
        .add_skill(test_skill(SPELL_KEY, EFFECT_KEY, 1), 3.0);

    harness.simulate_skill_input(0, Vec2::new(120.0, 0.0));

    assert!((harness.mana() - 20.0).abs() < EPSILON);
    assert!(harness.cooldown_finished(0));
}

#[test]
fn damage_skill_uses_scene_targets_in_headless_app() {
    let mut harness = SkillIntegrationHarness::new();
    harness
        .register_spell(SPELL_KEY, 20.0, Some(DAMAGE_AMOUNT_KEY), vec![40.0; 5])
        .register_effect(
            EFFECT_KEY,
            vec![SkillAction::Damage(ActionDamage {
                entity: Entity::PLACEHOLDER,
                skill: SPELL_KEY.into(),
                effects: vec![ActionDamageEffect {
                    shape: DamageShape::Circle { radius: 100.0 },
                    damage_list: vec![TargetDamage {
                        filter: TargetFilter::All,
                        amount: DAMAGE_AMOUNT_KEY,
                        damage_type: DamageType::Physical,
                    }],
                    particle: None,
                }],
            })],
        )
        .add_skill(test_skill(SPELL_KEY, EFFECT_KEY, 1), 3.0);

    harness.simulate_skill_input(0, Vec2::new(60.0, 0.0));

    assert!((harness.health(harness.enemy) - 960.0).abs() < EPSILON);
    assert!((harness.health(harness.ally) - 1000.0).abs() < EPSILON);
    assert!((harness.health(harness.far_enemy) - 1000.0).abs() < EPSILON);
}
