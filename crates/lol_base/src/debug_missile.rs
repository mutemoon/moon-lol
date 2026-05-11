use bevy::prelude::*;

/// Debug component for linear missile visualization.
/// Renders as a rectangular box showing the missile's collision width.
#[derive(Component)]
pub struct DebugMissile {
    pub width: f32,
    pub length: f32,
    pub color: Color,
}
