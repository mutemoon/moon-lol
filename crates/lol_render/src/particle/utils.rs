use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

pub trait MaterialPath {
    const FRAG_SHADER: league_utils::LeagueShader;
    const VERT_SHADER: league_utils::LeagueShader;
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

use std::collections::HashMap;

#[derive(Resource, Default)]
pub struct ResourceCache {
    image: HashMap<String, Handle<Image>>,
    mesh: HashMap<String, Handle<Mesh>>,
}

impl ResourceCache {
    pub fn get_image(&mut self, asset_server: &AssetServer, path: &str) -> Handle<Image> {
        match self.image.get(path) {
            Some(handle) => handle.clone(),
            None => {
                let handle = asset_server.load(path.to_string());
                self.image.insert(path.to_string(), handle.clone());
                handle
            }
        }
    }

    pub fn get_image_srgb(&mut self, asset_server: &AssetServer, path: &str) -> Handle<Image> {
        match self.image.get(path) {
            Some(handle) => handle.clone(),
            None => {
                let handle = asset_server.load(path.to_string());
                self.image.insert(path.to_string(), handle.clone());
                handle
            }
        }
    }

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

pub use lol_base::particle::{CombineMultiplicative, ProbabilityCurve, Sampler, StochasticSampler};
