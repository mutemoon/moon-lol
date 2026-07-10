//! Agent 子系统的 service 层（跨子系统协作模板）。
//!
//! 编排：AgentRepo + AgentLimitProvider（取槽位上限，抽象自 SubscriptionService 避免循环依赖）。

use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::agent::{
    Agent, AgentInput, assert_within_slot_limit, fork_name, validate_champion, validate_name,
};
use crate::domain::spawn_preset::Visibility;
use crate::domain::{ServiceError, ServiceResult};
use crate::repository::agent_repo::AgentRepo;

/// 取 Agent 槽位上限的抽象（由 SubscriptionService 实现，避免 service 间循环依赖）。
#[async_trait]
pub trait AgentLimitProvider: Send + Sync {
    async fn get_agent_limit(&self, user_id: i32) -> ServiceResult<usize>;
}

#[async_trait]
pub trait AgentService: Send + Sync {
    async fn list(&self, owner_id: i32) -> ServiceResult<Vec<Agent>>;
    async fn get(&self, requester_id: i32, id: Uuid) -> ServiceResult<Agent>;
    async fn create(&self, owner_id: i32, input: AgentInput) -> ServiceResult<Agent>;
    async fn update(&self, owner_id: i32, id: Uuid, input: AgentInput) -> ServiceResult<()>;
    async fn update_visibility(
        &self,
        owner_id: i32,
        id: Uuid,
        vis: Visibility,
    ) -> ServiceResult<()>;
    async fn delete(&self, owner_id: i32, id: Uuid) -> ServiceResult<()>;
    async fn fork(
        &self,
        requester_id: i32,
        source_id: Uuid,
        new_name: Option<String>,
    ) -> ServiceResult<Agent>;
}

pub struct AgentServiceImpl {
    pub repo: Arc<dyn AgentRepo>,
    pub limit_provider: Arc<dyn AgentLimitProvider>,
}

impl AgentServiceImpl {
    pub fn new(repo: Arc<dyn AgentRepo>, limit_provider: Arc<dyn AgentLimitProvider>) -> Self {
        Self {
            repo,
            limit_provider,
        }
    }

    fn validate_input(input: &AgentInput) -> ServiceResult<()> {
        if !validate_name(&input.name) {
            return Err(ServiceError::Validation(
                "名称不能为空且不超过 64 字符".into(),
            ));
        }
        if !validate_champion(&input.champion) {
            return Err(ServiceError::Validation(
                "英雄名不能为空且不超过 32 字符".into(),
            ));
        }
        Ok(())
    }
}

#[async_trait]
impl AgentService for AgentServiceImpl {
    async fn list(&self, owner_id: i32) -> ServiceResult<Vec<Agent>> {
        Ok(self.repo.list_by_owner(owner_id).await?)
    }

    async fn get(&self, requester_id: i32, id: Uuid) -> ServiceResult<Agent> {
        let agent = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or(ServiceError::NotFound)?;
        if !crate::domain::agent::can_view(&agent, requester_id) {
            return Err(ServiceError::NotFound);
        }
        Ok(agent)
    }

    async fn create(&self, owner_id: i32, input: AgentInput) -> ServiceResult<Agent> {
        Self::validate_input(&input)?;

        // 槽位限制
        let limit = self.limit_provider.get_agent_limit(owner_id).await?;
        let current = self.repo.count_by_owner(owner_id).await? as usize;
        if let Err(e) = assert_within_slot_limit(current, limit) {
            return Err(ServiceError::AgentSlotLimit {
                current: e.current,
                limit: e.limit,
            });
        }

        Ok(self.repo.insert(owner_id, &input).await?)
    }

