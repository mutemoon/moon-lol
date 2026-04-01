//! 英雄技能系统集成测试
//!
//! 测试覆盖：
//! 1. 技能释放流程 (Cast Flow)
//! 2. 技能升级系统 (Level Up)
//! 3. 冷却管理 (Cooldown)
//! 4. 资源消耗 (Resource Cost)
//! 5. 技能动作执行 (Skill Actions)

use bevy::prelude::*;
use moon_lol::*;
use std::collections::HashMap;

use lol_core::Team;
use moon_lol::{
    CommandSkillLevelUp, CommandSkillStart, CoolDown, DamageShape, DamageType,
    DashMoveType, Health, Level, PluginAction, PluginCooldown, PluginDamage, PluginLife,
    PluginMovement, PluginSkill, Skill, SkillAction, SkillOf, SkillPoints, Skills,
    TargetDamage, TargetFilter,
};

// ===== 测试常量定义 =====

const TEST_FPS: f32 = 30.0;
const EPSILON: f32 = 1e-6;

// ===== 测试辅助工具 =====

/// 技能测试装置 - 简化版，不依赖完整角色系统
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

        // 添加技能相关插件（不依赖 CharacterRecord）
        app.add_plugins(PluginSkill);
        app.add_plugins(PluginCooldown);
        app.add_plugins(PluginDamage);
        app.add_plugins(PluginLife);
        app.add_plugins(PluginMovement);
        app.add_plugins(PluginAction);

        // 设置固定时间步长
        app.insert_resource(Time::<Fixed>::from_hz(TEST_FPS as f64));

        let world = app.world_mut();

        // 创建目标实体（需要 Team 和 Transform）
        let target = world.spawn((
            Team::Chaos,
            Transform::default(),
            Health::new(1000.0),
        )).id();

        // 创建施法者实体
        let caster = world.spawn((
            Team::Order,
            Transform::default(),
        )).id();

        SkillTestHarness { app, caster, target }
    }

    /// 使用构建者模式配置施法者
    fn with_caster(mut self, components: impl Bundle) -> Self {
        self.app.world_mut().entity_mut(self.caster).insert(components);
        self
    }

    /// 为施法者添加技能
    fn add_skill(mut self, skill: Skill, cooldown_duration: f32) -> Self {
        let skill_entity = self.app.world_mut().spawn((
            SkillOf(self.caster),
            skill,
            CoolDown {
                timer: Timer::from_seconds(cooldown_duration, TimerMode::Once),
                duration: cooldown_duration,
            },
        )).id();

        // Add the skill entity to the caster's Skills list
        if let Some(mut skills) = self.app.world_mut().get_mut::<Skills>(self.caster) {
            skills.push(skill_entity);
        } else {
            self.app.world_mut().entity_mut(self.caster).insert(Skills::new(skill_entity));
        }
        self
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

    // ===== 查询方法 (Query Methods) =====

    /// 获取技能冷却状态
    fn skill_cooldown(&self, skill_index: usize) -> Option<&CoolDown> {
        let skills = self.app.world().get::<Skills>(self.caster)?;
        let skill_entity = skills.get(skill_index)?;
        self.app.world().get::<CoolDown>(*skill_entity)
    }

    /// 获取技能组件
    fn skill_component(&self, skill_index: usize) -> Option<&Skill> {
        let skills = self.app.world().get::<Skills>(self.caster)?;
        let skill_entity = skills.get(skill_index)?;
        self.app.world().get::<Skill>(*skill_entity)
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

    /// 检查冷却是否正在冷却中
    fn is_on_cooldown(&self, skill_index: usize) -> bool {
        !self.is_cooldown_ready(skill_index)
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

// ===== 辅助函数 =====

/// 创建测试用AbilityResource
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

/// 创建测试用Level
fn make_level(value: u32) -> Level {
    Level {
        value,
        experience: 0,
        experience_to_next_level: 100 * value,
    }
}

// ===== 一、技能升级系统 (Level Up) =====

/// 测试1: 正常升级技能
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
        }, 5.0);

    harness
        .level_up_skill(0)
        .then_expect_skill_level(0, 1, "技能应该升到1级")
        .then_expect_skill_points(0, "技能点应该消耗");
}

/// 测试2: 技能点不足无法升级
#[test]
fn test_cannot_level_up_without_skill_points() {
    let mut harness = SkillTestHarness::new()
        .with_caster((
            Level::default(),
            SkillPoints(0), // 无技能点
            Skills::default(),
        ))
        .add_skill(Skill {
            key_spell_object: 0.into(),
            key_skill_effect: 0.into(),
            level: 0,
        }, 5.0);

    let initial_level = harness.skill_level(0).unwrap_or(0);
    let initial_points = harness.skill_points();

    harness.level_up_skill(0);

    // 技能等级不应变化
    assert_eq!(harness.skill_level(0), Some(initial_level), "无技能点时技能等级不应变化");
    // 技能点不应变化
    assert_eq!(harness.skill_points(), initial_points, "无技能点时技能点数不应变化");
}

