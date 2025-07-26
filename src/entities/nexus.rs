use bevy::prelude::*;

use crate::{
    combat::{Bounding, Obstacle, Team},
    config::*,
};

#[derive(Component)]
#[require(Transform, Team, Bounding = Bounding { radius: NEXUS_RADIUS, sides: NEXUS_SIDES, height: NEXUS_HEIGHT }, Obstacle)]
pub struct Nexus;
