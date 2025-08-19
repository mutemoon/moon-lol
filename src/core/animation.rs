use std::collections::HashMap;

use bevy::{prelude::*, reflect::ReflectRef};
use rand::{
    distr::{weighted::WeightedIndex, Distribution},
    rng,
};

use crate::{
    core::{EventAttackCast, EventMovementEnd, EventMovementStart},
    league::LeagueLoader,
};

#[derive(Component)]
pub struct Animation {
    pub hash_to_node: HashMap<u32, AnimationNode>,
}

impl Animation {
    pub fn play(&self, player: &mut AnimationPlayer, key: u32, weight: f32) {
        let Some(node) = self.hash_to_node.get(&key) else {
            return;
        };

        match node {
            AnimationNode::Clip { node_index } => {
                player.play(*node_index).set_weight(weight).repeat();
            }
            AnimationNode::ConditionFloat { conditions, .. } => {
                for condition in conditions {
                    self.play(player, condition.key, weight);
                }
            }
            AnimationNode::Selector { probably_nodes } => {
                let weights = probably_nodes.iter().map(|v| v.value).collect::<Vec<_>>();
                let dist = WeightedIndex::new(weights).unwrap();
                let index = dist.sample(&mut rng());

                self.play(player, probably_nodes[index].key, weight);
            }
        }
    }
}

#[derive(Component)]
pub struct AnimationState {
    pub current_hash: u32,
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
    },
}

#[derive(Clone)]
pub struct AnimationNodeF32 {
    pub key: u32,
    pub value: f32,
}

#[derive(Default)]
pub struct PluginAnimation;

impl Plugin for PluginAnimation {
    fn build(&self, app: &mut App) {
        app.add_observer(on_movement_start);
        app.add_observer(on_movement_end);
        app.add_observer(on_command_attack_cast);
        app.add_systems(Update, on_animation_state_change);
        app.add_systems(Update, update_condition_animation);
    }
}

fn on_movement_start(trigger: Trigger<EventMovementStart>, mut query: Query<&mut AnimationState>) {
    let entity = trigger.target();

    let Ok(mut state) = query.get_mut(entity) else {
        return;
    };

    state.current_hash = LeagueLoader::hash_bin("Run");
}

fn on_movement_end(trigger: Trigger<EventMovementEnd>, mut query: Query<&mut AnimationState>) {
    let entity = trigger.target();

    let Ok(mut state) = query.get_mut(entity) else {
        return;
    };

    state.current_hash = LeagueLoader::hash_bin("Idle1");
}

fn on_command_attack_cast(
    trigger: Trigger<EventAttackCast>,
    mut query: Query<&mut AnimationState>,
) {
    let entity = trigger.target();

    let Ok(mut state) = query.get_mut(entity) else {
        return;
    };

    state.current_hash = LeagueLoader::hash_bin("Attack1");
}

fn on_animation_state_change(
    mut query: Query<(&mut AnimationPlayer, &Animation, &AnimationState), Changed<AnimationState>>,
) {
    for (mut player, animation, state) in query.iter_mut() {
        player.stop_all();
        animation.play(&mut player, state.current_hash, 1.0);
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
            animation.play(&mut player, key, weight);
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
