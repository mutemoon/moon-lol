//! Scenario wire DTO（场景预设：完整阵容编排）。

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Scenario {
    pub id: Uuid,
    pub owner_id: i32,
    pub name: String,
    pub agents: serde_json::Value,
    #[serde(default)]
    pub win_condition: Option<serde_json::Value>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateScenarioDto {
    pub name: String,
    pub agents: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateScenarioDto {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub agents: Option<serde_json::Value>,
}
