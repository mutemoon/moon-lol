pub use lol_client::protocol::*;
use serde::Deserialize;

// ── Command params（仅服务端解析使用）──

#[derive(Deserialize, Debug)]
pub struct SwitchChampionParams {
    pub name: String,
}

#[derive(Deserialize, Debug)]
pub struct GodModeParams {
    pub enabled: bool,
}

#[derive(Deserialize, Debug)]
pub struct ToggleCooldownParams {
    pub enabled: bool,
}

#[derive(Deserialize, Debug)]
pub struct SetSpeedParams {
    pub speed: f32,
}
