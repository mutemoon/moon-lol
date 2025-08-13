use std::collections::HashMap;

use bevy::prelude::*;

use crate::{
    core::{EventMovementStart, MovementState},
    league::LeagueLoader,
};

#[derive(Component)]

pub struct Animation {
    pub hash_to_node_index: HashMap<u32, AnimationNodeIndex>,
}

pub struct PluginAnimation;

impl Plugin for PluginAnimation {
    fn build(&self, app: &mut App) {
        app.add_observer(update_animation_move);
    }
}

fn update_animation_move(
    trigger: Trigger<EventMovementStart>,
    mut q_animation_move_state: Query<(&mut AnimationPlayer, &Animation, &MovementState)>,
) {
    let entity = trigger.target();

    let Ok((mut animation_player, animation, movement_state)) =
        q_animation_move_state.get_mut(entity)
    else {
        return;
    };

    if !movement_state.is_moving() {
        return;
    }

    let hash = LeagueLoader::hash_bin("Run");

    let Some(node_index) = animation.hash_to_node_index.get(&hash) else {
        return;
    };

    animation_player.play(*node_index).repeat();
}
