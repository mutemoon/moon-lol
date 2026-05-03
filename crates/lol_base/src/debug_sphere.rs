use bevy::prelude::*;

/// Debug sphere component for visualizing positions during development.
#[derive(Component)]
pub struct DebugSphere {
    pub radius: f32,
    pub color: Color,
}
