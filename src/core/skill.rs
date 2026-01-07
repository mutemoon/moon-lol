use std::ops::Deref;

use bevy::prelude::*;
use bevy_behave::prelude::{BehavePlugin, BehaveTree, Tree};
use bevy_behave::Behave;
use league_core::{
    EffectValueCalculationPart, EnumAbilityResourceByCoefficientCalculationPart,
    EnumGameCalculation, NamedDataValueCalculationPart, SpellObject,
    StatByCoefficientCalculationPart, StatByNamedDataValueCalculationPart,
    StatBySubPartCalculationPart,
};
use league_utils::hash_bin;
use lol_config::{HashKey, LoadHashKeyTrait};

use crate::{AbilityResource, EventLevelUp, Level};

#[derive(Default)]
pub struct PluginSkill;

impl Plugin for PluginSkill {
    fn build(&self, app: &mut App) {
        app.init_asset::<SkillEffect>();

        app.add_plugins(BehavePlugin::default());

        app.add_observer(on_skill_cast);
        app.add_observer(on_skill_level_up);
        app.add_observer(on_level_up);
    }
}

#[derive(Component, Debug)]
#[relationship(relationship_target = Skills)]
pub struct SkillOf(pub Entity);

#[derive(Component, Debug)]
#[relationship_target(relationship = SkillOf, linked_spawn)]
pub struct Skills(Vec<Entity>);

#[derive(Component, Debug)]
#[relationship(relationship_target = PassiveSkill)]
pub struct PassiveSkillOf(pub Entity);

#[derive(Component, Debug)]
#[relationship_target(relationship = PassiveSkillOf, linked_spawn)]
pub struct PassiveSkill(Entity);

impl Deref for Skills {
    type Target = Vec<Entity>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Component, Default)]
pub struct CoolDown {
    pub timer: Timer,
    pub duration: f32,
}

#[derive(Component)]
#[require(CoolDown)]
pub struct Skill {
    pub key_spell_object: HashKey<SpellObject>,
    pub key_skill_effect: HashKey<SkillEffect>,
    pub level: usize,
}

impl Default for Skill {
    fn default() -> Self {
        Self {
            key_spell_object: 0.into(),
            key_skill_effect: 0.into(),
            level: 0,
        }
    }
}

#[derive(Asset, TypePath)]
pub struct SkillEffect(pub Tree<Behave>);

#[derive(Component)]
pub struct SkillPoints(pub u32);

impl Default for SkillPoints {
    fn default() -> Self {
        Self(1)
    }
}

#[derive(Component)]
pub struct SkillEffectContext {
    pub point: Vec2,
}

#[derive(EntityEvent)]
pub struct CommandSkillStart {
    pub entity: Entity,
    pub index: usize,
    pub point: Vec2,
}

#[derive(EntityEvent)]
pub struct CommandSkillBeforeStart {
    pub entity: Entity,
    pub index: usize,
    pub point: Vec2,
}

fn on_skill_cast(
    trigger: On<CommandSkillStart>,
    mut commands: Commands,
    skills: Query<&Skills>,
    res_assets_spell_object: Res<Assets<SpellObject>>,
    res_assets_skill_effect: Res<Assets<SkillEffect>>,
    mut q_skill: Query<(&Skill, &mut CoolDown)>,
    mut q_ability_resource: Query<&mut AbilityResource>,
) {
    let entity = trigger.event_target();
    let Ok(skills) = skills.get(entity) else {
        return;
    };
    let Some(&skill_entity) = skills.0.get(trigger.index) else {
        return;
    };
    let Ok((skill, mut cooldown)) = q_skill.get_mut(skill_entity) else {
        return;
    };

    if !cooldown.timer.is_finished() {
        debug!(
            "{} 技能 {} 冷却中，剩余 {:.2}s",
            entity,
            trigger.index,
            cooldown.timer.remaining_secs()
        );
        return;
    }

    let spell_object = res_assets_spell_object
        .load_hash(skill.key_spell_object)
        .unwrap();

    if skill.level == 0 {
        debug!("{} 技能 {} 未学习，无法释放", entity, trigger.index);
        return;
    }

    let Ok(mut ability_resource) = q_ability_resource.get_mut(entity) else {
        return;
    };

    if let Some(ref mana) = spell_object.m_spell.as_ref().unwrap().mana {
        let &current_mana = mana.get(skill.level as usize).unwrap();

        if ability_resource.value < current_mana {
            debug!(
                "{} 技能 {} 蓝量不足，需要 {:.0}，当前 {:.0}",
                entity, trigger.index, current_mana, ability_resource.value
            );
            return;
        }

        ability_resource.value -= current_mana;
        debug!(
            "{} 技能 {} 消耗 {:.0} 蓝量，剩余 {:.0}",
            entity, trigger.index, current_mana, ability_resource.value
        );
    }

    let effect_key = skill.key_skill_effect;

    if let Some(effect) = res_assets_skill_effect.load_hash(effect_key) {
        debug!("{} 技能 {} 开始执行行为树", entity, trigger.index);
        commands.entity(entity).with_child((
            BehaveTree::new(effect.0.clone()),
            SkillEffectContext {
                point: trigger.point,
            },
        ));
    }

    cooldown.timer = Timer::from_seconds(cooldown.duration, TimerMode::Once);
    debug!(
        "{} 技能 {} 开始冷却 {}s",
        entity, trigger.index, cooldown.duration
    );
}

