use crate::{combat::*,  map::Lane};
use bevy::{
    app::Plugin,
    prelude::*,
    time::common_conditions::on_timer,
};
use std::time::Duration;

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
pub struct FoundAggroTarget {
    pub target: Entity,
}

#[derive(Event, Debug)]
pub struct ChasingTimeOut;

#[derive(Event, Debug)]
pub struct ActionContinueMinionPath;

#[derive(Event, Debug)]
pub struct SystemMoveByPath {
    pub path: Vec<Vec2>,
}

#[derive(Event, Debug)]
pub struct ChasingTooMuch;
#[derive(Component, Clone, Copy)]
#[require(
    Transform,
    Navigator,
    MoveSpeed = MoveSpeed(365.0),
    MoveVelocity,
    AttackState,
    AttackInfo,
    MinionState,
    Lane,
    Team,
    Obstacle,
    Attackable,
    AggroInfo
)]
pub enum Minion {
    Siege,
    Melee,
    Ranged,
}

impl Minion {}

pub const AGGRO_RANGE: f32 = 500.0;

pub struct PluginMinion;

impl Plugin for PluginMinion {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_event::<FoundAggroTarget>();
        app.add_event::<ChasingTimeOut>();
        app.add_event::<ChasingTooMuch>();
        app.add_event::<SystemMoveByPath>();
        app.init_resource::<SpawnTimer>()
            .add_systems(
                FixedUpdate,
                setup.run_if(on_timer(Duration::from_secs(10000))),
            )
            .add_systems(
                FixedUpdate,
                spawn_next_minion.run_if(on_timer(Duration::from_secs(1))),
            )
            .add_systems(FixedPostUpdate, minion_aggro);
        app.add_systems(FixedUpdate, on_spawn);
        app.add_observer(action_continue_minion_path);
        app.add_observer(on_system_move_by_path);
        app.add_observer(on_found_aggro_target);
        app.add_observer(on_target_dead);
    }
}

#[derive(Resource, Default)]
struct SpawnTimer {
    minion_type_index: usize,
}

fn spawn_next_minion(commands: Commands, mut spawn_timer: ResMut<SpawnTimer>) {
    let lanes = [Lane::Mid];
    let teams = [Team::Blue, Team::Red];
    let minion_types = [
        Minion::Melee,
        Minion::Melee,
        Minion::Melee,
        Minion::Siege,
        Minion::Ranged,
        Minion::Ranged,
        Minion::Ranged,
    ];

    // let lanes = [Lane::Top, Lane::Mid, Lane::Bottom];
    // let teams = [Team::Blue, Team::Red];
    // let minion_types = [Minion::Siege];

    if spawn_timer.minion_type_index < minion_types.len() {
        let minion_type = minion_types[spawn_timer.minion_type_index];

        // for &team in teams.iter() {
        //     for &lane in lanes.iter() {
        //         commands.spawn(Minion::bundle(minion_type, team, lane));
        //     }
        // }

        spawn_timer.minion_type_index += 1;
    }
}

fn setup(mut spawn_timer: ResMut<SpawnTimer>) {
    // 重置生成计时器
    *spawn_timer = SpawnTimer::default();
}

// fn get_mirror_spawn_position(team: &Team, lane: &Lane) -> Vec3 {
//     match team {
//         Team::Blue => get_spawn_position(&Team::Red, &lane),
//         Team::Red => get_spawn_position(&Team::Blue, &lane),
//     }
// }

pub fn minion_aggro(
    mut commands: Commands,
    q_minion: Query<(Entity, &Team, &Transform, &AggroInfo), With<Minion>>,
    q_attackable: Query<(Entity, &Team, &Transform), With<Attackable>>,
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
                FoundAggroTarget {
                    target: target_entity,
                },
                minion_entity,
            );
        }
    }
}

pub fn action_continue_minion_path(
    trigger: Trigger<ActionContinueMinionPath>,
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
        commands.trigger_targets(ActionSetMoveTarget(*first_point), trigger.target());
    }
}

fn get_is_in_attack_range_in_found_aggro_target(
    trigger: &Trigger<FoundAggroTarget>,
    q_attack_range: &Query<&AttackRange>,
    q_transform: &Query<&Transform>,
    q_bounding: &Query<&Bounding>,
) -> bool {
    let Ok(attack_range) = q_attack_range.get(trigger.target()) else {
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
    transform.translation.distance(target_transform.translation) <= attack_range.0 + bounding.radius
}

fn on_found_aggro_target(
    trigger: Trigger<FoundAggroTarget>,
    mut commands: Commands,
    mut q_attack_state: Query<&mut AttackState>,
    mut q_minion_state: Query<&mut MinionState>,

    q_attack_range: Query<&AttackRange>,
    q_transform: Query<&Transform>,
    q_bounding: Query<&Bounding>,
) {
    let entity = trigger.target();
    let event = trigger.event();

    let is_in_attack_range = get_is_in_attack_range_in_found_aggro_target(
        &trigger,
        &q_attack_range,
        &q_transform,
        &q_bounding,
    );

    if is_in_attack_range {
        commands.trigger_targets(MoveEnd, trigger.target());
    } else {
        // 获取目标位置并设置为移动目标
        if let Ok(target_transform) = q_transform.get(event.target) {
            commands.trigger_targets(
                ActionSetMoveTarget(target_transform.translation.xy()),
                trigger.target(),
            );
        }
    }

    if let Ok(mut attack_state) = q_attack_state.get_mut(entity) {
        match *attack_state {
            AttackState::Idle => {
                *attack_state = AttackState::Locking;
                println!(
                    "entity: {:?}, attack_state: {:?} -> {:?} event: {:?}",
                    entity,
                    AttackState::Idle,
                    AttackState::Locking,
                    event
                );
                let action = ActionAttackReset;
                commands.trigger_targets(action, trigger.target());

                let action = ActionSetLockTime {
                    target: event.target,
                };
                commands.trigger_targets(action, trigger.target());
            }
            _ => (),
        }
    }

    if let Ok(mut minion_state) = q_minion_state.get_mut(entity) {
        match *minion_state {
            MinionState::MovingOnPath => {
                *minion_state = MinionState::AttackingTarget;
                println!(
                    "entity: {:?}, minion_state: {:?} -> {:?} event: {:?}",
                    entity,
                    MinionState::MovingOnPath,
                    MinionState::AttackingTarget,
                    event
                );
                let action = ActionSetTarget {
                    target: event.target,
                };
                commands.trigger_targets(action, trigger.target());
            }
            _ => (),
        }
    }
}

pub fn action_attack_damage(
    trigger: Trigger<ActionAttackDamage>,
    mut q_attack_info: Query<&mut AttackInfo>,
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
                commands.trigger_targets(ActionRemoveTarget, trigger.target());
                commands.trigger_targets(ActionContinueMinionPath, trigger.target());
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
                let action = ActionContinueMinionPath;
                commands.trigger_targets(action, entity);
            }
            _ => (),
        }
    }
}
