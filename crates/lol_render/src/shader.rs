use bevy::prelude::*;
use league_utils::get_shader_handle_by_hash;

use crate::loaders::shader::ShaderMapAsset;

#[derive(Resource, Default)]
pub struct ResourceShaderMapHandle(pub Handle<ShaderMapAsset>);

pub fn startup_load_shaders(
    asset_server: Res<AssetServer>,
    mut res_shader_map_handle: ResMut<ResourceShaderMapHandle>,
) {
    res_shader_map_handle.0 = asset_server.load("shaders/map.ron");
}

pub fn update_shaders(
    mut commands: Commands,
    res_shader_map_handle: Option<Res<ResourceShaderMapHandle>>,
    mut res_assets_shader_map: ResMut<Assets<ShaderMapAsset>>,
    mut res_assets_shader: ResMut<Assets<Shader>>,
) {
    let Some(handle) = res_shader_map_handle.as_ref() else {
        return;
    };
    let Some(shader_map) = res_assets_shader_map.remove(handle.0.id()) else {
        return;
    };

    for (shader_type, inner_map) in shader_map.entries {
        for (u64_hash, shader) in inner_map {
            let _ = res_assets_shader.insert(
                get_shader_handle_by_hash(shader_type, u64_hash).id(),
                shader,
            );
        }
    }

    commands.remove_resource::<ResourceShaderMapHandle>();
}
