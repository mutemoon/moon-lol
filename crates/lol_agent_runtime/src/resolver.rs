//! 凭证解析 trait：屏蔽桌面端（providers.json）与云端（DB + ModelProviderService）的存储差异。

use async_trait::async_trait;

use crate::credentials::{AgentConfig, PlatformEnv, ResolvedCredentials};

/// 按单个 agent 的 `provider_id` 解析 LLM 凭证。
///
/// 返回 `None` 表示该 agent 无可用凭据（调用方应跳过该 agent）。
#[async_trait]
pub trait CredentialResolver: Send + Sync {
    async fn resolve(&self, agent: &AgentConfig, env: &PlatformEnv) -> Option<ResolvedCredentials>;
}
