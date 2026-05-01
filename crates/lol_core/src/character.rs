use bevy::ecs::entity::EntityHashMap;
use bevy::prelude::*;
use bevy::world_serialization::WorldInstanceSpawnError;
use lol_base::character::ConfigCharacterRecord;

use crate::base::level::{EventLevelUp, ExperienceDrop, Level};
use crate::life::EventDead;
use crate::team::Team;

#[derive(Default)]
pub struct PluginCharacter;

impl Plugin for PluginCharacter {
    fn build(&self, app: &mut App) {
        app.add_observer(on_event_dead);
        app.add_systems(FixedUpdate, try_load_config_characters);
    }
}

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
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

fn try_load_config_characters(
    mut commands: Commands,
    character_record_query: Query<(Entity, &ConfigCharacterRecord)>,
    dynamic_worlds: Res<Assets<DynamicWorld>>,
) {
    let char_len = character_record_query.iter().len();
    if char_len == 0 {
        return;
    }

    info!("加载 {} 个角色", char_len);

    // 处理 ConfigCharacterRecord - 写入角色数据到世界
    for (entity, config) in &character_record_query {
        if dynamic_worlds.get(&config.character_record).is_none() {
            return;
        }

        let handle = config.character_record.clone();
        commands.queue(move |world: &mut World| {
            world.resource_scope(|world, dynamic_worlds: Mut<Assets<DynamicWorld>>| {
                let dynamic_world = dynamic_worlds
                    .get(&handle)
                    .ok_or(WorldInstanceSpawnError::NonExistentDynamicWorld { id: handle.id() })?;

                let mut map = EntityHashMap::new();
                dynamic_world.entities.iter().for_each(|v| {
                    let components: Vec<_> = v
                        .components
                        .iter()
                        .map(|v| v.reflect_short_type_path())
                        .collect();
                    debug!("{}: [{}]", v.entity, components.join(", "));
                });
                let source_entity = dynamic_world
                    .entities
                    .iter()
                    .find(|v| {
                        v.components
                            .iter()
                            .any(|c| c.reflect_short_type_path().eq("Character"))
                    })
                    .expect("Character component not found in character config");
                map.entry(source_entity.entity).insert(entity);
                debug!("{} -> {}", source_entity.entity, entity);
                dynamic_world.write_to_world(world, &mut map)
            })
        });

        commands.entity(entity).remove::<ConfigCharacterRecord>();
    }
}
