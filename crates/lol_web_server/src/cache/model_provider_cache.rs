//! ModelProvider 子系统的缓存层。
//!
//! 运行时编排器按 provider_id 频繁解析凭证，缓存按 user_id 存整张列表（含明文密钥）。
//! Moka impl 用于生产，Noop impl 用于测试。

use std::sync::Arc;

use async_trait::async_trait;
use moka::future::Cache;

use crate::domain::model_provider::ModelProvider;

#[async_trait]
pub trait ModelProviderCache: Send + Sync {
    async fn get(&self, user_id: i32) -> Option<Vec<ModelProvider>>;
    async fn put(&self, user_id: i32, providers: Vec<ModelProvider>);
    async fn invalidate(&self, user_id: i32);
}

#[derive(Clone)]
pub struct MokaModelProviderCache {
    inner: Arc<Cache<i32, Vec<ModelProvider>>>,
}

impl MokaModelProviderCache {
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

impl Default for MokaModelProviderCache {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ModelProviderCache for MokaModelProviderCache {
    async fn get(&self, user_id: i32) -> Option<Vec<ModelProvider>> {
        self.inner.get(&user_id).await
    }

    async fn put(&self, user_id: i32, providers: Vec<ModelProvider>) {
        self.inner.insert(user_id, providers).await;
    }

    async fn invalidate(&self, user_id: i32) {
        self.inner.invalidate(&user_id).await;
    }
}

/// Noop 实现（测试默认）。
#[derive(Default, Clone, Copy)]
pub struct NoopModelProviderCache;

#[async_trait]
impl ModelProviderCache for NoopModelProviderCache {
    async fn get(&self, _user_id: i32) -> Option<Vec<ModelProvider>> {
        None
    }
    async fn put(&self, _user_id: i32, _providers: Vec<ModelProvider>) {}
    async fn invalidate(&self, _user_id: i32) {}
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::*;
    use crate::domain::model_provider::ModelProvider;

    fn sample(owner: i32) -> ModelProvider {
        ModelProvider {
            id: Uuid::new_v4(),
            owner_id: owner,
            name: "智谱".into(),
            category: "preset".into(),
            preset_type: "zhipu".into(),
            base_url: "https://open.bigmodel.cn/api/anthropic".into(),
            api_key: "sk".into(),
            api_format: "anthropic".into(),
            models: vec!["glm-5.1".into()],
            enabled: true,
            website_url: String::new(),
            api_key_url: String::new(),
            icon: "zhipu".into(),
            icon_color: "#0F62FE".into(),
            sort_order: 0,
        }
    }

    #[tokio::test]
    async fn put_then_get_roundtrip() {
        let cache = MokaModelProviderCache::new();
        cache.put(1, vec![sample(1)]).await;
        let got = cache.get(1).await;
        assert!(got.is_some_and(|v| v.len() == 1));
    }

    #[tokio::test]
    async fn invalidate_removes_entry() {
        let cache = MokaModelProviderCache::new();
        cache.put(1, vec![sample(1)]).await;
        cache.invalidate(1).await;
        assert!(cache.get(1).await.is_none());
    }

    #[tokio::test]
    async fn noop_returns_none() {
        let cache = NoopModelProviderCache;
        cache.put(1, vec![sample(1)]).await;
        assert!(cache.get(1).await.is_none());
    }
}
