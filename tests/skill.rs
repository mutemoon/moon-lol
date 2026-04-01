//! 英雄技能系统测试
//!
//! 测试覆盖范围：
//! 1. 技能释放流程 (Cast Flow)
//!    - 正常释放
//!    - 冷却中无法释放
//!    - 蓝量不足无法释放
//!    - 未学习技能无法释放
//!
//! 2. 技能升级系统 (Level Up)
//!    - 正常升级
//!    - 技能点不足无法升级
//!    - 等级限制 (6级前R技能无法升级)
//!    - 每技能最大3点限制
//!
//! 3. 冷却管理 (Cooldown)
//!    - 冷却时间准确性
//!    - 冷却结束后可释放
//!    - CDR影响
//!
//! 4. 资源消耗 (Resource Cost)
//!    - 蓝耗扣除
//!    - 蓝量不足检查
//!
//! 5. 技能动作执行 (Skill Actions)
//!    - Animation
//!    - Dash
//!    - Damage
//!    - Buff
//!    - Particle
//!
//! 6. 边缘情况 (Edge Cases)
//!    - 目标死亡后释放
//!    - 多技能同时冷却
//!    - 技能取消

use std::collections::HashMap;

use bevy::prelude::*;
use moon_lol::*;

use lol_core::Team;
use moon_lol::get_skill_value;
use moon_lol::{
    ActionAnimationPlay, ActionBuffSpawn, ActionDamage, ActionDash, ActionParticleDespawn,
    ActionParticleSpawn, CommandSkillLevelUp, CommandSkillStart, CoolDown, DashDamage, DashMoveType,
    Level, PluginAction, PluginAttack, PluginCooldown, PluginSkill, Skill, SkillAction, SkillEffect,
    SkillPoints, Skills,
};

// ===== 测试常量定义 =====

const TEST_FPS: f32 = 30.0;
const EPSILON: f32 = 1e-6;

// ===== 测试辅助工具 =====

/// 技能测试装置
struct SkillTestHarness {
    app: App,
    caster: Entity,
    target: Entity,
}

impl SkillTestHarness {
    /// 创建新的测试装置
    fn new() -> Self {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(AssetPlugin::default());
        app.add_plugins(PluginSkill);
        app.add_plugins(PluginAction);
        app.add_plugins(PluginCooldown);
        app.add_plugins(PluginAttack);
        app.insert_resource(Time::<Fixed>::from_hz(TEST_FPS as f64));

        let target = app.world_mut().spawn_empty().id();
        let caster = app.world_mut().spawn_empty().id();

        SkillTestHarness { app, caster, target }
    }

    /// 使用构建者模式配置施法者
    fn with_caster(mut self, components: impl Bundle) -> Self {
        self.app.world_mut().entity_mut(self.caster).insert(components);
        self
    }

    /// 为施法者添加技能
    fn add_skill(mut self, skill: Skill) -> Self {
        let skill_entity = self.app.world_mut().spawn((SkillOf(self.caster), skill, CoolDown::default())).id();
        // Add the skill entity to the caster's Skills list
        if let Some(mut skills) = self.app.world_mut().get_mut::<Skills>(self.caster) {
            skills.push(skill_entity);
        }
        self
    }

    /// 创建额外的目标实体
    fn spawn_target(&mut self) -> Entity {
        self.app.world_mut().spawn_empty().id()
    }

    /// 创建敌人实体
    fn spawn_enemy(&mut self, team: Team) -> Entity {
        self.app.world_mut().spawn((team, Transform::default())).id()
    }

    // ===== 动作方法 (Action Methods) =====

    /// 推进时间
    fn advance_time(&mut self, seconds: f32) -> &mut Self {
        let ticks = (seconds * TEST_FPS).ceil() as u32;
        for _ in 0..ticks {
            let mut time = self.app.world_mut().resource_mut::<Time<Fixed>>();
            time.advance_by(std::time::Duration::from_secs_f64(1.0 / TEST_FPS as f64));
            drop(time);
            self.app.world_mut().run_schedule(FixedUpdate);
        }
        self
    }

    /// 释放技能
    fn cast_skill(&mut self, index: usize, point: Vec2) -> &mut Self {
        self.app.world_mut().trigger(CommandSkillStart {
            entity: self.caster,
            index,
            point,
        });
        self.app.update();
        self
    }

    /// 升级技能
    fn level_up_skill(&mut self, index: usize) -> &mut Self {
        self.app.world_mut().trigger(CommandSkillLevelUp {
            entity: self.caster,
            index,
        });
        self.app.update();
        self
    }

