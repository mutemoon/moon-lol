use crate::combat::{
    Attack, AttackMachineState, AttackState, Bounding, CommandMovementMoveTo, CommandTargetRemove,
    CommandTargetSet, Dead, EventAttackAttack, EventMovementMoveEnd, Health, Navigator, Obstacle,
    Target, Team,
};
use bevy::{app::Plugin, prelude::*};
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize)]
#[require(MinionState, Navigator, Team, Obstacle, AggroInfo)]
pub enum Minion {
    Siege,
    Melee,
    Ranged,
    Super,
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

#[derive(Component)]
pub struct MinionPath(pub Vec<Vec2>);

#[derive(Event, Debug)]
pub struct EventMinionFoundTarget {
    pub target: Entity,
}

#[derive(Event, Debug)]
pub struct ChasingTimeOut;

#[derive(Event, Debug)]
pub struct CommandMinionContinuePath;

#[derive(Event, Debug)]
pub struct SystemMoveByPath {
    pub path: Vec<Vec2>,
}

#[derive(Event, Debug)]
pub struct ChasingTooMuch;

pub const AGGRO_RANGE: f32 = 500.0;

pub struct PluginMinion;

impl Plugin for PluginMinion {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_event::<EventMinionFoundTarget>();
        app.add_event::<ChasingTimeOut>();
        app.add_event::<ChasingTooMuch>();
        app.add_event::<SystemMoveByPath>();
        app.add_systems(FixedPostUpdate, minion_aggro);
        app.add_systems(FixedUpdate, on_spawn);
        app.add_observer(action_continue_minion_path);
        app.add_observer(on_system_move_by_path);
        app.add_observer(on_found_aggro_target);
        app.add_observer(on_target_dead);
    }
}

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
            // 忽略友方和死亡单位
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
    q_minion_path: Query<&MinionPath>,
    mut commands: Commands,
) {
    let Ok(minion_path) = q_minion_path.get(trigger.target()) else {
        return;
    };

    commands.trigger_targets(
        SystemMoveByPath {
            path: minion_path.0.clone(),
        },
        trigger.target(),
    );
}

fn on_system_move_by_path(trigger: Trigger<SystemMoveByPath>, mut commands: Commands) {
    // 由于移除了路径功能，这里需要外部系统来处理路径移动
    // 可以设置路径的第一个点作为目标，或者由外部路径管理系统处理
    if let Some(first_point) = trigger.path.first() {
        commands.trigger_targets(CommandMovementMoveTo(*first_point), trigger.target());
    }
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

    q_attack_state: Query<&AttackState>,
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
        commands.trigger_targets(EventMovementMoveEnd, trigger.target());
    } else {
        // 获取目标位置并设置为移动目标
        if let Ok(target_transform) = q_transform.get(event.target) {
            commands.trigger_targets(
                CommandMovementMoveTo(target_transform.translation.xz()),
                trigger.target(),
            );
        }
    }

    if let Ok(attack_state) = q_attack_state.get(entity) {
        match attack_state.get_state() {
            AttackMachineState::Idle => {}
            _ => (),
        }
    }

    if let Ok(mut minion_state) = q_minion_state.get_mut(entity) {
        match *minion_state {
            MinionState::MovingOnPath => {
                *minion_state = MinionState::AttackingTarget;
                let action = CommandTargetSet {
                    target: event.target,
                };
                commands.trigger_targets(action, trigger.target());
            }
            _ => (),
        }
    }
}

pub fn action_attack_damage(
    trigger: Trigger<EventAttackAttack>,
    mut q_attack_info: Query<&mut AttackState>,
    mut q_health: Query<&mut Health>,
    mut q_minion: Query<(Entity, &Team, &Transform, &mut AggroInfo), With<Minion>>,
    q_transform: Query<&Transform>,
    q_team: Query<&Team>,
) {
    let self_entity = trigger.target();
    let Ok(attack_info) = q_attack_info.get_mut(self_entity) else {
        return;
    };
    let Ok(self_transform) = q_transform.get(self_entity) else {
        return;
    };
    let Ok(self_team) = q_team.get(self_entity) else {
        return;
    };
    if let Some(target) = attack_info.target {
        if let Ok(mut health) = q_health.get_mut(target) {
            health.value -= 10.0;
        }
    }
    for (_, minion_team, minion_transform, mut aggro_info) in q_minion.iter_mut() {
        if self_team == minion_team {
            continue;
        }

        let distance = minion_transform
            .translation
            .distance(self_transform.translation);
        if distance >= AGGRO_RANGE {
            continue;
        }

        let aggro = aggro_info.aggros.get(&self_entity).copied().unwrap_or(0.0);

        aggro_info.aggros.insert(self_entity, aggro + 10.0);
    }
}

fn on_target_dead(
    trigger: Trigger<Dead>,
    mut commands: Commands,
    mut q_minion_state: Query<(&mut MinionState, &Target)>,
) {
    let dead_entity = trigger.target();

    for (mut minion_state, target) in q_minion_state.iter_mut() {
        if target.0 != dead_entity {
            continue;
        }

        match *minion_state {
            MinionState::AttackingTarget => {
                *minion_state = MinionState::MovingOnPath;
                commands.trigger_targets(CommandTargetRemove, trigger.target());
                commands.trigger_targets(CommandMinionContinuePath, trigger.target());
            }
            _ => (),
        }
    }
}

fn on_spawn(mut commands: Commands, mut q_minion_state: Query<(Entity, &mut MinionState)>) {
    for (entity, mut minion_state) in q_minion_state.iter_mut() {
        match *minion_state {
            MinionState::MovingOnPath => {
                *minion_state = MinionState::MovingOnPath;
                let action = CommandMinionContinuePath;
                commands.trigger_targets(action, entity);
            }
            _ => (),
        }
    }
}
