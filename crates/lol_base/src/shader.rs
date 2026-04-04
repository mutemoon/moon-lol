use std::collections::HashMap;

use bevy::prelude::*;

#[derive(Asset, TypePath)]
pub struct ResourceShaderPackage {
    pub handles: HashMap<u64, Handle<Shader>>,
}

#[derive(Asset, TypePath)]
pub struct ResourceShaderChunk {}
