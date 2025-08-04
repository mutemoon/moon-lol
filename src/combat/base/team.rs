use bevy::prelude::*;

#[derive(Component, Debug, Default, PartialEq)]
pub enum Team {
    #[default]
    Order,
    Chaos,
    Neutral,
}
