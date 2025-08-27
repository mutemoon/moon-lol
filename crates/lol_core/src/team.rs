use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub enum Team {
    #[default]
    Order = 100,
    Chaos = 200,
    Neutral = 300,
}

impl From<u32> for Team {
    fn from(value: u32) -> Self {
        match value {
            100 => Team::Order,
            200 => Team::Chaos,
            300 => Team::Neutral,
            _ => panic!("Unknown team value: {}", value),
        }
    }
}

impl From<Option<u32>> for Team {
    fn from(value: Option<u32>) -> Self {
        match value {
            Some(value) => From::from(value),
            None => Team::default(),
        }
    }
}
