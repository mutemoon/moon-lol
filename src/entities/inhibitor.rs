use bevy::prelude::*;

use crate::{
    combat::{Bounding, Obstacle, Team},
    config::*,
};

#[derive(Component)]
#[require(Transform, Team, Bounding = Bounding { radius: INHIBITOR_RADIUS, sides: INHIBITOR_SIDES, height: INHIBITOR_HEIGHT }, Obstacle)]
pub struct Inhibitor;
