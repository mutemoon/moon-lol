use bevy::prelude::*;
use league_core::SkinCharacterDataProperties;
use lol_config::{HashKey, LeagueProperties};

use crate::{CommandLoadPropBin, CommandSkinAnimationSpawn, CommandSkinMeshSpawn, Loading};

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

#[derive(TypePath)]
pub struct SkinSpawn(pub HashKey<SkinCharacterDataProperties>);

pub fn update_skin_scale(mut query: Query<(&Skin, &mut Transform)>) {
    for (skin, mut transform) in query.iter_mut() {
        transform.scale = Vec3::splat(skin.scale);
    }
}

pub fn on_command_skin_spawn(trigger: On<CommandSkinSpawn>, mut commands: Commands) {
    let entity = trigger.event_target();

    let key = (&trigger.key).into();

    let paths = vec![format!("data/{0}.bin", trigger.key)];

    commands.trigger(CommandLoadPropBin { paths });

    commands.entity(entity).insert(Loading::new(SkinSpawn(key)));
}

pub fn update_skin_spawn(
    mut commands: Commands,
    res_assets_skin_character_data_properties: Res<Assets<SkinCharacterDataProperties>>,
    res_league_properties: Res<LeagueProperties>,
    q_loading: Query<(Entity, &Loading<SkinSpawn>)>,
) {
    for (entity, loading) in q_loading.iter() {
        let Some(skin_character_data_properties) =
            res_league_properties.get(&res_assets_skin_character_data_properties, loading.0)
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
                key: loading.0,
                scale,
            },
        ));

        commands.trigger(CommandSkinMeshSpawn { entity });

        commands.trigger(CommandSkinAnimationSpawn { entity });

        commands.entity(entity).remove::<Loading<SkinSpawn>>();
    }
}
