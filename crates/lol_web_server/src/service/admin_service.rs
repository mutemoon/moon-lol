//! Admin 子系统的 service 层（算力监控 + 强制管理）。
//!
//! 复用 MatchRepo / RankQueueRepo / LocalGameService，不新建表。

use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::match_::MatchStatus;
use crate::domain::{ServiceError, ServiceResult};
use crate::repository::match_repo::MatchRepo;
use crate::repository::rank_repo::RankQueueRepo;
use crate::service::local_game_service::LocalGameService;

/// 算力监控快照。
#[derive(Debug, Clone, serde::Serialize)]
pub struct AdminMetrics {
    pub running_matches: usize,
    pub pending_matches: usize,
    pub queued_agents: usize,
    pub managed_processes: usize,
}

#[async_trait]
pub trait AdminService: Send + Sync {
    async fn metrics(&self) -> ServiceResult<AdminMetrics>;
    async fn list_running(&self) -> ServiceResult<Vec<crate::domain::match_::Match>>;
    async fn force_abort(&self, match_id: Uuid) -> ServiceResult<()>;
}

pub struct AdminServiceImpl {
    pub match_repo: Arc<dyn MatchRepo>,
    pub queue_repo: Arc<dyn RankQueueRepo>,
    pub local_game: Arc<dyn LocalGameService>,
}

impl AdminServiceImpl {
    pub fn new(
        match_repo: Arc<dyn MatchRepo>,
        queue_repo: Arc<dyn RankQueueRepo>,
        local_game: Arc<dyn LocalGameService>,
    ) -> Self {
        Self {
            match_repo,
            queue_repo,
            local_game,
        }
    }
}

#[async_trait]
impl AdminService for AdminServiceImpl {
    async fn metrics(&self) -> ServiceResult<AdminMetrics> {
        let running = self
            .match_repo
            .list_by_status(MatchStatus::Running, 1000)
            .await?;
        let pending = self
            .match_repo
            .list_by_status(MatchStatus::Pending, 1000)
            .await?;
        let processes = self.local_game.list_processes().await?;
        // 队列总数：遍历所有模式（简化：只查 top_solo）
        let queued = self.queue_repo.list_queued("top_solo").await?;
        Ok(AdminMetrics {
            running_matches: running.len(),
            pending_matches: pending.len(),
            queued_agents: queued.len(),
            managed_processes: processes.len(),
        })
    }

    async fn list_running(&self) -> ServiceResult<Vec<crate::domain::match_::Match>> {
        Ok(self
            .match_repo
            .list_by_status(MatchStatus::Running, 100)
            .await?)
    }

