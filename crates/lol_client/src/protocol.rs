use serde::{Deserialize, Serialize};

// ── 请求 (客户端 → 游戏) ──

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WsRequest {
    pub id: u64,
    pub cmd: String,
    pub params: serde_json::Value,
}

// ── 响应 / 事件 (游戏 → 客户端) ──

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WsEvent {
    #[serde(rename = "type")]
    pub msg_type: String,
    pub event: String,
    pub data: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct WsResponse {
    pub id: u64,
    #[serde(rename = "type", default)]
    pub msg_type: String,
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub data: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub error: Option<String>,
}

// ── 事件构造器 ──

impl WsEvent {
    pub fn game_loaded() -> Self {
        Self {
            msg_type: "event".into(),
            event: "game_loaded".into(),
            data: serde_json::json!({}),
        }
    }

    pub fn game_paused(paused: bool) -> Self {
        Self {
            msg_type: "event".into(),
            event: "game_paused".into(),
            data: serde_json::json!({"paused": paused}),
        }
    }

    pub fn champion_changed(name: &str) -> Self {
        Self {
            msg_type: "event".into(),
            event: "champion_changed".into(),
            data: serde_json::json!({"name": name}),
        }
    }

    pub fn entity_selected(entity_id: u32, kind: &str, name: &str) -> Self {
        Self {
            msg_type: "event".into(),
            event: "entity_selected".into(),
            data: serde_json::json!({"entity_id": entity_id, "kind": kind, "name": name}),
        }
    }

    pub fn game_close(reason: &str) -> Self {
        Self {
            msg_type: "event".into(),
            event: "game_close".into(),
            data: serde_json::json!({"reason": reason}),
        }
    }

    /// 对局结构化事件（champion_kill / turret_destroyed / cs_threshold / time_progress）。
    /// 由 lol_core 的 match_events 插件产出，经 WS 转发给 web server 的 match supervisor。
    pub fn match_event(payload: serde_json::Value) -> Self {
        Self {
            msg_type: "event".into(),
            event: "match_event".into(),
            data: payload,
        }
    }
}

impl WsResponse {
    pub fn ok(id: u64) -> Self {
        Self {
            id,
            msg_type: "result".into(),
            ok: true,
            data: None,
            error: None,
        }
    }

    pub fn ok_with_data(id: u64, data: serde_json::Value) -> Self {
        Self {
            id,
            msg_type: "result".into(),
            ok: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn err(id: u64, error: String) -> Self {
        Self {
            id,
            msg_type: "result".into(),
            ok: false,
            data: None,
            error: Some(error),
        }
    }
}
