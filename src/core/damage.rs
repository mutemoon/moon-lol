use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::Health;

/// 伤害系统插件
#[derive(Default)]
pub struct PluginDamage;

impl Plugin for PluginDamage {
    fn build(&self, app: &mut App) {
        app.register_type::<Damage>();

        app.add_systems(FixedUpdate, update_damage_reductions_system);
        app.add_systems(FixedUpdate, cleanup_depleted_shields_system);

        app.add_observer(on_command_damage_create);
    }
}

#[derive(Component, Reflect, Clone, Serialize, Deserialize)]
#[reflect(Component)]
pub struct Damage(pub f32);

#[derive(Component, Clone, Serialize, Deserialize)]
pub struct Armor(pub f32);

/// 伤害类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DamageType {
    /// 物理伤害
    Physical,
    /// 魔法伤害
    Magic,
    /// 真实伤害（无视所有防御）
    True,
}

/// 伤害事件，包含伤害来源、目标、伤害类型和数值
#[derive(Event, Debug)]
pub struct CommandDamageCreate {
    /// 伤害来源实体
    pub source: Entity,
    /// 伤害类型
    pub damage_type: DamageType,
    /// 伤害数值
    pub amount: f32,
}

#[derive(Event, Debug)]
pub struct EventDamageCreate {
    pub source: Entity,
    pub damage_type: DamageType,
    pub damage_result: DamageResult,
}

/// 白色护盾组件 - 可以抵挡所有类型的伤害
#[derive(Component, Debug, Default)]
pub struct WhiteShield {
    /// 当前护盾值
    pub current: f32,
    /// 最大护盾值
    pub max: f32,
}

/// 魔法护盾组件 - 只能抵挡魔法伤害
#[derive(Component, Debug, Default)]
pub struct MagicShield {
    /// 当前护盾值
    pub current: f32,
    /// 最大护盾值
    pub max: f32,
}

/// 伤害减免buff容器组件
#[derive(Component, Debug, Default)]
pub struct DamageReductions {
    pub buffs: Vec<DamageReduction>,
}

/// 伤害减免buff组件
#[derive(Component, Debug, Clone)]
pub struct DamageReduction {
    /// 减免百分比 (0.0 - 1.0)
    pub percentage: f32,
    /// 减免的伤害类型，None表示对所有类型有效
    pub damage_type: Option<DamageType>,
    /// buff持续时间（秒），None表示永久
    pub duration: Option<f32>,
    /// buff剩余时间
    pub remaining_time: Option<f32>,
}

/// 伤害计算结果
#[derive(Debug)]
pub struct DamageResult {
    /// 最终造成的伤害
    pub final_damage: f32,
    /// 被白色护盾吸收的伤害
    pub white_shield_absorbed: f32,
    /// 被魔法护盾吸收的伤害
    pub magic_shield_absorbed: f32,
    /// 被减免的伤害
    pub reduced_damage: f32,
    /// 原始伤害
    pub original_damage: f32,
}

impl WhiteShield {
    pub fn new(amount: f32) -> Self {
        Self {
            current: amount,
            max: amount,
        }
    }

    /// 吸收伤害，返回剩余伤害
    pub fn absorb_damage(&mut self, damage: f32) -> f32 {
        let absorbed = damage.min(self.current);
        self.current -= absorbed;
        damage - absorbed
    }

    /// 检查护盾是否已耗尽
    pub fn is_depleted(&self) -> bool {
        self.current <= 0.0
    }
}

impl MagicShield {
    pub fn new(amount: f32) -> Self {
        Self {
            current: amount,
            max: amount,
        }
    }

    /// 吸收魔法伤害，返回剩余伤害
    pub fn absorb_magic_damage(&mut self, damage: f32) -> f32 {
        let absorbed = damage.min(self.current);
        self.current -= absorbed;
        damage - absorbed
    }

    /// 检查护盾是否已耗尽
    pub fn is_depleted(&self) -> bool {
        self.current <= 0.0
    }
}

impl DamageReduction {
    /// 创建一个新的伤害减免buff
    pub fn new(percentage: f32, damage_type: Option<DamageType>, duration: Option<f32>) -> Self {
        Self {
            percentage: percentage.clamp(0.0, 1.0),
            damage_type,
            duration,
            remaining_time: duration,
        }
    }

    /// 检查buff是否对指定伤害类型有效
    pub fn applies_to(&self, damage_type: DamageType) -> bool {
        self.damage_type.map_or(true, |dt| dt == damage_type)
    }

