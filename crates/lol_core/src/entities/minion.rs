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
use crate::movement::{CommandMovement, MovementAction, MovementSource, MovementState, MovementWay};
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

#[derive(Component, Reflect, PartialEq, Debug, Default)]
#[reflect(Component)]
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
        app.register_type::<Minion>();
        app.register_type::<MinionState>();
        app.add_systems(FixedUpdate, fixed_update);

        app.add_observer(on_event_aggro_target_found);
        app.add_observer(on_event_dead);
        app.add_observer(on_reset_minions);
    }
}

pub fn on_reset_minions(
    _trigger: On<crate::action::EventReset>,
    mut commands: Commands,
    q_minions: Query<Entity, With<Minion>>,
) {
    for entity in q_minions.iter() {
        commands.entity(entity).despawn();
    }
}

pub fn fixed_update(
    mut commands: Commands,
    q_minion: Query<
        (Entity, &Transform, &Team, &Lane, &MinionState, Option<&MovementState>),
        (With<Minion>, Without<Death>),
    >,
    res_minion_path: Res<MinionPath>,
) {
    for (entity, transform, team, lane, minion_state, movement_state) in q_minion.iter() {
        if *minion_state != MinionState::MovingOnPath {
            continue;
        }

        // 如果小兵已有正在移动的路径且未完成，直接继续前行
        if let Some(state) = movement_state {
            if !state.completed && !state.path.is_empty() {
                continue;
            }
        }

        let Some(minion_path) = res_minion_path.0.get(lane) else {
            continue;
        };

        let mut path = minion_path.clone();

        if matches!(team, Team::Chaos) {
            path.reverse();
        }

        let Some(next_index) = find_next_point_index(&path, transform.translation.xz()) else {
            continue;
        };

        let current_y = transform.translation.y;
        let remaining_path_3d: Vec<Vec3> = path[next_index..]
            .iter()
            .map(|p| Vec3::new(p.x, current_y, p.y))
            .collect();

        if remaining_path_3d.is_empty() {
            continue;
        }

        commands.trigger(CommandLog {
            entity,
            info: format!("沿兵线直接移动，剩余路点: {}", remaining_path_3d.len()),
            category: EnumLogCategory::Minion,
        });

        commands.trigger(CommandMovement {
            entity,
            priority: 0,
            action: MovementAction::Start {
                way: MovementWay::Path(remaining_path_3d),
                speed: None,
                source: MovementSource::Run,
            },
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
