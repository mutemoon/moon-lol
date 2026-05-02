#![cfg(test)]

// ========== Integration tests migrated from root tests/skill.rs ==========

use std::collections::BTreeMap;
use std::time::Duration;

use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use league_utils::hash_key::HashKey;
use lol_base::prop::LoadHashKeyTrait;
use lol_base::spell::{DataSpell, Spell, ValuesEffect};
use lol_base::spell_calc::{CalculationPartEffectValue, CalculationSpell, CalculationType};

use crate::action::damage::{
    ActionDamage, ActionDamageEffect, DamageShape, TargetDamage, TargetFilter,
};
use crate::action::{Action, CommandAction, PluginAction};
use crate::base::ability_resource::{AbilityResource, AbilityResourceType};
use crate::base::level::Level;
use crate::cooldown::PluginCooldown;
use crate::damage::{DamageType, PluginDamage};
use crate::life::{Health, PluginLife};
use crate::skill::{
    CoolDown, EventSkillCast, PluginSkill, Skill, SkillCastLog, SkillCastResult, SkillOf,
    SkillPoints, SkillSlot, Skills,
};
use crate::team::Team;

const TEST_FPS: f32 = 30.0;
const SPELL_KEY: u32 = 0x5001;
const DAMAGE_AMOUNT_KEY: &str = "damage_amount";
const EPSILON: f32 = 1e-4;

fn spell_handle(key: u32) -> Handle<Spell> {
    Handle::from(HashKey::<Spell>::from(key))
}

#[derive(Component)]
struct ActionObserverSkill;

#[derive(Component)]
struct ActionDamageSkill;

#[derive(Resource, Default)]
struct ActionObserverTrace(Vec<Vec2>);

#[derive(Default)]
struct PluginActionObserverSkill;

impl Plugin for PluginActionObserverSkill {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActionObserverTrace>();
        app.add_observer(on_action_observer_skill_cast);
        app.add_observer(on_action_damage_skill_cast);
    }
}

fn on_action_observer_skill_cast(
    trigger: On<EventSkillCast>,
    mut trace: ResMut<ActionObserverTrace>,
    q_skill: Query<&Skill, With<ActionObserverSkill>>,
) {
    if q_skill.get(trigger.skill_entity).is_ok() {
        trace.0.push(trigger.point);
    }
}

fn on_action_damage_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_skill: Query<&Skill, With<ActionDamageSkill>>,
) {
    if q_skill.get(trigger.skill_entity).is_err() {
        return;
    }

    commands.trigger(ActionDamage {
        entity: trigger.entity,
        skill: spell_handle(SPELL_KEY),
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 120.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: DAMAGE_AMOUNT_KEY.to_string(),
                damage_type: DamageType::Physical,
            }],
            particle: None,
        }],
    });
}

struct ActionSkillHarness {
    app: App,
    caster: Entity,
    enemy: Entity,
}

impl ActionSkillHarness {
    fn new() -> Self {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(AssetPlugin::default());
        app.add_plugins(PluginAction);
        app.add_plugins(PluginCooldown);
        app.add_plugins(PluginDamage);
        app.add_plugins(PluginLife);
        app.add_plugins(PluginSkill);
        app.add_plugins(PluginActionObserverSkill);
        app.init_asset::<Spell>();
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

        Self { app, caster, enemy }
    }

