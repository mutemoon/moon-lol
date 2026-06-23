//! Community 子系统的 service 层（Fork + 社区浏览）。
//!
//! 复用 AgentRepo / AgentConfigRepo，不新建表。

use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::agent::{Agent, AgentInput, fork_name};
use crate::domain::community::{CommunitySort, can_fork, resolve_fork_name};
use crate::domain::spawn_preset::Visibility;
use crate::domain::{ServiceError, ServiceResult};
use crate::repository::agent_repo::AgentRepo;

#[async_trait]
pub trait CommunityService: Send + Sync {
    /// 浏览公开 Agent 列表。
    async fn browse_public(&self, sort: CommunitySort, limit: i64) -> ServiceResult<Vec<Agent>>;

    /// Fork 一个公开/好友 Agent 为自己的副本。
    async fn fork(
        &self,
        requester_id: i32,
        source_id: Uuid,
        new_name: Option<String>,
    ) -> ServiceResult<Agent>;
}

pub struct CommunityServiceImpl {
    pub agent_repo: Arc<dyn AgentRepo>,
}

impl CommunityServiceImpl {
    pub fn new(agent_repo: Arc<dyn AgentRepo>) -> Self {
        Self { agent_repo }
    }
}

#[async_trait]
impl CommunityService for CommunityServiceImpl {
    async fn browse_public(&self, _sort: CommunitySort, limit: i64) -> ServiceResult<Vec<Agent>> {
        // 简化：list_public 已按 name 排序；sort 维度的细化（rating/forks）留待后续
        let limit = limit.clamp(1, 100);
        let agents = self.agent_repo.list_public().await?;
        Ok(agents.into_iter().take(limit as usize).collect())
    }

    async fn fork(
        &self,
        requester_id: i32,
        source_id: Uuid,
        new_name: Option<String>,
    ) -> ServiceResult<Agent> {
        let source = self
            .agent_repo
            .find_by_id(source_id)
            .await?
            .ok_or(ServiceError::NotFound)?;

        // 校验可 Fork（非自己 + public/friends）
        if !can_fork(source.visibility, source.owner_id, requester_id) {
            return Err(ServiceError::Forbidden);
        }

        let name = resolve_fork_name(new_name.as_deref(), &source.name);

        // 创建 Fork 副本：引用同一 config/spawn，visibility 默认 private
        let input = AgentInput {
            name,
            champion: source.champion.clone(),
            agent_config_id: source.agent_config_id,
            spawn_preset_id: source.spawn_preset_id,
            visibility: Visibility::Private,
        };
        // 注意：agent_config 归属属于 source owner，create 时会校验归属失败。
        // Fork 路径跳过归属校验，直接 insert（forked_from/upstream 字段后续通过 repo 补充）。
        let forked = self.agent_repo.insert(requester_id, &input).await?;
        Ok(forked)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::RepoResult;
    use crate::domain::agent::Agent;
    use mockall::mock;
    use mockall::predicate::*;
    use uuid::Uuid;

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
            async fn delete(&self, id: Uuid) -> RepoResult<()>;
            async fn count_by_owner(&self, owner_id: i32) -> RepoResult<i64>;
        }
    }

    fn sample_agent(owner: i32, vis: Visibility) -> Agent {
        Agent {
            id: Uuid::new_v4(),
            owner_id: owner,
            name: "锐雯".into(),
            champion: "Riven".into(),
            agent_config_id: Uuid::new_v4(),
            spawn_preset_id: None,
            visibility: vis,
            forked_from: None,
            upstream_agent_id: None,
        }
    }

    fn build_service(repo: MockAgentRepo) -> CommunityServiceImpl {
        CommunityServiceImpl {
            agent_repo: Arc::new(repo),
        }
    }

    #[tokio::test]
    async fn browse_public_returns_list() {
        let mut repo = MockAgentRepo::new();
        repo.expect_list_public().returning(|| {
            Ok(vec![
                sample_agent(1, Visibility::Public),
                sample_agent(2, Visibility::Public),
            ])
        });
        let svc = build_service(repo);
        let agents = svc.browse_public(CommunitySort::Recent, 10).await.unwrap();
        assert_eq!(agents.len(), 2);
    }

    #[tokio::test]
    async fn browse_public_limit_clamped() {
        let mut repo = MockAgentRepo::new();
        repo.expect_list_public()
            .returning(|| Ok(vec![sample_agent(1, Visibility::Public)]));
        let svc = build_service(repo);
        let agents = svc.browse_public(CommunitySort::Recent, 0).await.unwrap();
        assert_eq!(agents.len(), 1); // clamp(1,100) → 1
    }

    #[tokio::test]
    async fn fork_public_agent_success() {
        let source = sample_agent(1, Visibility::Public);
        let source_clone = source.clone();
        let mut repo = MockAgentRepo::new();
        repo.expect_find_by_id()
            .returning(move |_| Ok(Some(source_clone.clone())));
        repo.expect_insert()
            .returning(|owner, _| Ok(sample_agent(owner, Visibility::Private)));
        let svc = build_service(repo);
        let forked = svc
            .fork(2, source.id, Some("我的锐雯".into()))
            .await
            .unwrap();
        assert_eq!(forked.owner_id, 2);
    }

    #[tokio::test]
    async fn fork_own_agent_forbidden() {
        let source = sample_agent(1, Visibility::Public);
        let source_clone = source.clone();
        let mut repo = MockAgentRepo::new();
        repo.expect_find_by_id()
            .returning(move |_| Ok(Some(source_clone.clone())));
        repo.expect_insert().times(0);
        let svc = build_service(repo);
        let err = svc.fork(1, source.id, None).await.unwrap_err();
        assert!(matches!(err, ServiceError::Forbidden));
    }

    #[tokio::test]
    async fn fork_private_agent_forbidden() {
        let source = sample_agent(1, Visibility::Private);
        let source_clone = source.clone();
        let mut repo = MockAgentRepo::new();
        repo.expect_find_by_id()
            .returning(move |_| Ok(Some(source_clone.clone())));
        repo.expect_insert().times(0);
        let svc = build_service(repo);
        let err = svc.fork(2, source.id, None).await.unwrap_err();
        assert!(matches!(err, ServiceError::Forbidden));
    }

    #[tokio::test]
    async fn fork_missing_agent_not_found() {
        let mut repo = MockAgentRepo::new();
        repo.expect_find_by_id().returning(|_| Ok(None));
        repo.expect_insert().times(0);
        let svc = build_service(repo);
        let err = svc.fork(2, Uuid::new_v4(), None).await.unwrap_err();
        assert!(matches!(err, ServiceError::NotFound));
    }

    #[tokio::test]
    async fn fork_friends_visible_allowed() {
        let source = sample_agent(1, Visibility::Friends);
        let source_clone = source.clone();
        let mut repo = MockAgentRepo::new();
        repo.expect_find_by_id()
            .returning(move |_| Ok(Some(source_clone.clone())));
        repo.expect_insert()
            .returning(|owner, _| Ok(sample_agent(owner, Visibility::Private)));
        let svc = build_service(repo);
        svc.fork(2, source.id, None).await.unwrap();
    }
}
