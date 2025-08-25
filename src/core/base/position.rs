use bevy::prelude::*;
use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer, Serialize,
};

#[derive(Component, Default, Debug, PartialEq, Eq, Hash, Clone, Serialize)]
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

struct PositionVisitor;

impl<'de> Visitor<'de> for PositionVisitor {
    type Value = Position;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a u16 representing a Position variant")
    }

    fn visit_u16<E>(self, value: u16) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(Position::from(value))
    }
}

impl<'de> Deserialize<'de> for Position {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_u16(PositionVisitor)
    }
}
