//! AgentConfig 子系统的缓存层。

use async_trait::async_trait;
use moka::future::Cache;
use std::sync::Arc;

use crate::domain::agent_config::AgentConfig;

#[async_trait]
pub trait AgentConfigCache: Send + Sync {
    async fn get(&self, id: uuid::Uuid) -> Option<AgentConfig>;
    async fn put(&self, config: AgentConfig);
    async fn invalidate(&self, id: uuid::Uuid);
}

#[derive(Clone)]
pub struct MokaAgentConfigCache {
    inner: Arc<Cache<uuid::Uuid, AgentConfig>>,
}

impl MokaAgentConfigCache {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(
                Cache::builder()
                    .max_capacity(10_000)
                    .time_to_live(std::time::Duration::from_secs(300))
                    .build(),
            ),
        }
    }
}

impl Default for MokaAgentConfigCache {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AgentConfigCache for MokaAgentConfigCache {
    async fn get(&self, id: uuid::Uuid) -> Option<AgentConfig> {
        self.inner.get(&id).await
    }
    async fn put(&self, config: AgentConfig) {
        self.inner.insert(config.id, config).await;
    }
    async fn invalidate(&self, id: uuid::Uuid) {
        self.inner.invalidate(&id).await;
    }
}

#[derive(Default, Clone, Copy)]
pub struct NoopAgentConfigCache;

#[async_trait]
impl AgentConfigCache for NoopAgentConfigCache {
    async fn get(&self, _id: uuid::Uuid) -> Option<AgentConfig> {
        None
    }
    async fn put(&self, _config: AgentConfig) {}
    async fn invalidate(&self, _id: uuid::Uuid) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::agent_config::{AgentConfig, AgentType};
    use crate::domain::spawn_preset::Visibility;

    fn sample(id: uuid::Uuid) -> AgentConfig {
        AgentConfig {
            id,
            owner_id: 1,
            name: "test".into(),
            agent_type: AgentType::Llm,
            prompt: "".into(),
            preamble: "".into(),
            model: "".into(),
            config_json: serde_json::json!({}),
            visibility: Visibility::Private,
            forked_from: None,
        }
    }

    #[tokio::test]
    async fn moka_put_get_invalidate() {
        let cache = MokaAgentConfigCache::new();
        let id = uuid::Uuid::new_v4();
        cache.put(sample(id)).await;
        assert!(cache.get(id).await.is_some());
        cache.invalidate(id).await;
        assert!(cache.get(id).await.is_none());
    }

    #[tokio::test]
    async fn moka_missing_returns_none() {
        let cache = MokaAgentConfigCache::new();
        assert!(cache.get(uuid::Uuid::new_v4()).await.is_none());
    }

    #[tokio::test]
    async fn noop_returns_none() {
        let cache = NoopAgentConfigCache;
        cache.put(sample(uuid::Uuid::new_v4())).await;
        assert!(cache.get(uuid::Uuid::new_v4()).await.is_none());
    }
}
