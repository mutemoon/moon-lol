use bevy::animation::AnimationTarget;
use bevy::color::palettes::tailwind::RED_500;
use bevy::prelude::*;
use league_core::{EnumMovement, SpellObject};
use league_utils::hash_joint;
use lol_config::{HashKey, LeagueProperties};
use serde::{Deserialize, Serialize};

use crate::{
    CommandDamageCreate, CommandMovement, CommandParticleSpawn, Damage, DamageType, DebugSphere,
    EntityCommandsTrigger, EventMovementEnd, Movement, MovementAction, MovementWay,
};

#[derive(Default)]
pub struct PluginMissile;

impl Plugin for PluginMissile {
    fn build(&self, app: &mut App) {
        app.add_observer(on_command_missile_create);
        app.add_observer(on_event_movement_end);

        app.add_systems(FixedUpdate, fixed_update);
    }
}

/// 攻击组件 - 包含攻击的基础属性
#[derive(Debug, Component, Clone)]
pub struct Missile {
    pub key: HashKey<SpellObject>,
    pub speed: f32,
}

/// 攻击状态机
#[derive(Component, Clone, Serialize, Deserialize)]
pub struct MissileState {
    pub source: Entity,
    /// 攻击目标
    pub target: Option<Entity>,
    pub target_bone: Option<Entity>,
}

#[derive(EntityEvent, Debug)]
pub struct CommandMissileCreate {
    pub entity: Entity,
    pub target: Entity,
    pub spell_key: HashKey<SpellObject>,
}

fn fixed_update(
    mut commands: Commands,
    q_missile: Query<(Entity, &Missile, &MissileState)>,
    q_transform: Query<&GlobalTransform>,
) {
    for (entity, missile, state) in q_missile.iter() {
        if let Some(target_bone) = state.target_bone {
            if let Ok(target_transform) = q_transform.get(target_bone) {
                let target_translation = target_transform.translation();
                commands.trigger(CommandMovement {
                    entity,
                    priority: 0,
                    action: MovementAction::Start {
                        way: MovementWay::Path(vec![target_translation]),
                        speed: Some(missile.speed),
                        source: "Missile".to_string(),
                    },
                });
            }
        }
    }
}

fn on_command_missile_create(
    trigger: On<CommandMissileCreate>,
    mut commands: Commands,
    res_assets_spell_object: Res<Assets<SpellObject>>,
    res_league_properties: Res<LeagueProperties>,
    q_global_transform: Query<&GlobalTransform>,
    q_children: Query<&Children>,
    q_joint_target: Query<&AnimationTarget>,
) {
    let entity = trigger.event_target();

    let spell_object = res_league_properties
        .get(&res_assets_spell_object, trigger.spell_key)
        .unwrap();

    let spell_data_resource = spell_object.m_spell.clone().unwrap();

    let speed = spell_data_resource.missile_speed.unwrap();

    let Ok(target_translation) = q_global_transform
        .get(trigger.target)
        .map(|v| v.translation())
    else {
        return;
    };

    let mut start_translation = None;

    if let Some(m_missile_spec) = spell_data_resource.m_missile_spec {
        match m_missile_spec.movement_component {
            EnumMovement::FixedSpeedMovement(fixed_speed_movement) => {
                if let Some(m_start_bone_name) = fixed_speed_movement.m_start_bone_name {
                    for child in q_children.iter_descendants(entity) {
                        let Ok(joint_target) = q_joint_target.get(child) else {
                            continue;
                        };
                        let id = joint_target.id.0.as_u128();
                        if hash_joint(&m_start_bone_name) as u128 == id {
                            start_translation =
                                Some(q_global_transform.get(child).unwrap().translation());
                        }
                    }
                }
            }
            _ => {
                // TODO: Implement other movement types
            }
        }
    }

    let mut end_entity = trigger.target;
    if let Some(m_hit_bone_name) = spell_data_resource.m_hit_bone_name {
        for child in q_children.iter_descendants(trigger.target) {
            let Ok(joint_target) = q_joint_target.get(child) else {
                continue;
            };
            let id = joint_target.id.0.as_u128();
            if hash_joint(&m_hit_bone_name) as u128 == id {
                end_entity = child;
                break;
            }
        }
    }

    let translation = match start_translation {
        Some(t) => t,
        None => {
            if let Ok(t) = q_global_transform.get(entity) {
                t.translation()
            } else {
                return;
            }
        }
    };

    debug!("{} 发射导弹 {:?}", entity, trigger.spell_key);
    let missile_entity = commands
        .spawn((
            Missile {
                key: trigger.spell_key,
                speed,
            },
            MissileState {
                source: entity,
                target: Some(trigger.target),
                target_bone: Some(end_entity),
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
            way: MovementWay::Path(vec![target_translation]),
            speed: Some(speed),
            source: "Missile".to_string(),
        },
    });
    if let Some(particle) = spell_data_resource.m_missile_effect_key {
        commands.trigger(CommandParticleSpawn {
            entity: missile_entity,
            hash: particle.into(),
        });
    } else {
        commands.entity(missile_entity).insert(DebugSphere {
            color: RED_500.into(),
            radius: 10.0,
        });
    }
}

fn on_event_movement_end(
    trigger: On<EventMovementEnd>,
    mut commands: Commands,
    q_missile: Query<&MissileState>,
    q_damage: Query<&Damage>,
) {
    let Ok(state) = q_missile.get(trigger.entity) else {
        return;
    };

    commands.entity(trigger.entity).despawn();

    let Some(target) = state.target else {
        return;
    };

    if let Ok(damage) = q_damage.get(state.source) {
        debug!("{} 对 {} 造成伤害 {}", state.source, target, damage.0);
        commands.try_trigger(CommandDamageCreate {
            entity: target,
            source: state.source,
            damage_type: DamageType::Physical,
            amount: damage.0,
        });
    }
}