    /// 切换当前目标
    fn switch_target(&mut self, new_target: Entity) -> &mut Self {
        self.target = new_target;
        self
    }

    // ===== 查询方法 (Query Methods) =====

    /// 获取技能冷却状态
    fn skill_cooldown(&self, skill_index: usize) -> Option<&CoolDown> {
        let skills = self.app.world().get::<Skills>(self.caster)?;
        let skill_entity = skills.get(skill_index)?;
        if let Some(cd) = self.app.world().get::<CoolDown>(*skill_entity) {
            Some(cd)
        } else {
            None
        }
    }

    /// 获取技能组件
    fn skill_component(&self, skill_index: usize) -> Option<&Skill> {
        let skills = self.app.world().get::<Skills>(self.caster)?;
        let skill_entity = skills.get(skill_index)?;
        if let Some(skill) = self.app.world().get::<Skill>(*skill_entity) {
            Some(skill)
        } else {
            None
        }
    }

    /// 获取技能等级
    fn skill_level(&self, skill_index: usize) -> Option<usize> {
        self.skill_component(skill_index).map(|s| s.level)
    }

    /// 检查冷却是否结束
    fn is_cooldown_ready(&self, skill_index: usize) -> bool {
        self.skill_cooldown(skill_index)
            .map(|cd| cd.timer.is_finished())
            .unwrap_or(true)
    }

    /// 获取当前蓝量
    fn current_mana(&self) -> f32 {
        self.app
            .world()
            .get::<AbilityResource>(self.caster)
            .map(|r| r.value)
            .unwrap_or(0.0)
    }

    /// 获取技能点
    fn skill_points(&self) -> u32 {
        self.app
            .world()
            .get::<SkillPoints>(self.caster)
            .map(|sp| sp.0)
            .unwrap_or(0)
    }

    /// 获取英雄等级
    fn champion_level(&self) -> u32 {
        self.app
            .world()
            .get::<Level>(self.caster)
            .map(|l| l.value)
            .unwrap_or(0)
    }

    // ===== 断言方法 (Assertion Methods) =====

    /// 断言技能可释放（冷却结束且已学习）
    fn then_expect_skill_ready(&mut self, index: usize, message: &str) -> &mut Self {
        assert!(
            self.is_cooldown_ready(index),
            "{}: 技能 {} 应该冷却结束",
            message,
            index
        );
        assert!(
            self.skill_level(index).unwrap_or(0) > 0,
            "{}: 技能 {} 应该已学习",
            message,
            index
        );
        self
    }

    /// 断言技能在冷却中
    fn then_expect_skill_on_cooldown(
        &mut self,
        index: usize,
        message: &str,
    ) -> &mut Self {
        let cd = self.skill_cooldown(index);
        assert!(
            cd.is_some(),
            "{}: 技能 {} 应该有冷却组件",
            message,
            index
        );
        assert!(
            !cd.unwrap().timer.is_finished(),
            "{}: 技能 {} 应该冷却中",
            message,
            index
        );
        self
    }

    /// 断言技能等级
    fn then_expect_skill_level(
        &mut self,
        index: usize,
        expected_level: usize,
        message: &str,
    ) -> &mut Self {
        let actual_level = self.skill_level(index).unwrap_or(0);
        assert_eq!(
            actual_level, expected_level,
            "{}: 技能 {} 等级应该是 {}, 实际是 {}",
            message, index, expected_level, actual_level
        );
        self
    }

    /// 断言蓝量
    fn then_expect_mana(&mut self, expected: f32, message: &str) -> &mut Self {
        let actual = self.current_mana();
        assert!(
            (actual - expected).abs() < EPSILON,
            "{}: 蓝量应该是 {:.2}, 实际是 {:.2}",
            message,
            expected,
            actual
        );
        self
    }

    /// 断言技能点
    fn then_expect_skill_points(&mut self, expected: u32, message: &str) -> &mut Self {
        let actual = self.skill_points();
        assert_eq!(
            actual, expected,
            "{}: 技能点应该是 {}, 实际是 {}",
            message, expected, actual
        );
        self
    }

    /// 断言英雄等级
    fn then_expect_level(&mut self, expected: u32, message: &str) -> &mut Self {
        let actual = self.champion_level();
        assert_eq!(
            actual, expected,
            "{}: 英雄等级应该是 {}, 实际是 {}",
            message, expected, actual
        );
        self
    }
}

// ===== 辅助工具 =====

/// 技能效果构建器
struct SkillEffectBuilder(Vec<SkillAction>);

impl SkillEffectBuilder {
    fn new() -> Self {
        Self(Vec::new())
    }

