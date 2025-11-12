use std::{collections::HashMap, time::Duration};

use bevy::prelude::*;
use league_core::{AnimationGraphDataMBlendDataTable, ConditionBoolClipDataUpdater};
use league_utils::hash_bin;
use rand::{
    distr::{weighted::WeightedIndex, Distribution},
    rng,
};

use crate::{Attack, Movement, State};

#[derive(Default)]
pub struct PluginAnimation;

impl Plugin for PluginAnimation {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, on_state_change);
        app.add_systems(Update, on_animation_state_change);
        app.add_systems(Update, update_transition_out);
        app.add_systems(Update, update_condition_animation);
        app.add_systems(Update, apply_animation_speed);

        app.add_observer(on_command_animation_play);
    }
}

#[derive(Component, Clone)]
pub struct Animation {
    pub hash_to_node: HashMap<u32, AnimationNode>,
    pub blend_data: HashMap<(u32, u32), AnimationGraphDataMBlendDataTable>,
}

#[derive(Component, Clone, Debug)]
pub struct AnimationState {
    pub current_hash: u32,
    pub last_hash: Option<u32>,
    pub current_duration: Option<f32>,
    pub repeat: bool,
}

#[derive(Component)]
pub struct AnimationTransitionOut {
    pub hash: u32,
    pub weight: f32,
    pub duration: Duration,
    pub start_time: f32,
}

#[derive(Event, Default)]
pub struct CommandAnimationPlay {
    pub hash: u32,
    pub repeat: bool,
    pub duration: Option<f32>,
}

#[derive(Clone)]
pub enum AnimationNode {
    Clip {
        node_index: AnimationNodeIndex,
    },
    ConditionFloat {
        updater: ConditionBoolClipDataUpdater,
        conditions: Vec<AnimationNodeF32>,
    },
    Selector {
        probably_nodes: Vec<AnimationNodeF32>,
        current_index: Option<usize>,
    },
    Sequence {
        hashes: Vec<u32>,
        current_index: Option<usize>,
    },
    ConditionBool {
        updater: ConditionBoolClipDataUpdater,
        true_node: u32,
        false_node: u32,
    },
}

#[derive(Clone)]
pub struct AnimationNodeF32 {
    pub key: u32,
    pub value: f32,
}

impl Animation {
    pub fn get_node_indices(&mut self, key: u32) -> Vec<AnimationNodeIndex> {
        let Some(node) = self.hash_to_node.get_mut(&key) else {
            return Vec::new();
        };

        let keys = match node {
            AnimationNode::Clip { node_index, .. } => {
                return vec![*node_index];
            }
            AnimationNode::ConditionFloat { conditions, .. } => {
                conditions.iter().map(|v| v.key).collect()
            }
            AnimationNode::Selector {
                probably_nodes,
                current_index,
            } => {
                let index = match *current_index {
                    Some(index) => index,
                    None => {
                        let weights = probably_nodes.iter().map(|v| v.value).collect::<Vec<_>>();
                        let dist = WeightedIndex::new(weights).unwrap();
                        dist.sample(&mut rng())
                    }
                };
                *current_index = Some(index);
                vec![probably_nodes[index].key]
            }
            AnimationNode::Sequence {
                hashes,
                current_index,
            } => {
                *current_index = Some(0);

                vec![hashes[0]]
            }
            AnimationNode::ConditionBool {
                true_node,
                false_node,
                updater,
            } => {
                vec![*false_node]
            }
        };

        keys.iter()
            .flat_map(|v| self.get_node_indices(*v))
            .collect()
    }

