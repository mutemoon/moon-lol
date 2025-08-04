use bevy::prelude::*;

#[derive(Component, Default)]
pub struct Bounding {
    pub radius: f32,
    pub sides: u32,
    pub height: f32,
}
