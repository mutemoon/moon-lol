use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub enum Lane {
    Top = 2,
    Mid = 1,
    Bot = 0,
}

impl From<u16> for Lane {
    fn from(value: u16) -> Self {
        match value {
            0 => Lane::Bot,
            1 => Lane::Mid,
            2 => Lane::Top,
            _ => panic!("Unknown lane value: {}", value),
        }
    }
}

impl From<Option<u16>> for Lane {
    fn from(value: Option<u16>) -> Self {
        match value {
            Some(value) => From::from(value),
            None => Lane::Bot,
        }
    }
}