    pub fn get_current_node_indices(&self, key: u32) -> Vec<AnimationNodeIndex> {
        let Some(node) = self.hash_to_node.get(&key) else {
            return Vec::new();
        };

        match node {
            AnimationNode::Clip { node_index, .. } => {
                return vec![*node_index];
            }
            AnimationNode::ConditionFloat { conditions, .. } => conditions
                .iter()
                .flat_map(|v| self.get_current_node_indices(v.key))
                .collect(),
            AnimationNode::Selector {
                probably_nodes,
                current_index,
            } => match current_index {
                Some(index) => self.get_current_node_indices(probably_nodes[*index].key),
                None => vec![],
            },
            AnimationNode::Sequence {
                hashes,
                current_index,
            } => match current_index {
                Some(index) => self.get_current_node_indices(hashes[*index]),
                None => vec![],
            },
            AnimationNode::ConditionBool {
                true_node,
                false_node,
                updater,
            } => self.get_current_node_indices(*false_node),
        }
    }

    pub fn get_current_nodes(&self, key: u32) -> Vec<u32> {
        let mut result = vec![key];

        let Some(node) = self.hash_to_node.get(&key) else {
            return Vec::new();
        };

        match node {
            AnimationNode::Clip { .. } => {}
            AnimationNode::ConditionFloat { conditions, .. } => {
                result.extend(
                    conditions
                        .iter()
                        .flat_map(|v| self.get_current_nodes(v.key)),
                );
            }
            AnimationNode::Selector {
                probably_nodes,
                current_index,
            } => match current_index {
                Some(index) => {
                    result.extend(self.get_current_nodes(probably_nodes[*index].key));
                }
                None => {}
            },
            AnimationNode::Sequence { hashes, .. } => {
                result.extend(hashes.iter().flat_map(|v| self.get_current_nodes(*v)));
            }
            AnimationNode::ConditionBool {
                true_node,
                false_node,
                updater,
            } => {
                result.extend(self.get_current_nodes(*false_node));
            }
        }

        result
    }

    pub fn play(&mut self, player: &mut AnimationPlayer, key: u32, weight: f32) {
        let node_indices = self.get_node_indices(key);

        for node_index in node_indices {
            player.play(node_index).set_weight(weight);
        }
    }

    pub fn repeat(&self, player: &mut AnimationPlayer, key: u32) {
        let node_indices = self.get_current_node_indices(key);
        for node_index in node_indices {
            if let Some(animation) = player.animation_mut(node_index) {
                animation.repeat();
            }
        }
    }

    pub fn stop(&mut self, player: &mut AnimationPlayer, key: u32) {
        let nodes = self.get_current_nodes(key);
        for node in nodes {
            let node = self.hash_to_node.get_mut(&node).unwrap();

            match node {
                AnimationNode::Clip { node_index, .. } => {
                    player.stop(*node_index);
                }
                AnimationNode::Selector { current_index, .. } => *current_index = None,
                _ => {}
            }
        }
    }

    pub fn set_speed(&self, player: &mut AnimationPlayer, key: u32, speed: f32) {
        let node_indices = self.get_current_node_indices(key);
        for node_index in node_indices {
            if let Some(animation) = player.animation_mut(node_index) {
                animation.set_speed(speed);
            }
        }
    }

    pub fn set_weight(&self, player: &mut AnimationPlayer, key: u32, weight: f32) {
        let node_indices = self.get_current_node_indices(key);
        for node_index in node_indices {
            if let Some(animation) = player.animation_mut(node_index) {
                animation.set_weight(weight);
            }
        }
    }

    pub fn get_weight(&self, player: &AnimationPlayer, key: u32) -> f32 {
        let node_indices = self.get_current_node_indices(key);
        let mut weight = 0.0;
        for node_index in node_indices {
            if let Some(animation) = player.animation(node_index) {
                weight = animation.weight().max(weight);
            }
        }
        weight
    }
}

impl AnimationState {
    pub fn update(&mut self, hash: u32) -> &mut Self {
        self.last_hash = Some(self.current_hash);
        self.current_hash = hash;
        self.current_duration = None;
        self.repeat = true;
        self
    }

    pub fn with_duration(&mut self, duration: f32) -> &mut Self {
        self.current_duration = Some(duration);
        self
    }

