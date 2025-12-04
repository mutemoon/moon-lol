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
