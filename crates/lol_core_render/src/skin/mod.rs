use bevy::prelude::*;

#[derive(Asset, TypePath)]
pub struct LeagueSkinMesh {
    pub submeshes: Vec<Handle<Mesh>>,
}

pub mod mesh_shadow;
