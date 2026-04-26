use std::collections::HashMap;

use bevy::animation::AnimationPlayer;
use bevy::animation::graph::AnimationNodeIndex;
use bevy::prelude::*;
use rand::distr::Distribution;
use rand::distr::weighted::WeightedIndex;
use rand::rng;
use serde::{Deserialize, Serialize};

/// Animation clip data - stores keyframe animation data
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ConfigAnimationClip {
    pub fps: f32,
    pub duration: f32,
    pub joint_hashes: Vec<u32>,
    pub translates: Vec<Vec<(f32, Vec3)>>,
    pub rotations: Vec<Vec<(f32, Quat)>>,
    pub scales: Vec<Vec<(f32, Vec3)>>,
}

/// Animation graph asset - stable version of lol_render::Animation
#[derive(Asset, TypePath, Clone, Serialize, Deserialize)]
pub struct ConfigAnimation {
    pub hash_to_node: HashMap<u32, ConfigAnimationNode>,
    pub blend_data: HashMap<(u32, u32), ConfigBlendData>,
}

/// Animation node types
#[derive(Clone, Serialize, Deserialize)]
pub enum ConfigAnimationNode {
    Clip {
        node_index: AnimationNodeIndex,
    },
    ConditionFloat {
        updater: ConfigParametricUpdater,
        conditions: Vec<ConfigAnimationNodeF32>,
    },
    Selector {
        probably_nodes: Vec<ConfigAnimationNodeF32>,
    },
    Sequence {
        hashes: Vec<u32>,
    },
    ConditionBool {
        updater: ConfigParametricUpdater,
        true_node: u32,
        false_node: u32,
    },
}

/// Float condition for animation nodes
#[derive(Clone, Serialize, Deserialize)]
pub struct ConfigAnimationNodeF32 {
    pub key: u32,
    pub value: f32,
}

/// Parametric updater types - stable version
#[derive(Clone, Serialize, Deserialize)]
pub enum ConfigParametricUpdater {
    AttackSpeed,
    Displacement,
    EquippedGear,
    FacingAndMovementAngle,
    Facing,
    IsAlly,
    IsHomeguard,
    IsInTerrain,
    IsMoving,
    IsRanged,
    IsTurning,
    LogicDriverBool,
    LogicDriverFloat,
    LookAtGoldRedirectTargetAngle,
    LookAtInterestAngle,
    LookAtInterestDistance,
    LookAtSpellTargetAngle,
    LookAtSpellTargetDistance,
    LookAtSpellTargetHeightOffset,
    MoveSpeed,
    MovementDirection,
    ParBarPercent,
    SkinScale,
    SlopeAngle,
    TotalTurnAngle,
    TurnAngle,
    TurnAngleRemaining,
    Unknown,
}

/// Blend data types - stable version
#[derive(Clone, Serialize, Deserialize)]
pub enum ConfigBlendData {
    Time { time: f32 },
    TransitionClip { clip_name: u32 },
}

/// Animation handler component - holds handle to ConfigAnimation asset
#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct AnimationHandler(pub Handle<ConfigAnimation>);

/// Per-entity animation state for Selector and Sequence nodes
#[derive(Component, Clone, Debug)]
pub struct AnimationState {
    pub current_hash: u32,
    pub last_hash: Option<u32>,
    pub current_duration: Option<f32>,
    pub repeat: bool,
    pub selector_states: HashMap<u32, usize>,
    pub sequence_states: HashMap<u32, usize>,
}

