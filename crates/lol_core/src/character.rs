use bevy::prelude::*;

use crate::base::level::{EventLevelUp, ExperienceDrop, Level};
use crate::life::EventDead;
use crate::team::Team;

#[derive(Default)]
pub struct PluginCharacter;

impl Plugin for PluginCharacter {
    fn build(&self, app: &mut App) {
        app.add_observer(on_event_dead);
    }
}

#[derive(Component, Debug)]
pub struct Character;

fn on_event_dead(
    event: On<EventDead>,
    query: Query<(Entity, &GlobalTransform, &ExperienceDrop, &Team)>,
    mut level_query: Query<(Entity, &GlobalTransform, &Team, &mut Level)>,
    mut commands: Commands,
) {
    let entity = event.event_target();

    let Ok((_, transform, exp_drop, team)) = query.get(entity) else {
        return;
    };

    if exp_drop.exp_given_on_death <= 0.0 {
        return;
    }

    let position = transform.translation();
    for (target_entity, target_transform, target_team, mut level) in level_query.iter_mut() {
        if target_team != team {
            continue;
        }

        if target_transform.translation().distance(position) > exp_drop.experience_radius {
            continue;
        }

        let exp_int = exp_drop.exp_given_on_death as u32;
        let levels_gained = level.add_experience(exp_int);
        if levels_gained == 0 {
            continue;
        }

        commands.trigger(EventLevelUp {
            entity: target_entity,
            level: level.value,
            delta: levels_gained,
        });
    }
}
