//! AgentConfig 子系统的 service 层。

use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

use crate::cache::agent_config_cache::AgentConfigCache;
use crate::domain::agent_config::{
    AgentConfig, AgentConfigInput, can_view_sensitive, validate_name,
};
use crate::domain::{ServiceError, ServiceResult};
use crate::repository::agent_config_repo::AgentConfigRepo;

/// 返回给前端的视图：敏感字段按权限裁剪。
#[derive(Debug, Clone, serde::Serialize)]
pub struct AgentConfigView {
    pub id: Uuid,
    pub owner_id: i32,
    pub name: String,
    pub agent_type: String,
    pub prompt: Option<String>,
    pub preamble: Option<String>,
    pub model: Option<String>,
    pub config_json: Option<serde_json::Value>,
    pub visibility: String,
    pub forked_from: Option<Uuid>,
}

impl AgentConfigView {
    /// 按权限裁剪敏感字段。
    pub fn from_config(config: AgentConfig, requester_id: i32, is_friend: bool) -> Self {
        let show_sensitive = can_view_sensitive(&config, requester_id, is_friend);
        AgentConfigView {
            id: config.id,
            owner_id: config.owner_id,
            name: config.name,
            agent_type: config.agent_type.as_str().to_string(),
            prompt: show_sensitive.then_some(config.prompt),
            preamble: show_sensitive.then_some(config.preamble),
            model: show_sensitive.then_some(config.model),
            config_json: show_sensitive.then_some(config.config_json),
            visibility: config.visibility.as_str().to_string(),
            forked_from: config.forked_from,
        }
    }
}

#[async_trait]
pub trait AgentConfigService: Send + Sync {
    async fn list(&self, owner_id: i32) -> ServiceResult<Vec<AgentConfigView>>;
    async fn get(
        &self,
        requester_id: i32,
        id: Uuid,
        is_friend: bool,
    ) -> ServiceResult<AgentConfigView>;
    async fn create(
        &self,
        owner_id: i32,
        input: AgentConfigInput,
    ) -> ServiceResult<AgentConfigView>;
    async fn update(&self, owner_id: i32, id: Uuid, input: AgentConfigInput) -> ServiceResult<()>;
    async fn delete(&self, owner_id: i32, id: Uuid) -> ServiceResult<()>;
}

pub struct AgentConfigServiceImpl {
    pub repo: Arc<dyn AgentConfigRepo>,
    pub cache: Arc<dyn AgentConfigCache>,
}

impl AgentConfigServiceImpl {
    pub fn new(repo: Arc<dyn AgentConfigRepo>, cache: Arc<dyn AgentConfigCache>) -> Self {
        Self { repo, cache }
    }
}

#[async_trait]
impl AgentConfigService for AgentConfigServiceImpl {
    async fn list(&self, owner_id: i32) -> ServiceResult<Vec<AgentConfigView>> {
        let configs = self.repo.list_by_owner(owner_id).await?;
        Ok(configs
            .into_iter()
            .map(|c| AgentConfigView::from_config(c, owner_id, false))
            .collect())
    }

    async fn get(
        &self,
        requester_id: i32,
        id: Uuid,
        is_friend: bool,
    ) -> ServiceResult<AgentConfigView> {
        // 1. 查缓存（仅命中时跳过 repo；缓存的内容已通过可见性校验过的）
        if let Some(cached) = self.cache.get(id).await {
            // 缓存命中后仍需对当前 requester 做可见性校验
            if cached.owner_id != requester_id
                && cached.visibility == crate::domain::spawn_preset::Visibility::Private
            {
                return Err(ServiceError::NotFound);
            }
            return Ok(AgentConfigView::from_config(
                cached,
                requester_id,
                is_friend,
            ));
        }
        // 2. miss → 查 repo
        let config = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or(ServiceError::NotFound)?;
        // 3. 可见性校验（在回填缓存之前，避免缓存他人不可见的配置）
        if config.owner_id != requester_id
            && config.visibility == crate::domain::spawn_preset::Visibility::Private
        {
            return Err(ServiceError::NotFound);
        }
        // 4. 校验通过才回填缓存
        self.cache.put(config.clone()).await;
        Ok(AgentConfigView::from_config(
            config,
            requester_id,
            is_friend,
        ))
    }

    async fn create(
        &self,
        owner_id: i32,
        input: AgentConfigInput,
    ) -> ServiceResult<AgentConfigView> {
        if !validate_name(&input.name) {
            return Err(ServiceError::Validation(
                "名称不能为空且不超过 64 字符".into(),
            ));
        }
        let config = self.repo.insert(owner_id, &input).await?;
        Ok(AgentConfigView::from_config(config, owner_id, false))
    }

