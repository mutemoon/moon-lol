use bevy::prelude::*;

use crate::aggro::{Aggro, EventAggroTargetFound};
use crate::attack_auto::{CommandAttackAutoStart, CommandAttackAutoStop};
use crate::damage::{Damage, EventDamageCreate};
use crate::entities::champion::Champion;

#[derive(Default)]
pub struct PluginTurret;

impl Plugin for PluginTurret {
    fn build(&self, app: &mut App) {
        app.add_observer(on_event_aggro_target_found);
        app.add_observer(on_command_attack_auto_stop);
        app.add_observer(on_event_damage_create);
    }
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
#[require(Aggro = Aggro { range: 1000.0 }, TurretHeat)]
pub struct Turret;

/// 防御塔加热机制组件
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct TurretHeat {
    /// 上一个攻击的目标
    pub last_target: Option<Entity>,
    /// 连续攻击同一目标的次数（加热层数，最高 3 层，每层 +40%）
    pub heat_level: u32,
    /// 基础攻击力（用于重置）
    pub base_damage: f32,
}

fn on_event_aggro_target_found(
    trigger: On<EventAggroTargetFound>,
    mut commands: Commands,
    mut q_turret: Query<(Entity, &mut TurretHeat), With<Turret>>,
) {
    let entity = trigger.event_target();
    let target = trigger.target;

    if let Ok((entity, mut heat)) = q_turret.get_mut(entity) {
        // 如果目标改变，重置加热状态
        if heat.last_target != Some(target) {
            heat.last_target = Some(target);
            heat.heat_level = 0;
            debug!("{} 目标改变，防御塔加热重置", entity);
        }

        debug!("{} 对仇恨目标 {} 发起攻击", entity, target);

        commands.trigger(CommandAttackAutoStart { entity, target });
    }
}

fn on_command_attack_auto_stop(
    trigger: On<CommandAttackAutoStop>,
    mut q_turret: Query<&mut TurretHeat, With<Turret>>,
) {
    let entity = trigger.event_target();
    if let Ok(mut heat) = q_turret.get_mut(entity) {
        heat.last_target = None;
        heat.heat_level = 0;
        debug!("{} 停止攻击，防御塔加热重置", entity);
    }
}

fn on_event_damage_create(
    trigger: On<EventDamageCreate>,
    mut q_turret: Query<(&mut TurretHeat, &mut Damage), With<Turret>>,
    q_champion: Query<&Champion>,
) {
    let source = trigger.source;
    let target = trigger.entity; // 被伤害的实体

    let Ok((mut heat, mut damage)) = q_turret.get_mut(source) else {
        return;
    };

    // 只有攻击英雄才会加热
    if !q_champion.contains(target) {
        // 如果攻击非英雄单位，虽然不加热，但如果目标没变也不重置（LoL 逻辑中攻击小兵不增加加热，但切换回英雄会从之前的层数继续？不，切换目标就重置了）
        // 简化逻辑：攻击非英雄单位不增加加热层数
        return;
    }

    // 增加加热层数（最高 3 层，即第 4 次攻击达到最大伤害）
    if heat.heat_level < 3 {
        heat.heat_level += 1;
        debug!("{} 攻击英雄，加热层数提升至 {}", source, heat.heat_level);
    }

    // 更新伤害：基础伤害 * (1 + 0.4 * 层数)
    if heat.base_damage == 0.0 {
        heat.base_damage = damage.0;
    }

    damage.0 = heat.base_damage * (1.0 + 0.4 * heat.heat_level as f32);
    debug!("{} 当前伤害提升至 {:.1}", source, damage.0);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::damage::{DamageResult, DamageType, EventDamageCreate};
    use crate::team::Team;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(PluginTurret);
        app.update();
        app
    }

    fn mock_damage_result() -> DamageResult {
        DamageResult {
            final_damage: 10.0,
            white_shield_absorbed: 0.0,
            magic_shield_absorbed: 0.0,
            reduced_damage: 0.0,
            armor_reduced_damage: 0.0,
            original_damage: 10.0,
        }
    }

    #[test]
    fn test_turret_heat_increase() {
        let mut app = setup_app();

        let turret = app
            .world_mut()
            .spawn((Turret, Damage(100.0), Team::Order))
            .id();
        let hero = app.world_mut().spawn((Champion, Team::Chaos)).id();

        // 模拟第一次攻击产生的伤害事件（触发加热）
        app.world_mut().trigger(EventDamageCreate {
            entity: hero,
            source: turret,
            damage_type: DamageType::Physical,
            damage_result: mock_damage_result(),
            tag: None,
        });

        // 检查伤害是否增加到 140
        let damage = app.world().get::<Damage>(turret).unwrap();
        assert_eq!(damage.0, 140.0, "第一次攻击英雄后伤害应提升至 140%");

        // 模拟第二次攻击
        app.world_mut().trigger(EventDamageCreate {
            entity: hero,
            source: turret,
            damage_type: DamageType::Physical,
            damage_result: mock_damage_result(),
            tag: None,
        });

        let damage = app.world().get::<Damage>(turret).unwrap();
        assert_eq!(damage.0, 180.0, "第二次攻击英雄后伤害应提升至 180%");

        // 模拟第三次攻击
        app.world_mut().trigger(EventDamageCreate {
            entity: hero,
            source: turret,
            damage_type: DamageType::Physical,
            damage_result: mock_damage_result(),
            tag: None,
        });

        let damage = app.world().get::<Damage>(turret).unwrap();
        assert_eq!(damage.0, 220.0, "第三次攻击英雄后伤害应提升至 220%");

        // 模拟第四次攻击（达到上限）
        app.world_mut().trigger(EventDamageCreate {
            entity: hero,
            source: turret,
            damage_type: DamageType::Physical,
            damage_result: mock_damage_result(),
            tag: None,
        });

        let damage = app.world().get::<Damage>(turret).unwrap();
        assert_eq!(damage.0, 220.0, "伤害应在 220% 封顶");
    }

    #[test]
    fn test_turret_heat_reset_on_target_change() {
        let mut app = setup_app();

        let turret = app
            .world_mut()
            .spawn((Turret, Damage(100.0), Team::Order))
            .id();
        let hero1 = app.world_mut().spawn((Champion, Team::Chaos)).id();
        let hero2 = app.world_mut().spawn((Champion, Team::Chaos)).id();

        // 攻击第一个英雄，产生加热
        app.world_mut().trigger(EventDamageCreate {
            entity: hero1,
            source: turret,
            damage_type: DamageType::Physical,
            damage_result: mock_damage_result(),
            tag: None,
        });

        assert_eq!(app.world().get::<Damage>(turret).unwrap().0, 140.0);

        // 切换目标
        app.world_mut().trigger(EventAggroTargetFound {
            entity: turret,
            target: hero2,
        });

        // 检查加热是否重置
        let heat = app.world().get::<TurretHeat>(turret).unwrap();
        assert_eq!(heat.heat_level, 0, "切换目标后加热等级应重置");
    }
}
