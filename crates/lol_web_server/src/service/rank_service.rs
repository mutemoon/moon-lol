//! Rank 子系统的 service 层（匹配队列 + ELO 更新 + 排行榜）。

use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::rank::{ELO_INITIAL, Outcome, elo_exchange, match_window_after_wait};
use crate::domain::{ServiceError, ServiceResult};
use crate::repository::rank_repo::{
    EloRating, EloRepo, NewQueueEntry, RankQueueEntry, RankQueueRepo, Season, SeasonRepo,
};

/// Rank 匹配成功后创建对局的抽象（由 MatchService 实现，避免循环依赖）。
#[async_trait]
pub trait RankMatchCreator: Send + Sync {
    /// 为两个队列条目创建一场 rank 对局，返回 match_id。
    async fn create_rank_match(
        &self,
        entry_a: &RankQueueEntry,
        entry_b: &RankQueueEntry,
    ) -> ServiceResult<Uuid>;
}

#[async_trait]
pub trait RankService: Send + Sync {
    async fn enqueue(
        &self,
        user_id: i32,
        agent_id: Uuid,
        agent_snapshot_id: Uuid,
        mode: &str,
    ) -> ServiceResult<RankQueueEntry>;
    async fn dequeue(&self, agent_id: Uuid) -> ServiceResult<()>;
    async fn list_my_queue(&self, user_id: i32) -> ServiceResult<Vec<RankQueueEntry>>;
    async fn try_match(&self, entry: &RankQueueEntry) -> ServiceResult<Option<Uuid>>;
    async fn get_elo(&self, agent_id: Uuid, mode: &str) -> ServiceResult<EloRating>;
    async fn record_result(
        &self,
        winner_agent_id: Uuid,
        loser_agent_id: Uuid,
        mode: &str,
        outcome: Outcome,
    ) -> ServiceResult<()>;
    async fn leaderboard(&self, mode: &str, limit: i64) -> ServiceResult<Vec<EloRating>>;
    async fn current_season(&self, mode: &str) -> ServiceResult<Season>;
}

pub struct RankServiceImpl {
    pub season_repo: Arc<dyn SeasonRepo>,
    pub queue_repo: Arc<dyn RankQueueRepo>,
    pub elo_repo: Arc<dyn EloRepo>,
    pub match_creator: Arc<dyn RankMatchCreator>,
}

impl RankServiceImpl {
    pub fn new(
        season_repo: Arc<dyn SeasonRepo>,
        queue_repo: Arc<dyn RankQueueRepo>,
        elo_repo: Arc<dyn EloRepo>,
        match_creator: Arc<dyn RankMatchCreator>,
    ) -> Self {
        Self {
            season_repo,
            queue_repo,
            elo_repo,
            match_creator,
        }
    }
}

#[async_trait]
impl RankService for RankServiceImpl {
    async fn enqueue(
        &self,
        user_id: i32,
        agent_id: Uuid,
        agent_snapshot_id: Uuid,
        mode: &str,
    ) -> ServiceResult<RankQueueEntry> {
        let season = self.current_season(mode).await?;

        // 校验未重复入队
        if let Some(existing) = self.queue_repo.find_by_agent(agent_id).await? {
            if existing.season_id == season.id {
                return Err(ServiceError::Conflict("该 Agent 已在队列中".into()));
            }
        }

        let entry = self
            .queue_repo
            .enqueue(&NewQueueEntry {
                agent_id,
                agent_snapshot_id,
                user_id,
                mode: mode.into(),
                season_id: season.id,
            })
            .await?;
        Ok(entry)
    }

    async fn dequeue(&self, agent_id: Uuid) -> ServiceResult<()> {
        self.queue_repo.dequeue(agent_id).await?;
        Ok(())
    }

    async fn list_my_queue(&self, user_id: i32) -> ServiceResult<Vec<RankQueueEntry>> {
        Ok(self.queue_repo.list_by_user(user_id).await?)
    }

