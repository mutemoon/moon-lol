use bevy::prelude::*;

use crate::combat::{MoveSpeed, MoveVelocity, Navigator, Obstacle, Team};

#[derive(Component, Default)]
#[require(Transform, Navigator, MoveSpeed = MoveSpeed(365.0), MoveVelocity, Team, Obstacle)]
pub struct Champion {}
