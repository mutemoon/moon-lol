use bevy::prelude::*;

use crate::combat::Obstacle;

#[derive(Component)]
#[require(Transform, Obstacle)]
pub struct Turret;
