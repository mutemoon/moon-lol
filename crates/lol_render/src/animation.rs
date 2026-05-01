use std::time::Duration;

use bevy::prelude::*;
use lol_base::animation::{
    AnimationState, ConfigParametricUpdater, LOLAnimationGraph, LOLAnimationGraphHandler,
};
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

        // app.add_observer(on_command_animation_play);
    }
}

#[derive(Component)]
pub struct AnimationTransitionOut {
    pub hash: String,
    pub weight: f32,
    pub duration: Duration,
    pub start_time: f32,
}

fn on_state_change(
    mut query: Query<(Entity, &State, &mut AnimationState), Changed<State>>,
    q_attack: Query<&Attack>,
) {
    for (entity, state, mut animation_state) in query.iter_mut() {
        match state {
            State::Idle => {
                animation_state.update("Idle1".to_string());
            }
            State::Running => {
                animation_state.update("Run".to_string());
            }
            State::Attacking => {
                let attack = q_attack.get(entity).unwrap();
                animation_state
                    .update("Attack".to_string())
                    .with_repeat(false)
                    .with_duration(attack.animation_duration());
            }
        }
    }
}

fn on_animation_state_change(
    mut query: Query<
        (
            Entity,
            &mut AnimationPlayer,
            &LOLAnimationGraphHandler,
            &mut AnimationState,
        ),
        Changed<AnimationState>,
    >,
    q_transition_out: Query<&AnimationTransitionOut>,
    res_animation: Res<Assets<LOLAnimationGraph>>,
    mut commands: Commands,
    time: Res<Time>,
) {
    for (entity, mut player, animation_handler, mut state) in query.iter_mut() {
        let Some(animation) = res_animation.get(&animation_handler.0) else {
            continue;
        };
        if let Ok(transition_out) = q_transition_out.get(entity) {
            animation.stop(&mut player, &transition_out.hash, &mut state);
        }

        let last_hash = state.last.clone();
        let current_hash = state.current.clone();
        let should_repeat = state.repeat;

        if let Some(ref last_hash_ref) = last_hash {
            if current_hash == *last_hash_ref {
                if should_repeat {
                    continue;
                } else {
                    animation.stop(&mut player, last_hash_ref, &mut state);
                }
            } else {
                commands.entity(entity).insert(AnimationTransitionOut {
                    hash: last_hash_ref.clone(),
                    weight: 1.0,
                    duration: Duration::from_millis(100),
                    start_time: time.elapsed_secs(),
                });
            }
        }

        animation.play(&mut player, &current_hash, 1.0, &mut state);
        if should_repeat {
            animation.repeat(&mut player, &current_hash, &state);
        }
    }
}

fn update_transition_out(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut AnimationPlayer,
        &LOLAnimationGraphHandler,
        &AnimationTransitionOut,
        &mut AnimationState,
    )>,
    res_animation: Res<Assets<LOLAnimationGraph>>,
    time: Res<Time>,
) {
    for (entity, mut player, animation_handler, transition_out, mut state) in query.iter_mut() {
        let Some(animation) = res_animation.get(&animation_handler.0) else {
            continue;
        };
        let now = time.elapsed_secs();

        let elapsed = now - transition_out.start_time;

        let duration = transition_out.duration.as_secs_f32();

        if elapsed > duration {
            animation.stop(&mut player, &transition_out.hash, &mut state);
            commands.entity(entity).remove::<AnimationTransitionOut>();
            continue;
        }

        let weight = transition_out.weight * (1.0 - (elapsed / duration));

        animation.set_weight(&mut player, &transition_out.hash, weight, &state);
    }
}

fn update_condition_animation(
    query: Query<(Entity, &LOLAnimationGraphHandler, &AnimationState)>,
    mut q_animation: Query<(
        &mut AnimationPlayer,
        &LOLAnimationGraphHandler,
        &AnimationState,
    )>,
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
            let Some(node) = animation
                .hash_to_node
                .get(&state.current)
                .map(|v| v.clone())
            else {
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
                ConfigParametricUpdater::MoveSpeed => q_movement.get(entity).unwrap().speed,
                _ => {
                    return None;
                }
            };

            if conditions.is_empty() {
                return None;
            }

            let mut found = false;

            return Some((
                entity,
                animation_handler.0.clone(),
                conditions
                    .iter()
                    .rev()
                    .map(|v| {
                        if found {
                            (v.key.clone(), 0.0)
                        } else {
                            if value >= v.value {
                                found = true;
                                (v.key.clone(), 1.0)
                            } else {
                                (v.key.clone(), 0.0)
                            }
                        }
                    })
                    .collect::<Vec<_>>(),
            ));
        })
        .collect::<Vec<_>>();

    for (entity, animation_handle, nodes) in play_list {
        let Ok((mut player, _, state)) = q_animation.get_mut(entity) else {
            continue;
        };
        let Some(animation) = res_animation.get(&animation_handle) else {
            continue;
        };

        for (key, weight) in nodes {
            animation.set_weight(&mut player, &key, weight, state);
        }
    }
}

fn apply_animation_speed(
    mut query: Query<(
        &mut AnimationPlayer,
        &LOLAnimationGraphHandler,
        &AnimationState,
        &AnimationGraphHandle,
    )>,
    res_animation_graph: Res<Assets<AnimationGraph>>,
    res_animation_clip: Res<Assets<AnimationClip>>,
    res_animation: Res<Assets<LOLAnimationGraph>>,
) {
    for (mut player, animation_handler, animation_state, animation_graph_handle) in query.iter_mut()
    {
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

            animation.set_speed(
                &mut player,
                &animation_state.current,
                duration / current_duration,
                animation_state,
            );
        }
    }
}
