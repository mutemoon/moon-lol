use std::collections::{BTreeMap, HashMap};

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
    /// 每个骨骼的蒙版权重，按 skeleton.influences 顺序排列
    /// None = 无蒙版，全部关节驱动
    /// Some(weights) = weights[i] 对应 influences[i] 的权重，0 = 不驱动
    pub mask_weights: Option<Vec<f32>>,
}

/// Animation graph asset - stable version of lol_render::Animation
#[derive(Asset, TypePath, Clone, Serialize, Deserialize)]
pub struct LOLAnimationGraph {
    pub gltf_path: String,
    pub hash_to_node: BTreeMap<String, ConfigAnimationNode>,
    pub blend_data: BTreeMap<(String, String), ConfigBlendData>,
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
        hashes: Vec<String>,
    },
    Parallel {
        hashes: Vec<String>,
    },
    Parametric {
        pairs: Vec<ConfigAnimationNodeF32>,
    },
    ConditionBool {
        updater: ConfigParametricUpdater,
        true_node: String,
        false_node: String,
    },
}

/// Float condition for animation nodes
#[derive(Clone, Serialize, Deserialize)]
pub struct ConfigAnimationNodeF32 {
    pub key: String,
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
    TransitionClip { clip_name: String },
}

/// Animation handler component - holds handle to ConfigAnimation asset
#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
#[require(LOLAnimationState)]
pub struct LOLAnimationGraphHandle(pub Handle<LOLAnimationGraph>);

/// Marks the entity that holds the AnimationPlayer and AnimationGraphHandle,
/// pointing back to the character entity that has the animation config.
#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
#[relationship(relationship_target = AnimationConfig)]
pub struct AnimationConfigOf(pub Entity);

/// Auto-maintained by Bevy on the character entity, pointing to the bone entity
/// that holds the AnimationPlayer.
#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
#[relationship_target(relationship = AnimationConfigOf, linked_spawn)]
pub struct AnimationConfig(Entity);

