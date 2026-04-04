use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Default, Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub enum Position {
    #[default]
    Outer = 0,
    Inhibitor = 1,
    Inner = 2,
    RightNexusTurretOrNexus = 4,
    LeftNexusTurret = 5,
}

impl From<u16> for Position {
    fn from(value: u16) -> Self {
        match value {
            0 => Position::Outer,
            1 => Position::Inhibitor,
            2 => Position::Inner,
            4 => Position::RightNexusTurretOrNexus,
            5 => Position::LeftNexusTurret,
            _ => panic!("Unknown position value: {}", value),
        }
    }
}

impl From<Option<u16>> for Position {
    fn from(value: Option<u16>) -> Self {
        match value {
            Some(value) => From::from(value),
            None => Position::Outer,
        }
    }
}
