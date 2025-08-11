use std::collections::HashMap;

use bevy::{
    ecs::resource::Resource,
    math::{Mat4, Vec2},
    transform::components::Transform,
};
use serde::{Deserialize, Serialize};

use crate::{
    core::{Health, Lane, Team},
    entities::Barrack,
};

#[derive(Resource, Default, Serialize, Deserialize)]
pub struct Configs {
    pub geometry_objects: Vec<ConfigGeometryObject>,
    pub environment_objects: Vec<(Transform, ConfigEnvironmentObject, Option<Health>)>,
    pub minion_paths: HashMap<Lane, Vec<Vec2>>,
    pub barracks: Vec<(Transform, Team, Lane, Barrack)>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ConfigGeometryObject {
    pub mesh_path: String,
    pub material_path: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ConfigEnvironmentObject {
    pub texture_path: String,
    pub submesh_paths: Vec<String>,
    pub joint_influences_indices: Vec<i16>,
    pub joints: Vec<ConfigJoint>,
    pub animation_graph: ConfigAnimationGraph,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ConfigAnimationGraph {
    pub clip_paths: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ConfigJoint {
    pub name: String,
    pub transform: Transform,
    pub inverse_bind_pose: Mat4,
    pub parent_index: i16,
}
