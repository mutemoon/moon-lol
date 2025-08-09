use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub enum Lane {
    Top,
    Mid,
    Bot,
}
