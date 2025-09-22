mod emitter;
mod particle;
mod ps;
mod utils;
mod vs;

pub use emitter::*;
pub use particle::*;
pub use ps::*;
pub use utils::*;
pub use vs::*;

use bevy::render::mesh::{MeshVertexAttribute, VertexFormat};

pub const ATTRIBUTE_WORLD_POSITION: MeshVertexAttribute =
    MeshVertexAttribute::new("Vertext_World_Position", 7, VertexFormat::Float32x3);

pub const ATTRIBUTE_UV_FRAME: MeshVertexAttribute =
    MeshVertexAttribute::new("Vertext_Frame", 8, VertexFormat::Float32x4);

pub const ATTRIBUTE_LIFETIME: MeshVertexAttribute =
    MeshVertexAttribute::new("Vertext_Life", 9, VertexFormat::Float32x2);
