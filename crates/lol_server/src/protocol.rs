use serde::{Deserialize, Serialize};

// ── Incoming (panel → game) ──

#[derive(Deserialize, Debug)]
pub struct WsRequest {
    pub id: u64,
    pub cmd: String,
    pub params: serde_json::Value,
}

// ── Outgoing (game → panel) ──

#[derive(Serialize, Debug, Clone)]
pub struct WsEvent {
    #[serde(rename = "type")]
    pub msg_type: &'static str,
    pub event: &'static str,
    pub data: serde_json::Value,
}

#[derive(Serialize, Debug, Clone)]
pub struct WsResponse {
    pub id: u64,
    #[serde(rename = "type")]
    pub msg_type: &'static str,
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

// ── Command params ──

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

// ── Event constructors ──

impl WsEvent {
    pub fn game_loaded() -> Self {
        Self {
            msg_type: "event",
            event: "game_loaded",
            data: serde_json::json!({}),
        }
    }

    pub fn game_paused(paused: bool) -> Self {
        Self {
            msg_type: "event",
            event: "game_paused",
            data: serde_json::json!({"paused": paused}),
        }
    }

    pub fn champion_changed(name: &str) -> Self {
        Self {
            msg_type: "event",
            event: "champion_changed",
            data: serde_json::json!({"name": name}),
        }
    }

    pub fn entity_selected(entity_id: u32, kind: &str, name: &str) -> Self {
        Self {
            msg_type: "event",
            event: "entity_selected",
            data: serde_json::json!({"entity_id": entity_id, "kind": kind, "name": name}),
        }
    }

    pub fn game_close(reason: &str) -> Self {
        Self {
            msg_type: "event",
            event: "game_close",
            data: serde_json::json!({"reason": reason}),
        }
    }

    /// 对局结构化事件（champion_kill / turret_destroyed / cs_threshold / time_progress）。
    /// 由 lol_core 的 match_events 插件产出，经 WS 转发给 web server 的 match supervisor。
    pub fn match_event(payload: serde_json::Value) -> Self {
        Self {
            msg_type: "event",
            event: "match_event",
            data: payload,
        }
    }
}

impl WsResponse {
    pub fn ok(id: u64) -> Self {
        Self {
            id,
            msg_type: "result",
            ok: true,
            data: None,
            error: None,
        }
    }

    pub fn ok_with_data(id: u64, data: serde_json::Value) -> Self {
        Self {
            id,
            msg_type: "result",
            ok: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn err(id: u64, error: String) -> Self {
        Self {
            id,
            msg_type: "result",
            ok: false,
            data: None,
            error: Some(error),
        }
    }
}
