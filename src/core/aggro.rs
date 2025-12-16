use std::collections::HashMap;

use bevy::prelude::*;
use lol_core::Team;

use crate::{DamageType, EventDamageCreate, EventDead};

#[derive(Default)]
pub struct PluginAggro;

impl Plugin for PluginAggro {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedPostUpdate, aggro_scan);
        app.add_observer(on_team_get_damage);
        app.add_observer(on_target_dead);
    }
}

#[derive(Component)]
#[require(AggroState)]
pub struct Aggro {
    pub range: f32,
}

#[derive(Component, Default)]
pub struct AggroState {
    pub aggros: HashMap<Entity, f32>,
}

#[derive(EntityEvent, Debug)]
pub struct EventAggroTargetFound {
    pub entity: Entity,
    pub target: Entity,
}

pub fn aggro_scan(
    mut commands: Commands,
    q_aggro: Query<(Entity, &Team, &Transform, &Aggro, &AggroState)>,
    q_attackable: Query<(Entity, &Team, &Transform)>,
) {
    for (entity, team, transform, aggro, aggro_state) in q_aggro.iter() {
        let mut best_aggro = 0.0;
        let mut closest_distance = f32::MAX;
        let mut target_entity = Entity::PLACEHOLDER;

        // 遍历所有可攻击单位筛选目标
        for (attackable_entity, attackable_team, attackable_transform) in q_attackable.iter() {
            // 忽略友方单位
            if attackable_team == team || *attackable_team == Team::Neutral {
                continue;
            }

            // 计算距离并检查是否在仇恨范围内
            let distance = transform
                .translation
                .distance(attackable_transform.translation);

            if distance >= aggro.range {
                continue;
            }

            // 获取仇恨值（默认为0）
            let aggro = aggro_state
                .aggros
                .get(&attackable_entity)
                .copied()
                .unwrap_or(0.0);

            // 优先选择仇恨值更高的目标，仇恨相同时选择更近的
            if aggro > best_aggro || (aggro == best_aggro && distance < closest_distance) {
                best_aggro = aggro;
                closest_distance = distance;
                target_entity = attackable_entity;
            }
        }

        // 如果找到有效目标则触发
        if target_entity != Entity::PLACEHOLDER {
            debug!("{} 找到仇恨目标 {}", entity, target_entity);
            commands.trigger(EventAggroTargetFound {
                entity,
                target: target_entity,
            });
        }
    }
}

pub fn on_team_get_damage(
    trigger: On<EventDamageCreate>,
    mut q_aggro: Query<(&Team, &Transform, &Aggro, &mut AggroState)>,
    q_transform: Query<&Transform>,
    q_team: Query<&Team>,
) {
    let source = trigger.source;
    let target = trigger.event_target();

    if trigger.damage_type != DamageType::Physical {
        return;
    }

    let Ok(source_transform) = q_transform.get(source) else {
        return;
    };

    let Ok(target_team) = q_team.get(target) else {
        return;
    };

    for (team, transform, aggro, mut aggro_state) in q_aggro.iter_mut() {
        if target_team != team {
            continue;
        }

        let distance = transform.translation.distance(source_transform.translation);

        if distance >= aggro.range {
            continue;
        }

        let aggro = aggro_state.aggros.get(&source).copied().unwrap_or(0.0);

        aggro_state.aggros.insert(source, aggro + 10.0);
    }
}

