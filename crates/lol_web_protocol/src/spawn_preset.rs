//! 出生点预设 wire DTO + 共享枚举 Team / Visibility。

use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── 共享枚举 ──

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Team {
    Order,
    Chaos,
}

impl Team {
    pub fn as_str(&self) -> &'static str {
        match self {
            Team::Order => "order",
            Team::Chaos => "chaos",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "order" => Some(Team::Order),
            "chaos" => Some(Team::Chaos),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Visibility {
    Private,
    Friends,
    Public,
}

impl Visibility {
    pub fn as_str(&self) -> &'static str {
        match self {
            Visibility::Private => "private",
            Visibility::Friends => "friends",
            Visibility::Public => "public",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "private" => Some(Visibility::Private),
            "friends" => Some(Visibility::Friends),
            "public" => Some(Visibility::Public),
            _ => None,
        }
    }
}

// ── 出生点预设 DTO ──

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SpawnPreset {
    pub id: Uuid,
    pub owner_id: i32,
    pub name: String,
    pub x: f32,
    pub z: f32,
    pub team: Team,
    pub visibility: Visibility,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSpawnPresetDto {
    pub name: String,
    pub x: f32,
    pub z: f32,
    pub team: Team,
    #[serde(default = "default_visibility")]
    pub visibility: Visibility,
}

fn default_visibility() -> Visibility {
    Visibility::Private
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateSpawnPresetDto {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub x: Option<f32>,
    #[serde(default)]
    pub z: Option<f32>,
    #[serde(default)]
    pub team: Option<Team>,
    #[serde(default)]
    pub visibility: Option<Visibility>,
}

// ── roundtrip 单测 ──

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn team_serializes_lowercase() {
        assert_eq!(serde_json::to_string(&Team::Order).unwrap(), r#""order""#);
        assert_eq!(serde_json::to_string(&Team::Chaos).unwrap(), r#""chaos""#);
    }

    #[test]
    fn team_roundtrip() {
        let cases = ["order", "chaos"];
        for s in cases {
            let t: Team = serde_json::from_str(&format!(r#""{s}""#)).unwrap();
            assert_eq!(serde_json::to_string(&t).unwrap(), format!(r#""{s}""#));
        }
    }

    #[test]
    fn visibility_serializes_lowercase() {
        assert_eq!(
            serde_json::to_string(&Visibility::Private).unwrap(),
            r#""private""#
        );
        assert_eq!(
            serde_json::to_string(&Visibility::Friends).unwrap(),
            r#""friends""#
        );
        assert_eq!(
            serde_json::to_string(&Visibility::Public).unwrap(),
            r#""public""#
        );
    }

    #[test]
    fn visibility_roundtrip() {
        let cases = ["private", "friends", "public"];
        for s in cases {
            let v: Visibility = serde_json::from_str(&format!(r#""{s}""#)).unwrap();
            assert_eq!(serde_json::to_string(&v).unwrap(), format!(r#""{s}""#));
        }
    }
}
