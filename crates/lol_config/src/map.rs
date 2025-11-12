use std::collections::HashMap;

use bevy::{math::bounding::Aabb3d, prelude::*};
use league_file::LeagueMapGeoMesh;
use serde::{Deserialize, Serialize};

use league_core::{
    BarracksConfig, Unk0x9d9f60d2, Unk0xad65d8c4, Unk0xba138ae3,
    VfxSystemDefinitionData,
};
use lol_core::Lane;

#[derive(Resource, Default, Serialize, Deserialize)]
pub struct ConfigMap {
    pub geometry_objects: Vec<ConfigGeometryObject>,
    pub minion_paths: HashMap<Lane, Vec<Vec2>>,
    pub barracks: HashMap<u32, Unk0xba138ae3>,
    pub characters: HashMap<u32, Unk0x9d9f60d2>,
    pub barrack_configs: HashMap<u32, BarracksConfig>,
    pub environment_objects: HashMap<u32, Unk0xad65d8c4>,
    pub vfx_system_definition_datas: HashMap<u32, VfxSystemDefinitionData>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ConfigGeometryObject {
    pub mesh_path: String,
    pub material_path: String,
    pub bounding_box: Aabb3d,
    pub geo_mesh: LeagueMapGeoMesh,
}
