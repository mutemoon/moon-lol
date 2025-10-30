use std::collections::HashMap;

use bevy::prelude::*;
use rand::random;

use league_utils::hash_bin;
use lol_core::Team;

use crate::{
    core::{
        is_in_direction, CommandDamageCreate, CommandParticleDespawn, CommandParticleSpawn,
        DamageType, Direction, EventDamageCreate, Health, SkillOf,
    },
    entities::Champion,
};

const VITAL_DISTANCE: f32 = 1000.0;
const VITAL_ADD_DURATION: f32 = 1.7;
const VITAL_DURATION: f32 = 4.0;
const VITAL_TIMEOUT: f32 = 1.5;

#[derive(Default)]
pub struct PluginFioraPassive;

impl Plugin for PluginFioraPassive {
    fn build(&self, app: &mut App) {
        app.init_resource::<FioraVitalLastDirection>();
        app.add_systems(FixedUpdate, update_add_vital);
        app.add_systems(FixedUpdate, update_remove_vital);
        app.add_observer(on_damage_create);
    }
}

#[derive(Resource, Default)]
struct FioraVitalLastDirection {
    entity_to_last_direction: HashMap<Entity, Direction>,
}

#[derive(Component, Default)]
pub struct AbilityFioraPassive;

#[derive(Component)]
pub struct Vital {
    pub direction: Direction,
    pub active_timer: Timer,
    pub remove_timer: Timer,
    pub timeout_red_triggered: bool,
}

impl Vital {
    pub fn is_active(&self) -> bool {
        self.active_timer.finished()
    }
}

fn get_particle_hash(direction: &Direction, suffix: &str) -> u32 {
    let base_name = match direction {
        Direction::Up => "Fiora_Passive_NE",
        Direction::Right => "Fiora_Passive_NW",
        Direction::Down => "Fiora_Passive_SW",
        Direction::Left => "Fiora_Passive_SE",
    };
    let full_name = format!("{}{}", base_name, suffix);
    hash_bin(&full_name)
}