    async fn try_match(&self, entry: &RankQueueEntry) -> ServiceResult<Option<Uuid>> {
        // 取自身 ELO（无则用初始值）
        let my_elo = self
            .elo_repo
            .find(entry.agent_id, &entry.mode, entry.season_id)
            .await?
            .map(|e| e.rating)
            .unwrap_or(ELO_INITIAL);

        // 计算匹配窗口（基于等待时间）
        let wait_secs = (chrono::Utc::now() - entry.enqueued_at)
            .num_seconds()
            .max(0);
        let window = match_window_after_wait(wait_secs);

        // 找对手
        let opponent = self
            .queue_repo
            .find_opponent(&entry.mode, entry.agent_id, my_elo, window)
            .await?;

        match opponent {
            None => Ok(None),
            Some(opp) => {
                // 双方标记为 matching
                self.queue_repo
                    .update_status(entry.id, crate::domain::rank::QueueStatus::Matching)
                    .await?;
                self.queue_repo
                    .update_status(opp.id, crate::domain::rank::QueueStatus::Matching)
                    .await?;

                // 创建对局
                let match_id = self.match_creator.create_rank_match(entry, &opp).await?;
                Ok(Some(match_id))
            }
        }
    }

    async fn get_elo(&self, agent_id: Uuid, mode: &str) -> ServiceResult<EloRating> {
        let season = self.current_season(mode).await?;
        Ok(self
            .elo_repo
            .upsert_initial(agent_id, mode, season.id)
            .await?)
    }

    async fn record_result(
        &self,
        winner_agent_id: Uuid,
        loser_agent_id: Uuid,
        mode: &str,
        outcome: Outcome,
    ) -> ServiceResult<()> {
        let season = self.current_season(mode).await?;

        let winner_elo = self
            .elo_repo
            .upsert_initial(winner_agent_id, mode, season.id)
            .await?;
        let loser_elo = self
            .elo_repo
            .upsert_initial(loser_agent_id, mode, season.id)
            .await?;

        let (new_winner, new_loser) = elo_exchange(winner_elo.rating, loser_elo.rating, outcome);

        self.elo_repo
            .update_after_match(winner_elo.id, new_winner, true, outcome == Outcome::Draw)
            .await?;
        self.elo_repo
            .update_after_match(loser_elo.id, new_loser, false, outcome == Outcome::Draw)
            .await?;
        Ok(())
    }

    async fn leaderboard(&self, mode: &str, limit: i64) -> ServiceResult<Vec<EloRating>> {
        let season = self.current_season(mode).await?;
        Ok(self.elo_repo.leaderboard(mode, season.id, limit).await?)
    }

    async fn current_season(&self, mode: &str) -> ServiceResult<Season> {
        self.season_repo
            .find_current(mode)
            .await?
            .ok_or_else(|| ServiceError::NotFound)
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, Utc};
    use mockall::mock;
    use mockall::predicate::*;
    use uuid::Uuid;

    use super::*;
    use crate::domain::RepoResult;
    use crate::domain::rank::{Outcome, QueueStatus, SeasonStatus};
    use crate::repository::rank_repo::NewSeason;

    mock! {
        pub SeasonRepo {}
        #[async_trait]
        impl SeasonRepo for SeasonRepo {
            async fn find_by_id(&self, id: Uuid) -> RepoResult<Option<Season>>;
            async fn find_current(&self, mode: &str) -> RepoResult<Option<Season>>;
            async fn list_by_mode(&self, mode: &str) -> RepoResult<Vec<Season>>;
            async fn insert(&self, s: &NewSeason) -> RepoResult<Season>;
            async fn update_status(&self, id: Uuid, status: SeasonStatus) -> RepoResult<()>;
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
            async fn update_status(&self, id: Uuid, status: QueueStatus) -> RepoResult<()>;
            async fn find_opponent(&self, mode: &str, agent_id: Uuid, rating: f64, window: f64) -> RepoResult<Option<RankQueueEntry>>;
        }
    }