    async fn update(&self, owner_id: i32, id: Uuid, input: AgentConfigInput) -> ServiceResult<()> {
        if !validate_name(&input.name) {
            return Err(ServiceError::Validation(
                "名称不能为空且不超过 64 字符".into(),
            ));
        }
        let existing = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or(ServiceError::NotFound)?;
        if existing.owner_id != owner_id {
            return Err(ServiceError::Forbidden);
        }
        self.repo.update(id, &input).await?;
        self.cache.invalidate(id).await;
        Ok(())
    }

    async fn delete(&self, owner_id: i32, id: Uuid) -> ServiceResult<()> {
        let existing = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or(ServiceError::NotFound)?;
        if existing.owner_id != owner_id {
            return Err(ServiceError::Forbidden);
        }
        self.repo.delete(id).await?;
        self.cache.invalidate(id).await;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::RepoResult;
    use crate::domain::agent_config::{AgentConfig, AgentConfigInput, AgentType};
    use crate::domain::spawn_preset::Visibility;
    use mockall::mock;
    use mockall::predicate::*;

    mock! {
        pub AgentConfigRepo {}
        #[async_trait]
        impl AgentConfigRepo for AgentConfigRepo {
            async fn find_by_id(&self, id: Uuid) -> RepoResult<Option<AgentConfig>>;
            async fn list_by_owner(&self, owner_id: i32) -> RepoResult<Vec<AgentConfig>>;
            async fn insert(&self, owner_id: i32, input: &AgentConfigInput) -> RepoResult<AgentConfig>;
            async fn update(&self, id: Uuid, input: &AgentConfigInput) -> RepoResult<()>;
            async fn delete(&self, id: Uuid) -> RepoResult<()>;
            async fn count_by_owner(&self, owner_id: i32) -> RepoResult<i64>;
        }
    }

    mock! {
        pub AgentConfigCache {}
        #[async_trait]
        impl AgentConfigCache for AgentConfigCache {
            async fn get(&self, id: Uuid) -> Option<AgentConfig>;
            async fn put(&self, config: AgentConfig);
            async fn invalidate(&self, id: Uuid);
        }
    }

    fn sample_input() -> AgentConfigInput {
        AgentConfigInput {
            name: "激进".into(),
            agent_type: AgentType::Llm,
            prompt: "aggro".into(),
            preamble: "".into(),
            model: "claude".into(),
            config_json: serde_json::json!({"depth": 2}),
            visibility: Visibility::Private,
        }
    }

    fn sample_config(owner: i32, vis: Visibility) -> AgentConfig {
        AgentConfig {
            id: Uuid::new_v4(),
            owner_id: owner,
            name: "激进".into(),
            agent_type: AgentType::Llm,
            prompt: "secret prompt".into(),
            preamble: "".into(),
            model: "secret model".into(),
            config_json: serde_json::json!({"secret": true}),
            visibility: vis,
            forked_from: None,
        }
    }

    fn build_service(
        repo: MockAgentConfigRepo,
        cache: MockAgentConfigCache,
    ) -> AgentConfigServiceImpl {
        AgentConfigServiceImpl {
            repo: Arc::new(repo),
            cache: Arc::new(cache),
        }
    }

    #[tokio::test]
    async fn create_invalid_name_rejected() {
        let svc = build_service(MockAgentConfigRepo::new(), MockAgentConfigCache::new());
        let mut input = sample_input();
        input.name = "".into();
        let err = svc.create(1, input).await.unwrap_err();
        assert!(matches!(err, ServiceError::Validation(_)));
    }

    #[tokio::test]
    async fn create_success() {
        let cfg = sample_config(1, Visibility::Private);
        let cfg_clone = cfg.clone();
        let mut repo = MockAgentConfigRepo::new();
        repo.expect_insert()
            .returning(move |_, _| Ok(cfg_clone.clone()));
        let svc = build_service(repo, MockAgentConfigCache::new());
        let view = svc.create(1, sample_input()).await.unwrap();
        assert!(view.prompt.is_some(), "owner 创建时应见敏感字段");
    }

    #[tokio::test]
    async fn get_owner_sees_sensitive() {
        let cfg = sample_config(1, Visibility::Private);
        let cfg_clone = cfg.clone();
        let mut repo = MockAgentConfigRepo::new();
        repo.expect_find_by_id()
            .returning(move |_| Ok(Some(cfg_clone.clone())));
        let mut cache = MockAgentConfigCache::new();
        cache.expect_get().returning(|_| None);
        cache.expect_put().return_const(());
        let svc = build_service(repo, cache);
        let view = svc.get(1, Uuid::new_v4(), false).await.unwrap();
        assert!(view.prompt.is_some());
    }

    #[tokio::test]
    async fn get_non_owner_private_returns_not_found() {
        let cfg = sample_config(1, Visibility::Private);
        let mut repo = MockAgentConfigRepo::new();
        repo.expect_find_by_id()
            .returning(move |_| Ok(Some(cfg.clone())));
        let mut cache = MockAgentConfigCache::new();
        cache.expect_get().returning(|_| None);
        cache.expect_put().times(0); // private 不该缓存后被他人读取（这里不缓存也无妨）
        let svc = build_service(repo, cache);
        let err = svc.get(2, Uuid::new_v4(), false).await.unwrap_err();
        assert!(matches!(err, ServiceError::NotFound));
    }

    #[tokio::test]
    async fn get_public_non_friend_hides_sensitive() {
        let cfg = sample_config(1, Visibility::Public);
        let mut repo = MockAgentConfigRepo::new();
        repo.expect_find_by_id()
            .returning(move |_| Ok(Some(cfg.clone())));
        let mut cache = MockAgentConfigCache::new();
        cache.expect_get().returning(|_| None);
        cache.expect_put().return_const(());
        let svc = build_service(repo, cache);
        let view = svc.get(2, Uuid::new_v4(), false).await.unwrap();
        assert!(view.prompt.is_none(), "公开但非好友，敏感字段不可见");
        assert!(view.model.is_none());
    }

    #[tokio::test]
    async fn get_public_friend_sees_sensitive() {
        let cfg = sample_config(1, Visibility::Public);
        let mut repo = MockAgentConfigRepo::new();
        repo.expect_find_by_id()
            .returning(move |_| Ok(Some(cfg.clone())));
        let mut cache = MockAgentConfigCache::new();
        cache.expect_get().returning(|_| None);
        cache.expect_put().return_const(());
        let svc = build_service(repo, cache);
        let view = svc.get(2, Uuid::new_v4(), true).await.unwrap();
        assert!(view.prompt.is_some(), "公开且好友，敏感字段可见");
    }

    #[tokio::test]
    async fn get_cache_hit_skips_repo() {
        let cfg = sample_config(1, Visibility::Public);
        let mut repo = MockAgentConfigRepo::new();
        repo.expect_find_by_id().times(0);
        let mut cache = MockAgentConfigCache::new();
        cache.expect_get().returning(move |_| Some(cfg.clone()));
        let svc = build_service(repo, cache);
        svc.get(1, Uuid::new_v4(), false).await.unwrap();
    }

    #[tokio::test]
    async fn update_non_owner_forbidden() {
        let cfg = sample_config(1, Visibility::Private);
        let mut repo = MockAgentConfigRepo::new();
        repo.expect_find_by_id()
            .returning(move |_| Ok(Some(cfg.clone())));
        repo.expect_update().times(0);
        let svc = build_service(repo, MockAgentConfigCache::new());
        let err = svc
            .update(2, Uuid::new_v4(), sample_input())
            .await
            .unwrap_err();
        assert!(matches!(err, ServiceError::Forbidden));
    }

    #[tokio::test]
    async fn update_invalidates_cache() {
        let cfg = sample_config(1, Visibility::Private);
        let mut repo = MockAgentConfigRepo::new();
        repo.expect_find_by_id()
            .returning(move |_| Ok(Some(cfg.clone())));
        repo.expect_update().returning(|_, _| Ok(()));
        let mut cache = MockAgentConfigCache::new();
        cache.expect_invalidate().times(1).return_const(());
        let svc = build_service(repo, cache);
        svc.update(1, Uuid::new_v4(), sample_input()).await.unwrap();
    }

    #[tokio::test]
    async fn delete_non_owner_forbidden() {
        let cfg = sample_config(1, Visibility::Private);
        let mut repo = MockAgentConfigRepo::new();
        repo.expect_find_by_id()
            .returning(move |_| Ok(Some(cfg.clone())));
        repo.expect_delete().times(0);
        let svc = build_service(repo, MockAgentConfigCache::new());
        let err = svc.delete(2, Uuid::new_v4()).await.unwrap_err();
        assert!(matches!(err, ServiceError::Forbidden));
    }

    #[tokio::test]
    async fn delete_owner_success_invalidates_cache() {
        let cfg = sample_config(1, Visibility::Private);
        let mut repo = MockAgentConfigRepo::new();
        repo.expect_find_by_id()
            .returning(move |_| Ok(Some(cfg.clone())));
        repo.expect_delete().returning(|_| Ok(()));
        let mut cache = MockAgentConfigCache::new();
        cache.expect_invalidate().times(1).return_const(());
        let svc = build_service(repo, cache);
        svc.delete(1, Uuid::new_v4()).await.unwrap();
    }
}
