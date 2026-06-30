//! ModelProvider 子系统的 service 层。
//!
//! 编排 repo（持久）+ cache（缓存）。list 返回脱敏 DTO，resolve_for_runtime
//! 供编排器取含明文密钥的记录。单测用 mockall mock repo 和 cache。

use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::cache::model_provider_cache::ModelProviderCache;
use crate::domain::model_provider::{
    ModelProvider, ModelProviderDto, ModelProviderInput, validate_api_format, validate_category,
    validate_name,
};
use crate::domain::{ServiceError, ServiceResult};
use crate::repository::model_provider_repo::ModelProviderRepo;

#[async_trait]
pub trait ModelProviderService: Send + Sync {
    /// 列出当前用户的供应商（脱敏）。
    async fn list(&self, owner_id: i32) -> ServiceResult<Vec<ModelProviderDto>>;
    /// 创建。
    async fn create(
        &self,
        owner_id: i32,
        input: ModelProviderInput,
    ) -> ServiceResult<ModelProviderDto>;
    /// 更新（api_key 空串保留旧值）。
    async fn update(&self, owner_id: i32, id: Uuid, input: ModelProviderInput)
    -> ServiceResult<()>;
    /// 删除。
    async fn delete(&self, owner_id: i32, id: Uuid) -> ServiceResult<()>;
    /// 运行时解析：返回含明文密钥的记录，校验归属。
    async fn resolve_for_runtime(
        &self,
        provider_id: Uuid,
        owner_id: i32,
    ) -> ServiceResult<Option<ModelProvider>>;
}

pub struct ModelProviderServiceImpl {
    pub repo: Arc<dyn ModelProviderRepo>,
    pub cache: Arc<dyn ModelProviderCache>,
}

impl ModelProviderServiceImpl {
    pub fn new(repo: Arc<dyn ModelProviderRepo>, cache: Arc<dyn ModelProviderCache>) -> Self {
        Self { repo, cache }
    }

    fn validate(input: &ModelProviderInput) -> ServiceResult<()> {
        if !validate_name(&input.name) {
            return Err(ServiceError::Validation(
                "名称不能为空且不超过 64 字符".into(),
            ));
        }
        if !validate_category(&input.category) {
            return Err(ServiceError::Validation(format!(
                "非法分类: {}",
                input.category
            )));
        }
        if !validate_api_format(&input.api_format) {
            return Err(ServiceError::Validation(format!(
                "非法 API 格式: {}",
                input.api_format
            )));
        }
        Ok(())
    }

    async fn invalidate(&self, owner_id: i32) {
        self.cache.invalidate(owner_id).await;
    }
}

#[async_trait]
impl ModelProviderService for ModelProviderServiceImpl {
    async fn list(&self, owner_id: i32) -> ServiceResult<Vec<ModelProviderDto>> {
        let list = self.repo.list_by_owner(owner_id).await?;
        Ok(list.iter().map(ModelProvider::to_dto).collect())
    }

    async fn create(
        &self,
        owner_id: i32,
        input: ModelProviderInput,
    ) -> ServiceResult<ModelProviderDto> {
        Self::validate(&input)?;
        let created = self
            .repo
            .insert(owner_id, &input)
            .await
            .map_err(map_repo_err)?;
        self.invalidate(owner_id).await;
        Ok(created.to_dto())
    }

    async fn update(
        &self,
        owner_id: i32,
        id: Uuid,
        input: ModelProviderInput,
    ) -> ServiceResult<()> {
        Self::validate(&input)?;
        let existing = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or(ServiceError::NotFound)?;
        if existing.owner_id != owner_id {
            return Err(ServiceError::Forbidden);
        }
        self.repo.update(id, &input).await.map_err(map_repo_err)?;
        self.invalidate(owner_id).await;
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
        self.invalidate(owner_id).await;
        Ok(())
    }

    async fn resolve_for_runtime(
        &self,
        provider_id: Uuid,
        owner_id: i32,
    ) -> ServiceResult<Option<ModelProvider>> {
        // 先查缓存（整张列表），命中则按 id 过滤。
        if let Some(list) = self.cache.get(owner_id).await {
            return Ok(list.into_iter().find(|p| p.id == provider_id));
        }
        let list = self.repo.list_by_owner(owner_id).await?;
        let found = list.iter().find(|p| p.id == provider_id).cloned();
        self.cache.put(owner_id, list).await;
        Ok(found)
    }
}

