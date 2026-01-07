use bevy::prelude::*;
use league_core::SkinCharacterDataProperties;
use lol_config::{HashKey, LoadHashKeyTrait};

use crate::{
    CommandLoadPropBin, CommandSkinAnimationSpawn, CommandSkinMeshSpawn, HealthBar, Loading,
    PropPath,
};

#[derive(Component, Debug, Clone, Copy)]
pub struct Skin {
    pub key: HashKey<SkinCharacterDataProperties>,
    pub scale: f32,
}

#[derive(EntityEvent)]
pub struct CommandSkinSpawn {
    pub entity: Entity,
    pub key: String,
}

pub fn update_skin_scale(mut query: Query<(&Skin, &mut Transform)>) {
    for (skin, mut transform) in query.iter_mut() {
        transform.scale = Vec3::splat(skin.scale);
    }
}

pub fn on_command_skin_spawn(trigger: On<CommandSkinSpawn>, mut commands: Commands) {
    let entity = trigger.event_target();

    let paths = vec![format!("data/{0}.bin", trigger.key)];

    commands.trigger(CommandLoadPropBin {
        path: PropPath::Path(paths),
        label: None,
    });

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
