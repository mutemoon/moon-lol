use bevy::prelude::*;
use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer, Serialize,
};

#[derive(Component, Debug, Default, PartialEq, Clone, Serialize)]
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

struct TeamVisitor;

impl<'de> Visitor<'de> for TeamVisitor {
    type Value = Team;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a u32 representing a Team variant")
    }

    fn visit_u32<E>(self, value: u32) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Team::from(value))
    }
}

impl<'de> Deserialize<'de> for Team {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_u32(TeamVisitor)
    }
}