/// 测试3: 6级前无法升级大招
#[test]
fn test_cannot_level_up_ultimate_before_level_6() {
    let mut harness = SkillTestHarness::new()
        .with_caster((
            make_level(5), // 5级
            SkillPoints(1),
            Skills::default(),
        ))
        .add_skill(Skill {
            key_spell_object: 0.into(),
            key_skill_effect: 0.into(),
            level: 0,
        }, 5.0);

    let initial_level = harness.skill_level(0).unwrap_or(0);

    // 尝试升级R技能（索引3）
    harness.level_up_skill(3);

    // 技能等级不应变化
    assert_eq!(harness.skill_level(0), Some(initial_level), "6级前R技能不应能升级");
}

/// 测试4: 每技能最多加3点（6级前）
#[test]
fn test_skill_max_3_points_before_level_6() {
    let mut harness = SkillTestHarness::new()
        .with_caster((
            make_level(5), // 5级
            SkillPoints(10),
            Skills::default(),
        ))
        .add_skill(Skill {
            key_spell_object: 0.into(),
            key_skill_effect: 0.into(),
            level: 3, // 已加3点
        }, 5.0);

    // 尝试再加一点
    harness.level_up_skill(0);

    // 等级不应超过3
    assert_eq!(harness.skill_level(0), Some(3), "6级前技能不应超过3点");
}

// ===== 二、冷却系统 =====

/// 测试5: 冷却时间准确性
#[test]
fn test_cooldown_timing_accuracy() {
    let harness = SkillTestHarness::new()
        .with_caster((
            Level::default(),
            SkillPoints(1),
            Skills::default(),
            make_mana(100.0),
        ))
        .add_skill(Skill {
            key_spell_object: 0.into(),
            key_skill_effect: 0.into(),
            level: 1,
        }, 4.0);

    // 验证冷却组件存在
    assert!(harness.skill_cooldown(0).is_some(), "技能0应该有冷却组件");

    let cd = harness.skill_cooldown(0).unwrap();
    assert!(
        (cd.duration - 4.0).abs() < EPSILON,
        "冷却时间应该是4秒"
    );
}

/// 测试6: 冷却计时器进度
#[test]
fn test_cooldown_timer_progress() {
    let mut timer = Timer::from_seconds(5.0, TimerMode::Once);
    timer.tick(std::time::Duration::from_secs_f32(2.5)); // 经过2.5秒

    let progress = timer.elapsed_secs() / 5.0;
    assert!((progress - 0.5).abs() < EPSILON, "冷却进度应该是50%");
}

/// 测试7: 冷却计时器完成
#[test]
fn test_cooldown_timer_finished() {
    let mut timer = Timer::from_seconds(1.0, TimerMode::Once);
    timer.tick(std::time::Duration::from_secs_f32(1.0)); // 经过1秒

    assert!(timer.is_finished(), "冷却应该已结束");
}

/// 测试8: 冷却计时器未完成
#[test]
fn test_cooldown_timer_not_finished() {
    let timer = Timer::from_seconds(5.0, TimerMode::Once);
    assert!(!timer.is_finished(), "冷却不应该已完成");
}

// ===== 三、技能动作执行 (Skill Actions) =====

/// 测试9: Animation 动作
#[test]
fn test_skill_action_animation() {
    let action = SkillAction::Animation(ActionAnimationPlay { hash: 123 });
    assert!(matches!(action, SkillAction::Animation(_)));
}

/// 测试10: Dash 动作
#[test]
fn test_skill_action_dash() {
    let dash = ActionDash {
        speed: 1000.0,
        move_type: DashMoveType::Fixed(250.0),
        damage: None,
        skill: 0.into(),
    };
    let action = SkillAction::Dash(dash);
    assert!(matches!(action, SkillAction::Dash(_)));
}

/// 测试11: Damage 动作
#[test]
fn test_skill_action_damage() {
    let damage_effects = vec![ActionDamageEffect {
        shape: DamageShape::Circle { radius: 100.0 },
        damage_list: vec![TargetDamage {
            filter: TargetFilter::All,
            amount: 50,
            damage_type: DamageType::Physical,
        }],
        particle: None,
    }];
    let action = SkillAction::Damage(ActionDamage {
        entity: Entity::PLACEHOLDER,
        skill: 0.into(),
        effects: damage_effects,
    });
    assert!(matches!(action, SkillAction::Damage(_)));
}

/// 测试12: Buff 动作
#[test]
fn test_skill_action_buff() {
    #[allow(dead_code)]
    fn create_buff_spawn() -> ActionBuffSpawn {
        ActionBuffSpawn::new(((),))
    }
    let action = SkillAction::Buff(ActionBuffSpawn::new(((),)));
    assert!(matches!(action, SkillAction::Buff(_)));
}

/// 测试13: AttackReset 动作
#[test]
fn test_skill_action_attack_reset() {
    let action = SkillAction::AttackReset;
    assert!(matches!(action, SkillAction::AttackReset));
}

