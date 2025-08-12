use bevy::prelude::*;

use crate::core::Team;

#[derive(Component, Default)]
#[require(Transform, Team)]
pub struct Champion {}
