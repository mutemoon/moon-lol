use bevy::prelude::*;

#[derive(Asset, TypePath)]
pub struct ResourceShaderPackage {}

#[derive(Resource)]
pub struct ResourceShaderHandles(pub Handle<ResourceShaderPackage>);

pub fn startup_load_shaders(
    _asset_server: Res<AssetServer>,
    _res_assets_shader: ResMut<Assets<Shader>>,
) {
    let _paths = vec![
        "assets/shaders/hlsl/environment/unlit_decal_ps.ps.glsl",
        "assets/shaders/hlsl/environment/unlit_decal_vs.vs.glsl",
    ];

    // for path in paths {
    //     let shader = asset_server.load(path);
    // }
}