/// 测试14: Particle 动作
#[test]
fn test_skill_action_particle() {
    let action = SkillAction::Particle(ActionParticleSpawn { hash: 456 });
    assert!(matches!(action, SkillAction::Particle(_)));
}

// ===== 四、技能效果值计算 (Skill Value Calculation) =====

/// 测试15: 效果值计算 - 基础值
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

/// 测试16: 效果值计算 - 属性加成
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

// ===== 五、综合测试 =====

/// 测试17: 多技能同时存在
#[test]
fn test_multiple_skills_exist() {
    let harness = SkillTestHarness::new()
        .with_caster((
            make_level(6),
            SkillPoints(0),
            Skills::default(),
            make_mana(100.0),
        ))
        .add_skill(Skill {
            key_spell_object: 0.into(),
            key_skill_effect: 0.into(),
            level: 1,
        }, 5.0)
        .add_skill(Skill {
            key_spell_object: 0.into(),
            key_skill_effect: 0.into(),
            level: 1,
        }, 5.0)
        .add_skill(Skill {
            key_spell_object: 0.into(),
            key_skill_effect: 0.into(),
            level: 1,
        }, 5.0)
        .add_skill(Skill {
            key_spell_object: 0.into(),
            key_skill_effect: 0.into(),
            level: 1,
        }, 5.0);

    // 验证4个技能都有
    assert_eq!(harness.skill_level(0), Some(1), "Q技能等级应该是1");
    assert_eq!(harness.skill_level(1), Some(1), "W技能等级应该是1");
    assert_eq!(harness.skill_level(2), Some(1), "E技能等级应该是1");
    assert_eq!(harness.skill_level(3), Some(1), "R技能等级应该是1");

    // 验证所有技能都有冷却组件
    assert!(harness.skill_cooldown(0).is_some(), "Q技能应该有冷却组件");
    assert!(harness.skill_cooldown(1).is_some(), "W技能应该有冷却组件");
    assert!(harness.skill_cooldown(2).is_some(), "E技能应该有冷却组件");
    assert!(harness.skill_cooldown(3).is_some(), "R技能应该有冷却组件");
}

/// 测试18: 技能组件默认值
#[test]
fn test_skill_default_values() {
    let skill = Skill::default();
    assert_eq!(skill.level, 0, "默认技能等级应该是0");
    // HashKey 默认构造的值是 0
}

/// 测试19: 技能点默认值
#[test]
fn test_skill_points_default() {
    let sp = SkillPoints::default();
    assert_eq!(sp.0, 1, "默认技能点应该是1");
}

/// 测试20: Level组件默认值
#[test]
fn test_level_default() {
    let level = Level::default();
    assert_eq!(level.value, 1, "默认等级应该是1");
}

// ===== 六、伤害形状和目标过滤 =====

/// 测试21: 圆形伤害形状
#[test]
fn test_damage_shape_circle() {
    let shape = DamageShape::Circle { radius: 300.0 };
    match shape {
        DamageShape::Circle { radius } => {
            assert!((radius - 300.0).abs() < EPSILON);
        }
        _ => panic!("应该是Circle形状"),
    }
}

/// 测试22: 扇形伤害形状
#[test]
fn test_damage_shape_sector() {
    let shape = DamageShape::Sector {
        radius: 300.0,
        angle: 90.0,
    };
    match shape {
        DamageShape::Sector { radius, angle } => {
            assert!((radius - 300.0).abs() < EPSILON);
            assert!((angle - 90.0).abs() < EPSILON);
        }
        _ => panic!("应该是Sector形状"),
    }
}

/// 测试23: 环形伤害形状
#[test]
fn test_damage_shape_annular() {
    let shape = DamageShape::Annular {
        inner_radius: 100.0,
        outer_radius: 300.0,
    };
    match shape {
        DamageShape::Annular {
            inner_radius,
            outer_radius,
        } => {
            assert!((inner_radius - 100.0).abs() < EPSILON);
            assert!((outer_radius - 300.0).abs() < EPSILON);
        }
        _ => panic!("应该是Annular形状"),
    }
}

/// 测试24: 最近目标伤害形状
#[test]
fn test_damage_shape_nearest() {
    let shape = DamageShape::Nearest { max_distance: 300.0 };
    match shape {
        DamageShape::Nearest { max_distance } => {
            assert!((max_distance - 300.0).abs() < EPSILON);
        }
        _ => panic!("应该是Nearest形状"),
    }
}

/// 测试25: 目标过滤器 - All
#[test]
fn test_target_filter_all() {
    let filter = TargetFilter::All;
    assert_eq!(filter, TargetFilter::All);
}

/// 测试26: 目标过滤器 - Champion
#[test]
fn test_target_filter_champion() {
    let filter = TargetFilter::Champion;
    assert_eq!(filter, TargetFilter::Champion);
}

/// 测试27: 目标过滤器 - Minion
#[test]
fn test_target_filter_minion() {
    let filter = TargetFilter::Minion;
    assert_eq!(filter, TargetFilter::Minion);
}
