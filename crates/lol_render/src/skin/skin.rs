use bevy::ecs::entity::EntityHashMap;
use bevy::prelude::*;
use bevy::world_serialization::WorldInstanceSpawnError;
use league_core::extract::SkinCharacterDataProperties;
use lol_base::character::ConfigSkin;
use lol_base::prop::{HashKey, LoadHashKeyTrait};
use lol_core::render_cmd::CommandSkinSpawn;
use lol_core::resource::loading::Loading;

// use lol_core::resource::prop_bin::{CommandLoadPropBin, PropPath};
use crate::skin::animation::CommandSkinAnimationSpawn;
use crate::skin::mesh::CommandSkinMeshSpawn;
use crate::ui::health_bar::HealthBar;

#[derive(Component, Debug, Clone, Copy)]
pub struct Skin {
    pub key: HashKey<SkinCharacterDataProperties>,
    pub scale: f32,
}

pub fn update_skin_scale(mut query: Query<(&Skin, &mut Transform)>) {
    for (skin, mut transform) in query.iter_mut() {
        transform.scale = Vec3::splat(skin.scale);
    }
}

pub fn on_command_skin_spawn(trigger: On<CommandSkinSpawn>, mut commands: Commands) {
    let entity = trigger.event_target();

    // let paths = vec![format!("data/{0}.bin", trigger.key)];
    //
    // commands.trigger(CommandLoadPropBin {
    //     path: PropPath::Path(paths),
    //     label: None,
    // });

    commands
        .entity(entity)
        .insert(Loading::new(HashKey::<SkinCharacterDataProperties>::from(
            &trigger.key,
        )));
}

pub fn update_skin_spawn(
    mut commands: Commands,
    res_assets_skin_character_data_properties: Res<Assets<SkinCharacterDataProperties>>,
    q_loading: Query<(Entity, &Loading<HashKey<SkinCharacterDataProperties>>)>,
) {
    for (entity, loading) in q_loading.iter() {
        let Some(skin_character_data_properties) =
            res_assets_skin_character_data_properties.load_hash(loading.value)
        else {
            continue;
        };

        let scale = skin_character_data_properties
            .skin_mesh_properties
            .as_ref()
            .unwrap()
            .skin_scale
            .unwrap_or(1.0);

        commands.entity(entity).insert((
            Visibility::default(),
            Skin {
                key: loading.value,
                scale,
            },
        ));

        commands.trigger(CommandSkinMeshSpawn { entity });

        commands.trigger(CommandSkinAnimationSpawn { entity });

        if let Some(bar_data) = &skin_character_data_properties.health_bar_data {
            commands.entity(entity).insert(HealthBar {
                bar_type: bar_data.unit_health_bar_style.unwrap(),
            });
        }

        commands
            .entity(entity)
            .remove::<Loading<HashKey<SkinCharacterDataProperties>>>();
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

    info!("加载 {} 个皮肤", skin_len);

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
