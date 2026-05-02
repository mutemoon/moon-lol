use std::ops::Deref;
use std::time::Duration;

use bevy::prelude::*;
use lol_base::animation::{
    AnimationConfig, AnimationConfigOf, ConfigParametricUpdater, LOLAnimationGraph,
    LOLAnimationGraphHandle, LOLAnimationState,
};
use lol_base::render_cmd::CommandAnimationPlay;
use lol_core::attack::Attack;
use lol_core::base::state::State;
use lol_core::movement::Movement;

use crate::loaders::animation::LoaderConfigAnimationLoader;

#[derive(Default)]
pub struct PluginAnimation;

impl Plugin for PluginAnimation {
    fn build(&self, app: &mut App) {
        app.init_asset::<LOLAnimationGraph>();

        app.init_asset_loader::<LoaderConfigAnimationLoader>();

        app.add_systems(Update, on_state_change);
        app.add_systems(Update, on_animation_state_change);
        app.add_systems(Update, update_transition_out);
        app.add_systems(Update, update_condition_animation);
        app.add_systems(Update, apply_animation_speed);

        app.add_observer(on_command_animation_play);
    }
}

#[derive(Component)]
pub struct AnimationTransitionOut {
    pub hash: String,
    pub weight: f32,
    pub duration: Duration,
    pub start_time: f32,
}

fn on_command_animation_play(
    trigger: On<CommandAnimationPlay>,
    mut query: Query<&mut LOLAnimationState>,
) {
    let event = trigger.event();
    let entity = trigger.event_target();

    let mut animation_state = query.get_mut(entity).unwrap();

    animation_state
        .update(event.hash.clone())
        .with_repeat(event.repeat);

    if let Some(duration) = event.duration {
        animation_state.with_duration(duration);
    }
}

fn on_state_change(
    mut query: Query<(Entity, &State, &mut LOLAnimationState), Changed<State>>,
    q_attack: Query<&Attack>,
) {
    for (entity, state, mut animation_state) in query.iter_mut() {
        match state {
            State::Idle => {
                debug!("[动画] 角色 {:?} 状态 -> Idle，切换动画 Idle1", entity);
                animation_state.update("Idle1".to_string());
            }
            State::Running => {
                debug!("[动画] 角色 {:?} 状态 -> Running，切换动画 Run", entity);
                animation_state.update("Run".to_string());
            }
            State::Attacking => {
                let attack = q_attack.get(entity).unwrap();
                debug!(
                    "[动画] 角色 {:?} 状态 -> Attacking，切换动画 Attack（duration={:?}）",
                    entity,
                    attack.animation_duration()
                );
                animation_state
                    .update("Attack".to_string())
                    .with_repeat(false)
                    .with_duration(attack.animation_duration());
            }
        }
    }
}

pub fn on_animation_state_change(
    mut q_bone: Query<(Entity, &mut AnimationPlayer, &AnimationGraphHandle)>,
    mut q_state: Query<
        (
            &LOLAnimationGraphHandle,
            &mut LOLAnimationState,
            &AnimationConfig,
        ),
        Or<(Added<LOLAnimationState>, Changed<LOLAnimationState>)>,
    >,
    q_transition_out: Query<&AnimationTransitionOut>,
    res_animation: Res<Assets<LOLAnimationGraph>>,
    mut commands: Commands,
    time: Res<Time>,
) {
    for (animation_handler, mut state, animation_config) in q_state.iter_mut() {
        let Ok((bone_entity, mut player, _)) = q_bone.get_mut(animation_config.deref().clone())
        else {
            warn!("[动画] 没有找到骨骼根节点实体",);
            continue;
        };
        let Some(animation) = res_animation.get(&animation_handler.0) else {
            warn!(
                "[动画] 骨骼实体 {:?} 的 LOLAnimationGraph 资源未加载",
                bone_entity
            );
            continue;
        };
        if let Ok(transition_out) = q_transition_out.get(bone_entity) {
            debug!(
                "[动画] 骨骼实体 {:?} 停止过渡动画 {}",
                bone_entity, transition_out.hash
            );
            animation.stop(&mut player, &transition_out.hash, &mut state);
        }

        let last_hash = state.last.clone();
        let current_hash = state.current.clone();
        let should_repeat = state.repeat;

        debug!(
            "[动画] 骨骼实体 {:?} 动画状态变更: {:?} -> {:?}（repeat={}）",
            bone_entity, last_hash, current_hash, should_repeat
        );

        if let Some(ref last_hash) = last_hash {
            if current_hash == *last_hash {
                if should_repeat {
                    continue;
                } else {
                    debug!(
                        "[动画] 骨骼实体 {:?} 停止不重复动画 {}",
                        bone_entity, last_hash
                    );
                    animation.stop(&mut player, last_hash, &mut state);
                }
            } else {
                debug!(
                    "[动画] 骨骼实体 {:?} 插入过渡: {} -> 0.0（100ms）",
                    bone_entity, last_hash
                );
                commands.entity(bone_entity).insert(AnimationTransitionOut {
                    hash: last_hash.clone(),
                    weight: 1.0,
                    duration: Duration::from_millis(100),
                    start_time: time.elapsed_secs(),
                });
            }
        }

        debug!(
            "[动画] 骨骼实体 {:?} 播放动画 {}（weight=1.0）",
            bone_entity, current_hash
        );
        animation.play(&mut player, &current_hash, 1.0, &mut state);
        if should_repeat {
            animation.repeat(&mut player, &current_hash, &state);
        }
    }
}

