use bevy::animation::AnimationTargetId;
use bevy::color::palettes::tailwind::RED_500;
use bevy::prelude::*;
use league_utils::hash_joint;
use lol_base::debug_sphere::DebugSphere;
use lol_base::movement::{MissileBehavior, MovementType};
use lol_base::render_cmd::CommandSkinParticleSpawn;
use lol_base::spell::Spell;
use serde::{Deserialize, Serialize};

use crate::attack::EntityCommandsTrigger;
use crate::damage::{CommandDamageCreate, Damage, DamageType};
use crate::movement::{CommandMovement, EventMovementEnd, Movement, MovementAction, MovementWay};
use crate::team::Team;

#[derive(Default)]
pub struct PluginMissile;

impl Plugin for PluginMissile {
    fn build(&self, app: &mut App) {
        app.add_observer(on_command_missile_create);
        app.add_observer(on_event_movement_end);

        app.add_systems(FixedUpdate, fixed_update);
        app.add_systems(FixedUpdate, linear_missile_collision);
    }
}

/// 攻击组件 - 包含攻击的基础属性
#[derive(Debug, Component, Clone)]
pub struct Missile {
    pub key: Handle<Spell>,
    pub speed: f32,
}

/// 攻击状态机
#[derive(Component, Clone, Serialize, Deserialize)]
pub struct MissileState {
    pub source: Entity,
    /// 追踪目标（None 表示直线导弹）
    pub target: Option<Entity>,
    pub target_bone: Option<Entity>,
    /// 直线导弹的目标点
    pub destination: Option<Vec3>,
}

/// 直线导弹标记组件
#[derive(Component, Debug)]
pub struct LinearMissile {
    pub width: f32,
    pub damage: f32,
    pub hit_enemies: Vec<Entity>,
}

#[derive(EntityEvent, Debug)]
pub struct CommandMissileCreate {
    pub entity: Entity,
    /// Some(entity) 为追踪导弹，None 为直线导弹
    pub target: Option<Entity>,
    /// 直线导弹的目标位置
    pub destination: Option<Vec3>,
    pub spell: Handle<Spell>,
    /// 直线导弹的伤害值（追踪导弹从 source.Damage 读取）
    pub damage: f32,
    /// 覆盖导弹飞行速度（None 则使用 spell data 中的 missileSpeed）
    pub speed: Option<f32>,
    /// 覆盖 missile effect particle（None 则使用 spell data 中的 missileEffectKey）
    pub particle_hash: Option<u32>,
}

