//! SpawnPreset 子系统的 service 层。

use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::spawn_preset::{SpawnPreset, SpawnPresetInput, validate_coord, validate_name};
use crate::domain::{ServiceError, ServiceResult};
use crate::repository::spawn_preset_repo::SpawnPresetRepo;

#[async_trait]
pub trait SpawnPresetService: Send + Sync {
    async fn list(&self, owner_id: i32) -> ServiceResult<Vec<SpawnPreset>>;
    async fn get(&self, requester_id: i32, id: Uuid) -> ServiceResult<SpawnPreset>;
    async fn create(&self, owner_id: i32, input: SpawnPresetInput) -> ServiceResult<SpawnPreset>;
    async fn update(&self, owner_id: i32, id: Uuid, input: SpawnPresetInput) -> ServiceResult<()>;
    async fn delete(&self, owner_id: i32, id: Uuid) -> ServiceResult<()>;
}

pub struct SpawnPresetServiceImpl {
    pub repo: Arc<dyn SpawnPresetRepo>,
}

impl SpawnPresetServiceImpl {
    pub fn new(repo: Arc<dyn SpawnPresetRepo>) -> Self {
        Self { repo }
    }

    fn validate_input(input: &SpawnPresetInput) -> ServiceResult<()> {
        if !validate_name(&input.name) {
            return Err(ServiceError::Validation(
                "名称不能为空且不超过 64 字符".into(),
            ));
        }
        if !validate_coord(input.x, input.z) {
            return Err(ServiceError::Validation(format!(
                "坐标 ({}, {}) 超出地图范围 [0, 15000]",
                input.x, input.z
            )));
        }
        Ok(())
    }
}

#[async_trait]
impl SpawnPresetService for SpawnPresetServiceImpl {
    async fn list(&self, owner_id: i32) -> ServiceResult<Vec<SpawnPreset>> {
        Ok(self.repo.list_by_owner(owner_id).await?)
    }

    async fn get(&self, requester_id: i32, id: Uuid) -> ServiceResult<SpawnPreset> {
        let preset = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or(ServiceError::NotFound)?;
        // 仅 owner 可查看（friend 系统未实现前，private/friends 同等对待）
        if preset.owner_id != requester_id
            && preset.visibility != crate::domain::spawn_preset::Visibility::Public
        {
            return Err(ServiceError::Forbidden);
        }
        Ok(preset)
    }

    async fn create(&self, owner_id: i32, input: SpawnPresetInput) -> ServiceResult<SpawnPreset> {
        Self::validate_input(&input)?;
        Ok(self.repo.insert(owner_id, &input).await?)
    }