impl std::ops::Deref for AnimationConfig {
    type Target = Entity;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Per-entity animation state for Selector and Sequence nodes
#[derive(Component, Clone, Debug)]
pub struct LOLAnimationState {
    pub current: String,
    pub last: Option<String>,
    pub current_duration: Option<f32>,
    pub repeat: bool,
    pub selector_states: HashMap<String, usize>,
    pub sequence_states: HashMap<String, usize>,
}

impl Default for LOLAnimationState {
    fn default() -> Self {
        Self {
            current: String::from("Idle1"),
            last: None,
            current_duration: None,
            repeat: true,
            selector_states: HashMap::new(),
            sequence_states: HashMap::new(),
        }
    }
}

impl LOLAnimationState {
    pub fn update(&mut self, key: String) -> &mut Self {
        self.last = Some(self.current.clone());
        self.current = key;
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

impl LOLAnimationGraph {
    pub fn figure_node_indices(
        &self,
        key: &str,
        state: &mut LOLAnimationState,
    ) -> Vec<AnimationNodeIndex> {
        let Some(node) = self.hash_to_node.get(key) else {
            return Vec::new();
        };

        match node {
            ConfigAnimationNode::Clip { node_index, .. } => {
                vec![*node_index]
            }
            ConfigAnimationNode::ConditionFloat { conditions, .. } => conditions
                .iter()
                .last()
                .map(|v| self.figure_node_indices(&v.key, state))
                .unwrap(),
            ConfigAnimationNode::Selector { probably_nodes } => {
                let index = state
                    .selector_states
                    .entry(key.to_string())
                    .or_insert_with(|| {
                        let weights = probably_nodes.iter().map(|v| v.value).collect::<Vec<_>>();
                        let dist = WeightedIndex::new(weights).unwrap();
                        dist.sample(&mut rng())
                    });
                info!("{:?}", probably_nodes[*index].key);
                self.figure_node_indices(&probably_nodes[*index].key, state)
            }
            ConfigAnimationNode::Sequence { hashes } => {
                state.sequence_states.entry(key.to_string()).or_insert(0);
                self.figure_node_indices(&hashes[0], state)
            }
            ConfigAnimationNode::Parallel { hashes } => hashes
                .iter()
                .flat_map(|h| self.figure_node_indices(h, state))
                .collect(),
            ConfigAnimationNode::Parametric { pairs, .. } => pairs
                .iter()
                .flat_map(|v| self.figure_node_indices(&v.key, state))
                .collect(),
            ConfigAnimationNode::ConditionBool { false_node, .. } => {
                self.figure_node_indices(&false_node, state)
            }
        }
    }

    pub fn get_current_node_indices(
        &self,
        key: &str,
        state: &LOLAnimationState,
    ) -> Vec<AnimationNodeIndex> {
        let Some(node) = self.hash_to_node.get(key) else {
            return Vec::new();
        };

        match node {
            ConfigAnimationNode::Clip { node_index, .. } => {
                return vec![*node_index];
            }
            ConfigAnimationNode::ConditionFloat { conditions, .. } => conditions
                .iter()
                .flat_map(|v| self.get_current_node_indices(&v.key, state))
                .collect(),
            ConfigAnimationNode::Selector { probably_nodes } => {
                match state.selector_states.get(key) {
                    Some(index) => {
                        self.get_current_node_indices(&probably_nodes[*index].key, state)
                    }
                    None => vec![],
                }
            }
            ConfigAnimationNode::Sequence { hashes } => match state.sequence_states.get(key) {
                Some(index) => self.get_current_node_indices(&hashes[*index], state),
                None => vec![],
            },
            ConfigAnimationNode::Parallel { hashes } => hashes
                .iter()
                .flat_map(|h| self.get_current_node_indices(h, state))
                .collect(),
            ConfigAnimationNode::Parametric { pairs, .. } => pairs
                .iter()
                .flat_map(|v| self.get_current_node_indices(&v.key, state))
                .collect(),
            ConfigAnimationNode::ConditionBool { false_node, .. } => {
                self.get_current_node_indices(&false_node, state)
            }
        }
    }

    pub fn get_current_nodes(&self, key: &str, state: &LOLAnimationState) -> Vec<String> {
        let mut result = vec![key.to_string()];

        let Some(node) = self.hash_to_node.get(key) else {
            return Vec::new();
        };

        match node {
            ConfigAnimationNode::Clip { .. } => {}
            ConfigAnimationNode::ConditionFloat { conditions, .. } => {
                result.extend(
                    conditions
                        .iter()
                        .flat_map(|v| self.get_current_nodes(&v.key, state)),
                );
            }
            ConfigAnimationNode::Selector { probably_nodes } => {
                match state.selector_states.get(key) {
                    Some(index) => {
                        result.extend(self.get_current_nodes(&probably_nodes[*index].key, state));
                    }
                    None => {}
                }
            }
            ConfigAnimationNode::Sequence { hashes, .. } => {
                result.extend(hashes.iter().flat_map(|v| self.get_current_nodes(v, state)));
            }
            ConfigAnimationNode::Parallel { hashes, .. } => {
                result.extend(hashes.iter().flat_map(|v| self.get_current_nodes(v, state)));
            }
            ConfigAnimationNode::Parametric { pairs, .. } => {
                result.extend(
                    pairs
                        .iter()
                        .flat_map(|v| self.get_current_nodes(&v.key, state)),
                );
            }
            ConfigAnimationNode::ConditionBool { false_node, .. } => {
                result.extend(self.get_current_nodes(&false_node, state));
            }
        }

        result
    }

    pub fn play(
        &self,
        player: &mut AnimationPlayer,
        key: &str,
        weight: f32,
        state: &mut LOLAnimationState,
    ) {
        let node_indices = self.figure_node_indices(key, state);

        for node_index in node_indices {
            player.play(node_index).set_weight(weight);
        }
    }

    pub fn repeat(&self, player: &mut AnimationPlayer, key: &str, state: &LOLAnimationState) {
        let node_indices = self.get_current_node_indices(key, state);
        for node_index in node_indices {
            if let Some(animation) = player.animation_mut(node_index) {
                animation.repeat();
            }
        }
    }

    pub fn stop(&self, player: &mut AnimationPlayer, key: &str, state: &mut LOLAnimationState) {
        let nodes = self.get_current_nodes(key, state);
        for node_name in nodes {
            let node = self.hash_to_node.get(&node_name).unwrap();

            match node {
                ConfigAnimationNode::Clip { node_index, .. } => {
                    player.stop(*node_index);
                }
                ConfigAnimationNode::Selector { .. } => {
                    state.selector_states.remove(&node_name);
                }
                _ => {}
            }
        }
    }

    pub fn set_speed(
        &self,
        player: &mut AnimationPlayer,
        key: &str,
        speed: f32,
        state: &LOLAnimationState,
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
        key: &str,
        weight: f32,
        state: &LOLAnimationState,
    ) {
        let node_indices = self.get_current_node_indices(key, state);
        for node_index in node_indices {
            if let Some(animation) = player.animation_mut(node_index) {
                animation.set_weight(weight);
            }
        }
    }

    pub fn get_weight(
        &self,
        player: &AnimationPlayer,
        key: &str,
        state: &LOLAnimationState,
    ) -> f32 {
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
