use bevy::prelude::*;

pub struct PluginMinion;

impl Plugin for PluginMinion {
    fn build(&self, _app: &mut App) {
        // app.add_systems(Update, insert_minion_model);
    }
}

// pub fn insert_minion_model(
//     mut commands: Commands,
//     mut gltf_assets: ResMut<GltfAssets>,
//     mut q_minion: Query<(Entity, &Team, &Minion), Added<Minion>>,
// ) {
//     for (entity, team, minion) in q_minion.iter_mut() {
//         let asset_type = match team {
//             Team::Blue => match minion {
//                 Minion::Melee => GltfAsset::MinionMeleeBlue,
//                 Minion::Ranged => GltfAsset::MinionRangedBlue,
//                 Minion::Siege => GltfAsset::MinionSiegeBlue,
//             },
//             Team::Red => match minion {
//                 Minion::Melee => GltfAsset::MinionMeleeRed,
//                 Minion::Ranged => GltfAsset::MinionRangedRed,
//                 Minion::Siege => GltfAsset::MinionSiegeRed,
//             },
//         };
//         gltf_assets.insert_scene_handle(entity, asset_type, &mut commands);
//     }
// }
