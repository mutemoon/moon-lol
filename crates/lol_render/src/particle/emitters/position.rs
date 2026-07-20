use bevy::prelude::*;
use league_core::extract::VfxSystemDefinitionData;
use lol_core::lifetime::Lifetime;

use super::state::{EmitterOf, ParticleEmitterState};
use crate::particle::ParticleId;

pub fn update_emitter_position(
    mut query: Query<(
        &mut Transform,
        &EmitterOf,
        &Lifetime,
        &ParticleEmitterState,
        &ParticleId,
    )>,
    res_assets_vfx_system_definition_data: Res<Assets<VfxSystemDefinitionData>>,
    q_global_transform: Query<&GlobalTransform>,
) {
    for (mut transform, emitter_of, lifetime, emitter, particle_id) in query.iter_mut() {
        let vfx_emitter_definition_data =
            particle_id.get_def(&res_assets_vfx_system_definition_data);

        let is_local_orientation = vfx_emitter_definition_data
            .is_local_orientation
            .unwrap_or(true);

        let progress = lifetime.progress();

        let emitter_position = emitter.emitter_position.sample_clamped(progress);

        let bind_weight = emitter.bind_weight.sample_clamped(progress);

        if bind_weight == 0. {
            continue;
        }

        let mut character_global_transform = q_global_transform
            .get(emitter_of.0)
            .unwrap()
            .compute_transform();

        if is_local_orientation {
            character_global_transform.translation += emitter_position;
            *transform = character_global_transform;
        } else {
            *transform = Transform::from_matrix(Mat4::from_scale_rotation_translation(
                character_global_transform.scale,
                Quat::default(),
                character_global_transform.translation + emitter_position,
            ));
        }
    }
}
