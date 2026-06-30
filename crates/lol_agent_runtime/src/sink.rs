//! 编排环副作用出口 trait：屏蔽桌面端（Tauri emit + 写盘 + 停进程）与云端（无副作用）的差异。

use async_trait::async_trait;
use rig::completion::Message;
use serde_json::Value;

use crate::credentials::AgentConfig;

/// 单个 agent 一局结束后的完整对话历史。
pub struct AgentRunResult {
    pub agent: AgentConfig,
    pub history: Vec<Message>,
}

/// 编排环向宿主暴露的回调。所有方法有默认空实现，宿主按需覆写。
#[async_trait]
pub trait OrchestratorSink: Send + Sync {
    /// 每个 agent 每轮思考后回调（用于 live 推送对话历史到前端）。
    async fn on_agent_thought(&self, _agent: &AgentConfig, _history: &[Message], _cycle: u64) {}

    /// 对局终结回调：最终观测原文、游戏时长、各 agent 的完整对话历史。
    async fn on_finished(
        &self,
        _final_observation: &Value,
        _last_game_time: f64,
        _results: &[AgentRunResult],
    ) {
    }

    /// 是否继续运行。返回 false 时编排环退出（桌面端用于检测游戏进程是否已被销毁）。
    async fn is_running(&self) -> bool {
        true
    }
}

/// 空实现，云端等不需要副作用的宿主使用。
pub struct NoopSink;

#[async_trait]
impl OrchestratorSink for NoopSink {}
