//! Scenario 子系统的 service 层。

use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::scenario::{Scenario, ScenarioInput, validate_name};
use crate::domain::{ServiceError, ServiceResult};
use crate::repository::scenario_repo::ScenarioRepo;

#[async_trait]
pub trait ScenarioService: Send + Sync {
    async fn list(&self, owner_id: i32) -> ServiceResult<Vec<Scenario>>;
    async fn get(&self, owner_id: i32, id: Uuid) -> ServiceResult<Scenario>;
    async fn create(&self, owner_id: i32, input: ScenarioInput) -> ServiceResult<Scenario>;
    async fn update(&self, owner_id: i32, id: Uuid, input: ScenarioInput) -> ServiceResult<()>;
    async fn delete(&self, owner_id: i32, id: Uuid) -> ServiceResult<()>;
    async fn get_win_condition(
        &self,
        owner_id: i32,
        id: Uuid,
    ) -> ServiceResult<Option<serde_json::Value>>;
    async fn save_win_condition(
        &self,
        owner_id: i32,
        id: Uuid,
        condition: serde_json::Value,
    ) -> ServiceResult<()>;
}

pub struct ScenarioServiceImpl {
    pub repo: Arc<dyn ScenarioRepo>,
}

impl ScenarioServiceImpl {
    pub fn new(repo: Arc<dyn ScenarioRepo>) -> Self {
        Self { repo }
    }

    fn validate_input(input: &ScenarioInput) -> ServiceResult<()> {
        if !validate_name(&input.name) {
            return Err(ServiceError::Validation(
                "名称不能为空且不超过 64 字符".into(),
            ));
        }
        Ok(())
    }

    async fn owned(&self, owner_id: i32, id: Uuid) -> ServiceResult<Scenario> {
        let s = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or(ServiceError::NotFound)?;
        if s.owner_id != owner_id {
            return Err(ServiceError::Forbidden);
        }
        Ok(s)
    }
}

#[async_trait]
impl ScenarioService for ScenarioServiceImpl {
    async fn list(&self, owner_id: i32) -> ServiceResult<Vec<Scenario>> {
        Ok(self.repo.list_by_owner(owner_id).await?)
    }

    async fn get(&self, owner_id: i32, id: Uuid) -> ServiceResult<Scenario> {
        let s = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or(ServiceError::NotFound)?;
        if s.owner_id != owner_id {
            return Err(ServiceError::NotFound);
        }
        Ok(s)
    }

    async fn create(&self, owner_id: i32, input: ScenarioInput) -> ServiceResult<Scenario> {
        Self::validate_input(&input)?;
        Ok(self.repo.insert(owner_id, &input).await?)
    }

    async fn update(&self, owner_id: i32, id: Uuid, input: ScenarioInput) -> ServiceResult<()> {
        Self::validate_input(&input)?;
        self.owned(owner_id, id).await?;
        self.repo.update(id, &input).await?;
        Ok(())
    }

    async fn delete(&self, owner_id: i32, id: Uuid) -> ServiceResult<()> {
        self.owned(owner_id, id).await?;
        self.repo.delete(id).await?;
        Ok(())
    }

    async fn get_win_condition(
        &self,
        owner_id: i32,
        id: Uuid,
    ) -> ServiceResult<Option<serde_json::Value>> {
        self.owned(owner_id, id).await?;
        Ok(self.repo.get_win_condition(owner_id, id).await?)
    }

