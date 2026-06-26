//! AgentSnapshot 子系统的 service 层。

use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::agent_snapshot::{AgentSnapshot, next_version};
use crate::domain::{ServiceError, ServiceResult};
use crate::repository::agent_repo::AgentRepo;
use crate::repository::agent_snapshot_repo::AgentSnapshotRepo;

/// 构造 publish 时冻结的 config_freeze（纯函数）。
pub fn build_config_freeze(
    agent: &crate::domain::agent::Agent,
    spawn: Option<&crate::domain::spawn_preset::SpawnPreset>,
    win_condition: Option<&serde_json::Value>,
) -> serde_json::Value {
    serde_json::json!({
        "champion": agent.champion,
        "agent": {
            "id": agent.id,
            "agent_type": agent.agent_type.as_str(),
            "prompt": agent.prompt,
            "preamble": agent.preamble,
            "model": agent.model,
            "config_json": agent.config_json,
        },
        "spawn": spawn.map(|s| serde_json::json!({
            "id": s.id, "x": s.x, "z": s.z, "team": s.team.as_str(),
        })),
        "win_condition": win_condition,
    })
}

#[async_trait]
pub trait AgentSnapshotService: Send + Sync {
    async fn publish(
        &self,
        owner_id: i32,
        agent_id: Uuid,
        config_freeze: serde_json::Value,
    ) -> ServiceResult<AgentSnapshot>;
    async fn list_by_agent(&self, agent_id: Uuid) -> ServiceResult<Vec<AgentSnapshot>>;
    async fn find_by_id(&self, id: Uuid) -> ServiceResult<Option<AgentSnapshot>>;
    async fn find_latest(&self, agent_id: Uuid) -> ServiceResult<Option<AgentSnapshot>>;
}

pub struct AgentSnapshotServiceImpl {
    pub repo: Arc<dyn AgentSnapshotRepo>,
    pub agent_repo: Arc<dyn AgentRepo>,
}

impl AgentSnapshotServiceImpl {
    pub fn new(repo: Arc<dyn AgentSnapshotRepo>, agent_repo: Arc<dyn AgentRepo>) -> Self {
        Self { repo, agent_repo }
    }
}

#[async_trait]
impl AgentSnapshotService for AgentSnapshotServiceImpl {
    async fn publish(
        &self,
        owner_id: i32,
        agent_id: Uuid,
        config_freeze: serde_json::Value,
    ) -> ServiceResult<AgentSnapshot> {
        let agent = self
            .agent_repo
            .find_by_id_with_owner_check(agent_id, owner_id)
            .await?
            .ok_or(ServiceError::NotFound)?;
        let max = self.repo.max_version(agent.id).await?;
        let version = next_version(max);
        Ok(self.repo.insert(agent.id, version, &config_freeze).await?)
    }

    async fn list_by_agent(&self, agent_id: Uuid) -> ServiceResult<Vec<AgentSnapshot>> {
        Ok(self.repo.list_by_agent(agent_id).await?)
    }

    async fn find_by_id(&self, id: Uuid) -> ServiceResult<Option<AgentSnapshot>> {
        Ok(self.repo.find_by_id(id).await?)
    }