#[derive(EntityEvent)]
pub struct CommandSkillLevelUp {
    pub entity: Entity,
    pub index: usize,
}

fn on_skill_level_up(
    trigger: On<CommandSkillLevelUp>,
    skills: Query<&Skills>,
    mut q_skill: Query<&mut Skill>,
    mut q_skill_points: Query<(&Level, &mut SkillPoints)>,
) {
    let entity = trigger.event_target();
    let Ok(skills) = skills.get(entity) else {
        return;
    };
    let Some(&skill_entity) = skills.0.get(trigger.index) else {
        return;
    };
    let Ok(mut skill) = q_skill.get_mut(skill_entity) else {
        return;
    };
    let Ok((level, mut skill_points)) = q_skill_points.get_mut(entity) else {
        return;
    };

    debug!("{} 尝试升级技能: 索引 {}", entity, trigger.index);

    if skill_points.0 == 0 {
        debug!("{} 升级失败: 技能点不足", entity);
        return;
    }

    // 1 级只能加点 q w e，6 级才能加点 r，6 级前一个技能最多加 3 点
    if level.value < 6 {
        if trigger.index == 3 {
            debug!(
                "{} 升级失败: 等级 {} 小于 6 级不能升级大招",
                entity, level.value
            );
            return;
        }
        if skill.level >= 3 {
            debug!(
                "{} 升级失败: 等级 {} 小于 6 级，技能 {} 已达上限 (3)",
                entity, level.value, trigger.index
            );
            return;
        }
    }

    skill.level += 1;
    skill_points.0 -= 1;
    debug!(
        "{} 技能升级成功: 索引 {}, 新等级 {}, 剩余技能点 {}",
        entity, trigger.index, skill.level, skill_points.0
    );
}

fn on_level_up(event: On<EventLevelUp>, mut q_skill_points: Query<&mut SkillPoints>) {
    let entity = event.event_target();
    if let Ok(mut skill_points) = q_skill_points.get_mut(entity) {
        skill_points.0 += event.delta;
        debug!(
            "{} 升级: 获得 {} 技能点，当前技能点 {}",
            entity, event.delta, skill_points.0
        );
    }
}

pub fn get_skill_value(
    skill_object: &SpellObject,
    hash: u32,
    level: usize,
    get_stat: impl Fn(u8) -> f32,
) -> Option<f32> {
    let spell = skill_object.m_spell.as_ref()?;
    let calculations = spell.m_spell_calculations.as_ref()?;
    let calculation = calculations.get(&hash)?;

    match calculation {
        EnumGameCalculation::GameCalculation(calc) => {
            let mut value = 0.0;
            if let Some(parts) = &calc.m_formula_parts {
                for part in parts {
                    value += calculate_part(part, skill_object, level, &get_stat);
                }
            }

            if let Some(multiplier) = &calc.m_multiplier {
                // m_multiplier is an EnumAbilityResourceByCoefficientCalculationPart
                // Wait, the struct definition says:
                // pub m_multiplier: Option<EnumAbilityResourceByCoefficientCalculationPart>,
                // So we can directly use it.
                value *= calculate_part(multiplier, skill_object, level, &get_stat);
            }
            Some(value)
        }
        _ => todo!(
            "EnumGameCalculation variant not implemented: {:?}",
            calculation
        ),
    }
}

