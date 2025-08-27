use std::{collections::HashMap, time::Duration};

use bevy::{prelude::*, reflect::ReflectRef};
use league_utils::hash_bin;
use rand::{
    distr::{weighted::WeightedIndex, Distribution},
    rng,
};

use crate::core::{Attack, State};

#[derive(Default)]
pub struct PluginAnimation;

impl Plugin for PluginAnimation {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, on_state_change);
        app.add_systems(Update, on_animation_state_change);
        app.add_systems(Update, update_transition_out);
        app.add_systems(Update, update_condition_animation);
        app.add_systems(Update, apply_animation_speed);
    }
}

#[derive(Component)]
pub struct Animation {
    pub hash_to_node: HashMap<u32, AnimationNode>,
}

#[derive(Component)]
pub struct AnimationState {
    pub current_hash: u32,
    pub last_hash: u32,
    pub current_duration: Option<f32>,
}

#[derive(Component)]
pub struct AnimationTransitionOut {
    pub hash: u32,
    pub weight: f32,
    pub duration: Duration,
    pub start_time: f32,
}

#[derive(Clone)]
pub enum AnimationNode {
    Clip {
        node_index: AnimationNodeIndex,
    },
    ConditionFloat {
        component_name: String,
        field_name: String,
        conditions: Vec<AnimationNodeF32>,
    },
    Selector {
        probably_nodes: Vec<AnimationNodeF32>,
        current_index: Option<usize>,
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
        }
    }

    pub fn play(&mut self, player: &mut AnimationPlayer, key: u32, weight: f32) {
        let node_indices = self.get_node_indices(key);
        for node_index in node_indices {
            player.play(node_index).set_weight(weight).repeat();
        }
    }

    pub fn stop(&self, player: &mut AnimationPlayer, key: u32) {
        let node_indices = self.get_current_node_indices(key);
        for node_index in node_indices {
            player.stop(node_index);
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
}

impl AnimationState {
    pub fn update(&mut self, hash: u32) {
        self.last_hash = self.current_hash;
        self.current_hash = hash;
        self.current_duration = None;
    }

    pub fn update_with_duration(&mut self, hash: u32, duration: f32) {
        self.last_hash = self.current_hash;
        self.current_hash = hash;
        self.current_duration = Some(duration);
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
            }
            State::Moving => {
                animation_state.update(hash_bin("Run"));
            }
            State::Attacking => {
                let attack = q_attack.get(entity).unwrap();
                animation_state
                    .update_with_duration(hash_bin("Attack1"), attack.total_duration_secs());
            }
        }
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
        if state.current_hash == state.last_hash {
            continue;
        }

        if let Ok(transition_out) = q_transition_out.get(entity) {
            animation.stop(&mut player, transition_out.hash);
        }

        commands.entity(entity).insert(AnimationTransitionOut {
            hash: state.last_hash,
            weight: 1.0,
            duration: Duration::from_millis(200),
            start_time: time.elapsed_secs(),
        });

        animation.play(&mut player, state.current_hash, 1.0);
    }
}

fn update_transition_out(
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut AnimationPlayer,
        &Animation,
        &AnimationTransitionOut,
    )>,
    time: Res<Time>,
) {
    for (entity, mut player, animation, transition_out) in query.iter_mut() {
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

fn update_condition_animation(world: &mut World) {
    let mut query = world.query::<(Entity, &Animation, &AnimationState)>();
    let mut player_query = world.query::<(&mut AnimationPlayer, &Animation)>();

    let play_list = query
        .iter(world)
        .filter_map(|(entity, animation, state)| {
            let Some(node) = animation
                .hash_to_node
                .get(&state.current_hash)
                .map(|v| v.clone())
            else {
                return None;
            };

            let AnimationNode::ConditionFloat {
                component_name,
                field_name,
                conditions,
                ..
            } = node
            else {
                return None;
            };

            let Some(value) =
                get_reflect_component_value(world, entity, &component_name, &field_name)
            else {
                return None;
            };

            if conditions.is_empty() {
                return None;
            }

            if value < conditions[0].value {
                return Some((entity, vec![(conditions[0].key, 1.0)]));
            }

            if let Some(condition) = conditions.last() {
                if value >= condition.value {
                    return Some((entity, vec![(condition.key, 1.0)]));
                }
            }

            let Some(window) = conditions
                .windows(2)
                .find(|w| value >= w[0].value && value < w[1].value)
            else {
                return None;
            };

            let lower = &window[0];
            let upper = &window[1];

            let range = upper.value - lower.value;

            let weight = if range > f32::EPSILON {
                ((value - lower.value) / range).clamp(0.0, 1.0)
            } else {
                0.0
            };

            return Some((entity, vec![(lower.key, 1.0 - weight), (upper.key, weight)]));
        })
        .collect::<Vec<_>>();

    for (entity, nodes) in play_list {
        let Ok((mut player, animation)) = player_query.get_mut(world, entity) else {
            continue;
        };

        for (key, weight) in nodes {
            animation.set_weight(&mut player, key, weight);
        }
    }
}

fn get_reflect_component_value(
    world: &World,
    entity: Entity,
    component_name: &str,
    field_name: &str,
) -> Option<f32> {
    let registry = world.resource::<AppTypeRegistry>().read();
    let Some(type_registration) = registry.get_with_short_type_path(component_name) else {
        return None;
    };
    let Some(reflect_component) = type_registration.data::<ReflectComponent>() else {
        return None;
    };
    let Ok(entity_ref) = world.get_entity(entity) else {
        return None;
    };
    let Some(component) = reflect_component.reflect(entity_ref) else {
        return None;
    };
    let ReflectRef::Struct(struct_ref) = component.reflect_ref() else {
        return None;
    };
    let Some(value) = struct_ref.get_field::<f32>(field_name) else {
        return None;
    };
    Some(*value)
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