    async fn save_win_condition(
        &self,
        owner_id: i32,
        id: Uuid,
        condition: serde_json::Value,
    ) -> ServiceResult<()> {
        self.owned(owner_id, id).await?;
        self.repo
            .save_win_condition(owner_id, id, &condition)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::RepoResult;
    use chrono::Utc;
    use mockall::mock;
    use mockall::predicate::*;

    mock! {
        pub ScenarioRepo {}
        #[async_trait]
        impl ScenarioRepo for ScenarioRepo {
            async fn find_by_id(&self, id: Uuid) -> RepoResult<Option<Scenario>>;
            async fn list_by_owner(&self, owner_id: i32) -> RepoResult<Vec<Scenario>>;
            async fn insert(&self, owner_id: i32, input: &ScenarioInput) -> RepoResult<Scenario>;
            async fn update(&self, id: Uuid, input: &ScenarioInput) -> RepoResult<()>;
            async fn delete(&self, id: Uuid) -> RepoResult<()>;
            async fn get_win_condition(&self, owner_id: i32, scenario_id: Uuid) -> RepoResult<Option<serde_json::Value>>;
            async fn save_win_condition(&self, owner_id: i32, scenario_id: Uuid, condition: &serde_json::Value) -> RepoResult<()>;
        }
    }

    fn build_service(repo: MockScenarioRepo) -> ScenarioServiceImpl {
        ScenarioServiceImpl {
            repo: Arc::new(repo),
        }
    }

    fn sample_input() -> ScenarioInput {
        ScenarioInput {
            name: "5v5 激进".into(),
            agents: serde_json::json!([{"champion": "Riven"}]),
        }
    }

    fn sample_scenario(owner: i32) -> Scenario {
        Scenario {
            id: Uuid::new_v4(),
            owner_id: owner,
            name: "5v5 激进".into(),
            agents: serde_json::json!([{"champion": "Riven"}]),
            created_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn create_success() {
        let mut repo = MockScenarioRepo::new();
        repo.expect_insert()
            .with(eq(1), always())
            .returning(|owner, _| Ok(sample_scenario(owner)));
        let svc = build_service(repo);
        assert_eq!(svc.create(1, sample_input()).await.unwrap().owner_id, 1);
    }

    #[tokio::test]
    async fn create_validates_name() {
        let mut repo = MockScenarioRepo::new();
        repo.expect_insert().times(0);
        let mut input = sample_input();
        input.name = "   ".into();
        let svc = build_service(repo);
        let err = svc.create(1, input).await.unwrap_err();
        assert!(matches!(err, ServiceError::Validation(_)));
    }

    #[tokio::test]
    async fn get_non_owner_not_found() {
        let mut repo = MockScenarioRepo::new();
        repo.expect_find_by_id()
            .returning(|_| Ok(Some(sample_scenario(1))));
        let svc = build_service(repo);
        let err = svc.get(2, Uuid::new_v4()).await.unwrap_err();
        assert!(matches!(err, ServiceError::NotFound));
    }

    #[tokio::test]
    async fn get_missing_not_found() {
        let mut repo = MockScenarioRepo::new();
        repo.expect_find_by_id().returning(|_| Ok(None));
        let svc = build_service(repo);
        let err = svc.get(1, Uuid::new_v4()).await.unwrap_err();
        assert!(matches!(err, ServiceError::NotFound));
    }

    #[tokio::test]
    async fn update_non_owner_forbidden() {
        let mut repo = MockScenarioRepo::new();
        repo.expect_find_by_id()
            .returning(|_| Ok(Some(sample_scenario(1))));
        repo.expect_update().times(0);
        let svc = build_service(repo);
        let err = svc
            .update(2, Uuid::new_v4(), sample_input())
            .await
            .unwrap_err();
        assert!(matches!(err, ServiceError::Forbidden));
    }

    #[tokio::test]
    async fn delete_non_owner_forbidden() {
        let mut repo = MockScenarioRepo::new();
        repo.expect_find_by_id()
            .returning(|_| Ok(Some(sample_scenario(1))));
        repo.expect_delete().times(0);
        let svc = build_service(repo);
        let err = svc.delete(2, Uuid::new_v4()).await.unwrap_err();
        assert!(matches!(err, ServiceError::Forbidden));
    }

    #[tokio::test]
    async fn update_owner_ok() {
        let mut repo = MockScenarioRepo::new();
        repo.expect_find_by_id()
            .returning(|_| Ok(Some(sample_scenario(1))));
        repo.expect_update().returning(|_, _| Ok(()));
        let svc = build_service(repo);
        svc.update(1, Uuid::new_v4(), sample_input()).await.unwrap();
    }

    #[tokio::test]
    async fn delete_owner_ok() {
        let mut repo = MockScenarioRepo::new();
        repo.expect_find_by_id()
            .returning(|_| Ok(Some(sample_scenario(1))));
        repo.expect_delete().returning(|_| Ok(()));
        let svc = build_service(repo);
        svc.delete(1, Uuid::new_v4()).await.unwrap();
    }

    #[tokio::test]
    async fn get_win_condition_owner_ok() {
        let cond = serde_json::json!({"type": "kill", "threshold": 10});
        let cond_clone = cond.clone();
        let mut repo = MockScenarioRepo::new();
        repo.expect_find_by_id()
            .returning(|_| Ok(Some(sample_scenario(1))));
        repo.expect_get_win_condition()
            .returning(move |_, _| Ok(Some(cond_clone.clone())));
        let svc = build_service(repo);
        let result = svc.get_win_condition(1, Uuid::new_v4()).await.unwrap();
        assert_eq!(result, Some(cond));
    }

    #[tokio::test]
    async fn save_win_condition_non_owner_forbidden() {
        let mut repo = MockScenarioRepo::new();
        repo.expect_find_by_id()
            .returning(|_| Ok(Some(sample_scenario(1))));
        repo.expect_save_win_condition().times(0);
        let svc = build_service(repo);
        let err = svc
            .save_win_condition(2, Uuid::new_v4(), serde_json::json!({}))
            .await
            .unwrap_err();
        assert!(matches!(err, ServiceError::Forbidden));
    }
}