fn on_target_dead(trigger: On<EventDead>, mut q_aggro: Query<&mut AggroState>) {
    let dead_entity = trigger.event_target();

    for mut aggro_state in q_aggro.iter_mut() {
        aggro_state.aggros.remove(&dead_entity);
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::prelude::*;
    use bevy::time::TimeUpdateStrategy;

    use super::*;
    use crate::{DamageResult, DamageType, EventDamageCreate, EventDead};

    // 用于测试中捕获系统选中的目标
    #[derive(Resource, Default)]
    struct LastTarget(Option<Entity>);

    // 测试辅助函数：构建 App 并注入必要的 Plugin 和 Resource
    fn setup_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(PluginAggro);
        // 手动控制时间更新，使得 app.update() 能运行一次 FixedUpdate
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_micros(
            15625,
        )));
        app.init_resource::<LastTarget>();

        // 注册观察者：当 Aggro 系统发出“找到目标”事件时，记录到 Resource 中
        app.add_observer(
            |event: On<EventAggroTargetFound>, mut res: ResMut<LastTarget>| {
                res.0 = Some(event.event().target);
            },
        );

        // 运行一次，使 TimePlugin 记录初始值
        app.update();

        app
    }

    // 辅助构建 DamageResult
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
    fn test_scan_closest_target_when_no_aggro() {
        let mut app = setup_app();
        let world = app.world_mut();

        // 1. 创建拥有仇恨系统的单位 (Order)
        let _me = world
            .spawn((Team::Order, Transform::default(), Aggro { range: 100.0 }))
            .id();

        // 2. 创建两个敌人，一个近(10m)，一个远(20m)
        let enemy_near = world
            .spawn((Team::Chaos, Transform::from_xyz(10.0, 0.0, 0.0)))
            .id();

        let _enemy_far = world
            .spawn((Team::Chaos, Transform::from_xyz(20.0, 0.0, 0.0)))
            .id();

        // 运行系统
        app.update();

        // 断言：在没有仇恨值的情况下，应当选择最近的敌人
        let target = app.world().resource::<LastTarget>().0;
        assert_eq!(target, Some(enemy_near), "应优先选择距离最近的目标");
    }

    #[test]
    fn test_scan_high_aggro_priority() {
        let mut app = setup_app();
        let world = app.world_mut();

        let me = world
            .spawn((Team::Order, Transform::default(), Aggro { range: 100.0 }))
            .id();

        let enemy_near = world
            .spawn((Team::Chaos, Transform::from_xyz(10.0, 0.0, 0.0)))
            .id();

        let enemy_far = world
            .spawn((Team::Chaos, Transform::from_xyz(50.0, 0.0, 0.0)))
            .id();

        // 手动注入仇恨值：远处的仇恨极高
        let mut aggro_state = world.get_mut::<AggroState>(me).unwrap();
        aggro_state.aggros.insert(enemy_near, 0.0);
        aggro_state.aggros.insert(enemy_far, 100.0);

        app.update();

        // 断言：应当忽略距离，选择仇恨值最高的目标
        let target = app.world().resource::<LastTarget>().0;
        assert_eq!(target, Some(enemy_far), "应优先选择仇恨值最高的目标");
    }

    #[test]
    fn test_scan_ignore_out_of_range() {
        let mut app = setup_app();
        let world = app.world_mut();

        // 自身范围只有 10
        world.spawn((Team::Order, Transform::default(), Aggro { range: 10.0 }));

        // 敌人在 20 处
        world.spawn((Team::Chaos, Transform::from_xyz(20.0, 0.0, 0.0)));

        app.update();

        // 断言：没有目标被选中
        let target = app.world().resource::<LastTarget>().0;
        assert_eq!(target, None, "超出范围的目标应被忽略");
    }

    #[test]
    fn test_damage_increases_aggro() {
        let mut app = setup_app();
        let world = app.world_mut();

        // 模拟场景：队友被攻击，附近的守卫应该对攻击者产生仇恨
        let attacker = world
            .spawn((Team::Order, Transform::from_xyz(5.0, 0.0, 0.0)))
            .id();
        let ally = world.spawn((Team::Chaos, Transform::default())).id();

        let guard = world
            .spawn((Team::Chaos, Transform::default(), Aggro { range: 50.0 }))
            .id();

        // 触发伤害事件
        world.trigger(EventDamageCreate {
            entity: ally,
            source: attacker,
            damage_type: DamageType::Physical,
            damage_result: mock_damage_result(),
        });

        app.update();

        // 验证守卫的仇恨列表
        let state = app.world().get::<AggroState>(guard).unwrap();
        let aggro_val = state.aggros.get(&attacker).copied().unwrap_or(0.0);

        assert_eq!(aggro_val, 10.0, "队友受击应增加攻击者的仇恨值");
    }

    #[test]
    fn test_remove_aggro_on_dead() {
        let mut app = setup_app();
        let world = app.world_mut();

        let enemy = world.spawn(Team::Chaos).id();
        let me = world.spawn((Team::Order, Aggro { range: 100.0 })).id();

        // 1. 初始化仇恨
        if let Some(mut state) = world.get_mut::<AggroState>(me) {
            state.aggros.insert(enemy, 50.0);
        }

        // 2. 触发目标死亡事件 (在目标实体上触发)
        // 注意：根据 on_target_dead 实现，EventDead 需触发在死亡实体上
        world.trigger(EventDead { entity: enemy });

        app.update();

        // 3. 验证清理逻辑
        let state = app.world().get::<AggroState>(me).unwrap();
        assert!(
            !state.aggros.contains_key(&enemy),
            "目标死亡后应从仇恨列表中移除"
        );
    }
}
