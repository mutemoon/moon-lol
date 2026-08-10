use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use lol_base_render::shader::LeagueShader;

pub trait MaterialPath {
    const FRAG_SHADER: LeagueShader;
    const VERT_SHADER: LeagueShader;
}

pub fn create_black_pixel_texture() -> Image {
    let image = Image::new_fill(
        Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Rgba8Unorm,
        default(),
    );

    image
}

pub fn create_white_pixel_texture() -> Image {
    let image = Image::new_fill(
        Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[255, 255, 255, 255],
        TextureFormat::Rgba8UnormSrgb,
        default(),
    );

    image
}

use std::collections::BTreeMap;

#[derive(Resource, Default)]
pub struct ResourceCache {
    mesh: BTreeMap<String, Handle<Mesh>>,
}

impl ResourceCache {
    pub fn get_mesh(&mut self, asset_server: &AssetServer, path: &str) -> Handle<Mesh> {
        match self.mesh.get(path) {
            Some(handle) => handle.clone(),
            None => {
                let handle = asset_server.load(path.to_string());
                self.mesh.insert(path.to_string(), handle.clone());
                handle
            }
        }
    }
}

pub use lol_base_render::particle::{
    CombineMultiplicative, ProbabilityCurve, Sampler, StochasticSampler,
};
