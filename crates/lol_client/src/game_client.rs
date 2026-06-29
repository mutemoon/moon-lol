use serde_json::{Value, json};

use crate::action::Action;
use crate::protocol::WsResponse;
use crate::session::WsSession;

/// 类型化游戏客户端：方法一一映射服务端 cmd 字符串，参数用纯 Rust 类型拼 JSON。
///
/// 不持有连接生命周期，内部复用 [`WsSession`]（可 Clone 共享同一连接）。
#[derive(Clone)]
pub struct GameClient {
    session: WsSession,
}

impl GameClient {
    pub fn new(session: WsSession) -> Self {
        Self { session }
    }

    /// 借用底层会话（供需要直接 send_cmd 的调用方使用）。
    pub fn session(&self) -> &WsSession {
        &self.session
    }

    async fn cmd(&self, cmd: &str, params: Value) -> Result<WsResponse, String> {
        self.session.send_cmd(cmd.to_string(), params).await
    }

    // ── observe / action（MCP 暴露的子集）──

    pub async fn observe(&self, entity_id: u64) -> Result<WsResponse, String> {
        self.cmd("get_observe", json!({ "entity_id": entity_id }))
            .await
    }

    pub async fn action(&self, entity_id: u64, action: Action) -> Result<WsResponse, String> {
        let action_val = serde_json::to_value(&action).map_err(|e| e.to_string())?;
        self.cmd(
            "action",
            json!({ "entity_id": entity_id, "action": action_val }),
        )
        .await
    }

    // ── 调试 / 控制面（CLI 完全控制）──

    pub async fn agents(&self) -> Result<WsResponse, String> {
        self.cmd("get_agents", Value::Null).await
    }

    pub async fn state(&self) -> Result<WsResponse, String> {
        self.cmd("get_state", Value::Null).await
    }

    pub async fn toggle_pause(&self) -> Result<WsResponse, String> {
        self.cmd("toggle_pause", Value::Null).await
    }

    pub async fn switch_champion(&self, name: &str) -> Result<WsResponse, String> {
        self.cmd("switch_champion", json!({ "name": name })).await
    }

    pub async fn god_mode(&self, enabled: bool) -> Result<WsResponse, String> {
        self.cmd("god_mode", json!({ "enabled": enabled })).await
    }

    pub async fn toggle_cooldown(&self, enabled: bool) -> Result<WsResponse, String> {
        self.cmd("toggle_cooldown", json!({ "enabled": enabled }))
            .await
    }

    pub async fn reset_position(&self) -> Result<WsResponse, String> {
        self.cmd("reset_position", Value::Null).await
    }

    pub async fn set_script(&self, entity_id: u64, source: &str) -> Result<WsResponse, String> {
        self.cmd(
            "set_script",
            json!({ "entity_id": entity_id, "source": source }),
        )
        .await
    }

    // ── 高频推理 / RL（packed）──

    pub async fn observe_packed(&self, entity_id: u64) -> Result<WsResponse, String> {
        self.cmd("get_observe_packed", json!({ "entity_id": entity_id }))
            .await
    }

    pub async fn action_packed(
        &self,
        entity_id: u64,
        msgpack_b64: &str,
    ) -> Result<WsResponse, String> {
        self.cmd(
            "action_packed",
            json!({ "entity_id": entity_id, "msgpack_b64": msgpack_b64 }),
        )
        .await
    }

    pub async fn rl_reset(
        &self,
        entity_id: u64,
        config_json: Option<Value>,
    ) -> Result<WsResponse, String> {
        let mut params = json!({ "entity_id": entity_id });
        if let Some(cfg) = config_json {
            params["config_json"] = cfg;
        }
        self.cmd("rl_reset", params).await
    }

    pub async fn rl_step(&self, entity_id: u64) -> Result<WsResponse, String> {
        self.cmd("rl_step", json!({ "entity_id": entity_id })).await
    }

    // ── 幂等暂停 / 恢复（先 get_state 再决定是否 toggle_pause）──

    /// 幂等暂停：已暂停则不操作。返回是否实际触发了切换。
    pub async fn pause(&self) -> Result<bool, String> {
        if self.is_paused().await? {
            return Ok(false);
        }
        self.toggle_pause().await?;
        Ok(true)
    }

    /// 幂等恢复：已运行则不操作。返回是否实际触发了切换。
    pub async fn unpause(&self) -> Result<bool, String> {
        if !self.is_paused().await? {
            return Ok(false);
        }
        self.toggle_pause().await?;
        Ok(true)
    }

    async fn is_paused(&self) -> Result<bool, String> {
        let resp = self.state().await?;
        Ok(resp
            .data
            .and_then(|d| d.get("paused").and_then(|p| p.as_bool()))
            .unwrap_or(false))
    }
}
