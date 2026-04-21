use std::ops::Deref;

use bevy::prelude::*;
use league_utils::hash_bin;
use lol_base::prop::{HashKey, LoadHashKeyTrait};
use lol_base::spell::Spell;
use lol_base::spell_calc::{
    CalculationPart, CalculationPartEffectValue, CalculationPartNamedDataValue,
    CalculationPartStatCoefficient, CalculationPartStatNamedDataValue, CalculationPartStatSub,
    CalculationType,
};

use crate::action::damage::{ActionDamage, ActionDamageEffect, DamageShape, TargetDamage};
use crate::action::dash::{ActionDash, DashDamageComponent, DashMoveType};
use crate::attack::CommandAttackReset;
use crate::base::ability_resource::AbilityResource;
use crate::base::level::{EventLevelUp, Level};
use crate::movement::{CommandMovement, MovementAction, MovementWay};
use crate::render_cmd::{
    CommandAnimationPlay, CommandSkinParticleDespawn, CommandSkinParticleSpawn,
};

#[derive(Default)]
pub struct PluginSkill;

impl Plugin for PluginSkill {
    fn build(&self, app: &mut App) {
        app.init_resource::<SkillCastLog>();

        app.add_observer(on_skill_cast);
        app.add_observer(on_skill_level_up);
        app.add_observer(on_level_up);
        app.add_systems(FixedUpdate, update_skill_recast_windows);
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

impl Default for Skills {
    fn default() -> Self {
        Skills(Vec::new())
    }
}

impl Skills {
    /// Create a new Skills list with a single skill entity
    pub fn new(entity: Entity) -> Self {
        Skills(vec![entity])
    }

    /// Add a skill entity to this Skills list
    pub fn push(&mut self, entity: Entity) {
        self.0.push(entity);
    }
}

#[derive(Component, Default)]
pub struct CoolDown {
    pub timer: Timer,
    pub duration: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect, Default)]
pub enum SkillSlot {
    Passive,
    #[default]
    Q,
    W,
    E,
    R,
    Custom(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect, Default)]
pub enum SkillCooldownMode {
    #[default]
    AfterCast,
    Manual,
}

#[derive(Component)]
#[require(CoolDown)]
pub struct Skill {
    pub key_spell_object: HashKey<Spell>,
    pub level: usize,
    pub slot: SkillSlot,
    pub cooldown_mode: SkillCooldownMode,
}

impl Default for Skill {
    fn default() -> Self {
        Self {
            key_spell_object: 0.into(),
            level: 0,
            slot: SkillSlot::Q,
            cooldown_mode: SkillCooldownMode::AfterCast,
        }
    }
}

impl Skill {
    pub fn new(slot: SkillSlot, key_spell_object: impl Into<HashKey<Spell>>) -> Self {
        Self {
            key_spell_object: key_spell_object.into(),
            level: 0,
            slot,
            cooldown_mode: SkillCooldownMode::AfterCast,
        }
    }

    pub fn with_level(mut self, level: usize) -> Self {
        self.level = level;
        self
    }

