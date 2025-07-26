use bevy::prelude::*;

use crate::{
    combat::{Bounding, Obstacle, Team},
    config::*,
};

#[derive(Component)]
#[require(Transform, Team, Bounding = Bounding { radius: TURRET_RADIUS, sides: TURRET_SIDES, height: TURRET_HEIGHT }, Obstacle)]
pub struct Turret;
