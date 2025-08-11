use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Clone, Serialize, Deserialize)]
pub struct Bounding {
    pub radius: f32,
    pub sides: u32,
    pub height: f32,
}
