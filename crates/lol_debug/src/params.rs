//! Debug 面命令参数类型，仅在 debug 构建下编译。

#![cfg(debug_assertions)]

use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize, Debug, Clone)]
pub struct SwitchChampionParams {
    pub name: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GodModeParams {
    pub enabled: bool,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ToggleCooldownParams {
    pub enabled: bool,
}

#[derive(Debug, Clone, Default)]
pub struct ResetPositionParams;

impl<'de> Deserialize<'de> for ResetPositionParams {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let _ = Value::deserialize(deserializer)?;
        Ok(ResetPositionParams)
    }
}

#[derive(Debug, Clone, Default)]
pub struct TogglePauseParams;

impl<'de> Deserialize<'de> for TogglePauseParams {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let _ = Value::deserialize(deserializer)?;
        Ok(TogglePauseParams)
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct SetSpeedParams {
    pub speed: f32,
}

#[derive(Debug, Clone, Default)]
pub struct GetStateParams;

impl<'de> Deserialize<'de> for GetStateParams {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let _ = Value::deserialize(deserializer)?;
        Ok(GetStateParams)
    }
}

#[derive(Debug, Clone, Default)]
pub struct GetTimeParams;

impl<'de> Deserialize<'de> for GetTimeParams {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let _ = Value::deserialize(deserializer)?;
        Ok(GetTimeParams)
    }
}