fn calculate_part(
    part: &EnumAbilityResourceByCoefficientCalculationPart,
    skill_object: &SpellObject,
    level: usize,
    get_stat: &impl Fn(u8) -> f32,
) -> f32 {
    match part {
        EnumAbilityResourceByCoefficientCalculationPart::EffectValueCalculationPart(
            EffectValueCalculationPart { m_effect_index },
        ) => {
            let index = m_effect_index.unwrap_or(1) - 1;
            if let Some(effect_amount) = skill_object
                .m_spell
                .as_ref()
                .and_then(|s| s.m_effect_amount.as_ref())
                .and_then(|v| v.get(index as usize))
            {
                if let Some(values) = &effect_amount.value {
                    // level is 1-based, so index is level - 1
                    // Ensure level is at least 1
                    let lvl_idx = if level > 0 { level - 1 } else { 0 };
                    return *values.get(lvl_idx).unwrap_or(&0.0);
                }
            }
            0.0
        }
        EnumAbilityResourceByCoefficientCalculationPart::StatByCoefficientCalculationPart(
            StatByCoefficientCalculationPart {
                m_stat,
                m_coefficient,
                ..
            },
        ) => {
            let stat = m_stat.unwrap_or(0);
            let coefficient = m_coefficient.unwrap_or(0.0);
            get_stat(stat) * coefficient
        }
        EnumAbilityResourceByCoefficientCalculationPart::NamedDataValueCalculationPart(
            NamedDataValueCalculationPart { m_data_value },
        ) => {
            if let Some(data_values) = skill_object
                .m_spell
                .as_ref()
                .and_then(|s| s.data_values.as_ref())
            {
                for dv in data_values {
                    // Check if hash of name matches m_data_value
                    // Assuming m_data_value is the hash of the name
                    let hash = hash_bin(&dv.m_name);
                    if hash == *m_data_value {
                        if let Some(values) = &dv.m_values {
                            let lvl_idx = if level > 0 { level - 1 } else { 0 };
                            return *values.get(lvl_idx).unwrap_or(&0.0);
                        }
                    }
                }
            }
            0.0
        }
        EnumAbilityResourceByCoefficientCalculationPart::StatBySubPartCalculationPart(
            StatBySubPartCalculationPart {
                m_stat, m_subpart, ..
            },
        ) => {
            let stat = m_stat.unwrap_or(0);
            let sub_val = calculate_part(m_subpart, skill_object, level, get_stat);
            get_stat(stat) * sub_val
        }
        EnumAbilityResourceByCoefficientCalculationPart::StatByNamedDataValueCalculationPart(
            StatByNamedDataValueCalculationPart {
                m_stat,
                m_data_value,
                ..
            },
        ) => {
            let stat = m_stat.unwrap_or(0);
            let mut data_val = 0.0;
            if let Some(data_values) = skill_object
                .m_spell
                .as_ref()
                .and_then(|s| s.data_values.as_ref())
            {
                for dv in data_values {
                    let hash = hash_bin(&dv.m_name);
                    if hash == *m_data_value {
                        if let Some(values) = &dv.m_values {
                            let lvl_idx = if level > 0 { level - 1 } else { 0 };
                            data_val = *values.get(lvl_idx).unwrap_or(&0.0);
                            break;
                        }
                    }
                }
            }
            get_stat(stat) * data_val
        }
        _ => todo!("Calculation part not implemented: {:?}", part),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use league_core::{
        EffectValueCalculationPart, EnumAbilityResourceByCoefficientCalculationPart,
        EnumGameCalculation, GameCalculation, NamedDataValueCalculationPart, SpellDataResource,
        SpellDataValue, SpellEffectAmount, SpellObject, StatByCoefficientCalculationPart,
    };

    use super::*;

    fn create_mock_spell_object(
        calculations: HashMap<u32, EnumGameCalculation>,
        effect_amounts: Option<Vec<SpellEffectAmount>>,
        data_values: Option<Vec<SpellDataValue>>,
    ) -> SpellObject {
        SpellObject {
            m_spell: Some(SpellDataResource {
                m_spell_calculations: Some(calculations),
                m_effect_amount: effect_amounts,
                data_values,
                ..default()
            }),
            bot_data: None,
            cc_behavior_data: None,
            m_buff: None,
            m_script_name: "".to_string(),
            object_name: "".to_string(),
            script: None,
        }
    }

    #[test]
    fn test_effect_value_calculation() {
        // Setup
        let hash = 123;
        let effect_index = 1;
        let expected_value_lvl1 = 10.0;
        let expected_value_lvl2 = 20.0;

        let calc_part = EnumAbilityResourceByCoefficientCalculationPart::EffectValueCalculationPart(
            EffectValueCalculationPart {
                m_effect_index: Some(effect_index),
            },
        );

        let game_calc = EnumGameCalculation::GameCalculation(GameCalculation {
            m_formula_parts: Some(vec![calc_part]),
            m_display_as_percent: None,
            m_expanded_tooltip_calculation_display: None,
            m_multiplier: None,
            m_precision: None,
            m_simple_tooltip_calculation_display: None,
            result_modifier: None,
            tooltip_only: None,
        });

        let mut calculations = HashMap::new();
        calculations.insert(hash, game_calc);

        let effect_amounts = vec![SpellEffectAmount {
            value: Some(vec![expected_value_lvl1, expected_value_lvl2, 30.0]),
        }];

        let spell_object = create_mock_spell_object(calculations, Some(effect_amounts), None);

        // Test Level 1
        let result = get_skill_value(&spell_object, hash, 1, |_| 0.0);
        assert_eq!(result, Some(expected_value_lvl1));

        // Test Level 2
        let result = get_skill_value(&spell_object, hash, 2, |_| 0.0);
        assert_eq!(result, Some(expected_value_lvl2));
    }

    #[test]
    fn test_stat_by_coefficient_calculation() {
        // Setup
        let hash = 456;
        let stat_id = 2; // e.g., Attack Damage
        let coefficient = 1.5;
        let stat_value = 100.0;
        let expected_value = stat_value * coefficient;

        let calc_part =
            EnumAbilityResourceByCoefficientCalculationPart::StatByCoefficientCalculationPart(
                StatByCoefficientCalculationPart {
                    m_stat: Some(stat_id),
                    m_coefficient: Some(coefficient),
                    m_stat_formula: None,
                    unk_0xa8cb9c14: None,
                },
            );

        let game_calc = EnumGameCalculation::GameCalculation(GameCalculation {
            m_formula_parts: Some(vec![calc_part]),
            m_display_as_percent: None,
            m_expanded_tooltip_calculation_display: None,
            m_multiplier: None,
            m_precision: None,
            m_simple_tooltip_calculation_display: None,
            result_modifier: None,
            tooltip_only: None,
        });

        let mut calculations = HashMap::new();
        calculations.insert(hash, game_calc);

        let spell_object = create_mock_spell_object(calculations, None, None);

        // Test
        let result = get_skill_value(&spell_object, hash, 1, |id| {
            if id == stat_id {
                stat_value
            } else {
                0.0
            }
        });
        assert_eq!(result, Some(expected_value));
    }

    #[test]
    fn test_named_data_value_calculation() {
        // Setup
        let hash = 789;
        let data_name = "BaseDamage";
        let data_name_hash = hash_bin(data_name);
        let expected_value = 50.0;

        let calc_part =
            EnumAbilityResourceByCoefficientCalculationPart::NamedDataValueCalculationPart(
                NamedDataValueCalculationPart {
                    m_data_value: data_name_hash,
                },
            );

        let game_calc = EnumGameCalculation::GameCalculation(GameCalculation {
            m_formula_parts: Some(vec![calc_part]),
            m_display_as_percent: None,
            m_expanded_tooltip_calculation_display: None,
            m_multiplier: None,
            m_precision: None,
            m_simple_tooltip_calculation_display: None,
            result_modifier: None,
            tooltip_only: None,
        });

        let mut calculations = HashMap::new();
        calculations.insert(hash, game_calc);

        let data_values = vec![SpellDataValue {
            m_name: data_name.to_string(),
            m_values: Some(vec![expected_value, 60.0, 70.0]),
        }];

        let spell_object = create_mock_spell_object(calculations, None, Some(data_values));

        // Test
        let result = get_skill_value(&spell_object, hash, 1, |_| 0.0);
        assert_eq!(result, Some(expected_value));
    }
}
