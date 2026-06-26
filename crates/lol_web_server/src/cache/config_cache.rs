//! Config 子系统的缓存层。
//!
//! Config 读多写少（设置页频繁加载），适合缓存。cache key 是 user_id。
//! Moka impl 用于生产，Noop impl 用于测试（service 单测默认不依赖缓存）。

use std::sync::Arc;

use async_trait::async_trait;
use moka::future::Cache;

use crate::domain::config::AiConfig;

/// Config 缓存 trait。
#[async_trait]
pub trait ConfigCache: Send + Sync {
    async fn get(&self, user_id: i32) -> Option<AiConfig>;
    async fn put(&self, user_id: i32, config: AiConfig);
    async fn invalidate(&self, user_id: i32);
}

/// Moka 实现（生产用）。
#[derive(Clone)]
pub struct MokaConfigCache {
    inner: Arc<Cache<i32, AiConfig>>,
}

impl MokaConfigCache {
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

impl Default for MokaConfigCache {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ConfigCache for MokaConfigCache {
    async fn get(&self, user_id: i32) -> Option<AiConfig> {
        self.inner.get(&user_id).await
    }

    async fn put(&self, user_id: i32, config: AiConfig) {
        self.inner.insert(user_id, config).await;
    }

    async fn invalidate(&self, user_id: i32) {
        self.inner.invalidate(&user_id).await;
    }
}

/// Noop 实现（测试默认：不缓存，所有操作 no-op）。
#[derive(Default, Clone, Copy)]
pub struct NoopConfigCache;

#[async_trait]
impl ConfigCache for NoopConfigCache {
    async fn get(&self, _user_id: i32) -> Option<AiConfig> {
        None
    }
    async fn put(&self, _user_id: i32, _config: AiConfig) {}
    async fn invalidate(&self, _user_id: i32) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn moka_put_then_get_roundtrip() {
        let cache = MokaConfigCache::new();
        let cfg = AiConfig {
            api_key: "sk-test".into(),
            base_url: "https://api.test".into(),
            preamble: "p".into(),
        };
        cache.put(1, cfg.clone()).await;
        let got = cache.get(1).await;
        assert_eq!(got, Some(cfg));
    }

    #[tokio::test]
    async fn moka_invalidate_removes_entry() {
        let cache = MokaConfigCache::new();
        cache.put(1, AiConfig::empty()).await;
        cache.invalidate(1).await;
        assert!(cache.get(1).await.is_none());
    }

    #[tokio::test]
    async fn moka_get_returns_none_for_missing() {
        let cache = MokaConfigCache::new();
        assert!(cache.get(999).await.is_none());
    }

    #[tokio::test]
    async fn noop_always_returns_none() {
        let cache = NoopConfigCache;
        cache.put(1, AiConfig::empty()).await;
        assert!(cache.get(1).await.is_none());
    }

    #[tokio::test]
    async fn moka_isolates_by_user() {
        let cache = MokaConfigCache::new();
        let cfg_a = AiConfig {
            api_key: "a".into(),
            ..AiConfig::empty()
        };
        let cfg_b = AiConfig {
            api_key: "b".into(),
            ..AiConfig::empty()
        };
        cache.put(1, cfg_a.clone()).await;
        cache.put(2, cfg_b.clone()).await;
        assert_eq!(cache.get(1).await, Some(cfg_a));
        assert_eq!(cache.get(2).await, Some(cfg_b));
        // 失效一个不影响另一个
        cache.invalidate(1).await;
        assert!(cache.get(1).await.is_none());
        assert!(cache.get(2).await.is_some());
    }
}
