use bevy::prelude::*;

use crate::entities::champion::Champion;

#[derive(Component)]
#[require(Champion)]
pub struct Fiora;