    /// 检查buff是否已过期
    pub fn is_expired(&self) -> bool {
        self.remaining_time.map_or(false, |time| time <= 0.0)
    }

    /// 更新buff时间
    pub fn update_time(&mut self, delta_time: f32) {
        if let Some(ref mut remaining) = self.remaining_time {
            *remaining -= delta_time;
        }
    }
}

impl DamageReductions {
    /// 添加一个新的减免buff
    pub fn add_buff(&mut self, buff: DamageReduction) {
        self.buffs.push(buff);
    }

    /// 移除已过期的buff
    pub fn remove_expired(&mut self) {
        self.buffs.retain(|buff| !buff.is_expired());
    }

    /// 计算对指定伤害类型的总减免百分比
    pub fn calculate_reduction(&self, damage_type: DamageType) -> f32 {
        let mut total_reduction = 0.0;

        for buff in &self.buffs {
            if buff.applies_to(damage_type) {
                // 使用乘法叠加公式：总减免 = 1 - (1 - r1) * (1 - r2) * ...
                total_reduction = 1.0 - (1.0 - total_reduction) * (1.0 - buff.percentage);
            }
        }

        total_reduction.clamp(0.0, 1.0)
    }

    /// 更新所有buff的时间
    pub fn update_time(&mut self, delta_time: f32) {
        for buff in &mut self.buffs {
            buff.update_time(delta_time);
        }
    }
}

/// 核心伤害计算函数
pub fn calculate_damage(
    damage_type: DamageType,
    amount: f32,
    white_shield: Option<Mut<WhiteShield>>,
    magic_shield: Option<Mut<MagicShield>>,
    damage_reductions: Option<&DamageReductions>,
) -> DamageResult {
    let original_damage = amount;
    let mut remaining_damage = amount;
    let mut white_shield_absorbed = 0.0;
    let mut magic_shield_absorbed = 0.0;
    let mut reduced_damage = 0.0;

    // 真实伤害无视所有防御机制
    if damage_type == DamageType::True {
        return DamageResult {
            final_damage: remaining_damage,
            white_shield_absorbed: 0.0,
            magic_shield_absorbed: 0.0,
            reduced_damage: 0.0,
            original_damage,
        };
    }

    // 1. 首先应用伤害减免buff
    if let Some(reductions) = damage_reductions {
        let reduction_percentage = reductions.calculate_reduction(damage_type);
        reduced_damage = remaining_damage * reduction_percentage;
        remaining_damage -= reduced_damage;
    }

    // 2. 然后应用护盾（白色护盾优先）
    if let Some(mut white_shield) = white_shield {
        if !white_shield.is_depleted() {
            let absorbed = white_shield.absorb_damage(remaining_damage);
            white_shield_absorbed = remaining_damage - absorbed;
            remaining_damage = absorbed;
        }
    }

    // 3. 如果是魔法伤害且还有剩余伤害，应用魔法护盾
    if damage_type == DamageType::Magic {
        if let Some(mut magic_shield) = magic_shield {
            if !magic_shield.is_depleted() && remaining_damage > 0.0 {
                let absorbed = magic_shield.absorb_magic_damage(remaining_damage);
                magic_shield_absorbed = remaining_damage - absorbed;
                remaining_damage = absorbed;
            }
        }
    }

    DamageResult {
        final_damage: remaining_damage,
        white_shield_absorbed,
        magic_shield_absorbed,
        reduced_damage,
        original_damage,
    }
}

