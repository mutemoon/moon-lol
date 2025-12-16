use bevy::mesh::skinning::SkinnedMesh;
use bevy::prelude::*;
use bevy::render::render_resource::Face;
use league_core::SkinCharacterDataProperties;
use lol_config::LoadHashKeyTrait;
use lol_core::LeagueSkinMesh;

use crate::{AssetServerLoadLeague, CommandSkinSkeletonSpawn, Loading, Skin};

#[derive(EntityEvent)]
pub struct CommandSkinMeshSpawn {
    pub entity: Entity,
}

#[derive(TypePath)]
pub struct SkinMeshSpawn(pub Handle<LeagueSkinMesh>, pub Handle<StandardMaterial>);

pub fn on_command_skin_mesh_spawn(
    trigger: On<CommandSkinMeshSpawn>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut res_assets_standard_material: ResMut<Assets<StandardMaterial>>,
    q_skin: Query<&Skin>,
    res_assets_skin_character_data_properties: Res<Assets<SkinCharacterDataProperties>>,
) {
    let entity = trigger.event_target();

    let skin = q_skin.get(entity).unwrap();

    let skin_character_data_properties = res_assets_skin_character_data_properties
        .load_hash(skin.key)
        .unwrap();

    let skin_mesh_properties = skin_character_data_properties
        .skin_mesh_properties
        .as_ref()
        .unwrap();

    commands.trigger(CommandSkinSkeletonSpawn {
        entity,
        path: skin_mesh_properties.skeleton.clone().unwrap(),
    });

    commands.entity(entity).insert(Loading::new(SkinMeshSpawn(
        asset_server.load_league(&skin_mesh_properties.simple_skin.clone().unwrap()),
        get_standard(
            &mut res_assets_standard_material,
            &asset_server,
            skin_mesh_properties.texture.clone(),
        ),
    )));
}

pub fn update_skin_mesh_spawn(
    mut commands: Commands,
    res_assets_league_skinned_mesh: Res<Assets<LeagueSkinMesh>>,
    q_loading_mesh: Query<(Entity, &Loading<SkinMeshSpawn>, Option<&SkinnedMesh>)>,
) {
    for (entity, loading, skinned_mesh) in q_loading_mesh.iter() {
        let Some(league_skinned_mesh) = res_assets_league_skinned_mesh.get(&loading.0) else {
            continue;
        };

        let Some(skinned_mesh) = skinned_mesh else {
            continue;
        };

        for mesh in league_skinned_mesh.submeshes.iter() {
            commands.entity(entity).with_child((
                Transform::default(),
                Mesh3d(mesh.clone()),
                MeshMaterial3d(loading.1.clone()),
                skinned_mesh.clone(),
            ));
        }

        commands.entity(entity).remove::<Loading<SkinMeshSpawn>>();
    }
}

pub fn get_standard(
    res_assets_standard_material: &mut ResMut<Assets<StandardMaterial>>,
    asset_server: &Res<AssetServer>,
    texture: Option<String>,
) -> Handle<StandardMaterial> {
    let material_handle = res_assets_standard_material.add(StandardMaterial {
        base_color_texture: texture.map(|v| asset_server.load_league_labeled(&v, "srgb")),
        unlit: true,
        cull_mode: Some(Face::Front),
        alpha_mode: AlphaMode::Mask(0.3),
        ..default()
    });

    material_handle
}