    mock! {
        pub EloRepo {}
        #[async_trait]
        impl EloRepo for EloRepo {
            async fn find(&self, agent_id: Uuid, mode: &str, season_id: Uuid) -> RepoResult<Option<EloRating>>;
            async fn upsert_initial(&self, agent_id: Uuid, mode: &str, season_id: Uuid) -> RepoResult<EloRating>;
            async fn update_after_match(&self, id: Uuid, new_rating: f64, is_win: bool, is_draw: bool) -> RepoResult<()>;
            async fn leaderboard(&self, mode: &str, season_id: Uuid, limit: i64) -> RepoResult<Vec<EloRating>>;
        }
    }

    mock! {
        pub MatchCreator {}
        #[async_trait]
        impl RankMatchCreator for MatchCreator {
            async fn create_rank_match(&self, a: &RankQueueEntry, b: &RankQueueEntry) -> ServiceResult<Uuid>;
        }
    }

    fn sample_season() -> Season {
        Season {
            id: Uuid::new_v4(),
            name: "2026 夏季赛".into(),
            mode: "top_solo".into(),
            starts_at: Utc::now() - Duration::days(10),
            ends_at: Utc::now() + Duration::days(80),
            status: SeasonStatus::Active,
        }
    }

    fn build_service(
        season: MockSeasonRepo,
        queue: MockQueueRepo,
        elo: MockEloRepo,
        creator: MockMatchCreator,
    ) -> RankServiceImpl {
        RankServiceImpl {
            season_repo: Arc::new(season),
            queue_repo: Arc::new(queue),
            elo_repo: Arc::new(elo),
            match_creator: Arc::new(creator),
        }
    }

    fn current_season_mocks(season_repo: &mut MockSeasonRepo, season: Season) {
        let s = season.clone();
        season_repo
            .expect_find_current()
            .returning(move |_| Ok(Some(s.clone())));
    }

    #[tokio::test]
    async fn enqueue_success() {
        let season = sample_season();
        let season_id = season.id;
        let mut sr = MockSeasonRepo::new();
        current_season_mocks(&mut sr, season);
        let mut qr = MockQueueRepo::new();
        qr.expect_find_by_agent().returning(|_| Ok(None));
        qr.expect_enqueue().returning(move |_| {
            Ok(RankQueueEntry {
                id: Uuid::new_v4(),
                agent_id: Uuid::new_v4(),
                agent_snapshot_id: Uuid::new_v4(),
                user_id: 1,
                mode: "top_solo".into(),
                season_id,
                status: QueueStatus::Queued,
                enqueued_at: Utc::now(),
                last_match_at: None,
            })
        });
        let svc = build_service(sr, qr, MockEloRepo::new(), MockMatchCreator::new());
        let entry = svc
            .enqueue(1, Uuid::new_v4(), Uuid::new_v4(), "top_solo")
            .await
            .unwrap();
        assert_eq!(entry.status, QueueStatus::Queued);
    }

    #[tokio::test]
    async fn enqueue_duplicate_rejected() {
        let season = sample_season();
        let season_id = season.id;
        let existing = RankQueueEntry {
            id: Uuid::new_v4(),
            agent_id: Uuid::new_v4(),
            agent_snapshot_id: Uuid::new_v4(),
            user_id: 1,
            mode: "top_solo".into(),
            season_id,
            status: QueueStatus::Queued,
            enqueued_at: Utc::now(),
            last_match_at: None,
        };
        let existing_clone = existing.clone();
        let mut sr = MockSeasonRepo::new();
        current_season_mocks(&mut sr, season);
        let mut qr = MockQueueRepo::new();
        qr.expect_find_by_agent()
            .returning(move |_| Ok(Some(existing_clone.clone())));
        qr.expect_enqueue().times(0);
        let svc = build_service(sr, qr, MockEloRepo::new(), MockMatchCreator::new());
        let err = svc
            .enqueue(1, Uuid::new_v4(), Uuid::new_v4(), "top_solo")
            .await
            .unwrap_err();
        assert!(matches!(err, ServiceError::Conflict(_)));
    }

