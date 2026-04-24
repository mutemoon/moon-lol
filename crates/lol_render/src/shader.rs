use std::collections::HashSet;

use bevy::prelude::*;
use league_utils::{get_shader_handle_by_hash, hash_wad};
use lol_base::shader::ResourceShaderPackage;
use serde::{Deserialize, Serialize};

use crate::particle::environment::unlit_decal::ParticleMaterialUnlitDecal;
use crate::particle::particle::distortion::ParticleMaterialDistortion;
use crate::particle::particle::quad::ParticleMaterialQuad;
use crate::particle::particle::quad_slice::ParticleMaterialQuadSlice;
use crate::particle::utils::MaterialPath;

#[derive(Resource, Default)]
pub struct ResourceShaderHandles(pub Vec<(String, Handle<ResourceShaderPackage>)>);

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct ShaderTocSettings(pub String);

pub trait AssetServerLoadShaderToc {
    fn load_shader_toc<'a>(&self, path: &str) -> Handle<ResourceShaderPackage>;
}

impl AssetServerLoadShaderToc for AssetServer {
    fn load_shader_toc<'a>(&self, path: &str) -> Handle<ResourceShaderPackage> {
        let original_path = path.to_string();
        self.load_builder()
            .with_settings(move |settings: &mut ShaderTocSettings| {
                settings.0 = original_path.clone()
            })
            .load(format!("data/{:x}.lol", hash_wad(path)))
    }
}

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
        ParticleMaterialDistortion::FRAG_PATH,
        ParticleMaterialDistortion::VERT_PATH,
    ]);

    for path in paths {
        let handle = asset_server.load_shader_toc(path);
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

        false
    });
}