    fn animation(mut self, hash: u32) -> Self {
        self.0.push(SkillAction::Animation(ActionAnimationPlay { hash }));
        self
    }

    fn particle(mut self, hash: u32) -> Self {
        self.0.push(SkillAction::Particle(ActionParticleSpawn { hash }));
        self
    }

    fn dash(mut self, speed: f32, move_type: DashMoveType, damage: Option<DashDamage>) -> Self {
        self.0.push(SkillAction::Dash(ActionDash {
            speed,
            move_type,
            damage,
            skill: 0.into(),
        }));
        self
    }

    fn damage(mut self, skill: u32, effects: Vec<ActionDamageEffect>) -> Self {
        self.0.push(SkillAction::Damage(ActionDamage { entity: Entity::PLACEHOLDER, skill: skill.into(), effects }));
        self
    }

    fn attack_reset(mut self) -> Self {
        self.0.push(SkillAction::AttackReset);
        self
    }

    fn build(self) -> SkillEffect {
        SkillEffect(self.0)
    }
}

// ===== 一、技能释放流程 (Cast Flow) =====

/// 目标 1: 正常释放已学习的技能
#[test]
fn test_cast_learned_skill_successfully() {
    // TODO: 设置施法者有足够的蓝量、已学习的技能
    // 释放技能
    // 验证冷却开始、蓝量消耗、技能动作执行
}

/// 目标 2: 冷却中无法释放技能
#[test]
fn test_cannot_cast_skill_during_cooldown() {
    // TODO: 设置施法者、技能正在冷却
    // 尝试释放技能
    // 验证技能未释放、冷却时间未重置
}

/// 目标 3: 蓝量不足无法释放技能
#[test]
fn test_cannot_cast_skill_insufficient_mana() {
    // TODO: 设置施法者蓝量低于技能消耗
    // 尝试释放技能
    // 验证蓝量未消耗、技能未释放
}

/// 目标 4: 未学习的技能无法释放
#[test]
fn test_cannot_cast_unlearned_skill() {
    // TODO: 技能等级为0
    // 尝试释放技能
    // 验证技能未释放
}

// ===== 二、技能升级系统 (Level Up) =====

/// 目标 5: 正常升级技能
#[test]
fn test_level_up_skill_normally() {
    let mut harness = SkillTestHarness::new()
        .with_caster((
            Level::default(),
            SkillPoints(1),
            Skills::default(),
        ))
        .add_skill(Skill {
            key_spell_object: 0.into(),
            key_skill_effect: 0.into(),
            level: 0,
        });

    harness
        .level_up_skill(0)
        .then_expect_skill_level(0, 1, "技能应该升到1级")
        .then_expect_skill_points(0, "技能点应该消耗");
}

/// 目标 6: 技能点不足无法升级
#[test]
fn test_cannot_level_up_without_skill_points() {
    // TODO: 技能点为0
    // 尝试升级
    // 验证技能等级未变化、技能点未消耗
}

/// 目标 7: 6级前无法升级大招
#[test]
fn test_cannot_level_up_ultimate_before_level_6() {
    // TODO: 英雄等级 < 6，尝试升级R技能
    // 验证升级失败
}

/// 目标 8: 每技能最多加3点（6级前）
#[test]
fn test_skill_max_3_points_before_level_6() {
    // TODO: 英雄等级 < 6，技能已加3点
    // 尝试再升级
    // 验证升级失败
}

// ===== 三、冷却管理 (Cooldown) =====

/// 目标 9: 冷却时间准确性
#[test]
fn test_cooldown_timing_accuracy() {
    // TODO: 设置冷却时间
    // 推进时间
    // 验证冷却结束时间精确
}

/// 目标 10: 冷却结束后可释放
#[test]
fn test_can_cast_after_cooldown_ends() {
    // TODO: 冷却结束
    // 释放技能
    // 验证成功
}

// ===== 四、资源消耗 (Resource Cost) =====

/// 目标 11: 蓝耗正确扣除
#[test]
fn test_mana_cost_deducted_correctly() {
    // TODO: 设置蓝量和技能蓝耗
    // 释放技能
    // 验证蓝量减少正确
}

/// 目标 12: 无蓝耗技能（部分技能不消耗资源）
#[test]
fn test_no_resource_cost_skill() {
    // TODO: 技能无蓝耗
    // 释放技能
    // 验证蓝量不变
}

// ===== 五、技能动作执行 (Skill Actions) =====

/// 目标 13: Animation 动作执行
#[test]
fn test_skill_action_animation() {
    // TODO: 技能包含动画动作
    // 释放技能
    // 验证动画事件触发
}

