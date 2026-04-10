use bevy::math::bounding::Aabb3d;
use bevy::prelude::*;

pub struct ConfigMapGeoSubmesh {
    pub mesh_path: String,
    pub material_name: String,
    pub aabb: Aabb3d,
}

pub struct ConfigMapGeoFile {
    pub mesh_path: String,
    pub submeshes: Vec<ConfigMapGeoFileSubmesh>,
}
pub struct ConfigMapGeoFileSubmesh {
    pub material_name: String,
    pub aabb: Aabb3d,
}

#[derive(Asset, TypePath)]
pub struct ConfigMapGeo {
    pub submeshes: Vec<(Handle<Mesh>, String, Aabb3d)>,
}
