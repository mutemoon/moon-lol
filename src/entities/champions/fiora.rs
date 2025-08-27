use std::collections::HashMap;

use bevy::prelude::*;
use lol_core::Team;
use rand::random;

use crate::entities::champion::Champion;

#[derive(Component)]
#[require(Champion)]
pub struct Fiora;

#[derive(Default)]
pub struct PluginFiora;

#[derive(Resource, Default)]
pub struct FioraVitalLastDirection {
    entity_to_last_direction: HashMap<Entity, Direction>,
}

impl Plugin for PluginFiora {
    fn build(&self, app: &mut App) {
        app.init_resource::<FioraVitalLastDirection>();
        app.add_systems(FixedUpdate, update_add_vital);
        app.add_systems(FixedUpdate, update_remove_vital);
    }
}

const VITAL_DISTANCE: f32 = 1000.0;
const VITAL_ADD_DURATION: f32 = 1.0;
const VITAL_DURATION: f32 = 10.0;

#[derive(Clone)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Component)]
pub struct FioraVital {
    pub direction: Direction,
    pub active_timer: Timer,
    pub remove_timer: Timer,
}

impl FioraVital {
    pub fn is_active(&self) -> bool {
        self.active_timer.finished()
    }
}

fn update_add_vital(
    mut commands: Commands,
    q_target_without_vital: Query<
        (Entity, &Transform, &Team),
        (With<Champion>, Without<FioraVital>),
    >,
    q_fiora: Query<(&Transform, &Team), With<Fiora>>,
    mut last_direction: ResMut<FioraVitalLastDirection>,
) {
    for (fiora_transform, fiora_team) in q_fiora.iter() {
        for (target_entity, target_transform, target_team) in q_target_without_vital.iter() {
            if target_team == fiora_team {
                continue;
            }

            let distance = target_transform
                .translation
                .xz()
                .distance(fiora_transform.translation.xz());

            if distance > VITAL_DISTANCE {
                continue;
            }

            let direction = match last_direction.entity_to_last_direction.get(&target_entity) {
                Some(direction) => match direction {
                    Direction::Up | Direction::Right => {
                        if random::<bool>() {
                            Direction::Up
                        } else {
                            Direction::Right
                        }
                    }
                    Direction::Left | Direction::Down => {
                        if random::<bool>() {
                            Direction::Left
                        } else {
                            Direction::Down
                        }
                    }
                },
                None => {
                    if random::<bool>() {
                        Direction::Up
                    } else {
                        Direction::Left
                    }
                }
            };

            last_direction
                .entity_to_last_direction
                .insert(target_entity, direction.clone());

            commands.entity(target_entity).insert(FioraVital {
                direction,
                active_timer: Timer::from_seconds(VITAL_ADD_DURATION, TimerMode::Once),
                remove_timer: Timer::from_seconds(VITAL_DURATION, TimerMode::Once),
            });
        }
    }
}

fn update_remove_vital(
    mut commands: Commands,
    mut q_target_with_vital: Query<
        (Entity, &Transform, &Team, &mut FioraVital),
        (With<Champion>, With<FioraVital>),
    >,
    q_fiora: Query<(&Transform, &Team), With<Fiora>>,
    time: Res<Time<Fixed>>,
) {
    for (fiora_transform, fiora_team) in q_fiora.iter() {
        for (target_entity, target_transform, target_team, mut vital) in
            q_target_with_vital.iter_mut()
        {
            if target_team == fiora_team {
                continue;
            }

            let distance = target_transform
                .translation
                .xz()
                .distance(fiora_transform.translation.xz());

            if distance > VITAL_DISTANCE {
                commands.entity(target_entity).remove::<FioraVital>();
                continue;
            }

            if !vital.is_active() {
                vital.active_timer.tick(time.delta());
                continue;
            }

            vital.remove_timer.tick(time.delta());

            if vital.remove_timer.finished() {
                commands.entity(target_entity).remove::<FioraVital>();
            }
        }
    }
}