    async fn update(&self, owner_id: i32, id: Uuid, input: AgentInput) -> ServiceResult<()> {
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

    async fn update_visibility(
        &self,
        owner_id: i32,
        id: Uuid,
        vis: Visibility,
    ) -> ServiceResult<()> {
        let existing = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or(ServiceError::NotFound)?;
        if existing.owner_id != owner_id {
            return Err(ServiceError::Forbidden);
        }
        self.repo.update_visibility(id, vis).await?;
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

    async fn fork(
        &self,
        requester_id: i32,
        source_id: Uuid,
        new_name: Option<String>,
    ) -> ServiceResult<Agent> {
        let source = self
            .repo
            .find_by_id(source_id)
            .await?
            .ok_or(ServiceError::NotFound)?;
        if !matches!(source.visibility, Visibility::Public | Visibility::Friends) {
            return Err(ServiceError::Forbidden);
        }

        // 槽位限制
        let limit = self.limit_provider.get_agent_limit(requester_id).await?;
        let current = self.repo.count_by_owner(requester_id).await? as usize;
        if let Err(e) = assert_within_slot_limit(current, limit) {
            return Err(ServiceError::AgentSlotLimit {
                current: e.current,
                limit: e.limit,
            });
        }

        let name = new_name.unwrap_or_else(|| fork_name(&source.name));

        let input = AgentInput {
            name,
            champion: source.champion.clone(),
            agent_type: source.agent_type,
            prompt: source.prompt.clone(),
            model: source.model.clone(),
            config_json: source.config_json.clone(),
            visibility: Visibility::Private,
        };
        let forked = self.repo.insert(requester_id, &input).await?;
        Ok(forked)
    }
}

#[cfg(test)]
mod tests {
    use mockall::mock;
    use mockall::predicate::*;

    use super::*;
    use crate::domain::RepoResult;
    use crate::domain::agent::{Agent, AgentInput, AgentType};
    use crate::domain::spawn_preset::Visibility;

    mock! {
        pub AgentRepo {}
        #[async_trait]
        impl AgentRepo for AgentRepo {
            async fn find_by_id(&self, id: Uuid) -> RepoResult<Option<Agent>>;
            async fn list_by_owner(&self, owner_id: i32) -> RepoResult<Vec<Agent>>;
            async fn list_public(&self) -> RepoResult<Vec<Agent>>;
            async fn find_by_id_with_owner_check(&self, id: Uuid, owner_id: i32) -> RepoResult<Option<Agent>>;
            async fn insert(&self, owner_id: i32, input: &AgentInput) -> RepoResult<Agent>;
            async fn update(&self, id: Uuid, input: &AgentInput) -> RepoResult<()>;
            async fn update_visibility(&self, id: Uuid, visibility: Visibility) -> RepoResult<()>;
            async fn set_fork_linkage(&self, id: Uuid, forked_from: Option<Uuid>, upstream: Option<Uuid>) -> RepoResult<()>;
            async fn delete(&self, id: Uuid) -> RepoResult<()>;
            async fn count_by_owner(&self, owner_id: i32) -> RepoResult<i64>;
        }
    }

    mock! {
        pub LimitProvider {}
        #[async_trait]
        impl AgentLimitProvider for LimitProvider {
            async fn get_agent_limit(&self, user_id: i32) -> ServiceResult<usize>;
        }
    }

    fn build_service(repo: MockAgentRepo, limit: MockLimitProvider) -> AgentServiceImpl {
        AgentServiceImpl {
            repo: Arc::new(repo),
            limit_provider: Arc::new(limit),
        }
    }

    fn sample_input() -> AgentInput {
        AgentInput {
            name: "锐雯 · 激进".into(),
            champion: "Riven".into(),
            agent_type: AgentType::Llm,
            prompt: "aggressive".into(),
            model: "gemini".into(),
            config_json: serde_json::json!({}),
            visibility: Visibility::Private,
        }
    }

    fn sample_agent(owner: i32) -> Agent {
        Agent {
            id: Uuid::new_v4(),
            owner_id: owner,
            name: "锐雯 · 激进".into(),
            champion: "Riven".into(),
            agent_type: AgentType::Llm,
            prompt: "aggressive".into(),
            model: "gemini".into(),
            config_json: serde_json::json!({}),
            visibility: Visibility::Private,
            forked_from: None,
            upstream_agent_id: None,
        }
    }

    // ── create ──
    #[tokio::test]
    async fn create_success() {
        let input = sample_input();
        let mut repo = MockAgentRepo::new();
        repo.expect_count_by_owner().returning(|_| Ok(2));
        repo.expect_insert()
            .returning(|owner, _| Ok(sample_agent(owner)));

        let mut limit = MockLimitProvider::new();
        limit.expect_get_agent_limit().returning(|_| Ok(5));

        let svc = build_service(repo, limit);
        let result = svc.create(1, input).await.unwrap();
        assert_eq!(result.owner_id, 1);
    }

    #[tokio::test]
    async fn create_validates_name() {
        let mut input = sample_input();
        input.name = "".into();
        let svc = build_service(MockAgentRepo::new(), MockLimitProvider::new());
        let err = svc.create(1, input).await.unwrap_err();
        assert!(matches!(err, ServiceError::Validation(_)));
    }

    #[tokio::test]
    async fn create_at_slot_limit_rejected() {
        let mut repo = MockAgentRepo::new();
        repo.expect_count_by_owner().returning(|_| Ok(5));
        repo.expect_insert().times(0);

        let mut limit = MockLimitProvider::new();
        limit.expect_get_agent_limit().returning(|_| Ok(5));

        let svc = build_service(repo, limit);
        let err = svc.create(1, sample_input()).await.unwrap_err();
        assert!(matches!(err, ServiceError::AgentSlotLimit { .. }));
    }

    // ── get 可见性 ──
    #[tokio::test]
    async fn get_owner_allowed() {
        let agent = sample_agent(1);
        let agent_clone = agent.clone();
        let mut repo = MockAgentRepo::new();
        repo.expect_find_by_id()
            .returning(move |_| Ok(Some(agent_clone.clone())));
        let svc = build_service(repo, MockLimitProvider::new());
        svc.get(1, Uuid::new_v4()).await.unwrap();
    }

    #[tokio::test]
    async fn get_non_owner_private_not_found() {
        let agent = sample_agent(1);
        let mut repo = MockAgentRepo::new();
        repo.expect_find_by_id()
            .returning(move |_| Ok(Some(agent.clone())));
        let svc = build_service(repo, MockLimitProvider::new());
        let err = svc.get(2, Uuid::new_v4()).await.unwrap_err();
        assert!(matches!(err, ServiceError::NotFound));
    }

    #[tokio::test]
    async fn get_missing_not_found() {
        let mut repo = MockAgentRepo::new();
        repo.expect_find_by_id().returning(|_| Ok(None));
        let svc = build_service(repo, MockLimitProvider::new());
        let err = svc.get(1, Uuid::new_v4()).await.unwrap_err();
        assert!(matches!(err, ServiceError::NotFound));
    }

    // ── update / delete 权限 ──
    #[tokio::test]
    async fn update_non_owner_forbidden() {
        let agent = sample_agent(1);
        let mut repo = MockAgentRepo::new();
        repo.expect_find_by_id()
            .returning(move |_| Ok(Some(agent.clone())));
        repo.expect_update().times(0);
        let svc = build_service(repo, MockLimitProvider::new());
        let err = svc
            .update(2, Uuid::new_v4(), sample_input())
            .await
            .unwrap_err();
        assert!(matches!(err, ServiceError::Forbidden));
    }

    #[tokio::test]
    async fn delete_non_owner_forbidden() {
        let agent = sample_agent(1);
        let mut repo = MockAgentRepo::new();
        repo.expect_find_by_id()
            .returning(move |_| Ok(Some(agent.clone())));
        repo.expect_delete().times(0);
        let svc = build_service(repo, MockLimitProvider::new());
        let err = svc.delete(2, Uuid::new_v4()).await.unwrap_err();
        assert!(matches!(err, ServiceError::Forbidden));
    }

    #[tokio::test]
    async fn update_visibility_owner_allowed() {
        let agent = sample_agent(1);
        let mut repo = MockAgentRepo::new();
        repo.expect_find_by_id()
            .returning(move |_| Ok(Some(agent.clone())));
        repo.expect_update_visibility().returning(|_, _| Ok(()));
        let svc = build_service(repo, MockLimitProvider::new());
        svc.update_visibility(1, Uuid::new_v4(), Visibility::Public)
            .await
            .unwrap();
    }
}