    async fn update(&self, owner_id: i32, id: Uuid, input: SpawnPresetInput) -> ServiceResult<()> {
        Self::validate_input(&input)?;
        let existing = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or(ServiceError::NotFound)?;
        if existing.owner_id != owner_id {
            return Err(ServiceError::Forbidden);
        }
        self.repo.update(id, &input).await?;
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
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::RepoResult;
    use crate::domain::spawn_preset::{Team, Visibility};
    use mockall::mock;
    use mockall::predicate::*;

    mock! {
        pub SpawnPresetRepo {}
        #[async_trait]
        impl SpawnPresetRepo for SpawnPresetRepo {
            async fn find_by_id(&self, id: Uuid) -> RepoResult<Option<SpawnPreset>>;
            async fn list_by_owner(&self, owner_id: i32) -> RepoResult<Vec<SpawnPreset>>;
            async fn insert(&self, owner_id: i32, input: &SpawnPresetInput) -> RepoResult<SpawnPreset>;
            async fn update(&self, id: Uuid, input: &SpawnPresetInput) -> RepoResult<()>;
            async fn delete(&self, id: Uuid) -> RepoResult<()>;
        }
    }

    fn sample_input() -> SpawnPresetInput {
        SpawnPresetInput {
            name: "上路一塔".into(),
            x: 1500.0,
            z: 13000.0,
            team: Team::Order,
            visibility: Visibility::Private,
        }
    }

    fn sample_preset(owner: i32) -> SpawnPreset {
        SpawnPreset {
            id: Uuid::new_v4(),
            owner_id: owner,
            name: "上路一塔".into(),
            x: 1500.0,
            z: 13000.0,
            team: Team::Order,
            visibility: Visibility::Private,
        }
    }

    fn build_service(repo: MockSpawnPresetRepo) -> SpawnPresetServiceImpl {
        SpawnPresetServiceImpl {
            repo: Arc::new(repo),
        }
    }

    #[tokio::test]
    async fn create_validates_name() {
        let mut repo = MockSpawnPresetRepo::new();
        repo.expect_insert().times(0);
        let svc = build_service(repo);
        let mut input = sample_input();
        input.name = "".into();
        let err = svc.create(1, input).await.unwrap_err();
        assert!(matches!(err, ServiceError::Validation(_)));
    }

    #[tokio::test]
    async fn create_validates_coord() {
        let mut repo = MockSpawnPresetRepo::new();
        repo.expect_insert().times(0);
        let svc = build_service(repo);
        let mut input = sample_input();
        input.x = -100.0;
        let err = svc.create(1, input).await.unwrap_err();
        assert!(matches!(err, ServiceError::Validation(_)));
    }

    #[tokio::test]
    async fn create_success() {
        let preset = sample_preset(1);
        let preset_clone = preset.clone();
        let mut repo = MockSpawnPresetRepo::new();
        repo.expect_insert()
            .with(eq(1), always())
            .returning(move |_, _| Ok(preset_clone.clone()));
        let svc = build_service(repo);
        let result = svc.create(1, sample_input()).await.unwrap();
        assert_eq!(result.owner_id, 1);
    }

    #[tokio::test]
    async fn get_owner_allowed() {
        let preset = sample_preset(1);
        let preset_clone = preset.clone();
        let mut repo = MockSpawnPresetRepo::new();
        repo.expect_find_by_id()
            .returning(move |_| Ok(Some(preset_clone.clone())));
        let svc = build_service(repo);
        let result = svc.get(1, Uuid::new_v4()).await.unwrap();
        assert_eq!(result.owner_id, 1);
    }

    #[tokio::test]
    async fn get_other_user_private_forbidden() {
        let preset = sample_preset(1);
        let mut repo = MockSpawnPresetRepo::new();
        repo.expect_find_by_id()
            .returning(move |_| Ok(Some(preset.clone())));
        let svc = build_service(repo);
        let err = svc.get(2, Uuid::new_v4()).await.unwrap_err();
        assert!(matches!(err, ServiceError::Forbidden));
    }

    #[tokio::test]
    async fn get_other_user_public_allowed() {
        let mut preset = sample_preset(1);
        preset.visibility = Visibility::Public;
        let preset_clone = preset.clone();
        let mut repo = MockSpawnPresetRepo::new();
        repo.expect_find_by_id()
            .returning(move |_| Ok(Some(preset_clone.clone())));
        let svc = build_service(repo);
        svc.get(2, Uuid::new_v4()).await.unwrap();
    }

    #[tokio::test]
    async fn get_missing_returns_not_found() {
        let mut repo = MockSpawnPresetRepo::new();
        repo.expect_find_by_id().returning(|_| Ok(None));
        let svc = build_service(repo);
        let err = svc.get(1, Uuid::new_v4()).await.unwrap_err();
        assert!(matches!(err, ServiceError::NotFound));
    }

    #[tokio::test]
    async fn update_non_owner_forbidden() {
        let preset = sample_preset(1);
        let mut repo = MockSpawnPresetRepo::new();
        repo.expect_find_by_id()
            .returning(move |_| Ok(Some(preset.clone())));
        repo.expect_update().times(0);
        let svc = build_service(repo);
        let err = svc
            .update(2, Uuid::new_v4(), sample_input())
            .await
            .unwrap_err();
        assert!(matches!(err, ServiceError::Forbidden));
    }

    #[tokio::test]
    async fn delete_owner_allowed() {
        let preset = sample_preset(1);
        let mut repo = MockSpawnPresetRepo::new();
        repo.expect_find_by_id()
            .returning(move |_| Ok(Some(preset.clone())));
        repo.expect_delete().with(always()).returning(|_| Ok(()));
        let svc = build_service(repo);
        svc.delete(1, Uuid::new_v4()).await.unwrap();
    }

    #[tokio::test]
    async fn delete_non_owner_forbidden() {
        let preset = sample_preset(1);
        let mut repo = MockSpawnPresetRepo::new();
        repo.expect_find_by_id()
            .returning(move |_| Ok(Some(preset.clone())));
        repo.expect_delete().times(0);
        let svc = build_service(repo);
        let err = svc.delete(2, Uuid::new_v4()).await.unwrap_err();
        assert!(matches!(err, ServiceError::Forbidden));
    }
}
