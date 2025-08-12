use bevy::prelude::*;

pub struct PluginAnimation;

impl Plugin for PluginAnimation {
    fn build(&self, _app: &mut App) {
        // app.add_systems(
        //     Update,
        //     (
        //         init_animation,
        //         animation_move,
        //         animation_idle,
        //         animation_attack,
        //         animation_play,
        //     ),
        // );
    }
}

// fn init_animation(
//     children: Query<&Children>,
//     gltfs: Res<Assets<Gltf>>,
//     mut commands: Commands,
//     mut gltf_assets: ResMut<GltfAssets>,
//     mut graphs: ResMut<Assets<AnimationGraph>>,
//     q_animation_handle: Query<(Entity, &GltfAnimationHandle)>,
// ) {
//     for (entity, gltf_animation_handle) in &q_animation_handle {
//         for child in children.iter_descendants(entity) {
//             gltf_assets.init_animation(
//                 child,
//                 gltf_animation_handle.0,
//                 &mut commands,
//                 entity,
//                 &gltfs,
//                 &mut graphs,
//             );
//         }
//     }
// }

// fn animation_move(
//     mut commands: Commands,
//     q_move_state: Query<Entity, (With<MoveDestination>, With<GltfAsset>)>,
// ) {
//     for entity in q_move_state.iter() {
//         if let Ok(mut entity_commands) = commands.get_entity(entity) {
//             entity_commands.insert(AnimationInfo {
//                 animation_type: AnimationType::Moving,
//             });
//         }
//     }
// }

// fn animation_idle(
//     mut commands: Commands,
//     q_move_state: Query<Entity, (Without<MoveDestination>, With<GltfAsset>)>,
// ) {
//     for entity in q_move_state.iter() {
//         if let Ok(mut entity_commands) = commands.get_entity(entity) {
//             entity_commands.insert(AnimationInfo {
//                 animation_type: AnimationType::Idle,
//             });
//         }
//     }
// }

// fn animation_attack(
//     mut commands: Commands,
//     q_attack_state: Query<(Entity, &AttackState), (Changed<AttackState>, With<GltfAsset>)>,
// ) {
//     for (entity, attack_state) in q_attack_state.iter() {
//         if attack_state.is_locked() || attack_state.is_attacking() {
//             if let Ok(mut entity_commands) = commands.get_entity(entity) {
//                 entity_commands.insert(AnimationInfo {
//                     animation_type: AnimationType::Attack,
//                 });
//             }
//         }
//     }
// }

// fn animation_play(
//     gltf_assets: Res<GltfAssets>,
//     q_animation_info: Query<(Entity, &GltfAsset, &AnimationInfo), Changed<AnimationInfo>>,
//     q_children: Query<&Children>,
//     mut q_animation: Query<(&mut AnimationTransitions, &mut AnimationPlayer)>,
// ) {
//     for (entity, asset_type, animation_info) in q_animation_info.iter() {
//         for child in q_children.iter_descendants(entity) {
//             let Ok((mut animation_transitions, mut animation_player)) = q_animation.get_mut(child)
//             else {
//                 continue;
//             };
//             animation_info.play(
//                 &gltf_assets,
//                 asset_type,
//                 &animation_info.animation_type,
//                 &mut animation_transitions,
//                 &mut animation_player,
//             );
//         }
//     }
// }
