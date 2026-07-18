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
        if !app.world().contains_resource::<crate::skill::GodMode>() {
            app.init_resource::<crate::skill::GodMode>();
        }
        if !app.world().contains_resource::<crate::skill::NoCooldown>() {
            app.init_resource::<crate::skill::NoCooldown>();
        }
        app.add_systems(FixedUpdate, (spawn_event, update_respawn, regen));
        app.add_systems(Update, apply_god_mode);
        app.add_observer(on_event_damage_create);
        app.add_observer(on_command_heal);
    }
}

#[derive(Component, Reflect, Clone, Debug, Default)]
#[reflect(Component)]
pub struct Death;

#[derive(Component, Reflect, Clone, Debug)]
#[reflect(Component)]
pub struct RespawnTimer(pub Timer);

#[derive(Component, Reflect, Serialize, Deserialize, Clone, Default)]
#[reflect(Component, Default)]
pub struct Health {
    pub value: f32,
    pub max: f32,
    /// 基础每秒生命回复（1 级时的回复速率）
    pub base_static_regen: f32,
    /// 每级额外每秒生命回复
    pub regen_per_level: f32,
}

#[derive(EntityEvent, Debug)]
pub struct EventDead {
    pub entity: Entity,
    pub killer: Option<Entity>,
}

/// 治疗命令：对目标实体施加瞬时治疗，不超过 health.max。
///
/// 用法：`commands.trigger(CommandHeal { entity, source, amount });`
#[derive(EntityEvent, Debug)]
pub struct CommandHeal {
    pub entity: Entity,
    pub source: Entity,
    pub amount: f32,
}

#[derive(EntityEvent, Debug)]
pub struct EventSpawn {
    entity: Entity,
}

