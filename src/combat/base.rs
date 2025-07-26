use bevy::prelude::*;

#[derive(Component, PartialEq, Default, Copy, Clone, Debug)]
pub enum Team {
    #[default]
    Blue,
    Red,
}

#[derive(Component, Default)]
pub struct Bounding {
    pub radius: f32,
    pub sides: u32,
    pub height: f32,
}
