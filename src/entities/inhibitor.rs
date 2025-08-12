use bevy::prelude::*;

use crate::core::Team;

#[derive(Component)]
#[require(Transform, Team)]
pub struct Inhibitor;
