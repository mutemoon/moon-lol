use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::EventDamageCreate;

#[derive(Default)]
pub struct PluginLife;

impl Plugin for PluginLife {
    fn build(&self, app: &mut App) {
        app.register_type::<Health>();

        app.add_event::<EventDead>();
        app.add_event::<EventSpawn>();
        app.add_systems(FixedUpdate, spawn_event);
        app.add_observer(on_event_damage_create);
    }
}

#[derive(Component, Reflect, Serialize, Deserialize, Clone)]
#[reflect(Component)]
pub struct Health {
    pub value: f32,
    pub max: f32,
}

#[derive(Event, Debug)]
pub struct EventDead;

#[derive(Event, Debug)]
pub struct EventSpawn;

impl Health {
    pub fn new(max: f32) -> Health {
        Health { value: max, max }
    }
}

pub fn spawn_event(mut commands: Commands, q_alive: Query<Entity, Added<Health>>) {
    let spawn_count = q_alive.iter().count();
    if spawn_count > 0 {
        // println!("Spawning {} new entities with health", spawn_count);
    }

    for entity in q_alive.iter() {
        // println!("Triggering spawn event for entity {:?}", entity);
        commands.entity(entity).trigger(EventSpawn);
    }
}

fn on_event_damage_create(
    trigger: Trigger<EventDamageCreate>,
    mut commands: Commands,
    q_health: Query<&Health>,
) {
    let entity = trigger.target();

    let Ok(health) = q_health.get(entity) else {
        return;
    };

    if health.value <= 0.0 {
        debug!("{:?} 死了", entity);
        commands.entity(entity).despawn();
        commands.trigger_targets(EventDead, entity);
    }
}
