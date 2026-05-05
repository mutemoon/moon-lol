use bevy::prelude::*;

/// Debug ground disc component for visualizing circular areas during development.
/// Renders as a flat semi-transparent disc on the ground plane.
/// The visual size is controlled by the entity's Transform.scale (unit mesh + scale).
#[derive(Component)]
pub struct DebugArea {
    pub color: Color,
}
