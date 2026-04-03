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
use moon_lol::*;

const TEST_FPS: f32 = 30.0;
const SPELL_KEY: u32 = 0x5001;
const DAMAGE_AMOUNT_KEY: u32 = 0x5003;
const EPSILON: f32 = 1e-4;

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

    skill_damage(
        &mut commands,
        trigger.entity,
        SPELL_KEY,
        DamageShape::Circle { radius: 120.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: DAMAGE_AMOUNT_KEY,
            damage_type: DamageType::Physical,
        }],
        None,
    );
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

        Self { app, caster, enemy }
    }

    fn register_spell(&mut self, mana_cost: f32, base_damage: f32) -> &mut Self {
        let mut calculations = HashMap::new();
        calculations.insert(
            DAMAGE_AMOUNT_KEY,
            EnumGameCalculation::GameCalculation(GameCalculation {
                m_formula_parts: Some(vec![
                    EnumAbilityResourceByCoefficientCalculationPart::EffectValueCalculationPart(
                        EffectValueCalculationPart {
                            m_effect_index: Some(1),
                        },
                    ),
                ]),
                m_display_as_percent: None,
                m_expanded_tooltip_calculation_display: None,
                m_multiplier: None,
                m_precision: None,
                m_simple_tooltip_calculation_display: None,
                result_modifier: None,
                tooltip_only: None,
            }),
        );

        self.app
            .world_mut()
            .resource_mut::<Assets<SpellObject>>()
            .add_hash(
                SPELL_KEY,
                SpellObject {
                    m_spell: Some(SpellDataResource {
                        mana: Some(vec![mana_cost; 6]),
                        m_spell_calculations: Some(calculations),
                        m_effect_amount: Some(vec![SpellEffectAmount {
                            value: Some(vec![base_damage; 6]),
                        }]),
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
                    timer,
                    duration: 3.0,
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
        Skill::new(SkillSlot::Q, SPELL_KEY).with_level(1),
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
        Skill::new(SkillSlot::W, SPELL_KEY).with_level(1),
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
    let skill_entity = harness.add_skill(Skill::new(SkillSlot::Q, SPELL_KEY), ());

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
