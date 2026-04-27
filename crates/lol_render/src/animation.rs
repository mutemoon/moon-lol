use std::time::Duration;

use bevy::prelude::*;
use league_utils::hash_bin;
use lol_base::animation::{
    AnimationHandler, AnimationState, ConfigAnimation, ConfigParametricUpdater,
};
use lol_base::render_cmd::CommandAnimationPlay;
use lol_core::attack::Attack;
use lol_core::base::state::State;
use lol_core::movement::Movement;

use crate::loaders::animation::LoaderConfigAnimation;

#[derive(Default)]
pub struct PluginAnimation;

impl Plugin for PluginAnimation {
    fn build(&self, app: &mut App) {
        app.init_asset::<ConfigAnimation>();

        app.init_asset_loader::<LoaderConfigAnimation>();

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
    pub hash: u32,
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
                animation_state.update(hash_bin("Idle1"));
            }
            State::Running => {
                animation_state.update(hash_bin("Run"));
            }
            State::Attacking => {
                let attack = q_attack.get(entity).unwrap();
                animation_state
                    .update(hash_bin("Attack"))
                    .with_repeat(false)
                    .with_duration(attack.animation_duration());
            }
        }
    }
}

fn on_command_animation_play(
    trigger: On<CommandAnimationPlay>,
    mut query: Query<&mut AnimationState>,
) {
    let event = trigger.event();
    let entity = trigger.event_target();

    let mut animation_state = query.get_mut(entity).unwrap();

    animation_state.update(event.hash).with_repeat(event.repeat);

    if let Some(duration) = event.duration {
        animation_state.with_duration(duration);
    }
}

fn on_animation_state_change(
    mut query: Query<
        (
            Entity,
            &mut AnimationPlayer,
            &AnimationHandler,
            &mut AnimationState,
        ),
        Changed<AnimationState>,
    >,
    q_transition_out: Query<&AnimationTransitionOut>,
    res_animation: Res<Assets<ConfigAnimation>>,
    mut commands: Commands,
    time: Res<Time>,
) {
    for (entity, mut player, animation_handler, mut state) in query.iter_mut() {
        let Some(animation) = res_animation.get(&animation_handler.0) else {
            continue;
        };
        if let Ok(transition_out) = q_transition_out.get(entity) {
            animation.stop(&mut player, transition_out.hash, &mut state);
        }

        if let Some(last_hash) = state.last_hash {
            if state.current_hash == last_hash {
                if state.repeat {
                    continue;
                } else {
                    animation.stop(&mut player, last_hash, &mut state);
                }
            } else {
                commands.entity(entity).insert(AnimationTransitionOut {
                    hash: last_hash,
                    weight: 1.0,
                    duration: Duration::from_millis(100),
                    start_time: time.elapsed_secs(),
                });
            }
        }

        animation.play(&mut player, state.current_hash, 1.0, &mut state);
        if state.repeat {
            animation.repeat(&mut player, state.current_hash, &state);
        }
    }
}

fn update_transition_out(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut AnimationPlayer,
        &AnimationHandler,
        &AnimationTransitionOut,
        &mut AnimationState,
    )>,
    res_animation: Res<Assets<ConfigAnimation>>,
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
            animation.stop(&mut player, transition_out.hash, &mut state);
            commands.entity(entity).remove::<AnimationTransitionOut>();
            continue;
        }

        let weight = transition_out.weight * (1.0 - (elapsed / duration));

        animation.set_weight(&mut player, transition_out.hash, weight, &state);
    }
}

fn update_condition_animation(
    query: Query<(Entity, &AnimationHandler, &AnimationState)>,
    mut q_animation: Query<(&mut AnimationPlayer, &AnimationHandler, &AnimationState)>,
    q_movement: Query<&Movement>,
    res_animation: Res<Assets<ConfigAnimation>>,
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
                .get(&state.current_hash)
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
                            (v.key, 0.0)
                        } else {
                            if value >= v.value {
                                found = true;
                                (v.key, 1.0)
                            } else {
                                (v.key, 0.0)
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
            animation.set_weight(&mut player, key, weight, state);
        }
    }
}

fn apply_animation_speed(
    mut query: Query<(
        &mut AnimationPlayer,
        &AnimationHandler,
        &AnimationState,
        &AnimationGraphHandle,
    )>,
    res_animation_graph: Res<Assets<AnimationGraph>>,
    res_animation_clip: Res<Assets<AnimationClip>>,
    res_animation: Res<Assets<ConfigAnimation>>,
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
            animation.get_current_node_indices(animation_state.current_hash, animation_state);

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
                animation_state.current_hash,
                duration / current_duration,
                animation_state,
            );
        }
    }
}
