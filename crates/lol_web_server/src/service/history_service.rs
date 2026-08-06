//! 游戏历史 service 层。

use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use lol_web_protocol::history::{GameHistorySummary, SavedAgentHistory, UploadHistoryRequest};
use uuid::Uuid;

use crate::domain::history::GameHistory;
use crate::domain::{ServiceError, ServiceResult};
use crate::repository::history_repo::HistoryRepo;

#[async_trait]
pub trait HistoryService: Send + Sync {
    async fn list(&self, user_id: i32) -> ServiceResult<Vec<GameHistorySummary>>;
    async fn get(&self, user_id: i32, id: Uuid) -> ServiceResult<Vec<SavedAgentHistory>>;
    async fn upload(&self, user_id: i32, req: UploadHistoryRequest) -> ServiceResult<()>;
    async fn delete(&self, user_id: i32, id: Uuid) -> ServiceResult<()>;
}

pub struct HistoryServiceImpl {
    pub repo: Arc<dyn HistoryRepo>,
}

impl HistoryServiceImpl {
    pub fn new(repo: Arc<dyn HistoryRepo>) -> Self {
        Self { repo }
    }

    fn validate_upload(req: &UploadHistoryRequest) -> ServiceResult<()> {
        if req.histories.is_empty() {
            return Err(ServiceError::Validation("histories 不能为空".into()));
        }
        Ok(())
    }
}

#[async_trait]
impl HistoryService for HistoryServiceImpl {
    async fn list(&self, user_id: i32) -> ServiceResult<Vec<GameHistorySummary>> {
        let rows = self.repo.list_by_user(user_id).await?;
        let summaries = rows
            .into_iter()
            .map(|row| {
                let agents = serde_json::from_value(row.agents).unwrap_or_default();
                GameHistorySummary {
                    id: Some(row.id.to_string()),
                    datetime: row.datetime.to_rfc3339(),
                    duration: row.game_duration,
                    agents,
                }
            })
            .collect();
        Ok(summaries)
    }

    async fn get(&self, user_id: i32, id: Uuid) -> ServiceResult<Vec<SavedAgentHistory>> {
        let row = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or(ServiceError::NotFound)?;
        if row.user_id != user_id {
            return Err(ServiceError::Forbidden);
        }
        let histories: Vec<SavedAgentHistory> =
            serde_json::from_value(row.histories).unwrap_or_default();
        Ok(histories)
    }

    async fn upload(&self, user_id: i32, req: UploadHistoryRequest) -> ServiceResult<()> {
        Self::validate_upload(&req)?;

        let datetime = req.histories[0]
            .datetime
            .parse::<chrono::DateTime<Utc>>()
            .map_err(|e| ServiceError::Validation(format!("datetime 格式错误: {e}")))?;
        let game_duration = req.histories[0].game_duration;

        let agents: Vec<serde_json::Value> = req
            .histories
            .iter()
            .map(|h| {
                serde_json::json!({
                    "agent_id": h.agent_id,
                    "champion": h.champion,
                    "team": h.team,
                })
            })
            .collect();

        let histories_json = serde_json::to_value(&req.histories)
            .map_err(|e| ServiceError::Internal(format!("序列化 histories 失败: {e}")))?;

        let domain = GameHistory {
            id: Uuid::new_v4(),
            user_id,
            datetime,
            game_duration,
            agents: serde_json::Value::Array(agents),
            histories: histories_json,
            created_at: Utc::now(),
        };

        self.repo.insert(user_id, &domain).await?;
        Ok(())
    }

