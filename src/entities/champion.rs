use bevy::prelude::*;

use crate::{
    combat::{Bounding, MoveSpeed, MoveVelocity, Navigator, Obstacle, Team},
    config::*,
};

#[derive(Component, Default)]
#[require(Transform, Navigator, MoveSpeed = MoveSpeed(365.0), MoveVelocity, Team, Bounding = Bounding { radius: COMMON_HERO_RADIUS, sides: COMMON_HERO_SIDES, height: COMMON_HERO_HEIGHT }, Obstacle)]
pub struct Champion {}