    pub fn with_repeat(&mut self, repeat: bool) -> &mut Self {
        self.repeat = repeat;
        self
    }
}

fn on_state_change(
    mut query: Query<(Entity, &State, &mut AnimationState), Changed<State>>,
    q_attack: Query<&Attack>,
) {
    for (entity, state, mut animation_state) in query.iter_mut() {
        match state {
            State::Idle => {
                animation_state.update(hash_bin("Idle1"));
                // animation_state.update(hash_bin("IdleIn"));
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
            _ => {}
        }
    }
}

fn on_command_animation_play(
    trigger: Trigger<CommandAnimationPlay>,
    mut query: Query<&mut AnimationState>,
) {
    let event = trigger.event();
    let entity = trigger.target();

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
            &mut Animation,
            &AnimationState,
        ),
        Changed<AnimationState>,
    >,
    q_transition_out: Query<&AnimationTransitionOut>,
    mut commands: Commands,
    time: Res<Time>,
) {
    for (entity, mut player, mut animation, state) in query.iter_mut() {
        if let Ok(transition_out) = q_transition_out.get(entity) {
            animation.stop(&mut player, transition_out.hash);
        }

        if let Some(last_hash) = state.last_hash {
            if state.current_hash == last_hash {
                if state.repeat {
                    continue;
                } else {
                    animation.stop(&mut player, last_hash);
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

        animation.play(&mut player, state.current_hash, 1.0);
        if state.repeat {
            animation.repeat(&mut player, state.current_hash);
        }
    }
}

fn update_transition_out(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut AnimationPlayer,
        &mut Animation,
        &AnimationTransitionOut,
    )>,
    time: Res<Time>,
) {
    for (entity, mut player, mut animation, transition_out) in query.iter_mut() {
        let now = time.elapsed_secs();

        let elapsed = now - transition_out.start_time;

        let duration = transition_out.duration.as_secs_f32();

        if elapsed > duration {
            animation.stop(&mut player, transition_out.hash);
            commands.entity(entity).remove::<AnimationTransitionOut>();
            continue;
        }

        let weight = transition_out.weight * (1.0 - (elapsed / duration));

        animation.set_weight(&mut player, transition_out.hash, weight);
    }
}

fn update_condition_animation(
    query: Query<(Entity, &Animation, &AnimationState)>,
    mut q_animation: Query<(&mut AnimationPlayer, &Animation)>,
    q_movement: Query<&Movement>,
) {
    let play_list = query
        .iter()
        .filter_map(|(entity, animation, state)| {
            let Some(node) = animation
                .hash_to_node
                .get(&state.current_hash)
                .map(|v| v.clone())
            else {
                return None;
            };

            let AnimationNode::ConditionFloat {
                conditions,
                updater,
                ..
            } = node
            else {
                return None;
            };

            let value = match updater {
                ConditionBoolClipDataUpdater::MoveSpeedParametricUpdater => {
                    q_movement.get(entity).unwrap().speed
                }
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

    for (entity, nodes) in play_list {
        let Ok((mut player, animation)) = q_animation.get_mut(entity) else {
            continue;
        };

        for (key, weight) in nodes {
            animation.set_weight(&mut player, key, weight);
        }
    }
}

fn apply_animation_speed(
    mut query: Query<(
        &mut AnimationPlayer,
        &Animation,
        &AnimationState,
        &AnimationGraphHandle,
    )>,
    res_animation_graph: Res<Assets<AnimationGraph>>,
    res_animation_clip: Res<Assets<AnimationClip>>,
) {
    for (mut player, animation, animation_state, animation_graph_handle) in query.iter_mut() {
        let Some(current_duration) = animation_state.current_duration else {
            continue;
        };

        let Some(animation_graph) = res_animation_graph.get(animation_graph_handle) else {
            continue;
        };

        let current_node_indices = animation.get_current_node_indices(animation_state.current_hash);

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
            );
        }
    }
}
