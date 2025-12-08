use bevy::{ecs::relationship::Relationship, prelude::*};
use league_utils::hash_bin;
use lol_core::Team;

use crate::{
    abilities::get_particle_hash,
    core::{
        is_in_direction, Buff, BuffOf, CommandCharacterParticleDespawn,
        CommandCharacterParticleSpawn, Direction, EventDamageCreate, Health,
    },
};

const VITAL_R_TIMEOUT: f32 = 1.5;
const FIORA_R_ACTIVE_DURATION: f32 = 0.5;
const FIORA_R_DURATION: f32 = 7.;

#[derive(Default)]
pub struct PluginFioraR;

impl Plugin for PluginFioraR {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, fixed_update);
        app.add_observer(on_damage_create);
    }
}

#[derive(Component)]
#[require(Buff = Buff { name: "FioraR" })]
pub struct BuffFioraR {
    pub vitals: Vec<Direction>,
    pub active_timer: Timer,
    pub remove_timer: Timer,
    pub timeout_red_triggered: bool,
}

impl BuffFioraR {
    pub fn is_active(&self) -> bool {
        self.active_timer.is_finished()
    }
}

impl Default for BuffFioraR {
    fn default() -> Self {
        Self {
            vitals: vec![
                Direction::Up,
                Direction::Right,
                Direction::Down,
                Direction::Left,
            ],
            active_timer: Timer::from_seconds(FIORA_R_ACTIVE_DURATION, TimerMode::Once),
            remove_timer: Timer::from_seconds(FIORA_R_DURATION, TimerMode::Once),
            timeout_red_triggered: false,
        }
    }
}

fn fixed_update(
    mut commands: Commands,
    mut q_buff_fiora_r: Query<(Entity, &BuffOf, &mut BuffFioraR)>,
    time: Res<Time<Fixed>>,
) {
    for (entity, buff_of, mut buff) in q_buff_fiora_r.iter_mut() {
        let target_entity = buff_of.get();

        if !buff.is_active() {
            buff.active_timer.tick(time.delta());

            if buff.is_active() {
                for direction in buff.vitals.iter() {
                    commands.trigger(CommandCharacterParticleSpawn {
                        entity: target_entity,
                        hash: get_particle_hash(direction, "Fiora_R_Mark_", ""),
                    });
                    commands.trigger(CommandCharacterParticleSpawn {
                        entity: target_entity,
                        hash: get_particle_hash(direction, "Fiora_R_Mark_", "_FioraOnly"),
                    });
                }
            }
            continue;
        }

        if !buff.timeout_red_triggered && buff.remove_timer.remaining_secs() <= VITAL_R_TIMEOUT {
            for direction in buff.vitals.iter() {
                commands.entity(target_entity);
                commands.trigger(CommandCharacterParticleDespawn {
                    entity: target_entity,
                    hash: get_particle_hash(direction, "Fiora_R_Mark_", ""),
                });
                commands.trigger(CommandCharacterParticleSpawn {
                    entity: target_entity,
                    hash: get_particle_hash(direction, "Fiora_R_", "_Timeout"),
                });
            }
            buff.timeout_red_triggered = true;
        }

        buff.remove_timer.tick(time.delta());

        if buff.remove_timer.is_finished() {
            // 清理所有剩余的粒子
            for direction in buff.vitals.iter() {
                commands.entity(target_entity);
                commands.trigger(CommandCharacterParticleDespawn {
                    entity: target_entity,
                    hash: get_particle_hash(direction, "Fiora_R_Mark_", ""),
                });
                commands.trigger(CommandCharacterParticleDespawn {
                    entity: target_entity,
                    hash: get_particle_hash(direction, "Fiora_R_Mark_", "_FioraOnly"),
                });
                commands.trigger(CommandCharacterParticleDespawn {
                    entity: target_entity,
                    hash: get_particle_hash(direction, "Fiora_R_", "_Timeout"),
                });
            }
            commands.entity(entity).despawn();
        }
    }
}

/// 监听伤害事件并创建伤害数字
fn on_damage_create(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_target_with_vital: Query<(&GlobalTransform, &Team, &Health)>,
    q_transform: Query<(&GlobalTransform, &Team)>,
    mut q_buff_fiora_r: Query<(Entity, &BuffOf, &mut BuffFioraR)>,
) {
    let target_entity = trigger.event_target();
    let Ok((transform, team)) = q_transform.get(trigger.source) else {
        return;
    };

    let Some((buff_entity, mut buff_fiora_r)) =
        q_buff_fiora_r
            .iter_mut()
            .find_map(|(entity, buff_of, buff_fiora_r)| {
                if buff_of.get() == target_entity {
                    Some((entity, buff_fiora_r))
                } else {
                    None
                }
            })
    else {
        return;
    };

    let Ok((target_transform, target_team, _hp)) = q_target_with_vital.get(target_entity) else {
        return;
    };

    if target_team == team {
        return;
    }

    if !buff_fiora_r.is_active() {
        return;
    }

    let source_position = transform.translation().xz();
    let target_position = target_transform.translation().xz();

    let mut hit_direction: Option<Direction> = None;
    buff_fiora_r.vitals.retain(|direction| {
        if hit_direction.is_none() && is_in_direction(source_position, target_position, direction) {
            hit_direction = Some(direction.clone());
            false // 移除此方向
        } else {
            true // 保留此方向
        }
    });

    let Some(direction) = hit_direction else {
        return;
    };

    commands.trigger(CommandCharacterParticleSpawn {
        entity: target_entity,
        hash: hash_bin("Fiora_Passive_Hit_Tar"),
    });
    commands.trigger(CommandCharacterParticleDespawn {
        entity: target_entity,
        hash: get_particle_hash(&direction, "Fiora_R_Mark_", ""),
    });
    commands.trigger(CommandCharacterParticleDespawn {
        entity: target_entity,
        hash: get_particle_hash(&direction, "Fiora_R_Mark_", "_FioraOnly"),
    });
    commands.trigger(CommandCharacterParticleDespawn {
        entity: target_entity,
        hash: get_particle_hash(&direction, "Fiora_R_", "_Timeout"),
    });

    if buff_fiora_r.vitals.is_empty() {
        // TODO: 在这里触发治疗光环
        // 完成所有破绽，移除 buff
        commands.entity(buff_entity).despawn();
    }
}
