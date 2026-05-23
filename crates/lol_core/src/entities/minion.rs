use bevy::app::Plugin;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::aggro::{Aggro, EventAggroTargetFound};
use crate::attack_auto::{AttackAuto, CommandAttackAutoStart, CommandAttackAutoStop};
use crate::base::state::State;
use crate::lane::Lane;
use crate::life::{Death, EventDead};
use crate::log::{CommandLog, EnumLogCategory};
use crate::map::MinionPath;
use crate::run::{CommandRunStart, Run, RunTarget};
use crate::team::Team;

#[derive(Component, Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
#[require(MinionState, Aggro = Aggro { range: 1000.0 }, State)]
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

#[derive(Default)]
pub struct PluginMinion;

impl Plugin for PluginMinion {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(FixedUpdate, fixed_update);

        app.add_observer(on_event_aggro_target_found);
        app.add_observer(on_event_dead);
    }
}

pub fn fixed_update(
    mut commands: Commands,
    q_minion: Query<
        (Entity, &Transform, &Team, &Lane, &MinionState, Option<&Run>),
        (With<Minion>, Without<Death>),
    >,
    res_minion_path: Res<MinionPath>,
) {
    for (entity, transform, team, lane, minion_state, _run) in q_minion.iter() {
        if *minion_state != MinionState::MovingOnPath {
            continue;
        }

        let minion_path = res_minion_path.0.get(lane).unwrap();

        let mut path = minion_path.clone();

        if matches!(team, Team::Chaos) {
            path.reverse();
        }

        let Some(closest_index) = find_next_point_index(&path, transform.translation.xz()) else {
            continue;
        };

        let target_pos = *path.get(closest_index).unwrap();

        // if let Some(run) = run {
        //     if let RunTarget::Position(pos) = run.target {
        //         if pos == target_pos {
        //             continue;
        //         }
        //     }
        // }

        commands.trigger(CommandLog {
            entity,
            info: format!("寻路到 {:?}", target_pos),
            category: EnumLogCategory::Minion,
        });
        commands.trigger(CommandRunStart {
            entity,
            target: RunTarget::Position(target_pos),
        });
    }
}

fn on_event_aggro_target_found(
    trigger: On<EventAggroTargetFound>,
    mut commands: Commands,
    mut q_minion: Query<(&mut MinionState, Option<&AttackAuto>)>,
) {
    let entity = trigger.event_target();

    let Ok((mut minion_state, attack_auto)) = q_minion.get_mut(entity) else {
        return;
    };

    let target = trigger.target;

    if *minion_state == MinionState::MovingOnPath {
        *minion_state = MinionState::AttackingTarget;
        commands.trigger(CommandLog {
            entity,
            info: format!("路径移动中发现仇恨目标 {:?}，开始自动攻击", target),
            category: EnumLogCategory::Minion,
        });
        commands.trigger(CommandAttackAutoStart { entity, target });
    } else if *minion_state == MinionState::AttackingTarget {
        if let Some(attack_auto) = attack_auto {
            if attack_auto.target != target {
                commands.trigger(CommandLog {
                    entity,
                    info: format!("切换攻击目标为新发现的仇恨目标 {:?}", target),
                    category: EnumLogCategory::Minion,
                });
                commands.trigger(CommandAttackAutoStart { entity, target });
            }
        } else {
            commands.trigger(CommandLog {
                entity,
                info: format!("发现仇恨目标 {:?}，开始自动攻击", target),
                category: EnumLogCategory::Minion,
            });
            commands.trigger(CommandAttackAutoStart { entity, target });
        }
    }
}

fn on_event_dead(
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

        if *minion_state == MinionState::AttackingTarget {
            *minion_state = MinionState::MovingOnPath;
            commands.trigger(CommandLog {
                entity,
                info: "目标死亡，转为沿路移动".to_string(),
                category: EnumLogCategory::Minion,
            });
            commands.trigger(CommandAttackAutoStop { entity });
        }
    }
}

fn find_next_point_index(path: &Vec<Vec2>, position: Vec2) -> Option<usize> {
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

    // // 确保不往回走。如果找到的最近点不是第一个点，检查小兵是否还没有越过该点
    // if closest_index > 0 {
    //     let prev_point = path[closest_index - 1];
    //     let curr_point = path[closest_index];

    //     // 当前路段的方向向量
    //     let path_direction = curr_point - prev_point;
    //     // 目标相对于最近点的偏移向量
    //     let position_offset = position - curr_point;

    //     // 如果点积小于或等于 0，说明还没有越过最近点，应该将 closest_index 减 1。
    //     // 这样外部 fixed_update 取 closest_index + 1 时，目标正好是 curr_point。
    //     if path_direction.dot(position_offset) > 0.0 {
    //         closest_index += 1;
    //     }
    // }

    Some(if closest_index + 1 >= path.len() {
        closest_index
    } else {
        closest_index + 1
    })
}
