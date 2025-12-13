use bevy::math::bounding::Aabb3d;
use bevy::prelude::*;

#[derive(Asset, TypePath)]
pub struct ConfigMapGeo {
    pub submeshes: Vec<(Handle<Mesh>, String, Aabb3d)>,
}
