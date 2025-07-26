use crate::combat::Team;
use crate::entities::{Inhibitor, Nexus, Turret};
use bevy::prelude::*;

pub struct PluginNexus;

impl Plugin for PluginNexus {
    fn build(&self, app: &mut App) {
        // app.add_systems(
        //     Update,
        //     (
        //         insert_nexus_model,
        //         insert_turret_model,
        //         insert_inhibitor_model,
        //     ),
        // );
    }
}

// pub fn insert_nexus_model(
//     mut commands: Commands,
//     mut gltf_assets: ResMut<GltfAssets>,
//     mut q_nexus: Query<(Entity, &Team), Added<Nexus>>,
// ) {
//     for (entity, team) in q_nexus.iter_mut() {
//         let asset_type = match team {
//             Team::Blue => GltfAsset::NexusBlue,
//             Team::Red => GltfAsset::NexusRed,
//         };
//         gltf_assets.insert_scene_handle(entity, asset_type, &mut commands);
//     }
// }

// pub fn insert_turret_model(
//     mut commands: Commands,
//     mut gltf_assets: ResMut<GltfAssets>,
//     mut q_turret: Query<(Entity, &Team), Added<Turret>>,
// ) {
//     for (entity, team) in q_turret.iter_mut() {
//         let asset_type = match team {
//             Team::Blue => GltfAsset::TurretBlue,
//             Team::Red => GltfAsset::TurretRed,
//         };
//         gltf_assets.insert_scene_handle(entity, asset_type, &mut commands);
//     }
// }

// pub fn insert_inhibitor_model(
//     mut commands: Commands,
//     mut gltf_assets: ResMut<GltfAssets>,
//     mut q_inhibitor: Query<(Entity, &Team), Added<Inhibitor>>,
// ) {
//     for (entity, team) in q_inhibitor.iter_mut() {
//         let asset_type = match team {
//             Team::Blue => GltfAsset::InhibitorBlue,
//             Team::Red => GltfAsset::InhibitorRed,
//         };
//         gltf_assets.insert_scene_handle(entity, asset_type, &mut commands);
//     }
// }
