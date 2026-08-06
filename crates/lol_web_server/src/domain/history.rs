//! 游戏历史 领域层。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 数据库一行：一次游戏的所有 agent 历史快照。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GameHistory {
    pub id: Uuid,
    pub user_id: i32,
    pub datetime: DateTime<Utc>,
    pub game_duration: i64,
    pub agents: serde_json::Value,
    pub histories: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn game_history_serde_roundtrip() {
        let h = GameHistory {
            id: Uuid::new_v4(),
            user_id: 1,
            datetime: Utc::now(),
            game_duration: 1800,
            agents: serde_json::json!([{"agent_id": "a1", "champion": "Riven", "team": "blue"}]),
            histories: serde_json::json!([{"agent_id": "a1", "champion": "Riven", "team": "blue", "prompt": "", "system_prompt": "", "history": [], "game_duration": 1800, "datetime": "2025-01-01T00:00:00Z"}]),
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&h).unwrap();
        let back: GameHistory = serde_json::from_str(&json).unwrap();
        assert_eq!(h, back);
    }
}
