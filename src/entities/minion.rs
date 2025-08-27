use bevy::{app::Plugin, prelude::*};
use lol_core::{Lane, Team};
use serde::{Deserialize, Serialize};

use crate::core::{
    Attack, AttackState, Bounding, CommandBehaviorAttack, CommandMovementStart, ConfigMap,
    DamageType, EventDamageCreate, EventDead, EventMovementEnd, EventSpawn, State,
};

#[derive(Default)]
pub struct PluginMinion;

impl Plugin for PluginMinion {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_event::<EventMinionFoundTarget>();
        app.add_event::<EventMinionChasingTimeout>();
        app.add_event::<ChasingTooMuch>();
        app.add_systems(FixedPostUpdate, minion_aggro);
        app.add_observer(on_spawn);
        app.add_observer(action_continue_minion_path);
        app.add_observer(on_found_aggro_target);
        app.add_observer(on_target_dead);
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[require(MinionState, AggroInfo, State)]
pub enum Minion {
    Siege,
    Melee,
    Ranged,
    Super,
}

impl From<u8> for Minion {
    fn from(value: u8) -> Self {
        match value {
            4 => Minion::Melee,
            6 => Minion::Siege,
            5 => Minion::Ranged,
            7 => Minion::Super,
            _ => panic!("unknown minion type"),
        }
    }
}

#[derive(Component, Default)]
pub struct AggroInfo {
    pub aggros: std::collections::HashMap<Entity, f32>,
}

#[derive(Component, PartialEq, Debug, Default)]
pub enum MinionState {
    #[default]
    MovingOnPath,
    AttackingTarget,
}

#[derive(Event, Debug)]
pub struct CommandMinionContinuePath;

#[derive(Event, Debug)]
pub struct EventMinionFoundTarget {
    pub target: Entity,
}

#[derive(Event, Debug)]
pub struct EventMinionChasingTimeout;

#[derive(Event, Debug)]
pub struct ChasingTooMuch;

pub const AGGRO_RANGE: f32 = 500.0;

pub fn minion_aggro(
    mut commands: Commands,
    q_minion: Query<(Entity, &Team, &Transform, &AggroInfo), With<Minion>>,
    q_attackable: Query<(Entity, &Team, &Transform)>,
) {
    for (minion_entity, minion_team, minion_transform, aggro_info) in q_minion.iter() {
        let mut best_aggro = 0.0;
        let mut closest_distance = f32::MAX;
        let mut target_entity = Entity::PLACEHOLDER;

        // 遍历所有可攻击单位筛选目标
        for (attackable_entity, attackable_team, attackable_transform) in q_attackable.iter() {
            // 忽略友方单位
            if attackable_team == minion_team {
                continue;
            }

            // 计算距离并检查是否在仇恨范围内
            let distance = minion_transform
                .translation
                .distance(attackable_transform.translation);
            if distance >= AGGRO_RANGE {
                continue;
            }

            // 获取仇恨值（默认为0）
            let aggro = aggro_info
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
            commands.trigger_targets(
                EventMinionFoundTarget {
                    target: target_entity,
                },
                minion_entity,
            );
        }
    }
}

pub fn action_continue_minion_path(
    trigger: Trigger<CommandMinionContinuePath>,
    query: Query<(&Transform, &Lane)>,
    res_config: Res<ConfigMap>,
    mut commands: Commands,
) {
    let Ok((transform, lane)) = query.get(trigger.target()) else {
        return;
    };

    let minion_path = res_config.minion_paths.get(lane).unwrap();

    let Some(closest_index) = find_closest_point_index(&minion_path, transform.translation.xz())
    else {
        return;
    };

    commands.trigger_targets(
        CommandMovementStart(minion_path[closest_index..].to_vec()),
        trigger.target(),
    );
}

fn get_is_in_attack_range_in_found_aggro_target(
    trigger: &Trigger<EventMinionFoundTarget>,
    q_attack: &Query<&Attack>,
    q_transform: &Query<&Transform>,
    q_bounding: &Query<&Bounding>,
) -> bool {
    let Ok(attack) = q_attack.get(trigger.target()) else {
        return false;
    };
    let Ok(transform) = q_transform.get(trigger.target()) else {
        return false;
    };
    let Ok(target_transform) = q_transform.get(trigger.target) else {
        return false;
    };
    let Ok(bounding) = q_bounding.get(trigger.target) else {
        return false;
    };
    transform.translation.distance(target_transform.translation) <= attack.range + bounding.radius
}

fn on_found_aggro_target(
    trigger: Trigger<EventMinionFoundTarget>,
    mut commands: Commands,
    mut q_minion_state: Query<&mut MinionState>,

    q_attack: Query<&Attack>,
    q_transform: Query<&Transform>,
    q_bounding: Query<&Bounding>,
) {
    let entity = trigger.target();
    let event = trigger.event();

    let is_in_attack_range = get_is_in_attack_range_in_found_aggro_target(
        &trigger,
        &q_attack,
        &q_transform,
        &q_bounding,
    );

    if is_in_attack_range {
        commands.trigger_targets(EventMovementEnd, trigger.target());
    } else {
        commands
            .entity(trigger.target())
            .trigger(CommandBehaviorAttack {
                target: event.target,
            });
    }

    if let Ok(mut minion_state) = q_minion_state.get_mut(entity) {
        match *minion_state {
            MinionState::MovingOnPath => {
                *minion_state = MinionState::AttackingTarget;
                commands.trigger_targets(
                    CommandBehaviorAttack {
                        target: event.target,
                    },
                    trigger.target(),
                );
            }
            _ => (),
        }
    }
}

pub fn on_team_get_damage(
    trigger: Trigger<EventDamageCreate>,
    mut q_minion: Query<(&Team, &Transform, &mut AggroInfo), With<Minion>>,
    q_transform: Query<&Transform>,
    q_team: Query<&Team>,
) {
    let source = trigger.source;
    let target = trigger.target();

    if trigger.damage_type != DamageType::Physical {
        return;
    }

    let Ok(source_transform) = q_transform.get(source) else {
        return;
    };

    let Ok(target_team) = q_team.get(target) else {
        return;
    };

    for (minion_team, minion_transform, mut aggro_info) in q_minion.iter_mut() {
        if target_team != minion_team {
            continue;
        }

        let distance = minion_transform
            .translation
            .distance(source_transform.translation);

        if distance >= AGGRO_RANGE {
            continue;
        }

        let aggro = aggro_info.aggros.get(&source).copied().unwrap_or(0.0);

        aggro_info.aggros.insert(source, aggro + 10.0);
    }
}

fn on_target_dead(
    trigger: Trigger<EventDead>,
    mut commands: Commands,
    mut q_minion_state: Query<(&mut MinionState, &AttackState)>,
) {
    let dead_entity = trigger.target();

    for (mut minion_state, attack_state) in q_minion_state.iter_mut() {
        let Some(target) = attack_state.target else {
            continue;
        };

        if target != dead_entity {
            continue;
        }

        match *minion_state {
            MinionState::AttackingTarget => {
                *minion_state = MinionState::MovingOnPath;
                commands.trigger_targets(CommandMinionContinuePath, trigger.target());
            }
            _ => (),
        }
    }
}

fn on_spawn(
    trigger: Trigger<EventSpawn>,
    mut commands: Commands,
    mut q_minion_state: Query<&mut MinionState>,
) {
    let entity = trigger.target();
    if let Ok(mut minion_state) = q_minion_state.get_mut(entity) {
        match *minion_state {
            MinionState::MovingOnPath => {
                *minion_state = MinionState::MovingOnPath;
                commands.trigger_targets(CommandMinionContinuePath, entity);
            }
            _ => (),
        }
    }
}

fn find_closest_point_index(path: &Vec<Vec2>, position: Vec2) -> Option<usize> {
    if path.is_empty() {
        return None;
    }

    let mut closest_index = 0;
    let mut min_distance = f32::INFINITY;

    for (i, &point) in path.iter().enumerate() {
        let distance = position.distance(point);
        if distance < min_distance {
            min_distance = distance;
            closest_index = i;
        }
    }

    // 确保不往回走，如果找到的最近点不是第一个点，检查是否应该选择下一个点
    if closest_index > 0 {
        let prev_point = path[closest_index - 1];
        let curr_point = path[closest_index];

        // 计算从前一个点到当前点的方向向量
        let path_direction = (curr_point - prev_point).normalize();
        // 计算从前一个点到当前位置的向量
        let position_direction = (position - prev_point).normalize();

        // 如果当前位置在路径方向的前方，选择当前点；否则选择下一个点（如果存在）
        if path_direction.dot(position_direction) > 0.0 && closest_index + 1 < path.len() {
            closest_index += 1;
        }
    }

    Some(closest_index)
}
