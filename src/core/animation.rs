use std::collections::HashMap;

use bevy::{prelude::*, reflect::ReflectRef};

use crate::{
    core::{EventMovementEnd, EventMovementStart},
    league::LeagueLoader,
};

#[derive(Component)]
pub struct Animation {
    pub hash_to_node: HashMap<u32, AnimationNode>,
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
        segments: Vec<(f32, AnimationNodeIndex)>,
        node_index: AnimationNodeIndex,
    },
}

pub struct PluginAnimation;

impl Plugin for PluginAnimation {
    fn build(&self, app: &mut App) {
        app.add_observer(on_movement_start);
        app.add_observer(on_movement_end);
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

fn update_condition_animation(world: &mut World) {
    let mut query = world.query::<(Entity, &Animation, &AnimationState)>();
    let mut player_query = world.query::<&mut AnimationPlayer>();

    let iter = query
        .iter(world)
        .filter_map(|(entity, animation, state)| {
            let Some(node) = animation
                .hash_to_node
                .get(&state.current_hash)
                .map(|v| v.clone())
            else {
                return None;
            };

            match node {
                AnimationNode::Clip { node_index } => {
                    return Some((entity, vec![(node_index, 1.0)]));
                }
                AnimationNode::ConditionFloat {
                    component_name,
                    field_name,
                    segments,
                    ..
                } => {
                    let Some(value) =
                        get_reflect_component_value(world, entity, &component_name, &field_name)
                    else {
                        return None;
                    };

                    if segments.is_empty() {
                        return None;
                    }

                    if value < segments[0].0 {
                        let (_, node_index) = segments[0];
                        return Some((entity, vec![(node_index, 1.0)]));
                    }

                    if let Some((_, node_index)) = segments.last() {
                        if value >= segments.last().unwrap().0 {
                            return Some((entity, vec![(*node_index, 1.0)]));
                        }
                    }

                    let Some(window) = segments
                        .windows(2)
                        .find(|w| value >= w[0].0 && value < w[1].0)
                    else {
                        return None;
                    };

                    let lower = &window[0];
                    let upper = &window[1];

                    let (lower_threshold, lower_index) = *lower;
                    let (upper_threshold, upper_index) = *upper;

                    let range = upper_threshold - lower_threshold;

                    let weight = if range > f32::EPSILON {
                        ((value - lower_threshold) / range).clamp(0.0, 1.0)
                    } else {
                        0.0
                    };

                    return Some((
                        entity,
                        vec![(lower_index, 1.0 - weight), (upper_index, weight)],
                    ));
                }
            }
        })
        .collect::<Vec<_>>();

    for (entity, nodes) in iter {
        let Ok(mut player) = player_query.get_mut(world, entity) else {
            return;
        };

        for (node_index, weight) in nodes {
            player.play(node_index).set_weight(weight).repeat();
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