    async fn find_latest(&self, agent_id: Uuid) -> ServiceResult<Option<AgentSnapshot>> {
        Ok(self.repo.find_latest(agent_id).await?)
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use mockall::mock;
    use mockall::predicate::*;

    use super::*;
    use crate::domain::agent::{Agent, AgentType};
    use crate::domain::spawn_preset::{SpawnPreset, Team, Visibility};
    use crate::domain::{RepoError, RepoResult};

    mock! {
        pub SnapshotRepo {}
        #[async_trait]
        impl AgentSnapshotRepo for SnapshotRepo {
            async fn insert(&self, agent_id: Uuid, version: i32, config_freeze: &serde_json::Value) -> RepoResult<AgentSnapshot>;
            async fn find_by_id(&self, id: Uuid) -> RepoResult<Option<AgentSnapshot>>;
            async fn list_by_agent(&self, agent_id: Uuid) -> RepoResult<Vec<AgentSnapshot>>;
            async fn max_version(&self, agent_id: Uuid) -> RepoResult<Option<i32>>;
            async fn find_latest(&self, agent_id: Uuid) -> RepoResult<Option<AgentSnapshot>>;
            async fn delete(&self, id: Uuid) -> RepoResult<()>;
        }
    }

    mock! {
        pub AgentRepo {}
        #[async_trait]
        impl AgentRepo for AgentRepo {
            async fn find_by_id(&self, id: Uuid) -> RepoResult<Option<Agent>>;
            async fn list_by_owner(&self, owner_id: i32) -> RepoResult<Vec<Agent>>;
            async fn list_public(&self) -> RepoResult<Vec<Agent>>;
            async fn find_by_id_with_owner_check(&self, id: Uuid, owner_id: i32) -> RepoResult<Option<Agent>>;
            async fn insert(&self, owner_id: i32, input: &crate::domain::agent::AgentInput) -> RepoResult<Agent>;
            async fn update(&self, id: Uuid, input: &crate::domain::agent::AgentInput) -> RepoResult<()>;
            async fn update_visibility(&self, id: Uuid, visibility: Visibility) -> RepoResult<()>;
            async fn set_fork_linkage(&self, id: Uuid, forked_from: Option<Uuid>, upstream: Option<Uuid>) -> RepoResult<()>;
            async fn delete(&self, id: Uuid) -> RepoResult<()>;
            async fn count_by_owner(&self, owner_id: i32) -> RepoResult<i64>;
        }
    }

    fn build_service(
        repo: MockSnapshotRepo,
        agent_repo: MockAgentRepo,
    ) -> AgentSnapshotServiceImpl {
        AgentSnapshotServiceImpl {
            repo: Arc::new(repo),
            agent_repo: Arc::new(agent_repo),
        }
    }

    fn sample_agent(owner: i32) -> Agent {
        Agent {
            id: Uuid::new_v4(),
            owner_id: owner,
            name: "锐雯".into(),
            champion: "Riven".into(),
            agent_type: AgentType::Llm,
            prompt: "prompt".into(),
            preamble: "preamble".into(),
            model: "model".into(),
            config_json: serde_json::json!({}),
            visibility: Visibility::Private,
            forked_from: None,
            upstream_agent_id: None,
        }
    }

    fn sample_snapshot(agent_id: Uuid, version: i32) -> AgentSnapshot {
        AgentSnapshot {
            id: Uuid::new_v4(),
            agent_id,
            version,
            config_freeze: serde_json::json!({"champion": "Riven"}),
            published_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn publish_first_version_when_no_existing() {
        let agent = sample_agent(1);
        let agent_clone = agent.clone();
        let mut agent_repo = MockAgentRepo::new();
        agent_repo
            .expect_find_by_id_with_owner_check()
            .with(eq(agent.id), eq(1))
            .times(1)
            .returning(move |_, _| Ok(Some(agent_clone.clone())));

        let mut repo = MockSnapshotRepo::new();
        repo.expect_max_version()
            .with(eq(agent.id))
            .times(1)
            .returning(|_| Ok(None));
        let aid = agent.id;
        repo.expect_insert()
            .with(eq(aid), eq(1), always())
            .times(1)
            .returning(move |a, v, _| Ok(sample_snapshot(a, v)));

        let svc = build_service(repo, agent_repo);
        assert_eq!(
            svc.publish(1, agent.id, serde_json::json!({}))
                .await
                .unwrap()
                .version,
            1
        );
    }

    #[tokio::test]
    async fn publish_increments_version_from_max() {
        let agent = sample_agent(1);
        let agent_clone = agent.clone();
        let mut agent_repo = MockAgentRepo::new();
        agent_repo
            .expect_find_by_id_with_owner_check()
            .returning(move |_, _| Ok(Some(agent_clone.clone())));

        let mut repo = MockSnapshotRepo::new();
        repo.expect_max_version().returning(|_| Ok(Some(3)));
        let expected = sample_snapshot(agent.id, 4);
        let expected_clone = expected.clone();
        repo.expect_insert()
            .with(eq(agent.id), eq(4), always())
            .times(1)
            .returning(move |_, _, _| Ok(expected_clone.clone()));

        let svc = build_service(repo, agent_repo);
        assert_eq!(
            svc.publish(1, agent.id, serde_json::json!({}))
                .await
                .unwrap()
                .version,
            4
        );
    }

    #[tokio::test]
    async fn publish_agent_not_found() {
        let mut agent_repo = MockAgentRepo::new();
        agent_repo
            .expect_find_by_id_with_owner_check()
            .returning(|_, _| Ok(None));
        let mut repo = MockSnapshotRepo::new();
        repo.expect_max_version().times(0);
        repo.expect_insert().times(0);
        let svc = build_service(repo, agent_repo);
        let err = svc
            .publish(1, Uuid::new_v4(), serde_json::json!({}))
            .await
            .unwrap_err();
        assert!(matches!(err, ServiceError::NotFound));
    }

    #[tokio::test]
    async fn publish_unique_violation_propagates_as_conflict() {
        let agent = sample_agent(1);
        let agent_clone = agent.clone();
        let mut agent_repo = MockAgentRepo::new();
        agent_repo
            .expect_find_by_id_with_owner_check()
            .returning(move |_, _| Ok(Some(agent_clone.clone())));
        let mut repo = MockSnapshotRepo::new();
        repo.expect_max_version().returning(|_| Ok(None));
        repo.expect_insert()
            .returning(|_, _, _| Err(RepoError::UniqueViolation));
        let svc = build_service(repo, agent_repo);
        let err = svc
            .publish(1, agent.id, serde_json::json!({}))
            .await
            .unwrap_err();
        assert!(matches!(err, ServiceError::Conflict(_)));
    }

    #[tokio::test]
    async fn list_by_agent_delegates() {
        let agent_id = Uuid::new_v4();
        let mut repo = MockSnapshotRepo::new();
        repo.expect_list_by_agent()
            .with(eq(agent_id))
            .times(1)
            .returning(|id| Ok(vec![sample_snapshot(id, 1), sample_snapshot(id, 2)]));
        let svc = build_service(repo, MockAgentRepo::new());
        assert_eq!(svc.list_by_agent(agent_id).await.unwrap().len(), 2);
    }

    #[tokio::test]
    async fn find_latest_delegates() {
        let agent_id = Uuid::new_v4();
        let mut repo = MockSnapshotRepo::new();
        repo.expect_find_latest()
            .with(eq(agent_id))
            .times(1)
            .returning(move |id| Ok(Some(sample_snapshot(id, 5))));
        let svc = build_service(repo, MockAgentRepo::new());
        assert_eq!(svc.find_latest(agent_id).await.unwrap().unwrap().version, 5);
    }

    #[test]
    fn build_config_freeze_includes_all_fields() {
        let agent = Agent {
            id: Uuid::new_v4(),
            owner_id: 1,
            name: "锐雯".into(),
            champion: "Riven".into(),
            agent_type: AgentType::Llm,
            prompt: "p".into(),
            preamble: "pb".into(),
            model: "m".into(),
            config_json: serde_json::json!({"k": 1}),
            visibility: Visibility::Private,
            forked_from: None,
            upstream_agent_id: None,
        };
        let spawn = SpawnPreset {
            id: Uuid::new_v4(),
            owner_id: 1,
            name: "sp".into(),
            x: 1000.0,
            z: 1000.0,
            team: Team::Order,
            visibility: Visibility::Private,
        };
        let win = serde_json::json!({"type": "eliminate"});
        let freeze = build_config_freeze(&agent, Some(&spawn), Some(&win));
        assert_eq!(freeze["champion"], "Riven");
        assert_eq!(freeze["agent"]["agent_type"], "llm");
        assert_eq!(freeze["spawn"]["team"], "order");
        assert_eq!(freeze["win_condition"]["type"], "eliminate");
    }

    #[test]
    fn build_config_freeze_null_optionals() {
        let agent = Agent {
            id: Uuid::new_v4(),
            owner_id: 1,
            name: "菲奥娜".into(),
            champion: "Fiora".into(),
            agent_type: AgentType::Script,
            prompt: "".into(),
            preamble: "".into(),
            model: "".into(),
            config_json: serde_json::json!({}),
            visibility: Visibility::Private,
            forked_from: None,
            upstream_agent_id: None,
        };
        let freeze = build_config_freeze(&agent, None, None);
        assert_eq!(freeze["spawn"], serde_json::Value::Null);
        assert_eq!(freeze["win_condition"], serde_json::Value::Null);
    }
}
