use crate::core::Configs;
use crate::core::{AttackState, MovementDestination, Target};
use crate::league::{spawn_environment_objects_from_configs, spawn_geometry_objects_from_configs};

use bevy::color::palettes;
use bevy::prelude::*;
use bevy::render::mesh::skinning::SkinnedMeshInverseBindposes;

pub const MAP_WIDTH: f32 = 17000.0;
pub const MAP_HEIGHT: f32 = 17000.0;

pub const MAP_OFFSET_X: f32 = 500.0;
pub const MAP_OFFSET_Y: f32 = 500.0;

#[derive(Component)]
#[require(Visibility)]
pub struct Map;

pub struct PluginMap;

impl Plugin for PluginMap {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
}

fn setup(
    mut commands: Commands,
    configs: Res<Configs>,
    mut res_animation_graphs: ResMut<Assets<AnimationGraph>>,
    mut res_materials: ResMut<Assets<StandardMaterial>>,
    mut res_skinned_mesh_inverse_bindposes: ResMut<Assets<SkinnedMeshInverseBindposes>>,
    asset_server: Res<AssetServer>,
) {
    spawn_geometry_objects_from_configs(&mut commands, &asset_server, &configs);

    spawn_environment_objects_from_configs(
        &mut commands,
        &mut res_animation_graphs,
        &mut res_materials,
        &mut res_skinned_mesh_inverse_bindposes,
        &asset_server,
        &configs,
    );
}

pub fn draw_attack(
    mut gizmos: Gizmos,
    q_attack: Query<(&Transform, &AttackState)>,
    q_movement_destination: Query<(&Transform, &MovementDestination)>,
    q_target: Query<(&Transform, &Target)>,
    q_transform: Query<&Transform>,
) {
    for (transform, attack_info) in q_attack.iter() {
        let Some(target) = attack_info.target else {
            continue;
        };
        let Ok(target_transform) = q_transform.get(target) else {
            continue;
        };
        gizmos.line(
            transform.translation,
            target_transform.translation,
            Color::Srgba(palettes::tailwind::RED_500),
        );
    }

    for (transform, movement_destination) in q_movement_destination.iter() {
        let destination = movement_destination.0;

        gizmos.line(
            transform.translation,
            transform
                .translation
                .clone()
                .with_x(destination.x)
                .with_z(destination.y),
            Color::Srgba(palettes::tailwind::GREEN_500),
        );
    }

    for (transform, target) in q_target.iter() {
        let Ok(target_transform) = q_transform.get(target.0) else {
            continue;
        };
        gizmos.line(
            transform.translation,
            target_transform.translation,
            Color::Srgba(palettes::tailwind::YELLOW_500),
        );
    }
}
