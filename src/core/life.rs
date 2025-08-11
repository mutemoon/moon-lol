use crate::{system_debug, system_info, system_warn};
use bevy::prelude::*;
#[derive(Component, Default, Reflect)]
pub struct Health {
    pub value: f32,
    pub max: f32,
}

#[derive(Event, Debug)]
pub struct EventDead;

#[derive(Event, Debug)]
pub struct EventSpawn;

pub struct PluginLife;

impl Plugin for PluginLife {
    fn build(&self, app: &mut App) {
        app.add_event::<EventDead>();
        app.add_event::<EventSpawn>();
        app.add_systems(FixedUpdate, (detect_death, spawn_event));
        app.add_observer(on_dead);
    }
}

pub fn spawn_event(mut commands: Commands, q_alive: Query<Entity, Added<Health>>) {
    let spawn_count = q_alive.iter().count();
    if spawn_count > 0 {
        system_info!(
            "spawn_event",
            "Spawning {} new entities with health",
            spawn_count
        );
    }

    for entity in q_alive.iter() {
        system_debug!(
            "spawn_event",
            "Triggering spawn event for entity {:?}",
            entity
        );
        commands.trigger_targets(EventSpawn, entity);
    }
}

pub fn detect_death(mut commands: Commands, q_health: Query<(Entity, &Health)>) {
    let mut death_count = 0;

    for (entity, health) in q_health.iter() {
        if health.value <= 0.0 {
            system_warn!(
                "detect_death",
                "Entity {:?} has died (health={:.1})",
                entity,
                health.value
            );
            commands.trigger_targets(EventDead, entity);
            death_count += 1;
        }
    }

    if death_count > 0 {
        system_info!("detect_death", "Detected {} deaths this frame", death_count);
    }
}

fn on_dead(trigger: Trigger<EventDead>, mut commands: Commands) {
    let entity = trigger.target();
    system_info!("on_dead", "Despawning dead entity {:?}", entity);
    commands.entity(entity).despawn();
}