    #[tokio::test]
    async fn try_match_no_opponent_returns_none() {
        let entry = RankQueueEntry {
            id: Uuid::new_v4(),
            agent_id: Uuid::new_v4(),
            agent_snapshot_id: Uuid::new_v4(),
            user_id: 1,
            mode: "top_solo".into(),
            season_id: Uuid::new_v4(),
            status: QueueStatus::Queued,
            enqueued_at: Utc::now(),
            last_match_at: None,
        };
        let entry_clone = entry.clone();
        let mut er = MockEloRepo::new();
        er.expect_find().returning(|_, _, _| Ok(None));
        let mut qr = MockQueueRepo::new();
        qr.expect_find_opponent().returning(|_, _, _, _| Ok(None));
        qr.expect_update_status().times(0);
        let mut mc = MockMatchCreator::new();
        mc.expect_create_rank_match().times(0);
        let svc = build_service(MockSeasonRepo::new(), qr, er, mc);
        let result = svc.try_match(&entry_clone).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn try_match_finds_opponent_creates_match() {
        let entry = RankQueueEntry {
            id: Uuid::new_v4(),
            agent_id: Uuid::new_v4(),
            agent_snapshot_id: Uuid::new_v4(),
            user_id: 1,
            mode: "top_solo".into(),
            season_id: Uuid::new_v4(),
            status: QueueStatus::Queued,
            enqueued_at: Utc::now(),
            last_match_at: None,
        };
        let opp = RankQueueEntry {
            id: Uuid::new_v4(),
            agent_id: Uuid::new_v4(),
            agent_snapshot_id: Uuid::new_v4(),
            user_id: 2,
            mode: "top_solo".into(),
            season_id: Uuid::new_v4(),
            status: QueueStatus::Queued,
            enqueued_at: Utc::now(),
            last_match_at: None,
        };
        let opp_clone = opp.clone();
        let mut er = MockEloRepo::new();
        er.expect_find().returning(|_, _, _| Ok(None));
        let mut qr = MockQueueRepo::new();
        qr.expect_find_opponent()
            .returning(move |_, _, _, _| Ok(Some(opp_clone.clone())));
        qr.expect_update_status().returning(|_, _| Ok(()));
        let mut mc = MockMatchCreator::new();
        mc.expect_create_rank_match()
            .returning(|_, _| Ok(Uuid::new_v4()));
        let svc = build_service(MockSeasonRepo::new(), qr, er, mc);
        let result = svc.try_match(&entry).await.unwrap();
        assert!(result.is_some());
    }

    #[tokio::test]
    async fn record_result_updates_both_elo() {
        let season = sample_season();
        let season_id = season.id;
        let mut sr = MockSeasonRepo::new();
        current_season_mocks(&mut sr, season);
        let winner_id = Uuid::new_v4();
        let loser_id = Uuid::new_v4();
        let mut er = MockEloRepo::new();
        er.expect_upsert_initial()
            .returning(move |aid, _mode, sid| {
                Ok(EloRating {
                    id: Uuid::new_v4(),
                    agent_id: aid,
                    mode: "top_solo".into(),
                    season_id: sid,
                    rating: 1200.0,
                    wins: 0,
                    losses: 0,
                    draws: 0,
                })
            });
        er.expect_update_after_match()
            .times(2)
            .returning(|_, _, _, _| Ok(()));
        let svc = build_service(sr, MockQueueRepo::new(), er, MockMatchCreator::new());
        svc.record_result(winner_id, loser_id, "top_solo", Outcome::Win)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn current_season_not_found_when_no_active() {
        let mut sr = MockSeasonRepo::new();
        sr.expect_find_current().returning(|_| Ok(None));
        let svc = build_service(
            sr,
            MockQueueRepo::new(),
            MockEloRepo::new(),
            MockMatchCreator::new(),
        );
        let err = svc.current_season("top_solo").await.unwrap_err();
        assert!(matches!(err, ServiceError::NotFound));
    }
}