impl Default for AnimationState {
    fn default() -> Self {
        Self {
            current_hash: 0,
            last_hash: None,
            current_duration: None,
            repeat: true,
            selector_states: HashMap::new(),
            sequence_states: HashMap::new(),
        }
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

impl ConfigAnimation {
    pub fn get_node_indices(
        &self,
        key: u32,
        state: &mut AnimationState,
    ) -> Vec<AnimationNodeIndex> {
        let Some(node) = self.hash_to_node.get(&key) else {
            return Vec::new();
        };

        let keys = match node {
            ConfigAnimationNode::Clip { node_index, .. } => {
                return vec![*node_index];
            }
            ConfigAnimationNode::ConditionFloat { conditions, .. } => {
                conditions.iter().map(|v| v.key).collect()
            }
            ConfigAnimationNode::Selector { probably_nodes } => {
                let index = state.selector_states.entry(key).or_insert_with(|| {
                    let weights = probably_nodes.iter().map(|v| v.value).collect::<Vec<_>>();
                    let dist = WeightedIndex::new(weights).unwrap();
                    dist.sample(&mut rng())
                });
                vec![probably_nodes[*index].key]
            }
            ConfigAnimationNode::Sequence { hashes } => {
                state.sequence_states.entry(key).or_insert(0);
                vec![hashes[0]]
            }
            ConfigAnimationNode::ConditionBool { false_node, .. } => {
                vec![*false_node]
            }
        };

        keys.iter()
            .flat_map(|v| self.get_node_indices(*v, state))
            .collect()
    }

    pub fn get_current_node_indices(
        &self,
        key: u32,
        state: &AnimationState,
    ) -> Vec<AnimationNodeIndex> {
        let Some(node) = self.hash_to_node.get(&key) else {
            return Vec::new();
        };

        match node {
            ConfigAnimationNode::Clip { node_index, .. } => {
                return vec![*node_index];
            }
            ConfigAnimationNode::ConditionFloat { conditions, .. } => conditions
                .iter()
                .flat_map(|v| self.get_current_node_indices(v.key, state))
                .collect(),
            ConfigAnimationNode::Selector { probably_nodes } => {
                match state.selector_states.get(&key) {
                    Some(index) => self.get_current_node_indices(probably_nodes[*index].key, state),
                    None => vec![],
                }
            }
            ConfigAnimationNode::Sequence { hashes } => match state.sequence_states.get(&key) {
                Some(index) => self.get_current_node_indices(hashes[*index], state),
                None => vec![],
            },
            ConfigAnimationNode::ConditionBool { false_node, .. } => {
                self.get_current_node_indices(*false_node, state)
            }
        }
    }

    pub fn get_current_nodes(&self, key: u32, state: &AnimationState) -> Vec<u32> {
        let mut result = vec![key];

        let Some(node) = self.hash_to_node.get(&key) else {
            return Vec::new();
        };

        match node {
            ConfigAnimationNode::Clip { .. } => {}
            ConfigAnimationNode::ConditionFloat { conditions, .. } => {
                result.extend(
                    conditions
                        .iter()
                        .flat_map(|v| self.get_current_nodes(v.key, state)),
                );
            }
            ConfigAnimationNode::Selector { probably_nodes } => {
                match state.selector_states.get(&key) {
                    Some(index) => {
                        result.extend(self.get_current_nodes(probably_nodes[*index].key, state));
                    }
                    None => {}
                }
            }
            ConfigAnimationNode::Sequence { hashes, .. } => {
                result.extend(
                    hashes
                        .iter()
                        .flat_map(|v| self.get_current_nodes(*v, state)),
                );
            }
            ConfigAnimationNode::ConditionBool { false_node, .. } => {
                result.extend(self.get_current_nodes(*false_node, state));
            }
        }

        result
    }

    pub fn play(
        &self,
        player: &mut AnimationPlayer,
        key: u32,
        weight: f32,
        state: &mut AnimationState,
    ) {
        let node_indices = self.get_node_indices(key, state);

        for node_index in node_indices {
            player.play(node_index).set_weight(weight);
        }
    }

    pub fn repeat(&self, player: &mut AnimationPlayer, key: u32, state: &AnimationState) {
        let node_indices = self.get_current_node_indices(key, state);
        for node_index in node_indices {
            if let Some(animation) = player.animation_mut(node_index) {
                animation.repeat();
            }
        }
    }

    pub fn stop(&self, player: &mut AnimationPlayer, key: u32, state: &mut AnimationState) {
        let nodes = self.get_current_nodes(key, state);
        for node_hash in nodes {
            let node = self.hash_to_node.get(&node_hash).unwrap();

            match node {
                ConfigAnimationNode::Clip { node_index, .. } => {
                    player.stop(*node_index);
                }
                ConfigAnimationNode::Selector { .. } => {
                    state.selector_states.remove(&node_hash);
                }
                _ => {}
            }
        }
    }

    pub fn set_speed(
        &self,
        player: &mut AnimationPlayer,
        key: u32,
        speed: f32,
        state: &AnimationState,
    ) {
        let node_indices = self.get_current_node_indices(key, state);
        for node_index in node_indices {
            if let Some(animation) = player.animation_mut(node_index) {
                animation.set_speed(speed);
            }
        }
    }

    pub fn set_weight(
        &self,
        player: &mut AnimationPlayer,
        key: u32,
        weight: f32,
        state: &AnimationState,
    ) {
        let node_indices = self.get_current_node_indices(key, state);
        for node_index in node_indices {
            if let Some(animation) = player.animation_mut(node_index) {
                animation.set_weight(weight);
            }
        }
    }

    pub fn get_weight(&self, player: &AnimationPlayer, key: u32, state: &AnimationState) -> f32 {
        let node_indices = self.get_current_node_indices(key, state);
        let mut weight = 0.0;
        for node_index in node_indices {
            if let Some(animation) = player.animation(node_index) {
                weight = animation.weight().max(weight);
            }
        }
        weight
    }
}
