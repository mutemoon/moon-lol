use bevy::{animation::AnimationTarget, prelude::*};
use league_core::MissileSpecificationMovementComponent;
use league_utils::{hash_bin, hash_joint};
use serde::{Deserialize, Serialize};

use crate::{
    CommandMovement, CommandParticleSpawn, Movement, MovementAction, MovementWay, ResourceCache,
};

#[derive(Default)]
pub struct PluginMissile;

impl Plugin for PluginMissile {
    fn build(&self, app: &mut App) {
        app.add_observer(on_command_missile_create);

        app.add_systems(FixedUpdate, fixed_update);
    }
}

/// 攻击组件 - 包含攻击的基础属性
#[derive(Debug, Component, Clone)]
pub struct Missile {
    pub key: u32,
    pub speed: f32,
}

/// 攻击状态机
#[derive(Component, Clone, Serialize, Deserialize)]
pub struct MissileState {
    /// 攻击目标
    pub target: Option<Entity>,
}

#[derive(EntityEvent, Debug)]
pub struct CommandMissileCreate {
    pub entity: Entity,
    pub target: Entity,
    pub spell_key: u32,
}

fn fixed_update() {}

fn on_command_missile_create(
    trigger: On<CommandMissileCreate>,
    mut commands: Commands,
    resource_cache: Res<ResourceCache>,
    q_global_transform: Query<&GlobalTransform>,
    q_children: Query<&Children>,
    q_joint_target: Query<&AnimationTarget>,
) {
    let entity = trigger.event_target();

    let spell_object = resource_cache.spells.get(&trigger.spell_key).unwrap();

    let spell_data_resource = spell_object.m_spell.clone().unwrap();

    let speed = spell_data_resource.missile_speed.unwrap();

    let target_translation = q_global_transform
        .get(trigger.target)
        .unwrap()
        .translation();

    let mut start_translation = None;

    if let Some(m_missile_spec) = spell_data_resource.m_missile_spec {
        match m_missile_spec.movement_component {
            MissileSpecificationMovementComponent::FixedSpeedMovement(fixed_speed_movement) => {
                if let Some(m_start_bone_name) = fixed_speed_movement.m_start_bone_name {
                    for child in q_children.iter_descendants(entity) {
                        let Ok(joint_target) = q_joint_target.get(child) else {
                            continue;
                        };
                        let id = joint_target.id.0.as_u128();
                        if hash_joint(&m_start_bone_name) as u128 == id {
                            start_translation =
                                Some(q_global_transform.get(child).unwrap().translation());
                            println!("start_translation: {:?}", start_translation);
                        }
                    }
                }
            }
            MissileSpecificationMovementComponent::TrackMouseMovement(track_mouse_movement) => {
                todo!()
            }
            MissileSpecificationMovementComponent::WallFollowMovement(wall_follow_movement) => {
                todo!()
            }
            MissileSpecificationMovementComponent::FixedTimeMovement(fixed_time_movement) => {
                todo!()
            }
            MissileSpecificationMovementComponent::FixedSpeedSplineMovement(
                fixed_speed_spline_movement,
            ) => todo!(),
            MissileSpecificationMovementComponent::DecelToLocationMovement(
                decel_to_location_movement,
            ) => todo!(),
            MissileSpecificationMovementComponent::PhysicsMovement(physics_movement) => todo!(),
            MissileSpecificationMovementComponent::AcceleratingMovement(accelerating_movement) => {
                todo!()
            }
            MissileSpecificationMovementComponent::ParametricMovement(parametric_movement) => {
                todo!()
            }
            MissileSpecificationMovementComponent::FixedTimeSplineMovement(
                fixed_time_spline_movement,
            ) => todo!(),
            MissileSpecificationMovementComponent::SyncCircleMovement(sync_circle_movement) => {
                todo!()
            }
            MissileSpecificationMovementComponent::CircleMovement(circle_movement) => {
                todo!()
            }
        }
    }

    let translation = match start_translation {
        Some(t) => t,
        None => q_global_transform.get(entity).unwrap().translation(),
    };

    let missile_entity = commands
        .spawn((
            Missile {
                key: trigger.spell_key,
                speed,
            },
            Transform::from_translation(translation),
            Movement { speed },
        ))
        .id();

    q_children.iter_descendants(entity);

    commands.trigger(CommandMovement {
        entity: missile_entity,
        priority: 0,
        action: MovementAction::Start {
            way: MovementWay::Path(vec![target_translation.xz()]),
            speed: Some(speed),
            source: "Missile".to_string(),
        },
    });
    commands.trigger(CommandParticleSpawn {
        entity: missile_entity,
        particle: spell_data_resource.m_missile_effect_key.unwrap(),
    });
}
