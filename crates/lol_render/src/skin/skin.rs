use bevy::ecs::entity::EntityHashMap;
use bevy::prelude::*;
use bevy::world_serialization::WorldInstanceSpawnError;
use lol_base::character::{ConfigSkin, Skin};

pub fn update_skin_scale(mut query: Query<(&Skin, &mut Transform)>) {
    for (skin, mut transform) in query.iter_mut() {
        transform.scale = Vec3::splat(skin.scale);
    }
}

pub fn try_load_config_skin_characters(
    mut commands: Commands,
    skin_query: Query<(Entity, &ConfigSkin)>,
    dynamic_worlds: Res<Assets<DynamicWorld>>,
) {
    let skin_len = skin_query.iter().len();
    if skin_len == 0 {
        return;
    }

    // info!("加载 {} 个皮肤", skin_len);

    // 处理 ConfigSkin - 写入皮肤数据到世界
    for (entity, config) in &skin_query {
        if dynamic_worlds.get(&config.skin).is_none() {
            return;
        }

        let handle = config.skin.clone();
        commands.queue(move |world: &mut World| {
            world.resource_scope(|world, dynamic_worlds: Mut<Assets<DynamicWorld>>| {
                let dynamic_world = dynamic_worlds
                    .get(&handle)
                    .ok_or(WorldInstanceSpawnError::NonExistentDynamicWorld { id: handle.id() })?;

                let mut map = EntityHashMap::new();
                map.entry(dynamic_world.entities.get(0).unwrap().entity)
                    .insert(entity);
                dynamic_world.write_to_world(world, &mut map)
            })
        });

        commands.entity(entity).remove::<ConfigSkin>();
    }
}
