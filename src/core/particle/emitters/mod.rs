mod distortion;
mod mesh;
mod quad;
mod quad_slice;

use bevy::mesh::skinning::{SkinnedMesh, SkinnedMeshInverseBindposes};
use bevy::mesh::VertexAttributeValues;
use bevy::prelude::*;
use league_core::{EnumVfxPrimitive, VfxSystemDefinitionData};

use crate::{
    CameraState, Lifetime, ParticleEmitterState, ParticleId, ParticleMaterialSkinnedMeshParticle,
    ParticleMaterialUnlitDecal, ATTRIBUTE_LIFETIME, ATTRIBUTE_WORLD_POSITION,
    ATTRIBUTE_WORLD_POSITION_VEC4,
};
