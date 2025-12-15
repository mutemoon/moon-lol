use std::collections::HashSet;

use bevy::prelude::*;
use league_utils::hash_shader_spec;
use lol_config::ResourceShaderPackage;

use crate::{
    get_shader_handle_by_hash, MaterialPath, ParticleMaterialQuad, ParticleMaterialQuadSlice,
    ParticleMaterialUnlitDecal,
};

#[derive(Resource, Default)]
pub struct ResourceShaderHandles(pub Vec<(String, Handle<ResourceShaderPackage>)>);

pub fn startup_load_shaders(
    asset_server: Res<AssetServer>,
    mut res_resource_shader_handles: ResMut<ResourceShaderHandles>,
) {
    let paths = HashSet::from([
        ParticleMaterialQuadSlice::FRAG_PATH,
        ParticleMaterialQuadSlice::VERT_PATH,
        ParticleMaterialQuad::FRAG_PATH,
        ParticleMaterialQuad::VERT_PATH,
        ParticleMaterialUnlitDecal::FRAG_PATH,
        ParticleMaterialUnlitDecal::VERT_PATH,
    ]);

    for path in paths {
        let handle = asset_server.load::<ResourceShaderPackage>(path);
        res_resource_shader_handles
            .0
            .push((path.to_string(), handle));
    }
}

pub fn update_shaders(
    mut res_resource_shader_handles: ResMut<ResourceShaderHandles>,
    res_assets_shader_package: ResMut<Assets<ResourceShaderPackage>>,
    mut res_assets_shader: ResMut<Assets<Shader>>,
) {
    res_resource_shader_handles.0.retain(|(path, handle)| {
        let Some(shader_package) = res_assets_shader_package.get(handle) else {
            return true;
        };

        for (&u64_hash, handle) in shader_package.handles.iter() {
            let shader = res_assets_shader.get(handle).unwrap().clone();

            res_assets_shader
                .insert(get_shader_handle_by_hash(&path, u64_hash).id(), shader)
                .unwrap();
        }
        // true
        false
    });
}
