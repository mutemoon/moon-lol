use bevy::math::bounding::Aabb3d;
use bevy::prelude::*;

#[derive(Component)]
pub struct MapGeometry {
    pub bounding_box: Aabb3d,
}