impl Health {
    pub fn new(max: f32) -> Health {
        Health {
            value: max,
            max,
            base_static_regen: 0.0,
            regen_per_level: 0.0,
        }
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

/// 持续回复生命与蓝量：每秒回复 `base_static_regen + regen_per_level * (level - 1)`，
/// 按 FixedUpdate 的 delta 累加并夹取到 `[0, max]`；死亡中的实体不回复。
pub fn regen(
    time: Res<Time<Fixed>>,
    mut q: Query<
        (
            Option<&mut Health>,
            Option<&mut AbilityResource>,
            Option<&Level>,
        ),
        (Without<Death>, Or<(With<Health>, With<AbilityResource>)>),
    >,
) {
    let dt = time.delta().as_secs_f32();
    if dt <= 0.0 {
        return;
    }

    for (health, ar, level) in q.iter_mut() {
        // 缺少 Level 组件时按 1 级处理（仅基础回复）
        let lvl = level.map(|l| l.value).unwrap_or(1);
        let lvl_factor = lvl.saturating_sub(1) as f32;

        // 生命回复
        if let Some(mut health) = health {
            if health.max > 0.0 && health.value < health.max {
                let rate = health.base_static_regen + health.regen_per_level * lvl_factor;
                if rate > 0.0 {
                    health.value = (health.value + rate * dt).min(health.max);
                }
            }
        }

        // 蓝量/能量回复
        if let Some(mut ar) = ar {
            if ar.max > 0.0 && ar.value < ar.max {
                let rate = ar.base_static_regen + ar.regen_per_level * lvl_factor;
                if rate > 0.0 {
                    ar.value = (ar.value + rate * dt).min(ar.max);
                }
            }
        }
    }
}

/// 处理瞬时治疗命令。治疗量夹取到 health.max，死亡目标不生效。
pub fn on_command_heal(trigger: On<CommandHeal>, mut q_health: Query<&mut Health>) {
    let Ok(mut health) = q_health.get_mut(trigger.entity) else {
        return;
    };
    if health.value <= 0.0 || health.max <= 0.0 {
        return;
    }
    let before = health.value;
    health.value = (health.value + trigger.amount).min(health.max);
    debug!(
        "{:?} 治疗 {:?} {:.1} HP（{:.1} → {:.1}）",
        trigger.source, trigger.entity, trigger.amount, before, health.value,
    );
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

    /// 推进固定时间并执行一次 `regen` 系统。
    fn advance_and_regen(app: &mut App, delta: Duration) {
        {
            let mut time = app.world_mut().resource_mut::<Time<Fixed>>();
            time.advance_by(delta);
        }
        let _ = app.world_mut().run_system_once(regen);
    }

    #[test]
    fn test_mana_regen_basic() {
        let mut app = setup_app();
        let e = app
            .world_mut()
            .spawn(AbilityResource {
                value: 10.0,
                max: 100.0,
                base_static_regen: 2.0,
                regen_per_level: 0.0,
                ..default()
            })
            .id();

        advance_and_regen(&mut app, Duration::from_secs(1));
        assert_eq!(app.world().get::<AbilityResource>(e).unwrap().value, 12.0);

        advance_and_regen(&mut app, Duration::from_secs(1));
        assert_eq!(
            app.world().get::<AbilityResource>(e).unwrap().value,
            14.0,
            "蓝量应持续回复"
        );
    }

    #[test]
    fn test_mana_regen_clamps_to_max() {
        let mut app = setup_app();
        let e = app
            .world_mut()
            .spawn(AbilityResource {
                value: 99.0,
                max: 100.0,
                base_static_regen: 5.0,
                ..default()
            })
            .id();

        advance_and_regen(&mut app, Duration::from_secs(1));
        assert_eq!(
            app.world().get::<AbilityResource>(e).unwrap().value,
            100.0,
            "蓝量不应超过上限"
        );
    }

    #[test]
    fn test_mana_regen_scales_with_level() {
        let mut app = setup_app();
        // level 3 -> regen/sec = 2.0 + 1.0 * (3 - 1) = 4.0
        let e = app
            .world_mut()
            .spawn((
                AbilityResource {
                    value: 10.0,
                    max: 100.0,
                    base_static_regen: 2.0,
                    regen_per_level: 1.0,
                    ..default()
                },
                Level {
                    value: 3,
                    ..default()
                },
            ))
            .id();

        advance_and_regen(&mut app, Duration::from_secs(1));
        assert_eq!(app.world().get::<AbilityResource>(e).unwrap().value, 14.0);
    }

    #[test]
    fn test_mana_regen_zero_rate_no_change() {
        let mut app = setup_app();
        let e = app
            .world_mut()
            .spawn(AbilityResource {
                value: 10.0,
                max: 100.0,
                base_static_regen: 0.0,
                regen_per_level: 0.0,
                ..default()
            })
            .id();

        advance_and_regen(&mut app, Duration::from_secs(1));
        assert_eq!(
            app.world().get::<AbilityResource>(e).unwrap().value,
            10.0,
            "回复速率为 0 时蓝量不变"
        );
    }

    #[test]
    fn test_regen_skipped_when_dead() {
        let mut app = setup_app();
        let e = app
            .world_mut()
            .spawn((
                Health {
                    value: 10.0,
                    max: 100.0,
                    base_static_regen: 5.0,
                    ..default()
                },
                AbilityResource {
                    value: 10.0,
                    max: 100.0,
                    base_static_regen: 5.0,
                    ..default()
                },
                Death,
            ))
            .id();

        advance_and_regen(&mut app, Duration::from_secs(1));
        assert_eq!(
            app.world().get::<Health>(e).unwrap().value,
            10.0,
            "死亡时不回复血量"
        );
        assert_eq!(
            app.world().get::<AbilityResource>(e).unwrap().value,
            10.0,
            "死亡时不回复蓝量"
        );
    }

    #[test]
    fn test_health_regen_basic() {
        let mut app = setup_app();
        let e = app
            .world_mut()
            .spawn(Health {
                value: 10.0,
                max: 100.0,
                base_static_regen: 3.0,
                ..default()
            })
            .id();

        advance_and_regen(&mut app, Duration::from_secs(1));
        assert_eq!(app.world().get::<Health>(e).unwrap().value, 13.0);
    }

    #[test]
    fn test_health_regen_clamps_to_max() {
        let mut app = setup_app();
        let e = app
            .world_mut()
            .spawn(Health {
                value: 99.0,
                max: 100.0,
                base_static_regen: 5.0,
                ..default()
            })
            .id();

        advance_and_regen(&mut app, Duration::from_secs(1));
        assert_eq!(
            app.world().get::<Health>(e).unwrap().value,
            100.0,
            "血量不应超过上限"
        );
    }

    #[test]
    fn test_health_regen_scales_with_level() {
        let mut app = setup_app();
        // level 3 -> regen/sec = 2.0 + 1.5 * (3 - 1) = 5.0
        let e = app
            .world_mut()
            .spawn((
                Health {
                    value: 10.0,
                    max: 100.0,
                    base_static_regen: 2.0,
                    regen_per_level: 1.5,
                    ..default()
                },
                Level {
                    value: 3,
                    ..default()
                },
            ))
            .id();

        advance_and_regen(&mut app, Duration::from_secs(1));
        assert_eq!(app.world().get::<Health>(e).unwrap().value, 15.0);
    }

    #[test]
    fn test_regen_both_health_and_mana() {
        let mut app = setup_app();
        let e = app
            .world_mut()
            .spawn((
                Health {
                    value: 10.0,
                    max: 100.0,
                    base_static_regen: 2.0,
                    ..default()
                },
                AbilityResource {
                    value: 20.0,
                    max: 100.0,
                    base_static_regen: 3.0,
                    ..default()
                },
            ))
            .id();

        advance_and_regen(&mut app, Duration::from_secs(1));
        assert_eq!(app.world().get::<Health>(e).unwrap().value, 12.0);
        assert_eq!(app.world().get::<AbilityResource>(e).unwrap().value, 23.0);
    }
}

fn apply_god_mode(
    god_mode: Res<crate::skill::GodMode>,
    mut commands: Commands,
    mut q_champions: Query<
        (
            Entity,
            &mut Health,
            Option<&mut AbilityResource>,
            &mut Level,
            Option<&mut crate::skill::SkillPoints>,
        ),
        With<Champion>,
    >,
) {
    if god_mode.is_changed() && !god_mode.0 {
        // 变为禁用时，移除无敌 buff
        for (entity, _, _, _, _) in q_champions.iter() {
            commands
                .entity(entity)
                .remove::<crate::buffs::damage_reduction::BuffDamageReduction>();
        }
    } else if god_mode.0 {
        // 启用时，提升等级到 6 并加相应技能点，每帧回满生命值和能量值，并附加无敌 buff
        for (entity, mut health, ar_opt, mut level, skill_points_opt) in q_champions.iter_mut() {
            if level.value < 6 {
                let delta = 6 - level.value;
                level.value = 6;
                if let Some(mut sp) = skill_points_opt {
                    sp.0 += delta;
                }
            }
            health.value = health.max;
            if let Some(mut ar) = ar_opt {
                ar.value = ar.max;
            }
            commands
                .entity(entity)
                .insert(crate::buffs::damage_reduction::BuffDamageReduction {
                    percentage: 1.0,
                    damage_type: None,
                });
        }
    }
}
