//! Config 子系统的 service 层（业务编排）。
//!
//! 黄金示例：service 编排 repo（持久）+ cache（缓存）。
//! service 单测会用 mockall mock repo 和 cache，完全不碰 DB。

use std::sync::Arc;

use async_trait::async_trait;

use crate::cache::config_cache::ConfigCache;
use crate::domain::ServiceResult;
use crate::domain::config::AiConfig;
use crate::repository::config_repo::ConfigRepo;

/// Config service trait（业务接口）。
#[async_trait]
pub trait ConfigService: Send + Sync {
    /// 获取用户配置。未设置时返回空配置（不报错）。
    async fn get_config(&self, user_id: i32) -> ServiceResult<AiConfig>;

    /// 保存用户配置（upsert）。
    async fn set_config(&self, user_id: i32, config: AiConfig) -> ServiceResult<()>;
}

/// service 实现：持有 repo + cache（均为 trait object，可 mock）。
pub struct ConfigServiceImpl {
    pub repo: Arc<dyn ConfigRepo>,
    pub cache: Arc<dyn ConfigCache>,
}

impl ConfigServiceImpl {
    pub fn new(repo: Arc<dyn ConfigRepo>, cache: Arc<dyn ConfigCache>) -> Self {
        Self { repo, cache }
    }
}

#[async_trait]
impl ConfigService for ConfigServiceImpl {
    async fn get_config(&self, user_id: i32) -> ServiceResult<AiConfig> {
        // 1. 先查缓存
        if let Some(cfg) = self.cache.get(user_id).await {
            return Ok(cfg);
        }
        // 2. miss → 查 repo
        let cfg = self.repo.find_by_user(user_id).await?.unwrap_or_default();
        // 3. 回填缓存（即使是空配置也缓存，避免反复查 DB）
        self.cache.put(user_id, cfg.clone()).await;
        Ok(cfg)
    }

    async fn set_config(&self, user_id: i32, config: AiConfig) -> ServiceResult<()> {
        // 1. 落库
        self.repo.upsert(user_id, &config).await?;
        // 2. 失效缓存（下次 get 重新加载）
        self.cache.invalidate(user_id).await;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use mockall::mock;
    use mockall::predicate::*;

    use super::*;
    use crate::domain::{RepoError, RepoResult};

    // ── 用 mockall 生成 repo 和 cache 的 mock ──
    mock! {
        pub ConfigRepo {}
        #[async_trait]
        impl ConfigRepo for ConfigRepo {
            async fn find_by_user(&self, user_id: i32) -> RepoResult<Option<AiConfig>>;
            async fn upsert(&self, user_id: i32, config: &AiConfig) -> RepoResult<()>;
        }
    }

    mock! {
        pub ConfigCache {}
        #[async_trait]
        impl ConfigCache for ConfigCache {
            async fn get(&self, user_id: i32) -> Option<AiConfig>;
            async fn put(&self, user_id: i32, config: AiConfig);
            async fn invalidate(&self, user_id: i32);
        }
    }

    fn build_service(repo: MockConfigRepo, cache: MockConfigCache) -> ConfigServiceImpl {
        ConfigServiceImpl {
            repo: Arc::new(repo),
            cache: Arc::new(cache),
        }
    }

    #[tokio::test]
    async fn get_config_cache_hit_does_not_query_repo() {
        let mut repo = MockConfigRepo::new();
        repo.expect_find_by_user().times(0); // 断言：缓存命中时不查 DB

        let mut cache = MockConfigCache::new();
        cache.expect_get().with(eq(1)).returning(|_| {
            Some(AiConfig {
                api_key: "cached".into(),
                ..AiConfig::empty()
            })
        });

        let svc = build_service(repo, cache);
        let cfg = svc.get_config(1).await.unwrap();
        assert_eq!(cfg.api_key, "cached");
    }

    #[tokio::test]
    async fn get_config_cache_miss_falls_back_to_repo_and_backfills() {
        let mut repo = MockConfigRepo::new();
        repo.expect_find_by_user().with(eq(1)).returning(|_| {
            Ok(Some(AiConfig {
                api_key: "from-db".into(),
                ..AiConfig::empty()
            }))
        });

        let mut cache = MockConfigCache::new();
        cache.expect_get().with(eq(1)).returning(|_| None); // miss
        cache.expect_put().times(1).return_const(()); // 断言：回填

        let svc = build_service(repo, cache);
        let cfg = svc.get_config(1).await.unwrap();
        assert_eq!(cfg.api_key, "from-db");
    }

    #[tokio::test]
    async fn get_config_returns_empty_when_user_has_no_config() {
        let mut repo = MockConfigRepo::new();
        repo.expect_find_by_user().returning(|_| Ok(None)); // 用户从未设置

        let mut cache = MockConfigCache::new();
        cache.expect_get().returning(|_| None);
        cache.expect_put().times(1).return_const(()); // 空配置也回填

        let svc = build_service(repo, cache);
        let cfg = svc.get_config(1).await.unwrap();
        assert!(cfg.is_empty());
    }

    #[tokio::test]
    async fn get_config_propagates_repo_error() {
        let mut repo = MockConfigRepo::new();
        repo.expect_find_by_user()
            .returning(|_| Err(RepoError::NotFound));

        let mut cache = MockConfigCache::new();
        cache.expect_get().returning(|_| None);
        cache.expect_put().times(0); // 错误时不回填

        let svc = build_service(repo, cache);
        let err = svc.get_config(1).await.unwrap_err();
        assert!(matches!(err, ServiceError::NotFound));
    }

    #[tokio::test]
    async fn set_config_persists_then_invalidates_cache() {
        let mut repo = MockConfigRepo::new();
        repo.expect_upsert()
            .with(eq(1), always())
            .returning(|_, _| Ok(()));

        let mut cache = MockConfigCache::new();
        cache
            .expect_invalidate()
            .with(eq(1))
            .times(1)
            .return_const(());

        let svc = build_service(repo, cache);
        svc.set_config(1, AiConfig::empty()).await.unwrap();
    }

    #[tokio::test]
    async fn set_config_does_not_invalidate_cache_if_repo_fails() {
        let mut repo = MockConfigRepo::new();
        repo.expect_upsert()
            .returning(|_, _| Err(RepoError::UniqueViolation));

        let mut cache = MockConfigCache::new();
        cache.expect_invalidate().times(0); // 断言：落库失败不清缓存

        let svc = build_service(repo, cache);
        let err = svc.set_config(1, AiConfig::empty()).await.unwrap_err();
        assert!(matches!(err, ServiceError::Conflict(_)));
    }
}
