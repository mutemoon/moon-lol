use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::base::ability_resource::AbilityResource;
use crate::base::level::Level;
use crate::damage::EventDamageCreate;
use crate::entities::champion::Champion;
use crate::team::Team;

#[derive(Default)]
pub struct PluginLife;

impl Plugin for PluginLife {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, (spawn_event, update_respawn));
        app.add_observer(on_event_damage_create);
    }
}

#[derive(Component, Reflect, Clone, Debug, Default)]
#[reflect(Component)]
pub struct Death;

#[derive(Component, Reflect, Clone, Debug)]
#[reflect(Component)]
pub struct RespawnTimer(pub Timer);

#[derive(Component, Reflect, Serialize, Deserialize, Clone)]
#[reflect(Component)]
pub struct Health {
    pub value: f32,
    pub max: f32,
}

#[derive(EntityEvent, Debug)]
pub struct EventDead {
    pub entity: Entity,
    pub killer: Option<Entity>,
}

#[derive(EntityEvent, Debug)]
pub struct EventSpawn {
    entity: Entity,
}

impl Health {
    pub fn new(max: f32) -> Health {
        Health { value: max, max }
    }
}

pub fn spawn_event(mut commands: Commands, q_alive: Query<Entity, Added<Health>>) {
    let spawn_count = q_alive.iter().count();
    if spawn_count > 0 {
        debug!("生成 {} 个新实体", spawn_count);
    }

    for entity in q_alive.iter() {
        commands.trigger(EventSpawn { entity });
    }
}

fn on_event_damage_create(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_health: Query<&Health>,
    q_champion: Query<&Champion>,
    q_level: Query<&Level>,
) {
    let entity = trigger.event_target();

    let Ok(health) = q_health.get(entity) else {
        return;
    };

    if health.value <= 0.0 {
        debug!("{:?} 死了", entity);
        commands.trigger(EventDead {
            entity,
            killer: Some(trigger.source),
        });

        if q_champion.get(entity).is_ok() {
            commands.entity(entity).insert(Death);

            let level = q_level.get(entity).map(|l| l.value).unwrap_or(1);
            let duration = 5.0 + level as f32 * 2.0;
            commands
                .entity(entity)
                .insert(RespawnTimer(Timer::from_seconds(duration, TimerMode::Once)));
            debug!("{:?} 将在 {:.1} 秒后复活", entity, duration);
        } else {
            commands.entity(entity).despawn();
        }
    }
}

pub fn update_respawn(
    mut commands: Commands,
    mut q_respawn: Query<(
        Entity,
        &mut RespawnTimer,
        &mut Health,
        Option<&mut AbilityResource>,
        &Team,
        &mut Transform,
    )>,
    time: Res<Time<Fixed>>,
) {
    for (entity, mut timer, mut health, ar, team, mut transform) in q_respawn.iter_mut() {
        timer.0.tick(time.delta());

        if timer.0.just_finished() {
            // 复活位置
            let spawn_pos = match team {
                Team::Order => Vec3::new(1000.0, 0.0, 1000.0),
                Team::Chaos => Vec3::new(14000.0, 0.0, 14000.0),
                Team::Neutral => transform.translation, // 应该不会发生
            };

            transform.translation = spawn_pos;
            health.value = health.max;
            if let Some(mut ar) = ar {
                ar.value = ar.max;
            }

            commands.entity(entity).remove::<Death>();
            commands.entity(entity).remove::<RespawnTimer>();

            debug!("{:?} 已在 {:?} 泉水复活", entity, team);
            commands.trigger(EventSpawn { entity });
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::ecs::system::RunSystemOnce;

    use super::*;
    use crate::damage::{DamageResult, DamageType, EventDamageCreate};

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(PluginLife);
        app.insert_resource(Time::<Fixed>::from_duration(Duration::from_secs(1)));
        app.update();
        app
    }

    #[test]
    fn test_hero_respawn() {
        let mut app = setup_app();

        let hero = app
            .world_mut()
            .spawn((
                Champion,
                Team::Order,
                Health::new(100.0),
                AbilityResource {
                    value: 0.0,
                    max: 100.0,
                    ..default()
                },
                Level {
                    value: 1,
                    ..default()
                },
                Transform::from_xyz(5000.0, 0.0, 5000.0),
            ))
            .id();

        // 1. 模拟英雄死亡
        // 先将生命值降为 0
        if let Some(mut health) = app.world_mut().get_mut::<Health>(hero) {
            health.value = 0.0;
        }

        app.world_mut().trigger(EventDamageCreate {
            entity: hero,
            source: Entity::PLACEHOLDER,
            damage_type: DamageType::True,
            damage_result: DamageResult {
                final_damage: 100.0,
                white_shield_absorbed: 0.0,
                magic_shield_absorbed: 0.0,
                reduced_damage: 0.0,
                armor_reduced_damage: 0.0,
                original_damage: 100.0,
            },
            tag: None,
        });

        // 触发伤害事件处理
        app.update();

        // 2. 检查是否添加了 Death 和 RespawnTimer
        assert!(app.world().get::<Death>(hero).is_some());
        assert!(app.world().get::<RespawnTimer>(hero).is_some());

        // 3. 模拟时间流逝 (level 1 -> 5+1*2 = 7s)
        for _ in 0..8 {
            let delta = Duration::from_secs(1);
            {
                let mut time = app.world_mut().resource_mut::<Time<Fixed>>();
                time.advance_by(delta);
            }

            // 手动执行系统以确保其运行
            let _ = app.world_mut().run_system_once(update_respawn);
        }

        // 4. 检查是否复活
        assert!(app.world().get::<Death>(hero).is_none());
        assert!(app.world().get::<RespawnTimer>(hero).is_none());

        let health = app.world().get::<Health>(hero).unwrap();
        assert_eq!(health.value, 100.0);

        let ar = app.world().get::<AbilityResource>(hero).unwrap();
        assert_eq!(ar.value, 100.0, "复活后蓝量应回满");

        let transform = app.world().get::<Transform>(hero).unwrap();
        assert_eq!(transform.translation, Vec3::new(1000.0, 0.0, 1000.0));
    }
}
