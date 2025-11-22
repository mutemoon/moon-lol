use bevy::{app::Plugin, prelude::*};
use serde::{Deserialize, Serialize};

use lol_config::ConfigMap;
use lol_core::{Lane, Team};

use crate::{
    Aggro, AttackAuto, CommandAttackAutoStart, CommandAttackAutoStop, CommandMovement,
    EventAggroTargetFound, EventDead, HealthBar, HealthBarType, MovementAction, MovementWay, State,
};

#[derive(Default)]
pub struct PluginMinion;

impl Plugin for PluginMinion {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(FixedUpdate, fixed_update);

        app.add_observer(on_event_aggro_target_found);
        app.add_observer(on_target_dead);
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[require(
    MinionState,
    Aggro = Aggro { range: 500.0 },
    State,
    HealthBar = HealthBar { bar_type: HealthBarType::Minion }
)]
pub enum Minion {
    Siege,
    Melee,
    Ranged,
    Super,
}

#[derive(Component, PartialEq, Debug, Default)]
pub enum MinionState {
    #[default]
    MovingOnPath,
    AttackingTarget,
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

pub fn fixed_update(
    mut commands: Commands,
    res_config: Res<ConfigMap>,
    q_minion: Query<(Entity, &Transform, &Team, &Lane, &MinionState), With<Minion>>,
) {
    for (entity, transform, team, lane, minion_state) in q_minion.iter() {
        if *minion_state == MinionState::AttackingTarget {
            continue;
        }

        let minion_path = res_config.minion_paths.get(lane).unwrap();

        let mut path = minion_path.clone();

        if matches!(team, Team::Chaos) {
            path.reverse();
        }

        let Some(closest_index) = find_closest_point_index(&path, transform.translation.xz())
        else {
            return;
        };

        commands.trigger(CommandMovement {
            entity,
            priority: 0,
            action: MovementAction::Start {
                way: MovementWay::Pathfind(
                    *path.get(closest_index + 1).unwrap_or(&path[closest_index]),
                ),
                speed: None,
                source: "Minion".to_string(),
            },
        });
    }
}

fn on_event_aggro_target_found(
    trigger: On<EventAggroTargetFound>,
    mut commands: Commands,
    mut q_minion_state: Query<&mut MinionState>,
) {
    let entity = trigger.event_target();

    if let Ok(mut minion_state) = q_minion_state.get_mut(entity) {
        match *minion_state {
            MinionState::MovingOnPath => {
                *minion_state = MinionState::AttackingTarget;
                commands.trigger(CommandAttackAutoStart {
                    entity,
                    target: trigger.target,
                });
            }
            _ => (),
        }
    }
}

fn on_target_dead(
    trigger: On<EventDead>,
    mut commands: Commands,
    mut q_minion_state: Query<(Entity, &mut MinionState, &AttackAuto)>,
) {
    let dead_entity = trigger.event_target();

    for (entity, mut minion_state, attack_state) in q_minion_state.iter_mut() {
        let target = attack_state.target;

        if target != dead_entity {
            continue;
        }

        match *minion_state {
            MinionState::AttackingTarget => {
                *minion_state = MinionState::MovingOnPath;
                commands.trigger(CommandAttackAutoStop { entity });
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