fn map_repo_err(e: crate::domain::RepoError) -> ServiceError {
    match e {
        crate::domain::RepoError::UniqueViolation => {
            ServiceError::Conflict("同名供应商已存在".into())
        }
        other => ServiceError::Internal(other.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use mockall::mock;
    use mockall::predicate::*;

    use super::*;
    use crate::domain::RepoResult;

    mock! {
        pub ModelProviderRepo {}
        #[async_trait]
        impl ModelProviderRepo for ModelProviderRepo {
            async fn list_by_owner(&self, owner_id: i32) -> RepoResult<Vec<ModelProvider>>;
            async fn find_by_id(&self, id: Uuid) -> RepoResult<Option<ModelProvider>>;
            async fn find_for_runtime(&self, id: Uuid, owner_id: i32) -> RepoResult<Option<ModelProvider>>;
            async fn insert(&self, owner_id: i32, input: &ModelProviderInput) -> RepoResult<ModelProvider>;
            async fn update(&self, id: Uuid, input: &ModelProviderInput) -> RepoResult<()>;
            async fn delete(&self, id: Uuid) -> RepoResult<()>;
        }
    }

    mock! {
        pub ModelProviderCache {}
        #[async_trait]
        impl ModelProviderCache for ModelProviderCache {
            async fn get(&self, user_id: i32) -> Option<Vec<ModelProvider>>;
            async fn put(&self, user_id: i32, providers: Vec<ModelProvider>);
            async fn invalidate(&self, user_id: i32);
        }
    }

    fn sample_input() -> ModelProviderInput {
        ModelProviderInput {
            name: "智谱".into(),
            category: "preset".into(),
            api_format: "anthropic".into(),
            ..ModelProviderInput::default()
        }
    }

    fn sample_provider(owner: i32) -> ModelProvider {
        ModelProvider {
            id: Uuid::new_v4(),
            owner_id: owner,
            name: "智谱".into(),
            category: "preset".into(),
            preset_type: "zhipu".into(),
            base_url: String::new(),
            api_key: "sk".into(),
            api_format: "anthropic".into(),
            models: vec![],
            enabled: true,
            website_url: String::new(),
            api_key_url: String::new(),
            icon: String::new(),
            icon_color: String::new(),
            sort_order: 0,
        }
    }

    fn build_service(
        repo: MockModelProviderRepo,
        cache: MockModelProviderCache,
    ) -> ModelProviderServiceImpl {
        ModelProviderServiceImpl {
            repo: Arc::new(repo),
            cache: Arc::new(cache),
        }
    }

    #[tokio::test]
    async fn create_validates_name() {
        let mut repo = MockModelProviderRepo::new();
        repo.expect_insert().times(0);
        let cache = MockModelProviderCache::new();
        let svc = build_service(repo, cache);
        let mut input = sample_input();
        input.name = "".into();
        let err = svc.create(1, input).await.unwrap_err();
        assert!(matches!(err, ServiceError::Validation(_)));
    }

    #[tokio::test]
    async fn create_success_masks_key() {
        let p = sample_provider(1);
        let p_clone = p.clone();
        let mut repo = MockModelProviderRepo::new();
        repo.expect_insert()
            .with(eq(1), always())
            .returning(move |_, _| Ok(p_clone.clone()));
        let mut cache = MockModelProviderCache::new();
        cache.expect_invalidate().return_const(());
        let svc = build_service(repo, cache);
        let dto = svc.create(1, sample_input()).await.unwrap();
        assert!(dto.has_api_key);
    }

    #[tokio::test]
    async fn update_non_owner_forbidden() {
        let p = sample_provider(1);
        let mut repo = MockModelProviderRepo::new();
        repo.expect_find_by_id()
            .returning(move |_| Ok(Some(p.clone())));
        repo.expect_update().times(0);
        let cache = MockModelProviderCache::new();
        let svc = build_service(repo, cache);
        let err = svc
            .update(2, Uuid::new_v4(), sample_input())
            .await
            .unwrap_err();
        assert!(matches!(err, ServiceError::Forbidden));
    }

    #[tokio::test]
    async fn delete_owner_allowed() {
        let p = sample_provider(1);
        let mut repo = MockModelProviderRepo::new();
        repo.expect_find_by_id()
            .returning(move |_| Ok(Some(p.clone())));
        repo.expect_delete().returning(|_| Ok(()));
        let mut cache = MockModelProviderCache::new();
        cache.expect_invalidate().return_const(());
        let svc = build_service(repo, cache);
        svc.delete(1, Uuid::new_v4()).await.unwrap();
    }

    #[tokio::test]
    async fn resolve_for_runtime_cache_hit() {
        let p = sample_provider(1);
        let target_id = p.id;
        let mut repo = MockModelProviderRepo::new();
        repo.expect_list_by_owner().times(0); // 缓存命中不查 DB
        let mut cache = MockModelProviderCache::new();
        cache.expect_get().returning(move |_| Some(vec![p.clone()]));
        let svc = build_service(repo, cache);
        let found = svc.resolve_for_runtime(target_id, 1).await.unwrap();
        assert!(found.is_some());
    }

    #[tokio::test]
    async fn resolve_for_runtime_miss_falls_back_to_repo() {
        let p = sample_provider(1);
        let target_id = p.id;
        let mut repo = MockModelProviderRepo::new();
        repo.expect_list_by_owner()
            .returning(move |_| Ok(vec![p.clone()]));
        let mut cache = MockModelProviderCache::new();
        cache.expect_get().returning(|_| None);
        cache.expect_put().return_const(());
        let svc = build_service(repo, cache);
        let found = svc.resolve_for_runtime(target_id, 1).await.unwrap();
        assert!(found.is_some());
    }

    #[tokio::test]
    async fn create_unique_violation_maps_to_conflict() {
        use crate::domain::RepoError;
        let mut repo = MockModelProviderRepo::new();
        repo.expect_insert()
            .returning(|_, _| Err(RepoError::UniqueViolation));
        let cache = MockModelProviderCache::new();
        let svc = build_service(repo, cache);
        let err = svc.create(1, sample_input()).await.unwrap_err();
        assert!(matches!(err, ServiceError::Conflict(_)));
    }
}
