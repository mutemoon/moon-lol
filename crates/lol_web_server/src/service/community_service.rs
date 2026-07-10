//! Community 子系统的 service 层（Fork + 社区浏览）。
//!
//! 复用 AgentRepo / AgentConfigRepo，不新建表。

use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::agent::{Agent, AgentInput, can_view};
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

    /// 从上游拉取最新策略，覆盖当前 Fork 副本的编辑态（保留自己的名称与可见性）。
    /// 拉取后视为「待发布」改动，需用户重新发布快照才在 Rank 生效。
    async fn pull_upstream(&self, requester_id: i32, id: Uuid) -> ServiceResult<Agent>;
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

        // 创建 Fork 副本：拷贝策略配置，visibility 默认 private
        let input = AgentInput {
            name,
            champion: source.champion.clone(),
            agent_type: source.agent_type,
            prompt: source.prompt.clone(),
            model: source.model.clone(),
            config_json: source.config_json.clone(),
            visibility: Visibility::Private,
        };
        // 注意：agent_config 归属属于 source owner，create 时会校验归属失败。
        // Fork 路径跳过归属校验，直接 insert，随后补写 forked_from / upstream 溯源关系。
        let mut forked = self.agent_repo.insert(requester_id, &input).await?;
        self.agent_repo
            .set_fork_linkage(forked.id, Some(source.id), Some(source.id))
            .await?;
        forked.forked_from = Some(source.id);
        forked.upstream_agent_id = Some(source.id);
        Ok(forked)
    }

    async fn pull_upstream(&self, requester_id: i32, id: Uuid) -> ServiceResult<Agent> {
        let agent = self
            .agent_repo
            .find_by_id(id)
            .await?
            .ok_or(ServiceError::NotFound)?;
        if agent.owner_id != requester_id {
            return Err(ServiceError::Forbidden);
        }

        let upstream_id = agent.upstream_agent_id.ok_or_else(|| {
            ServiceError::Validation("该选手不是 Fork 副本，没有可拉取的上游".into())
        })?;
        let upstream = self
            .agent_repo
            .find_by_id(upstream_id)
            .await?
            .ok_or(ServiceError::NotFound)?;
        // 上游若已转为私有则不再允许拉取（可见性即权限）。
        if !can_view(&upstream, requester_id) {
            return Err(ServiceError::Forbidden);
        }

        // 用上游策略覆盖编辑态，保留自己的名称与可见性；update 会刷新 updated_at，
        // 因此前端的「未发布改动」指示会随之点亮，提醒重新发布快照。
        let input = AgentInput {
            name: agent.name.clone(),
            champion: upstream.champion.clone(),
            agent_type: upstream.agent_type,
            prompt: upstream.prompt.clone(),
            model: upstream.model.clone(),
            config_json: upstream.config_json.clone(),
            visibility: agent.visibility,
        };
        self.agent_repo.update(id, &input).await?;

        let updated = self
            .agent_repo
            .find_by_id(id)
            .await?
            .ok_or(ServiceError::NotFound)?;
        Ok(updated)
    }
}

#[cfg(test)]
mod tests {
    use mockall::mock;
    use mockall::predicate::*;
    use uuid::Uuid;

    use super::*;
    use crate::domain::RepoResult;
    use crate::domain::agent::Agent;

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

    fn sample_agent(owner: i32, vis: Visibility) -> Agent {
        Agent {
            id: Uuid::new_v4(),
            owner_id: owner,
            name: "锐雯".into(),
            champion: "Riven".into(),
            agent_type: crate::domain::agent::AgentType::Llm,
            prompt: "prompt".into(),
            model: "model".into(),
            config_json: serde_json::json!({}),
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
        repo.expect_set_fork_linkage().returning(|_, _, _| Ok(()));
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
        repo.expect_set_fork_linkage().returning(|_, _, _| Ok(()));
        let svc = build_service(repo);
        svc.fork(2, source.id, None).await.unwrap();
    }

    fn agent_with(owner: i32, id: Uuid, upstream: Option<Uuid>) -> Agent {
        Agent {
            id,
            owner_id: owner,
            name: "我的锐雯".into(),
            champion: "Riven".into(),
            agent_type: crate::domain::agent::AgentType::Llm,
            prompt: "mine".into(),
            model: "model".into(),
            config_json: serde_json::json!({}),
            visibility: Visibility::Private,
            forked_from: upstream,
            upstream_agent_id: upstream,
        }
    }

    #[tokio::test]
    async fn pull_upstream_overwrites_from_source() {
        let upstream_id = Uuid::new_v4();
        let agent_id = Uuid::new_v4();
        let agent = agent_with(2, agent_id, Some(upstream_id));
        let mut upstream = sample_agent(1, Visibility::Public);
        upstream.id = upstream_id;
        upstream.prompt = "upstream-strategy".into();
        let mut repo = MockAgentRepo::new();
        repo.expect_find_by_id().returning(move |qid| {
            if qid == upstream_id {
                Ok(Some(upstream.clone()))
            } else {
                Ok(Some(agent.clone()))
            }
        });
        repo.expect_update().returning(|_, _| Ok(()));
        let svc = build_service(repo);
        let updated = svc.pull_upstream(2, agent_id).await.unwrap();
        assert_eq!(updated.owner_id, 2);
    }

    #[tokio::test]
    async fn pull_upstream_non_owner_forbidden() {
        let agent_id = Uuid::new_v4();
        let agent = agent_with(2, agent_id, Some(Uuid::new_v4()));
        let mut repo = MockAgentRepo::new();
        repo.expect_find_by_id()
            .returning(move |_| Ok(Some(agent.clone())));
        repo.expect_update().times(0);
        let svc = build_service(repo);
        let err = svc.pull_upstream(99, agent_id).await.unwrap_err();
        assert!(matches!(err, ServiceError::Forbidden));
    }

    #[tokio::test]
    async fn pull_upstream_without_upstream_is_validation_error() {
        let agent_id = Uuid::new_v4();
        let agent = agent_with(2, agent_id, None);
        let mut repo = MockAgentRepo::new();
        repo.expect_find_by_id()
            .returning(move |_| Ok(Some(agent.clone())));
        repo.expect_update().times(0);
        let svc = build_service(repo);
        let err = svc.pull_upstream(2, agent_id).await.unwrap_err();
        assert!(matches!(err, ServiceError::Validation(_)));
    }
}
