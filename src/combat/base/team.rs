use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub enum Team {
    #[default]
    Order,
    Chaos,
    Neutral,
}