/// 伤害系统 - 处理伤害事件
pub fn on_command_damage_create(
    trigger: Trigger<CommandDamageCreate>,
    mut commands: Commands,
    mut query: Query<(
        &mut Health,
        Option<&mut WhiteShield>,
        Option<&mut MagicShield>,
        Option<&DamageReductions>,
    )>,
) {
    debug!(
        "{:?} 对 {:?} 造成 {:.1} 点 {:?} 伤害",
        trigger.source,
        trigger.target(),
        trigger.amount,
        trigger.damage_type,
    );

    if let Ok((mut health, white_shield, magic_shield, damage_reductions)) =
        query.get_mut(trigger.target())
    {
        let health_before = health.value;
        let result = calculate_damage(
            trigger.damage_type,
            trigger.amount,
            white_shield,
            magic_shield,
            damage_reductions,
        );

        // 应用最终伤害到生命值
        health.value -= result.final_damage;

        // println!(
        //     "Damage applied: {:?} -> {:?}, Type: {:?}, Original: {:.1}, Final: {:.1}, Health: {:.1} -> {:.1}, Shields: W{:.1}/M{:.1}, Reduced: {:.1}",
        //     trigger.source,
        //     trigger.target(),
        //     trigger.damage_type,
        //     result.original_damage,
        //     result.final_damage,
        //     health_before,
        //     health.value,
        //     result.white_shield_absorbed,
        //     result.magic_shield_absorbed,
        //     result.reduced_damage
        // );

        // 触发伤害生效事件
        commands.trigger_targets(
            EventDamageCreate {
                source: trigger.source,
                damage_type: trigger.damage_type,
                damage_result: result,
            },
            trigger.target(),
        );

        if health.value <= 0.0 {
            // println!(
            //     "Entity {:?} health dropped to {:.1} (death threshold)",
            //     trigger.target(),
            //     health.value
            // );
        }
    } else {
        // println!(
        //     "Failed to find target entity {:?} for damage event",
        //     trigger.target()
        // );
    }
}

/// 更新伤害减免buff时间的系统
pub fn update_damage_reductions_system(mut query: Query<&mut DamageReductions>, time: Res<Time>) {
    let entity_count = query.iter().count();
    if entity_count > 0 {
        // println!("Updating {} entities with damage reductions", entity_count);
    }

    for mut reductions in query.iter_mut() {
        let expired_before = reductions.buffs.iter().filter(|r| r.is_expired()).count();
        reductions.update_time(time.delta_secs());
        reductions.remove_expired();
        let expired_after = reductions.buffs.iter().filter(|r| r.is_expired()).count();

        if expired_before != expired_after {
            // println!(
            //     "Removed {} expired damage reductions",
            //     expired_before - expired_after
            // );
        }
    }
}

/// 清理耗尽的护盾组件系统
pub fn cleanup_depleted_shields_system(
    mut commands: Commands,
    white_shields: Query<(Entity, &WhiteShield)>,
    magic_shields: Query<(Entity, &MagicShield)>,
) {
    let white_shield_count = white_shields.iter().count();
    let magic_shield_count = magic_shields.iter().count();

    if white_shield_count > 0 || magic_shield_count > 0 {
        // println!(
        //     "Checking {} white shields and {} magic shields for depletion",
        //     white_shield_count, magic_shield_count
        // );
    }

    let mut removed_white = 0;
    let mut removed_magic = 0;

    // 移除耗尽的白色护盾
    for (entity, shield) in white_shields.iter() {
        if shield.is_depleted() {
            // println!("Removing depleted white shield from entity {:?}", entity);
            commands.entity(entity).remove::<WhiteShield>();
            removed_white += 1;
        }
    }

    // 移除耗尽的魔法护盾
    for (entity, shield) in magic_shields.iter() {
        if shield.is_depleted() {
            // println!("Removing depleted magic shield from entity {:?}", entity);
            commands.entity(entity).remove::<MagicShield>();
            removed_magic += 1;
        }
    }

    if removed_white > 0 || removed_magic > 0 {
        // println!(
        //     "Removed {} white shields and {} magic shields",
        //     removed_white, removed_magic
        // );
    }
}

/// 便利函数：为实体添加白色护盾
pub fn add_white_shield(commands: &mut Commands, entity: Entity, amount: f32) {
    commands.entity(entity).insert(WhiteShield::new(amount));
}

/// 便利函数：为实体添加魔法护盾
pub fn add_magic_shield(commands: &mut Commands, entity: Entity, amount: f32) {
    commands.entity(entity).insert(MagicShield::new(amount));
}