fn fixed_update(
    mut commands: Commands,
    q_missile: Query<(Entity, &Missile, &MissileState), Without<LinearMissile>>,
    q_transform: Query<&GlobalTransform>,
) {
    for (entity, missile, state) in q_missile.iter() {
        let Some(target_bone) = state.target_bone else {
            continue;
        };

        let Ok(target_transform) = q_transform.get(target_bone) else {
            continue;
        };

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

fn linear_missile_collision(
    mut commands: Commands,
    mut q_missile: Query<(
        Entity,
        &Missile,
        &MissileState,
        &mut LinearMissile,
        &mut Transform,
    )>,
    q_targets: Query<(Entity, &Team, &Transform), Without<LinearMissile>>,
    q_source_team: Query<&Team>,
    time: Res<Time<Fixed>>,
) {
    let dt = time.delta_secs();

    for (missile_entity, missile, state, mut linear, mut transform) in q_missile.iter_mut() {
        let Some(destination) = state.destination else {
            continue;
        };

        let current = transform.translation;
        let diff = destination - current;
        let distance = diff.length();

        if distance < 1.0 {
            commands.entity(missile_entity).despawn();
            continue;
        }

        let step = (missile.speed * dt).min(distance);
        let direction = diff / distance;
        transform.translation = current + direction * step;

        let Ok(source_team) = q_source_team.get(state.source) else {
            continue;
        };

        for (target, team, target_transform) in q_targets.iter() {
            if team == source_team {
                continue;
            }
            if linear.hit_enemies.contains(&target) {
                continue;
            }

            let to_target = target_transform.translation - transform.translation;
            let dist = to_target.length();
            if dist < linear.width {
                linear.hit_enemies.push(target);
                commands.trigger(CommandDamageCreate {
                    entity: target,
                    source: state.source,
                    damage_type: DamageType::Physical,
                    amount: linear.damage,
                });
            }
        }
    }
}

fn on_command_missile_create(
    trigger: On<CommandMissileCreate>,
    mut commands: Commands,
    res_assets_spell_object: Res<Assets<Spell>>,
    q_global_transform: Query<&GlobalTransform>,
    q_children: Query<&Children>,
    q_joint_target: Query<&AnimationTargetId>,
) {
    let entity = trigger.event_target();

    let spell_data = res_assets_spell_object
        .get(&trigger.spell)
        .and_then(|s| s.spell_data.as_ref());

    let speed = trigger
        .speed
        .or_else(|| spell_data.and_then(|d| d.missile_speed))
        .unwrap_or(1200.0);

    // 读取 missile spec 数据（start_bone, width, behaviors）
    let missile_spec = spell_data.and_then(|d| d.missile_spec.as_ref());
    let start_bone = missile_spec.and_then(|s| match &s.movement_component {
        MovementType::MovementTypeFixedSpeed(f) => f.start_bone_name.clone(),
    });
    let missile_width = missile_spec
        .and_then(|s| s.missile_width)
        .or(spell_data.and_then(|d| d.line_width));
    let _has_destroy_on_end = missile_spec
        .and_then(|s| {
            s.behaviors.as_ref().map(|b| {
                b.iter()
                    .any(|beh| matches!(beh, MissileBehavior::DestroyOnMovementComplete))
            })
        })
        .unwrap_or(true);
    let missile_effect = trigger
        .particle_hash
        .or(spell_data.and_then(|d| d.missile_effect_key));

    let translation = match &start_bone {
        Some(bone_name) => find_bone_translation(
            entity,
            bone_name,
            &q_children,
            &q_joint_target,
            &q_global_transform,
        )
        .unwrap_or_else(|| {
            q_global_transform
                .get(entity)
                .map(|t| t.translation())
                .unwrap_or_default()
        }),
        None => q_global_transform
            .get(entity)
            .map(|t| t.translation())
            .unwrap_or_default(),
    };

    // 直线导弹
    if trigger.target.is_none() {
        let Some(destination) = trigger.destination else {
            return;
        };

        let missile_entity = commands
            .spawn((
                Missile {
                    key: trigger.spell.clone(),
                    speed,
                },
                MissileState {
                    source: entity,
                    target: None,
                    target_bone: None,
                    destination: Some(destination),
                },
                LinearMissile {
                    width: missile_width.unwrap_or(100.0),
                    damage: trigger.damage,
                    hit_enemies: Vec::new(),
                },
                Transform::from_translation(translation),
                Movement { speed },
            ))
            .id();

        commands.trigger(CommandMovement {
            entity: missile_entity,
            priority: 0,
            action: MovementAction::Start {
                way: MovementWay::Path(vec![destination]),
                speed: Some(speed),
                source: "Missile".to_string(),
            },
        });

        if let Some(particle) = missile_effect {
            commands.trigger(CommandSkinParticleSpawn {
                entity: missile_entity,
                hash: particle,
            });
        } else {
            commands.entity(missile_entity).insert(DebugSphere {
                color: RED_500.into(),
                radius: 10.0,
            });
        }

        return;
    }

    // 追踪导弹
    let target = trigger.target.unwrap();

    let Ok(target_translation) = q_global_transform.get(target).map(|v| v.translation()) else {
        return;
    };

    let mut end_entity = target;
    if let Some(hit_bone_name) = spell_data.and_then(|d| d.hit_bone_name.clone()) {
        for child in q_children.iter_descendants(target) {
            let Ok(joint_target) = q_joint_target.get(child) else {
                continue;
            };
            let id = joint_target.0.as_u128();
            if hash_joint(&hit_bone_name) as u128 == id {
                end_entity = child;
                break;
            }
        }
    }

    debug!("{} 发射导弹 {:?}", entity, trigger.spell);
    let missile_entity = commands
        .spawn((
            Missile {
                key: trigger.spell.clone(),
                speed,
            },
            MissileState {
                source: entity,
                target: Some(target),
                target_bone: Some(end_entity),
                destination: None,
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
    if let Some(particle) = missile_effect {
        commands.trigger(CommandSkinParticleSpawn {
            entity: missile_entity,
            hash: particle,
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
    q_linear: Query<&LinearMissile>,
    q_damage: Query<&Damage>,
) {
    // 直线导弹到达终点：直接销毁（碰撞伤害已在 linear_missile_collision 中处理）
    if q_linear.get(trigger.entity).is_ok() {
        commands.entity(trigger.entity).despawn();
        return;
    }

    // 追踪导弹
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

fn find_bone_translation(
    entity: Entity,
    bone_name: &str,
    q_children: &Query<&Children>,
    q_joint_target: &Query<&AnimationTargetId>,
    q_global_transform: &Query<&GlobalTransform>,
) -> Option<Vec3> {
    for child in q_children.iter_descendants(entity) {
        let Ok(joint_target) = q_joint_target.get(child) else {
            continue;
        };
        let id = joint_target.0.as_u128();
        if hash_joint(bone_name) as u128 == id {
            return Some(q_global_transform.get(child).unwrap().translation());
        }
    }
    None
}
