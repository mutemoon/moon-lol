// 权威协议类型统一在 Bevy-free 的 lol_client 中定义，此处 re-export 供服务端使用。
pub use lol_client::protocol::{WsEvent, WsRequest, WsResponse};
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