fn update_add_vital(
    mut commands: Commands,
    q_target_without_vital: Query<(Entity, &Transform, &Team), (With<Champion>, Without<Vital>)>,
    q_skill_of_with_ability: Query<&SkillOf, With<AbilityFioraPassive>>,
    q_transform_team: Query<(&Transform, &Team)>,
    mut last_direction: ResMut<FioraVitalLastDirection>,
) {
    for skill_of in q_skill_of_with_ability.iter() {
        let entity = skill_of.0;

        let (transform, team) = q_transform_team.get(entity).unwrap();

        for (target_entity, target_transform, target_team) in q_target_without_vital.iter() {
            if target_team == team {
                continue;
            }

            let distance = target_transform
                .translation
                .xz()
                .distance(transform.translation.xz());

            if distance > VITAL_DISTANCE {
                continue;
            }

            let direction = match last_direction.entity_to_last_direction.get(&target_entity) {
                Some(direction) => match direction {
                    Direction::Up | Direction::Right => {
                        if random::<bool>() {
                            Direction::Left
                        } else {
                            Direction::Down
                        }
                    }
                    Direction::Left | Direction::Down => {
                        if random::<bool>() {
                            Direction::Up
                        } else {
                            Direction::Right
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

            commands.entity(target_entity).insert(Vital {
                direction: direction.clone(),
                active_timer: Timer::from_seconds(VITAL_ADD_DURATION, TimerMode::Once),
                remove_timer: Timer::from_seconds(VITAL_DURATION, TimerMode::Once),
                timeout_red_triggered: false,
            });

            commands
                .entity(target_entity)
                .trigger(CommandParticleSpawn {
                    particle: get_particle_hash(&direction, "_Warning"),
                });
        }
    }
}

fn update_remove_vital(
    mut commands: Commands,
    mut q_target_with_vital: Query<
        (Entity, &Transform, &Team, &mut Vital),
        (With<Champion>, With<Vital>),
    >,
    q_skill_of_with_ability: Query<&SkillOf, With<AbilityFioraPassive>>,
    q_transform_team: Query<(&Transform, &Team)>,
    time: Res<Time<Fixed>>,
) {
    for skill_of in q_skill_of_with_ability.iter() {
        let entity = skill_of.0;

        let (fiora_transform, fiora_team) = q_transform_team.get(entity).unwrap();

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
                commands.entity(target_entity).remove::<Vital>();
                continue;
            }

            if !vital.is_active() {
                vital.active_timer.tick(time.delta());

                if vital.is_active() {
                    commands
                        .entity(target_entity)
                        .trigger(CommandParticleSpawn {
                            particle: get_particle_hash(&vital.direction, ""),
                        });
                }
                continue;
            }

            if !vital.timeout_red_triggered && vital.remove_timer.remaining_secs() <= VITAL_TIMEOUT
            {
                commands
                    .entity(target_entity)
                    .trigger(CommandParticleDespawn {
                        particle: get_particle_hash(&vital.direction, ""),
                    });
                commands
                    .entity(target_entity)
                    .trigger(CommandParticleSpawn {
                        particle: get_particle_hash(&vital.direction, "_TimeOut_Red"),
                    });

                vital.timeout_red_triggered = true;
            }

            vital.remove_timer.tick(time.delta());

            if vital.remove_timer.finished() {
                commands.entity(target_entity).remove::<Vital>();
            }
        }
    }
}

/// 监听伤害事件并创建伤害数字
fn on_damage_create(
    trigger: Trigger<EventDamageCreate>,
    mut commands: Commands,
    q_target_with_vital: Query<(&GlobalTransform, &Team, &Health, &Vital)>,
    q_transform: Query<(&GlobalTransform, &Team)>,
    mut last_direction: ResMut<FioraVitalLastDirection>,
) {
    let target_entity = trigger.target();
    let (transform, team) = q_transform.get(trigger.source).unwrap();

    let Ok((target_transform, target_team, hp, vital)) = q_target_with_vital.get(target_entity)
    else {
        return;
    };

    if target_team == team {
        return;
    }

    if !vital.is_active() {
        return;
    }

    let source_position = transform.translation().xz();
    let target_position = target_transform.translation().xz();

    if !is_in_direction(source_position, target_position, &vital.direction) {
        return;
    }

    commands
        .entity(target_entity)
        .trigger(CommandParticleSpawn {
            particle: hash_bin("Fiora_Passive_Hit_Tar"),
        });
    commands
        .entity(target_entity)
        .trigger(CommandParticleDespawn {
            particle: get_particle_hash(&vital.direction, ""),
        });

    let distance = source_position.distance(target_position);

    if distance > VITAL_DISTANCE {
        return;
    }

    let direction = match last_direction.entity_to_last_direction.get(&target_entity) {
        Some(direction) => match direction {
            Direction::Up | Direction::Right => {
                if random::<bool>() {
                    Direction::Left
                } else {
                    Direction::Down
                }
            }
            Direction::Left | Direction::Down => {
                if random::<bool>() {
                    Direction::Up
                } else {
                    Direction::Right
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

    commands.entity(target_entity).insert(Vital {
        direction: direction.clone(),
        active_timer: Timer::from_seconds(VITAL_ADD_DURATION, TimerMode::Once),
        remove_timer: Timer::from_seconds(VITAL_DURATION, TimerMode::Once),
        timeout_red_triggered: false,
    });

    commands
        .entity(target_entity)
        .trigger(CommandParticleSpawn {
            particle: get_particle_hash(&direction, "_Warning"),
        });

    commands.entity(target_entity).trigger(CommandDamageCreate {
        source: trigger.source,
        damage_type: DamageType::True,
        amount: hp.max * 0.05,
    });
}