    fn register_spell(&mut self, mana_cost: f32, base_damage: f32) -> &mut Self {
        use lol_base::spell_calc::CalculationPart;

        let mut calculations = BTreeMap::new();
        calculations.insert(
            DAMAGE_AMOUNT_KEY.to_string(),
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
            values: Some(vec![base_damage; 6]),
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

    fn add_skill(&mut self, skill: Skill, extra: impl Bundle) -> Entity {
        let mut timer = Timer::from_seconds(3.0, TimerMode::Once);
        timer.tick(Duration::from_secs_f32(3.0));

        let skill_entity = self
            .app
            .world_mut()
            .spawn((
                SkillOf(self.caster),
                skill,
                CoolDown {
                    duration: 3.0,
                    timer: Some(timer),
                },
                extra,
            ))
            .id();

        self.app
            .world_mut()
            .get_mut::<Skills>(self.caster)
            .unwrap()
            .push(skill_entity);

        skill_entity
    }

    fn send_action(&mut self, action: Action) -> &mut Self {
        self.app.world_mut().trigger(CommandAction {
            entity: self.caster,
            action,
        });
        self.app.update();
        self
    }

    fn mana(&self) -> f32 {
        self.app
            .world()
            .get::<AbilityResource>(self.caster)
            .unwrap()
            .value
    }

    fn health(&self) -> f32 {
        self.app.world().get::<Health>(self.enemy).unwrap().value
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
fn action_input_can_drive_code_driven_damage_skill_end_to_end() {
    let mut harness = ActionSkillHarness::new();
    harness.register_spell(25.0, 35.0).add_skill(
        Skill::new(SkillSlot::Q, spell_handle(SPELL_KEY)).with_level(1),
        ActionDamageSkill,
    );

    harness.send_action(Action::Skill {
        index: 0,
        point: Vec2::new(60.0, 0.0),
    });

    assert!((harness.mana() - 75.0).abs() < EPSILON);
    assert!((harness.health() - 965.0).abs() < EPSILON);
    assert!(matches!(
        harness
            .app
            .world()
            .resource::<SkillCastLog>()
            .0
            .last()
            .unwrap()
            .result,
        SkillCastResult::Started
    ));
}

#[test]
fn action_input_can_reach_code_driven_skill_observer() {
    let mut harness = ActionSkillHarness::new();
    harness.register_spell(10.0, 0.0).add_skill(
        Skill::new(SkillSlot::W, spell_handle(SPELL_KEY)).with_level(1),
        ActionObserverSkill,
    );

    harness.send_action(Action::Skill {
        index: 0,
        point: Vec2::new(180.0, 12.0),
    });

    let trace = &harness.app.world().resource::<ActionObserverTrace>().0;
    assert_eq!(trace.as_slice(), &[Vec2::new(180.0, 12.0)]);
    assert!((harness.mana() - 90.0).abs() < EPSILON);
}

#[test]
fn action_input_can_level_up_skill_through_same_pipeline() {
    let mut harness = ActionSkillHarness::new();
    let skill_entity = harness.add_skill(Skill::new(SkillSlot::Q, spell_handle(SPELL_KEY)), ());

    harness.send_action(Action::SkillLevelUp(0));

    assert_eq!(
        harness
            .app
            .world()
            .get::<Skill>(skill_entity)
            .unwrap()
            .level,
        1
    );
    assert_eq!(
        harness
            .app
            .world()
            .get::<SkillPoints>(harness.caster)
            .unwrap()
            .0,
        0
    );
}

// ========== Unit tests for spell value calculation ==========

use league_utils::hash_bin;

use super::*;
use crate::skill::get_skill_value;

fn create_mock_spell(
    calculations: std::collections::BTreeMap<String, lol_base::spell_calc::CalculationType>,
    effect_amounts: Option<Vec<lol_base::spell::ValuesEffect>>,
    data_values: Option<Vec<lol_base::spell::ValuesData>>,
) -> Spell {
    Spell {
        spell_data: Some(DataSpell {
            calculations: Some(calculations),
            effect_amounts,
            data_values,
            mana: None,
            missile_spec: None,
            hit_bone_name: None,
            missile_speed: None,
            missile_effect_key: None,
            cast_type: None,
        }),
    }
}

#[test]
fn test_effect_value_calculation() {
    use std::collections::BTreeMap;

    use lol_base::spell::ValuesEffect;
    use lol_base::spell_calc::{
        CalculationPart, CalculationPartEffectValue, CalculationSpell, CalculationType,
    };

    let key = "effect_damage".to_string();
    let effect_index = 1;
    let expected_value_lvl1 = 10.0;
    let expected_value_lvl2 = 20.0;

    let calc_part = CalculationPart::CalculationPartEffectValue(CalculationPartEffectValue {
        effect_index: Some(effect_index),
    });

    let calc = CalculationType::CalculationSpell(CalculationSpell {
        formula_parts: Some(vec![calc_part]),
        multiplier: None,
        precision: None,
    });

    let mut calculations = BTreeMap::new();
    calculations.insert(key.clone(), calc);

    let effect_amounts = vec![ValuesEffect {
        values: Some(vec![expected_value_lvl1, expected_value_lvl2, 30.0]),
    }];

    let spell = create_mock_spell(calculations, Some(effect_amounts), None);

    let result = get_skill_value(&spell, &key, 1, |_| 0.0);
    assert_eq!(result, Some(expected_value_lvl1));

    let result = get_skill_value(&spell, &key, 2, |_| 0.0);
    assert_eq!(result, Some(expected_value_lvl2));
}

#[test]
fn test_stat_by_coefficient_calculation() {
    use std::collections::BTreeMap;

    use lol_base::spell::ValuesEffect;
    use lol_base::spell_calc::{
        CalculationPart, CalculationPartStatCoefficient, CalculationSpell, CalculationType,
    };

    let key = "stat_coefficient".to_string();
    let stat_id = 2;
    let coefficient = 1.5;
    let stat_value = 100.0;
    let expected_value = stat_value * coefficient;

    let calc_part =
        CalculationPart::CalculationPartStatCoefficient(CalculationPartStatCoefficient {
            stat: Some(stat_id),
            coefficient: Some(coefficient),
            stat_formula: None,
        });

    let calc = CalculationType::CalculationSpell(CalculationSpell {
        formula_parts: Some(vec![calc_part]),
        multiplier: None,
        precision: None,
    });

    let mut calculations = BTreeMap::new();
    calculations.insert(key.clone(), calc);

    let spell = create_mock_spell(calculations, None, None);

    let result = get_skill_value(&spell, &key, 1, |id| {
        if id == stat_id { stat_value } else { 0.0 }
    });
    assert_eq!(result, Some(expected_value));
}

#[test]
fn test_named_data_value_calculation() {
    use std::collections::BTreeMap;

    use lol_base::spell::{ValuesData, ValuesEffect};
    use lol_base::spell_calc::{
        CalculationPart, CalculationPartNamedDataValue, CalculationSpell, CalculationType,
    };

    let key = "named_data_value".to_string();
    let data_name = "BaseDamage";
    let data_value_name = "base_damage".to_string();
    let expected_value = 50.0;

    let calc_part = CalculationPart::CalculationPartNamedDataValue(CalculationPartNamedDataValue {
        data_value: data_value_name,
    });

    let calc = CalculationType::CalculationSpell(CalculationSpell {
        formula_parts: Some(vec![calc_part]),
        multiplier: None,
        precision: None,
    });

    let mut calculations = BTreeMap::new();
    calculations.insert(key.clone(), calc);

    let data_values = vec![ValuesData {
        name: data_name.to_string(),
        values: Some(vec![expected_value, 60.0, 70.0]),
    }];

    let spell = create_mock_spell(calculations, None, Some(data_values));

    let result = get_skill_value(&spell, &key, 1, |_| 0.0);
    assert_eq!(result, Some(expected_value));
}