    pub fn with_cooldown_mode(mut self, cooldown_mode: SkillCooldownMode) -> Self {
        self.cooldown_mode = cooldown_mode;
        self
    }
}

pub fn skill_slot_from_index(index: usize) -> SkillSlot {
    match index {
        0 => SkillSlot::Q,
        1 => SkillSlot::W,
        2 => SkillSlot::E,
        3 => SkillSlot::R,
        other => SkillSlot::Custom(other as u8),
    }
}

#[derive(Component)]
pub struct SkillPoints(pub u32);

impl Default for SkillPoints {
    fn default() -> Self {
        Self(1)
    }
}

#[derive(Component, Debug, Clone)]
pub struct SkillRecastWindow {
    pub stage: u8,
    pub max_stage: u8,
    pub timer: Timer,
}

impl SkillRecastWindow {
    pub fn new(stage: u8, max_stage: u8, duration: f32) -> Self {
        Self {
            stage,
            max_stage,
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillCastFailureReason {
    MissingSkills,
    InvalidSkillIndex,
    MissingSkillEntity,
    MissingSpellObject,
    NotLearned,
    MissingAbilityResource,
    InsufficientAbilityResource,
    CoolingDown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillCastResult {
    Started,
    Failed(SkillCastFailureReason),
}

#[derive(Debug, Clone)]
pub struct SkillCastRecord {
    pub caster: Entity,
    pub skill_entity: Option<Entity>,
    pub index: usize,
    pub slot: Option<SkillSlot>,
    pub point: Vec2,
    pub result: SkillCastResult,
}

#[derive(Resource, Default, Debug)]
pub struct SkillCastLog(pub Vec<SkillCastRecord>);

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

#[derive(EntityEvent, Debug, Clone, Copy)]
pub struct EventSkillCast {
    pub entity: Entity,
    pub skill_entity: Entity,
    pub index: usize,
    pub point: Vec2,
}

fn push_skill_log(
    log: &mut ResMut<SkillCastLog>,
    caster: Entity,
    skill_entity: Option<Entity>,
    index: usize,
    slot: Option<SkillSlot>,
    point: Vec2,
    result: SkillCastResult,
) {
    log.0.push(SkillCastRecord {
        caster,
        skill_entity,
        index,
        slot,
        point,
        result,
    });
}

fn on_skill_cast(
    trigger: On<CommandSkillStart>,
    mut commands: Commands,
    skills: Query<&Skills>,
    res_assets_spell_object: Res<Assets<Spell>>,
    mut q_skill: Query<(&Skill, &mut CoolDown)>,
    mut q_ability_resource: Query<&mut AbilityResource>,
    mut log: ResMut<SkillCastLog>,
) {
    let entity = trigger.event_target();
    let Ok(skills) = skills.get(entity) else {
        push_skill_log(
            &mut log,
            entity,
            None,
            trigger.index,
            None,
            trigger.point,
            SkillCastResult::Failed(SkillCastFailureReason::MissingSkills),
        );
        return;
    };
    let Some(&skill_entity) = skills.0.get(trigger.index) else {
        push_skill_log(
            &mut log,
            entity,
            None,
            trigger.index,
            None,
            trigger.point,
            SkillCastResult::Failed(SkillCastFailureReason::InvalidSkillIndex),
        );
        return;
    };
    let Ok((skill, mut cooldown)) = q_skill.get_mut(skill_entity) else {
        push_skill_log(
            &mut log,
            entity,
            Some(skill_entity),
            trigger.index,
            None,
            trigger.point,
            SkillCastResult::Failed(SkillCastFailureReason::MissingSkillEntity),
        );
        return;
    };

    if !cooldown.timer.is_finished() {
        debug!(
            "{} 技能 {} 冷却中，剩余 {:.2}s",
            entity,
            trigger.index,
            cooldown.timer.remaining_secs()
        );
        push_skill_log(
            &mut log,
            entity,
            Some(skill_entity),
            trigger.index,
            Some(skill.slot),
            trigger.point,
            SkillCastResult::Failed(SkillCastFailureReason::CoolingDown),
        );
        return;
    }

    let Some(spell_object) = res_assets_spell_object.load_hash(skill.key_spell_object) else {
        push_skill_log(
            &mut log,
            entity,
            Some(skill_entity),
            trigger.index,
            Some(skill.slot),
            trigger.point,
            SkillCastResult::Failed(SkillCastFailureReason::MissingSpellObject),
        );
        return;
    };

    if skill.level == 0 {
        debug!("{} 技能 {} 未学习，无法释放", entity, trigger.index);
        push_skill_log(
            &mut log,
            entity,
            Some(skill_entity),
            trigger.index,
            Some(skill.slot),
            trigger.point,
            SkillCastResult::Failed(SkillCastFailureReason::NotLearned),
        );
        return;
    }

    let Ok(mut ability_resource) = q_ability_resource.get_mut(entity) else {
        push_skill_log(
            &mut log,
            entity,
            Some(skill_entity),
            trigger.index,
            Some(skill.slot),
            trigger.point,
            SkillCastResult::Failed(SkillCastFailureReason::MissingAbilityResource),
        );
        return;
    };

    if let Some(ref mana) = spell_object.spell_data.as_ref().unwrap().mana {
        let &current_mana = mana.get(skill.level as usize).unwrap();

        if ability_resource.value < current_mana {
            debug!(
                "{} 技能 {} 蓝量不足，需要 {:.0}，当前 {:.0}",
                entity, trigger.index, current_mana, ability_resource.value
            );
            push_skill_log(
                &mut log,
                entity,
                Some(skill_entity),
                trigger.index,
                Some(skill.slot),
                trigger.point,
                SkillCastResult::Failed(SkillCastFailureReason::InsufficientAbilityResource),
            );
            return;
        }

        ability_resource.value -= current_mana;
        debug!(
            "{} 技能 {} 消耗 {:.0} 蓝量，剩余 {:.0}",
            entity, trigger.index, current_mana, ability_resource.value
        );
    }

    push_skill_log(
        &mut log,
        entity,
        Some(skill_entity),
        trigger.index,
        Some(skill.slot),
        trigger.point,
        SkillCastResult::Started,
    );

    let cast_event = EventSkillCast {
        entity,
        skill_entity,
        index: trigger.index,
        point: trigger.point,
    };

    debug!("{} 技能 {} 进入代码驱动观察者流程", entity, trigger.index);
    commands.trigger(cast_event);

    if skill.cooldown_mode == SkillCooldownMode::AfterCast {
        cooldown.timer = Timer::from_seconds(cooldown.duration, TimerMode::Once);
        debug!(
            "{} 技能 {} 开始冷却 {}s",
            entity, trigger.index, cooldown.duration
        );
    }
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

fn update_skill_recast_windows(
    time: Res<Time<Fixed>>,
    mut commands: Commands,
    mut q_skill_window: Query<(Entity, &mut SkillRecastWindow)>,
) {
    for (entity, mut window) in q_skill_window.iter_mut() {
        window.timer.tick(time.delta());
        if window.timer.is_finished() {
            commands.entity(entity).remove::<SkillRecastWindow>();
        }
    }
}

pub fn play_skill_animation(commands: &mut Commands, entity: Entity, hash: u32) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash,
        repeat: false,
        duration: None,
    });
}

pub fn spawn_skill_particle(commands: &mut Commands, entity: Entity, hash: u32) {
    commands.trigger(CommandSkinParticleSpawn { entity, hash });
}

pub fn despawn_skill_particle(commands: &mut Commands, entity: Entity, hash: u32) {
    commands.trigger(CommandSkinParticleDespawn { entity, hash });
}

pub fn reset_skill_attack(commands: &mut Commands, entity: Entity) {
    commands.trigger(CommandAttackReset { entity });
}

pub fn skill_damage(
    commands: &mut Commands,
    entity: Entity,
    skill: impl Into<HashKey<Spell>>,
    shape: DamageShape,
    damage_list: Vec<TargetDamage>,
    particle: Option<u32>,
) {
    commands.trigger(ActionDamage {
        entity,
        skill: skill.into(),
        effects: vec![ActionDamageEffect {
            shape,
            damage_list,
            particle,
        }],
    });
}

pub fn skill_dash(
    commands: &mut Commands,
    q_transform: &Query<&Transform>,
    entity: Entity,
    point: Vec2,
    dash: &ActionDash,
) {
    let Ok(transform) = q_transform.get(entity) else {
        return;
    };
    let vector = point - transform.translation.xz();
    let distance = vector.length();

    let destination = match dash.move_type {
        DashMoveType::Fixed(fixed_distance) => {
            let direction = if distance < 0.001 {
                transform.forward().xz().normalize()
            } else {
                vector.normalize()
            };
            transform.translation.xz() + direction * fixed_distance
        }
        DashMoveType::Pointer { max } => {
            if distance < max {
                point
            } else {
                let direction = vector.normalize();
                transform.translation.xz() + direction * max
            }
        }
    };

    if let Some(damage) = &dash.damage {
        commands.entity(entity).insert(DashDamageComponent {
            start_pos: transform.translation,
            target_pos: Vec3::new(destination.x, transform.translation.y, destination.y),
            damage: damage.clone(),
            skill: dash.skill,
            hit_entities: std::collections::HashSet::default(),
        });
    }

    commands.trigger(CommandMovement {
        entity,
        priority: 100,
        action: MovementAction::Start {
            way: MovementWay::Path(vec![Vec3::new(
                destination.x,
                transform.translation.y,
                destination.y,
            )]),
            speed: Some(dash.speed),
            source: "Dash".to_string(),
        },
    });
}

pub fn get_skill_value(
    skill_object: &Spell,
    hash: u32,
    level: usize,
    get_stat: impl Fn(u8) -> f32,
) -> Option<f32> {
    let spell = skill_object.spell_data.as_ref()?;
    let calculations = spell.calculations.as_ref()?;
    let calculation = calculations.get(&hash)?;

    match calculation {
        CalculationType::CalculationSpell(calc) => {
            let mut value = 0.0;
            if let Some(parts) = &calc.formula_parts {
                for part in parts {
                    value += calculate_part(part, skill_object, level, &get_stat);
                }
            }

            if let Some(multiplier) = &calc.multiplier {
                value *= calculate_part(multiplier, skill_object, level, &get_stat);
            }
            Some(value)
        }
        _ => todo!("CalculationType variant not implemented: {:?}", calculation),
    }
}

fn calculate_part(
    part: &CalculationPart,
    skill_object: &Spell,
    level: usize,
    get_stat: &impl Fn(u8) -> f32,
) -> f32 {
    match part {
        CalculationPart::CalculationPartEffectValue(CalculationPartEffectValue {
            effect_index,
        }) => {
            let index = effect_index.unwrap_or(1) - 1;
            if let Some(effect_amount) = skill_object
                .spell_data
                .as_ref()
                .and_then(|s| s.effect_amounts.as_ref())
                .and_then(|v| v.get(index as usize))
            {
                if let Some(values) = &effect_amount.values {
                    // level is 1-based, so index is level - 1
                    // Ensure level is at least 1
                    let lvl_idx = if level > 0 { level - 1 } else { 0 };
                    return *values.get(lvl_idx).unwrap_or(&0.0);
                }
            }
            0.0
        }
        CalculationPart::CalculationPartStatCoefficient(CalculationPartStatCoefficient {
            stat,
            coefficient,
            ..
        }) => {
            let stat = stat.unwrap_or(0);
            let coefficient = coefficient.unwrap_or(0.0);
            get_stat(stat) * coefficient
        }
        CalculationPart::CalculationPartNamedDataValue(CalculationPartNamedDataValue {
            data_value,
        }) => {
            if let Some(data_values) = skill_object
                .spell_data
                .as_ref()
                .and_then(|s| s.data_values.as_ref())
            {
                for dv in data_values {
                    // Check if hash of name matches data_value
                    let hash = hash_bin(&dv.name);
                    if hash == *data_value {
                        if let Some(values) = &dv.values {
                            let lvl_idx = if level > 0 { level - 1 } else { 0 };
                            return *values.get(lvl_idx).unwrap_or(&0.0);
                        }
                    }
                }
            }
            0.0
        }
        CalculationPart::CalculationPartStatSub(CalculationPartStatSub {
            stat, subpart, ..
        }) => {
            let stat_val = stat.unwrap_or(0);
            let sub_val = if let Some(sub) = subpart {
                calculate_part(sub, skill_object, level, get_stat)
            } else {
                0.0
            };
            get_stat(stat_val) * sub_val
        }
        CalculationPart::CalculationPartStatNamedDataValue(CalculationPartStatNamedDataValue {
            stat,
            data_value,
            ..
        }) => {
            let stat = stat.unwrap_or(0);
            let mut data_val = 0.0;
            if let Some(data_values) = skill_object
                .spell_data
                .as_ref()
                .and_then(|s| s.data_values.as_ref())
            {
                for dv in data_values {
                    let hash = hash_bin(&dv.name);
                    if hash == *data_value {
                        if let Some(values) = &dv.values {
                            let lvl_idx = if level > 0 { level - 1 } else { 0 };
                            data_val = *values.get(lvl_idx).unwrap_or(&0.0);
                            break;
                        }
                    }
                }
            }
            get_stat(stat) * data_val
        }
        _ => todo!("CalculationPart not implemented: {:?}", part),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use lol_base::spell::{DataSpell, Spell, ValuesData, ValuesEffect};
    use lol_base::spell_calc::{
        CalculationPart, CalculationPartEffectValue, CalculationPartNamedDataValue,
        CalculationPartStatCoefficient, CalculationSpell, CalculationType,
    };

    use super::*;

    fn create_mock_spell(
        calculations: HashMap<u32, CalculationType>,
        effect_amounts: Option<Vec<ValuesEffect>>,
        data_values: Option<Vec<ValuesData>>,
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
        // Setup
        let hash = 123;
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

        let mut calculations = HashMap::new();
        calculations.insert(hash, calc);

        let effect_amounts = vec![ValuesEffect {
            values: Some(vec![expected_value_lvl1, expected_value_lvl2, 30.0]),
        }];

        let spell = create_mock_spell(calculations, Some(effect_amounts), None);

        // Test Level 1
        let result = get_skill_value(&spell, hash, 1, |_| 0.0);
        assert_eq!(result, Some(expected_value_lvl1));

        // Test Level 2
        let result = get_skill_value(&spell, hash, 2, |_| 0.0);
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

        let mut calculations = HashMap::new();
        calculations.insert(hash, calc);

        let spell = create_mock_spell(calculations, None, None);

        // Test
        let result = get_skill_value(&spell, hash, 1, |id| {
            if id == stat_id { stat_value } else { 0.0 }
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
            CalculationPart::CalculationPartNamedDataValue(CalculationPartNamedDataValue {
                data_value: data_name_hash,
            });

        let calc = CalculationType::CalculationSpell(CalculationSpell {
            formula_parts: Some(vec![calc_part]),
            multiplier: None,
            precision: None,
        });

        let mut calculations = HashMap::new();
        calculations.insert(hash, calc);

        let data_values = vec![ValuesData {
            name: data_name.to_string(),
            values: Some(vec![expected_value, 60.0, 70.0]),
        }];

        let spell = create_mock_spell(calculations, None, Some(data_values));

        // Test
        let result = get_skill_value(&spell, hash, 1, |_| 0.0);
        assert_eq!(result, Some(expected_value));
    }
}