/// 目标 14: Dash 动作执行
#[test]
fn test_skill_action_dash() {
    // TODO: 技能包含位移动作
    // 释放技能
    // 验证位移事件触发、位置变化正确
}

/// 目标 15: Damage 动作执行
#[test]
fn test_skill_action_damage() {
    // TODO: 技能包含伤害动作
    // 释放技能
    // 验证伤害事件触发
}

/// 目标 16: Buff 动作执行
#[test]
fn test_skill_action_buff() {
    // TODO: 技能包含Buff动作
    // 释放技能
    // 验证Buff应用正确
}

// ===== 六、边缘情况 (Edge Cases) =====

/// 目标 17: 多技能同时冷却
#[test]
fn test_multiple_skills_on_cooldown() {
    // TODO: 多个技能同时释放
    // 验证各自独立冷却
}

/// 目标 18: 连续释放同一技能
#[test]
fn test_consecutive_cast_same_skill() {
    // TODO: 第一个技能释放后立即再次释放
    // 验证第二次失败（冷却中）
}

/// 目标 19: 技能升级后立即释放
#[test]
fn test_cast_after_immediate_level_up() {
    // TODO: 升级后立即释放
    // 验证成功
}

// ===== 七、技能效果值计算 (Skill Value Calculation) =====

/// 目标 20: 效果值计算 - 基础值
#[test]
fn test_skill_value_calculation_effect_amount() {
    use league_core::{
        EffectValueCalculationPart, EnumAbilityResourceByCoefficientCalculationPart,
        EnumGameCalculation, GameCalculation, SpellObject,
    };

    let mut calculations = HashMap::new();
    let hash = 123u32;

    let calc_part =
        EnumAbilityResourceByCoefficientCalculationPart::EffectValueCalculationPart(
            EffectValueCalculationPart {
                m_effect_index: Some(1),
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

    calculations.insert(hash, game_calc);

    let effect_amounts = vec![league_core::SpellEffectAmount {
        value: Some(vec![10.0, 20.0, 30.0, 40.0, 50.0]),
    }];

    let spell_object = SpellObject {
        m_spell: Some(league_core::SpellDataResource {
            m_spell_calculations: Some(calculations),
            m_effect_amount: Some(effect_amounts),
            data_values: None,
            ..Default::default()
        }),
        bot_data: None,
        cc_behavior_data: None,
        m_buff: None,
        m_script_name: String::new(),
        object_name: String::new(),
        script: None,
    };

    // 测试各等级效果值
    assert_eq!(get_skill_value(&spell_object, hash, 1, |_| 0.0), Some(10.0));
    assert_eq!(get_skill_value(&spell_object, hash, 2, |_| 0.0), Some(20.0));
    assert_eq!(get_skill_value(&spell_object, hash, 3, |_| 0.0), Some(30.0));
    assert_eq!(get_skill_value(&spell_object, hash, 4, |_| 0.0), Some(40.0));
    assert_eq!(get_skill_value(&spell_object, hash, 5, |_| 0.0), Some(50.0));
}

/// 目标 21: 效果值计算 - 属性加成
#[test]
fn test_skill_value_calculation_stat_coefficient() {
    use league_core::{
        EnumAbilityResourceByCoefficientCalculationPart, EnumGameCalculation,
        GameCalculation, StatByCoefficientCalculationPart, SpellObject,
    };

    let mut calculations = HashMap::new();
    let hash = 456u32;
    let stat_id = 2u8; // Attack Damage
    let coefficient = 1.5;
    let stat_value = 100.0;
    let expected = stat_value * coefficient;

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

    calculations.insert(hash, game_calc);

    let spell_object = SpellObject {
        m_spell: Some(league_core::SpellDataResource {
            m_spell_calculations: Some(calculations),
            m_effect_amount: None,
            data_values: None,
            ..Default::default()
        }),
        bot_data: None,
        cc_behavior_data: None,
        m_buff: None,
        m_script_name: String::new(),
        object_name: String::new(),
        script: None,
    };

    let result = get_skill_value(&spell_object, hash, 1, |id| {
        if id == stat_id {
            stat_value
        } else {
            0.0
        }
    });

    assert_eq!(result, Some(expected));
}

// ===== 八、集成测试 (Integration Tests) =====

/// 目标 22: 完整技能连招测试
#[test]
fn test_full_skill_combo() {
    // TODO: 测试完整的技能连招
    // 例如: R > Q > W > E
    // 验证每个技能正确执行、冷却正确、资源正确消耗
}

/// 目标 23: 技能与普攻交互
#[test]
fn test_skill_attack_interaction() {
    // TODO: 技能释放后接普攻
    // 验证互相影响（重置普攻、被打断等）
}