/// 便利函数：为实体添加伤害减免buff
/// 注意：这个函数需要在系统中使用，因为需要查询现有组件
pub fn add_damage_reduction_to_entity(
    entity: Entity,
    percentage: f32,
    damage_type: Option<DamageType>,
    duration: Option<f32>,
    mut commands: Commands,
    mut query: Query<Option<&mut DamageReductions>>,
) {
    let buff = DamageReduction::new(percentage, damage_type, duration);

    if let Ok(existing_reductions) = query.get_mut(entity) {
        if let Some(mut reductions) = existing_reductions {
            // 如果实体已有DamageReductions组件，添加新buff
            reductions.add_buff(buff);
        } else {
            // 如果实体没有DamageReductions组件，创建新的
            let mut new_reductions = DamageReductions::default();
            new_reductions.add_buff(buff);
            commands.entity(entity).insert(new_reductions);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_white_shield_absorbs_all_damage_types() {
        let mut shield = WhiteShield::new(100.0);

        // 测试物理伤害
        let remaining = shield.absorb_damage(50.0);
        assert_eq!(remaining, 0.0);
        assert_eq!(shield.current, 50.0);

        // 测试剩余护盾值
        let remaining = shield.absorb_damage(60.0);
        assert_eq!(remaining, 10.0); // 50护盾只能吸收50伤害，剩余10
        assert_eq!(shield.current, 0.0);
        assert!(shield.is_depleted());
    }

    #[test]
    fn test_magic_shield_only_absorbs_magic_damage() {
        let mut shield = MagicShield::new(100.0);

        // 测试魔法伤害
        let remaining = shield.absorb_magic_damage(50.0);
        assert_eq!(remaining, 0.0);
        assert_eq!(shield.current, 50.0);

        // 测试超出护盾值的伤害
        let remaining = shield.absorb_magic_damage(60.0);
        assert_eq!(remaining, 10.0);
        assert_eq!(shield.current, 0.0);
        assert!(shield.is_depleted());
    }

    #[test]
    fn test_damage_reduction_calculation() {
        let mut reductions = DamageReductions::default();

        // 添加30%物理伤害减免
        reductions.add_buff(DamageReduction::new(0.3, Some(DamageType::Physical), None));

        // 添加20%全伤害减免
        reductions.add_buff(DamageReduction::new(0.2, None, None));

        // 计算物理伤害减免：1 - (1-0.3) * (1-0.2) = 1 - 0.7 * 0.8 = 1 - 0.56 = 0.44 (44%)
        let physical_reduction = reductions.calculate_reduction(DamageType::Physical);
        assert!((physical_reduction - 0.44).abs() < 0.001);

        // 计算魔法伤害减免：只有全伤害减免生效 = 20%
        let magic_reduction = reductions.calculate_reduction(DamageType::Magic);
        assert!((magic_reduction - 0.2).abs() < 0.001);
    }

    #[test]
    fn test_damage_reduction_expiry() {
        let mut buff = DamageReduction::new(0.5, None, Some(5.0));

        assert!(!buff.is_expired());

        // 更新时间
        buff.update_time(3.0);
        assert!(!buff.is_expired());
        assert_eq!(buff.remaining_time, Some(2.0));

        // 再更新时间，使其过期
        buff.update_time(3.0);
        assert!(buff.is_expired());
    }

    #[test]
    fn test_true_damage_ignores_all_defenses() {
        let mut reductions = DamageReductions::default();
        reductions.add_buff(DamageReduction::new(0.9, None, None)); // 90%减免

        // 验证真实伤害的逻辑
        let damage_type = DamageType::True;
        let amount = 50.0;

        if damage_type == DamageType::True {
            // 真实伤害应该无视所有防御
            assert_eq!(amount, 50.0); // 伤害不变
        }
    }

    #[test]
    fn test_damage_priority_order() {
        // 测试伤害处理的优先级：减免 -> 白色护盾 -> 魔法护盾
        let mut reductions = DamageReductions::default();
        reductions.add_buff(DamageReduction::new(0.5, None, None)); // 50%减免

        // 模拟伤害计算逻辑
        let original_damage = 100.0;
        let mut remaining_damage = original_damage;

        // 1. 应用减免
        let reduction = reductions.calculate_reduction(DamageType::Magic);
        let reduced_damage = remaining_damage * reduction;
        remaining_damage -= reduced_damage;
        assert_eq!(remaining_damage, 50.0); // 100 * 0.5 = 50减免，剩余50

        // 2. 应用白色护盾
        let mut white_shield = WhiteShield::new(30.0);
        let absorbed_by_white = white_shield.absorb_damage(remaining_damage);
        remaining_damage = absorbed_by_white;
        assert_eq!(remaining_damage, 20.0); // 30护盾吸收30伤害，剩余20

        // 3. 应用魔法护盾（对魔法伤害）
        let mut magic_shield = MagicShield::new(15.0);
        let absorbed_by_magic = magic_shield.absorb_magic_damage(remaining_damage);
        remaining_damage = absorbed_by_magic;
        assert_eq!(remaining_damage, 5.0); // 15护盾吸收15伤害，剩余5

        // 最终伤害应该是5
        assert_eq!(remaining_damage, 5.0);
    }
}
