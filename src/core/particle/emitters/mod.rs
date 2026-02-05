mod decal;
mod mesh;
mod position;
mod quad;
mod skinned_mesh;
mod state;
mod unlit_decal;
mod utils;

pub use decal::{update_decal_intersections, ParticleDecal, ParticleDecalGeometry};
pub use mesh::{attach_mesh_visuals, update_emitter_mesh};
pub use position::update_emitter_position;
pub use quad::{attach_quad_visuals, update_emitter_quad};
pub use skinned_mesh::{attach_skinned_mesh_visuals, update_emitter_skinned_mesh};
pub use state::{EmitterOf, Emitters, ParticleEmitterState};
pub use unlit_decal::{attach_unlit_decal_visuals, update_emitter_decal};
