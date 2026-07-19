use serde_json::{Value, json};

use crate::action::Action;
use crate::protocol::*;
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

    /// 检查连接是否已关闭。
    pub fn is_closed(&self) -> bool {
        self.session.is_closed()
    }

    async fn cmd(&self, cmd: &str, params: Value) -> Result<WsResponse, String> {
        self.session.send_cmd(cmd.to_string(), params).await
    }

    pub async fn observe(&self, entity_id: u64, json: bool) -> Result<WsResponse, String> {
        self.cmd(CMD_OBSERVE, json!({ "entity_id": entity_id, "json": json }))
            .await
    }

    pub async fn action(&self, entity_id: u64, action: Action) -> Result<WsResponse, String> {
        let action_val = serde_json::to_value(&action).map_err(|e| e.to_string())?;
        self.cmd(
            CMD_ACTION,
            json!({ "entity_id": entity_id, "action": action_val }),
        )
        .await
    }

    // ── 调试 / 控制面（CLI 完全控制）──

    pub async fn agents(&self) -> Result<WsResponse, String> {
        self.cmd(CMD_GET_AGENTS, Value::Null).await
    }

    pub async fn state(&self) -> Result<WsResponse, String> {
        self.cmd(CMD_GET_STATE, Value::Null).await
    }

    /// 查询当前游戏内时间（秒）。
    pub async fn get_time(&self) -> Result<f64, String> {
        let resp = self.cmd(CMD_GET_TIME, Value::Null).await?;
        if !resp.ok {
            return Err(resp.error.unwrap_or_else(|| "get_time 失败".to_string()));
        }
        resp.data
            .and_then(|d| d.get("time").and_then(|t| t.as_f64()))
            .ok_or_else(|| "get_time 响应缺少 time 字段".to_string())
    }

    pub async fn toggle_pause(&self) -> Result<WsResponse, String> {
        self.cmd(CMD_TOGGLE_PAUSE, Value::Null).await
    }

    pub async fn set_speed(&self, speed: f32) -> Result<WsResponse, String> {
        self.cmd(CMD_SET_SPEED, json!({ "speed": speed })).await
    }

    pub async fn switch_champion(&self, name: &str) -> Result<WsResponse, String> {
        self.cmd(CMD_SWITCH_CHAMPION, json!({ "name": name })).await
    }

    pub async fn god_mode(&self, enabled: bool) -> Result<WsResponse, String> {
        self.cmd(CMD_GOD_MODE, json!({ "enabled": enabled })).await
    }

    pub async fn toggle_cooldown(&self, enabled: bool) -> Result<WsResponse, String> {
        self.cmd(CMD_TOGGLE_COOLDOWN, json!({ "enabled": enabled }))
            .await
    }

    pub async fn reset_position(&self) -> Result<WsResponse, String> {
        self.cmd(CMD_RESET_POSITION, Value::Null).await
    }

    pub async fn set_script(&self, entity_id: u64, source: &str) -> Result<WsResponse, String> {
        self.cmd(
            CMD_SET_SCRIPT,
            json!({ "entity_id": entity_id, "source": source }),
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
        self.cmd(CMD_RL_RESET, params).await
    }

    pub async fn rl_step(&self, entity_id: u64, frames: Option<u32>) -> Result<WsResponse, String> {
        let mut params = json!({ "entity_id": entity_id });
        if let Some(f) = frames {
            params["frames"] = json!(f);
        }
        self.cmd(CMD_RL_STEP, params).await
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
