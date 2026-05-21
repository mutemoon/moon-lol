use bevy::animation::graph::AnimationGraphHandle;
use bevy::asset::RecursiveDependencyLoadState;
use bevy::ecs::entity::EntityHashMap;
use bevy::prelude::*;
use bevy::world_serialization::{WorldInstanceReady, WorldInstanceSpawnError};
use lol_base::animation::AnimationConfigOf;
use lol_base::character::{ConfigSkin, Skin};

pub fn update_skin_scale(mut query: Query<(&Skin, &mut Transform)>) {
    for (skin, mut transform) in query.iter_mut() {
        transform.scale = Vec3::splat(skin.scale);
    }
}

pub fn try_load_config_skin_characters(
    mut commands: Commands,
    skin_query: Query<(Entity, &ConfigSkin)>,
    _dynamic_worlds: Res<Assets<DynamicWorld>>,
    asset_server: Res<AssetServer>,
) {
    let skin_len = skin_query.iter().len();
    if skin_len == 0 {
        return;
    }

    let mut loaded_count = 0;

    // 处理 ConfigSkin - 写入皮肤数据到世界
    for (entity, config) in &skin_query {
        if matches!(
            asset_server.get_recursive_dependency_load_state(&config.skin),
            Some(RecursiveDependencyLoadState::Loaded)
        ) {
            // info!("Character config loaded: {:?}", config.character_record);
        } else {
            // info!("Character config not loaded: {:?}", config.character_record);
            continue;
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

        commands
            .entity(entity)
            .remove::<ConfigSkin>()
            .observe(migrate_animation_graph_handle);

        loaded_count += 1;
    }

    if loaded_count > 0 {
        if skin_len - loaded_count > 0 {
            info!(
                "加载 {} 个皮肤，还剩 {} 个皮肤",
                loaded_count,
                skin_len - loaded_count
            );
        } else {
            debug!("加载 {} 个皮肤", loaded_count);
        }
    }
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct BoneRoot;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct SkinReady;

pub fn migrate_animation_graph_handle(
    trigger: On<WorldInstanceReady>,
    q_character: Query<&AnimationGraphHandle>,
    q_children: Query<&Children>,
    q_bone: Query<&AnimationPlayer>,
    mut commands: Commands,
) {
    let root_entity = trigger.event_target();
    let graph_handle = q_character.get(root_entity).unwrap();
    for descendant in q_children.iter_descendants(root_entity) {
        if q_bone.contains(descendant) {
            debug!(
                "角色实体 {:?} 加载动画 {:?} 骨骼数量 {}",
                root_entity,
                descendant,
                q_bone.count()
            );
            commands
                .entity(descendant)
                .insert(graph_handle.clone())
                .insert(BoneRoot)
                .insert(AnimationConfigOf(root_entity));
            commands.entity(root_entity).insert(SkinReady);
        }
    }
}
