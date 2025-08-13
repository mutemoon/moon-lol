use std::collections::HashMap;

use bevy::{
    ecs::resource::Resource,
    math::{Mat4, Vec2, Vec3},
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
    pub navigation_grid: ConfigNavigationGrid,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ConfigGeometryObject {
    pub mesh_path: String,
    pub material_path: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ConfigEnvironmentObject {
    pub animation_graph_path: String,
    pub inverse_bind_pose_path: String,
    pub joint_influences_indices: Vec<i16>,
    pub joints: Vec<ConfigJoint>,
    pub material_path: String,
    pub skin_scale: Option<f32>,
    pub submesh_paths: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ConfigAnimationGraph {
    pub clip_paths: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ConfigJoint {
    pub hash: u32,
    pub transform: Transform,
    pub parent_index: i16,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ConfigSkinnedMeshInverseBindposes {
    pub inverse_bindposes: Vec<Mat4>,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct ConfigNavigationGrid {
    pub min_grid_pos: Vec3,
    pub cell_size: f32,
    pub x_len: usize,
    pub y_len: usize,
    pub cells: Vec<Vec<ConfigNavigationGridCell>>,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct ConfigNavigationGridCell {
    pub y: f32,
    pub heuristic: f32,
    pub flags: u16,
}

impl ConfigNavigationGrid {
    pub fn get_cell_pos(&self, x: usize, y: usize) -> Vec3 {
        Vec3::new(
            self.min_grid_pos.x + x as f32 * self.cell_size,
            self.cells[x][y].y,
            -(self.min_grid_pos.z + y as f32 * self.cell_size),
        )
    }

    pub fn get_cell_by_pos(&self, pos: Vec3) -> &ConfigNavigationGridCell {
        let x = ((pos.x - self.min_grid_pos.x) / self.cell_size).round() as usize;
        let y = ((-pos.z - self.min_grid_pos.z) / self.cell_size).round() as usize;

        &self.cells[x.clamp(0, self.x_len - 1)][y.clamp(0, self.y_len - 1)]
    }

    pub fn get_center_pos(&self) -> Vec3 {
        Vec3::new(
            self.min_grid_pos.x + self.cell_size * self.x_len as f32 / 2.0,
            0.0,
            -(self.min_grid_pos.z + self.cell_size * self.y_len as f32 / 2.0),
        )
    }
}