    async fn delete(&self, user_id: i32, id: Uuid) -> ServiceResult<()> {
        let row = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or(ServiceError::NotFound)?;
        if row.user_id != user_id {
            return Err(ServiceError::Forbidden);
        }
        self.repo.delete(id).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use mockall::mock;
    use mockall::predicate::*;

    use super::*;
    use crate::domain::RepoResult;

    mock! {
        pub HistoryRepo {}
        #[async_trait]
        impl HistoryRepo for HistoryRepo {
            async fn list_by_user(&self, user_id: i32) -> RepoResult<Vec<GameHistory>>;
            async fn find_by_id(&self, id: Uuid) -> RepoResult<Option<GameHistory>>;
            async fn insert(&self, user_id: i32, history: &GameHistory) -> RepoResult<GameHistory>;
            async fn delete(&self, id: Uuid) -> RepoResult<()>;
        }
    }

    fn build_service(repo: MockHistoryRepo) -> HistoryServiceImpl {
        HistoryServiceImpl {
            repo: Arc::new(repo),
        }
    }

    fn sample_domain(user_id: i32) -> GameHistory {
        GameHistory {
            id: Uuid::new_v4(),
            user_id,
            datetime: Utc::now(),
            game_duration: 1800,
            agents: serde_json::json!([{"agent_id": "a1", "champion": "Riven", "team": "blue"}]),
            histories: serde_json::json!([{
                "agent_id": "a1",
                "champion": "Riven",
                "team": "blue",
                "prompt": "",
                "system_prompt": "",
                "history": [],
                "game_duration": 1800,
                "datetime": "2025-08-01T00:00:00Z"
            }]),
            created_at: Utc::now(),
        }
    }

    fn sample_upload() -> UploadHistoryRequest {
        UploadHistoryRequest {
            histories: vec![SavedAgentHistory {
                agent_id: "a1".into(),
                champion: "Riven".into(),
                team: "blue".into(),
                prompt: "".into(),
                system_prompt: "".into(),
                history: vec![],
                game_duration: 1800,
                datetime: "2025-08-01T00:00:00Z".into(),
            }],
        }
    }

    #[tokio::test]
    async fn list_returns_summaries() {
        let mut repo = MockHistoryRepo::new();
        repo.expect_list_by_user()
            .with(eq(1))
            .returning(|_| Ok(vec![sample_domain(1)]));
        let svc = build_service(repo);
        let summaries = svc.list(1).await.unwrap();
        assert_eq!(summaries.len(), 1);
        assert!(summaries[0].id.is_some());
    }

    #[tokio::test]
    async fn get_non_owner_forbidden() {
        let mut repo = MockHistoryRepo::new();
        repo.expect_find_by_id()
            .returning(|_| Ok(Some(sample_domain(1))));
        let svc = build_service(repo);
        let err = svc.get(2, Uuid::new_v4()).await.unwrap_err();
        assert!(matches!(err, ServiceError::Forbidden));
    }

    #[tokio::test]
    async fn get_missing_not_found() {
        let mut repo = MockHistoryRepo::new();
        repo.expect_find_by_id().returning(|_| Ok(None));
        let svc = build_service(repo);
        let err = svc.get(1, Uuid::new_v4()).await.unwrap_err();
        assert!(matches!(err, ServiceError::NotFound));
    }

    #[tokio::test]
    async fn upload_empty_rejected() {
        let mut repo = MockHistoryRepo::new();
        repo.expect_insert().times(0);
        let svc = build_service(repo);
        let req = UploadHistoryRequest { histories: vec![] };
        let err = svc.upload(1, req).await.unwrap_err();
        assert!(matches!(err, ServiceError::Validation(_)));
    }

    #[tokio::test]
    async fn upload_success() {
        let mut repo = MockHistoryRepo::new();
        repo.expect_insert()
            .with(eq(1), always())
            .returning(|user_id, _| Ok(sample_domain(user_id)));
        let svc = build_service(repo);
        svc.upload(1, sample_upload()).await.unwrap();
    }

    #[tokio::test]
    async fn delete_non_owner_forbidden() {
        let mut repo = MockHistoryRepo::new();
        repo.expect_find_by_id()
            .returning(|_| Ok(Some(sample_domain(1))));
        repo.expect_delete().times(0);
        let svc = build_service(repo);
        let err = svc.delete(2, Uuid::new_v4()).await.unwrap_err();
        assert!(matches!(err, ServiceError::Forbidden));
    }

    #[tokio::test]
    async fn delete_owner_ok() {
        let mut repo = MockHistoryRepo::new();
        repo.expect_find_by_id()
            .returning(|_| Ok(Some(sample_domain(1))));
        repo.expect_delete().returning(|_| Ok(()));
        let svc = build_service(repo);
        svc.delete(1, Uuid::new_v4()).await.unwrap();
    }
}