    async fn force_abort(&self, match_id: Uuid) -> ServiceResult<()> {
        let m = self
            .match_repo
            .find_by_id(match_id)
            .await?
            .ok_or(ServiceError::NotFound)?;
        if m.status == MatchStatus::Finished || m.status == MatchStatus::Aborted {
            return Err(ServiceError::Conflict("对局已结束，无法中止".into()));
        }
        self.match_repo
            .update_abort(match_id, m.status, "admin_force_abort")
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use mockall::mock;
    use mockall::predicate::*;
    use uuid::Uuid;

    use super::*;
    use crate::domain::RepoResult;
    use crate::domain::local_game::ManagedProcess;
    use crate::domain::match_::{Match, MatchForm, MatchStatus, Winner};
    use crate::repository::match_repo::MatchInput;
    use crate::repository::rank_repo::{NewQueueEntry, RankQueueEntry};

    mock! {
        pub MatchRepo {}
        #[async_trait]
        impl MatchRepo for MatchRepo {
            async fn find_by_id(&self, id: Uuid) -> RepoResult<Option<Match>>;
            async fn list_by_owner(&self, owner_id: i32, limit: i64) -> RepoResult<Vec<Match>>;
            async fn list_by_status(&self, status: MatchStatus, limit: i64) -> RepoResult<Vec<Match>>;
            async fn insert(&self, owner_id: i32, input: &MatchInput) -> RepoResult<Match>;
            async fn update_status(&self, id: Uuid, from: MatchStatus, to: MatchStatus) -> RepoResult<()>;
            async fn update_result(&self, id: Uuid, winner: Winner) -> RepoResult<()>;
            async fn update_abort(&self, id: Uuid, from: MatchStatus, reason: &str) -> RepoResult<()>;
            async fn update_ports(&self, id: Uuid, bevy_port: Option<i32>, ws_port: Option<i32>) -> RepoResult<()>;
        }
    }

    mock! {
        pub QueueRepo {}
        #[async_trait]
        impl RankQueueRepo for QueueRepo {
            async fn enqueue(&self, entry: &NewQueueEntry) -> RepoResult<RankQueueEntry>;
            async fn dequeue(&self, agent_id: Uuid) -> RepoResult<()>;
            async fn find_by_agent(&self, agent_id: Uuid) -> RepoResult<Option<RankQueueEntry>>;
            async fn list_by_user(&self, user_id: i32) -> RepoResult<Vec<RankQueueEntry>>;
            async fn list_queued(&self, mode: &str) -> RepoResult<Vec<RankQueueEntry>>;
            async fn update_status(&self, id: Uuid, status: crate::domain::rank::QueueStatus) -> RepoResult<()>;
            async fn find_opponent(&self, mode: &str, agent_id: Uuid, rating: f64, window: f64) -> RepoResult<Option<RankQueueEntry>>;
        }
    }

    mock! {
        pub LocalGame {}
        #[async_trait]
        impl LocalGameService for LocalGame {
            async fn start(&self, owner_id: i32, input: crate::service::local_game_service::LocalStartInput) -> ServiceResult<(Uuid, i32)>;
            async fn stop(&self, owner_id: i32, match_id: Uuid) -> ServiceResult<()>;
            async fn list_processes(&self) -> ServiceResult<Vec<ManagedProcess>>;
            async fn cleanup(&self) -> ServiceResult<usize>;
        }
    }

    fn sample_match(status: MatchStatus) -> Match {
        Match {
            id: Uuid::new_v4(),
            form: MatchForm::Local,
            room_id: None,
            owner_id: 1,
            mode: "1v1".into(),
            status,
            bevy_port: None,
            winner_team: None,
            abort_reason: None,
        }
    }

    fn build_service(
        repo: MockMatchRepo,
        queue: MockQueueRepo,
        lg: MockLocalGame,
    ) -> AdminServiceImpl {
        AdminServiceImpl {
            match_repo: Arc::new(repo),
            queue_repo: Arc::new(queue),
            local_game: Arc::new(lg),
        }
    }

    #[tokio::test]
    async fn metrics_aggregates_counts() {
        let mut repo = MockMatchRepo::new();
        repo.expect_list_by_status().returning(|s, _| {
            if s == MatchStatus::Running {
                Ok(vec![sample_match(MatchStatus::Running)])
            } else {
                Ok(vec![])
            }
        });
        // list_by_status 被调用两次（running + pending）
        repo.expect_list_by_status().returning(|_, _| Ok(vec![]));
        let mut queue = MockQueueRepo::new();
        queue.expect_list_queued().returning(|_| Ok(vec![]));
        let mut lg = MockLocalGame::new();
        lg.expect_list_processes().returning(|| Ok(vec![]));
        let svc = build_service(repo, queue, lg);
        let m = svc.metrics().await.unwrap();
        assert_eq!(m.running_matches, 1);
    }

    #[tokio::test]
    async fn list_running_delegates() {
        let mut repo = MockMatchRepo::new();
        repo.expect_list_by_status()
            .with(eq(MatchStatus::Running), eq(100i64))
            .returning(|_, _| Ok(vec![sample_match(MatchStatus::Running)]));
        let svc = build_service(repo, MockQueueRepo::new(), MockLocalGame::new());
        assert_eq!(svc.list_running().await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn force_abort_running_success() {
        let m = sample_match(MatchStatus::Running);
        let mid = m.id;
        let m_clone = m.clone();
        let mut repo = MockMatchRepo::new();
        repo.expect_find_by_id()
            .returning(move |_| Ok(Some(m_clone.clone())));
        repo.expect_update_abort()
            .with(eq(mid), eq(MatchStatus::Running), eq("admin_force_abort"))
            .returning(|_, _, _| Ok(()));
        let svc = build_service(repo, MockQueueRepo::new(), MockLocalGame::new());
        svc.force_abort(mid).await.unwrap();
    }

    #[tokio::test]
    async fn force_abort_finished_rejected() {
        let m = sample_match(MatchStatus::Finished);
        let m_clone = m.clone();
        let mut repo = MockMatchRepo::new();
        repo.expect_find_by_id()
            .returning(move |_| Ok(Some(m_clone.clone())));
        repo.expect_update_abort().times(0);
        let svc = build_service(repo, MockQueueRepo::new(), MockLocalGame::new());
        let err = svc.force_abort(m.id).await.unwrap_err();
        assert!(matches!(err, ServiceError::Conflict(_)));
    }

    #[tokio::test]
    async fn force_abort_missing_not_found() {
        let mut repo = MockMatchRepo::new();
        repo.expect_find_by_id().returning(|_| Ok(None));
        let svc = build_service(repo, MockQueueRepo::new(), MockLocalGame::new());
        let err = svc.force_abort(Uuid::new_v4()).await.unwrap_err();
        assert!(matches!(err, ServiceError::NotFound));
    }
}