fn update_transition_out(
    mut commands: Commands,
    mut q_bone: Query<(
        Entity,
        &mut AnimationPlayer,
        &AnimationGraphHandle,
        &AnimationConfigOf,
        &AnimationTransitionOut,
    )>,
    mut q_state: Query<(&LOLAnimationGraphHandle, &mut LOLAnimationState)>,
    res_animation: Res<Assets<LOLAnimationGraph>>,
    time: Res<Time>,
) {
    for (bone_entity, mut player, _, anim_config_of, transition_out) in q_bone.iter_mut() {
        let Ok((animation_handler, mut state)) = q_state.get_mut(anim_config_of.0) else {
            continue;
        };
        let Some(animation) = res_animation.get(&animation_handler.0) else {
            continue;
        };
        let now = time.elapsed_secs();
        let elapsed = now - transition_out.start_time;
        let duration = transition_out.duration.as_secs_f32();

        if elapsed > duration {
            debug!(
                "[动画] 骨骼实体 {:?} 过渡完成，停止动画 {}",
                bone_entity, transition_out.hash
            );
            animation.stop(&mut player, &transition_out.hash, &mut state);
            commands
                .entity(bone_entity)
                .remove::<AnimationTransitionOut>();
            continue;
        }

        let weight = transition_out.weight * (1.0 - (elapsed / duration));
        animation.set_weight(&mut player, &transition_out.hash, weight, &state);
    }
}

fn update_condition_animation(
    query: Query<(Entity, &LOLAnimationGraphHandle, &LOLAnimationState)>,
    mut q_bone: Query<(&mut AnimationPlayer, &AnimationConfigOf)>,
    q_movement: Query<&Movement>,
    res_animation: Res<Assets<LOLAnimationGraph>>,
) {
    use lol_base::animation::ConfigAnimationNode;

    let play_list = query
        .iter()
        .filter_map(|(entity, animation_handler, state)| {
            let Some(animation) = res_animation.get(&animation_handler.0) else {
                return None;
            };
            let Some(node) = animation.hash_to_node.get(&state.current) else {
                return None;
            };

            let ConfigAnimationNode::ConditionFloat {
                conditions,
                updater,
                ..
            } = node
            else {
                return None;
            };

            let value = match updater {
                ConfigParametricUpdater::MoveSpeed => match q_movement.get(entity) {
                    Ok(m) => m.speed,
                    Err(_) => return None,
                },
                _ => return None,
            };

            if conditions.is_empty() {
                return None;
            }

            let mut found = false;
            let nodes: Vec<_> = conditions
                .iter()
                .rev()
                .map(|v| {
                    if found {
                        (v.key.clone(), 0.0)
                    } else if value >= v.value {
                        found = true;
                        (v.key.clone(), 1.0)
                    } else {
                        (v.key.clone(), 0.0)
                    }
                })
                .collect();

            debug!(
                "[动画] 角色 {:?} 条件动画评估: state={}, move_speed={:.1}, 权重={:?}",
                entity, state.current, value, nodes
            );

            Some((entity, animation_handler.0.clone(), state.clone(), nodes))
        })
        .collect::<Vec<_>>();

    for (entity, animation_handle, state, nodes) in play_list {
        let Some(animation) = res_animation.get(&animation_handle) else {
            continue;
        };
        let Some((mut player, _)) = q_bone.iter_mut().find(|(_, cf)| cf.0 == entity) else {
            continue;
        };

        for (key, weight) in nodes {
            animation.set_weight(&mut player, &key, weight, &state);
        }
    }
}

fn apply_animation_speed(
    mut q_bone: Query<(
        &mut AnimationPlayer,
        &AnimationGraphHandle,
        &AnimationConfigOf,
    )>,
    q_state: Query<(&LOLAnimationGraphHandle, &LOLAnimationState)>,
    res_animation_graph: Res<Assets<AnimationGraph>>,
    res_animation_clip: Res<Assets<AnimationClip>>,
    res_animation: Res<Assets<LOLAnimationGraph>>,
) {
    for (mut player, animation_graph_handle, anim_config_of) in q_bone.iter_mut() {
        let Ok((animation_handler, animation_state)) = q_state.get(anim_config_of.0) else {
            continue;
        };
        let Some(animation) = res_animation.get(&animation_handler.0) else {
            continue;
        };
        let Some(current_duration) = animation_state.current_duration else {
            continue;
        };

        let Some(animation_graph) = res_animation_graph.get(animation_graph_handle) else {
            continue;
        };

        let current_node_indices =
            animation.get_current_node_indices(&animation_state.current, animation_state);

        for index in current_node_indices {
            let Some(node) = animation_graph.get(index) else {
                continue;
            };

            let AnimationNodeType::Clip(ref clip_handle) = node.node_type else {
                continue;
            };

            let Some(clip) = res_animation_clip.get(clip_handle) else {
                continue;
            };

            let duration = clip.duration();
            let speed = duration / current_duration;

            debug!(
                "[动画] 角色 {:?} 动画速度: state={}, clip_duration={:.2}, target_duration={:.2}, speed={:.2}",
                anim_config_of.0, animation_state.current, duration, current_duration, speed
            );

            animation.set_speed(
                &mut player,
                &animation_state.current,
                speed,
                animation_state,
            );
        }
    }
}
