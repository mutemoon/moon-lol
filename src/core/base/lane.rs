use bevy::prelude::*;
use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer, Serialize,
};

#[derive(Component, Debug, PartialEq, Eq, Hash, Clone, Serialize)]
pub enum Lane {
    Top,
    Mid,
    Bot,
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

struct LaneVisitor;

impl<'de> Visitor<'de> for LaneVisitor {
    type Value = Lane;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a u16 representing a Lane variant")
    }

    fn visit_u16<E>(self, value: u16) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Lane::from(value))
    }
}

impl<'de> Deserialize<'de> for Lane {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_u16(LaneVisitor)
    }
}
