use bevy::prelude::*;

use crate::combat::{Obstacle, Team};

#[derive(Component)]
#[require(Transform, Team, Obstacle)]
pub struct Inhibitor;
