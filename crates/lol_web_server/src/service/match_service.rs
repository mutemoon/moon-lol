//! Match 子系统的 service 层。

use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::match_::{
    Match, MatchEvent, MatchForm, MatchStatus, ParticipantResult, Winner, can_transition,
};
use crate::domain::spawn_preset::Team;
use crate::domain::{ServiceError, ServiceResult};
use crate::repository::match_repo::{
    MatchEventInput, MatchEventRepo, MatchInput, MatchParticipantRepo, MatchRepo, ParticipantInput,
};
use crate::repository::rank_repo::RankQueueEntry;
use crate::service::rank_service::RankMatchCreator;

const DEFAULT_LIST_LIMIT: i64 = 100;

#[async_trait]
pub trait MatchService: Send + Sync {
    async fn create(&self, owner_id: i32, input: MatchInput) -> ServiceResult<Match>;
    async fn get(&self, requester_id: i32, id: Uuid) -> ServiceResult<Match>;
    async fn list_mine(&self, owner_id: i32) -> ServiceResult<Vec<Match>>;
    async fn list_by_status(&self, status: MatchStatus) -> ServiceResult<Vec<Match>>;
    async fn start(
        &self,
        requester_id: i32,
        id: Uuid,
        bevy_port: i32,
        ws_port: i32,
    ) -> ServiceResult<Match>;
    async fn finish(&self, requester_id: i32, id: Uuid, winner: Winner) -> ServiceResult<Match>;
    /// 内部结束对局（不校验 requester 归属）：供 match supervisor 在引擎产出胜负后调用。
    async fn finish_internal(&self, id: Uuid, winner: Winner) -> ServiceResult<Match>;
    async fn abort(&self, requester_id: i32, id: Uuid, reason: String) -> ServiceResult<Match>;
    async fn append_event(
        &self,
        requester_id: i32,
        id: Uuid,
        event: MatchEventInput,
    ) -> ServiceResult<MatchEvent>;
    /// 内部追加事件（不校验 requester 归属）：供 match supervisor 转发引擎事件流。
    async fn append_event_internal(
        &self,
        id: Uuid,
        event: MatchEventInput,
    ) -> ServiceResult<MatchEvent>;
    async fn get_events(
        &self,
        requester_id: i32,
        id: Uuid,
        from_seq: i32,
        limit: i64,
    ) -> ServiceResult<Vec<MatchEvent>>;
}

pub struct MatchServiceImpl {
    pub repo: Arc<dyn MatchRepo>,
    pub participant_repo: Arc<dyn MatchParticipantRepo>,
    pub event_repo: Arc<dyn MatchEventRepo>,
}

impl MatchServiceImpl {
    pub fn new(
        repo: Arc<dyn MatchRepo>,
        participant_repo: Arc<dyn MatchParticipantRepo>,
        event_repo: Arc<dyn MatchEventRepo>,
    ) -> Self {
        Self {
            repo,
            participant_repo,
            event_repo,
        }
    }

    fn validate_input(input: &MatchInput) -> ServiceResult<()> {
        let mode = input.mode.trim();
        if mode.is_empty() || mode.len() > 64 {
            return Err(ServiceError::Validation(
                "mode 不能为空且不超过 64 字符".into(),
            ));
        }
        Ok(())
    }

    async fn get_owned(&self, requester_id: i32, id: Uuid) -> ServiceResult<Match> {
        let m = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or(ServiceError::NotFound)?;
        if m.owner_id != requester_id {
            return Err(ServiceError::Forbidden);
        }
        Ok(m)
    }
}

#[async_trait]
impl MatchService for MatchServiceImpl {
    async fn create(&self, owner_id: i32, input: MatchInput) -> ServiceResult<Match> {
        Self::validate_input(&input)?;
        Ok(self.repo.insert(owner_id, &input).await?)
    }

    async fn get(&self, _requester_id: i32, id: Uuid) -> ServiceResult<Match> {
        self.repo
            .find_by_id(id)
            .await?
            .ok_or(ServiceError::NotFound)
    }

    async fn list_mine(&self, owner_id: i32) -> ServiceResult<Vec<Match>> {
        Ok(self
            .repo
            .list_by_owner(owner_id, DEFAULT_LIST_LIMIT)
            .await?)
    }

    async fn list_by_status(&self, status: MatchStatus) -> ServiceResult<Vec<Match>> {
        Ok(self.repo.list_by_status(status, DEFAULT_LIST_LIMIT).await?)
    }

    async fn start(
        &self,
        requester_id: i32,
        id: Uuid,
        bevy_port: i32,
        ws_port: i32,
    ) -> ServiceResult<Match> {
        let m = self.get_owned(requester_id, id).await?;
        if !can_transition(m.status, MatchStatus::Running) {
            return Err(ServiceError::Conflict(format!(
                "不能从 {} 启动对局",
                m.status.as_str()
            )));
        }
        self.repo
            .update_ports(id, Some(bevy_port), Some(ws_port))
            .await?;
        self.repo
            .update_status(id, m.status, MatchStatus::Running)
            .await?;
        self.repo
            .find_by_id(id)
            .await?
            .ok_or(ServiceError::NotFound)
    }

    async fn finish(&self, requester_id: i32, id: Uuid, winner: Winner) -> ServiceResult<Match> {
        self.get_owned(requester_id, id).await?;
        self.finish_internal(id, winner).await
    }

    async fn finish_internal(&self, id: Uuid, winner: Winner) -> ServiceResult<Match> {
        let m = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or(ServiceError::NotFound)?;
        if !can_transition(m.status, MatchStatus::Finished) {
            return Err(ServiceError::Conflict(format!(
                "不能从 {} 结束对局",
                m.status.as_str()
            )));
        }
        self.repo.update_result(id, winner).await?;
        self.participant_repo
            .update_result_by_team(id, Team::Order, winner.result_for_team(Team::Order))
            .await?;
        self.participant_repo
            .update_result_by_team(id, Team::Chaos, winner.result_for_team(Team::Chaos))
            .await?;
        self.repo
            .find_by_id(id)
            .await?
            .ok_or(ServiceError::NotFound)
    }

    async fn abort(&self, requester_id: i32, id: Uuid, reason: String) -> ServiceResult<Match> {
        let m = self.get_owned(requester_id, id).await?;
        if !can_transition(m.status, MatchStatus::Aborted) {
            return Err(ServiceError::Conflict(format!(
                "不能从 {} 中止对局",
                m.status.as_str()
            )));
        }
        let reason = reason.trim();
        if reason.is_empty() {
            return Err(ServiceError::Validation("abort reason 不能为空".into()));
        }
        self.repo.update_abort(id, m.status, reason).await?;
        self.participant_repo
            .update_result_by_team(id, Team::Order, ParticipantResult::None)
            .await?;
        self.participant_repo
            .update_result_by_team(id, Team::Chaos, ParticipantResult::None)
            .await?;
        self.repo
            .find_by_id(id)
            .await?
            .ok_or(ServiceError::NotFound)
    }

    async fn append_event(
        &self,
        requester_id: i32,
        id: Uuid,
        event: MatchEventInput,
    ) -> ServiceResult<MatchEvent> {
        self.get_owned(requester_id, id).await?;
        Ok(self.event_repo.append(id, &event).await?)
    }

    async fn append_event_internal(
        &self,
        id: Uuid,
        event: MatchEventInput,
    ) -> ServiceResult<MatchEvent> {
        Ok(self.event_repo.append(id, &event).await?)
    }

    async fn get_events(
        &self,
        requester_id: i32,
        id: Uuid,
        from_seq: i32,
        limit: i64,
    ) -> ServiceResult<Vec<MatchEvent>> {
        self.get_owned(requester_id, id).await?;
        Ok(self.event_repo.list_by_match(id, from_seq, limit).await?)
    }
}

#[async_trait]
impl RankMatchCreator for MatchServiceImpl {
    async fn create_rank_match(
        &self,
        entry_a: &RankQueueEntry,
        entry_b: &RankQueueEntry,
    ) -> ServiceResult<Uuid> {
        let m = self
            .repo
            .insert(
                entry_a.user_id,
                &MatchInput {
                    form: MatchForm::Rank,
                    room_id: None,
                    mode: entry_a.mode.clone(),
                    scenario_id: None,
                    win_condition: None,
                },
            )
            .await?;

        self.participant_repo
            .insert(
                m.id,
                &ParticipantInput {
                    agent_snapshot_id: entry_a.agent_snapshot_id,
                    agent_id: entry_a.agent_id,
                    user_id: entry_a.user_id,
                    team: Team::Order,
                },
            )
            .await?;

        self.participant_repo
            .insert(
                m.id,
                &ParticipantInput {
                    agent_snapshot_id: entry_b.agent_snapshot_id,
                    agent_id: entry_b.agent_id,
                    user_id: entry_b.user_id,
                    team: Team::Chaos,
                },
            )
            .await?;

        Ok(m.id)
    }
}

#[cfg(test)]
mod tests {
    use mockall::mock;
    use mockall::predicate::*;

    use super::*;
    use crate::domain::RepoResult;
    use crate::domain::match_::{
        Match, MatchEvent, MatchForm, MatchParticipant, MatchStatus, ParticipantResult, Winner,
    };
    use crate::domain::spawn_preset::Team;
    use crate::repository::match_repo::ParticipantInput;

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
        pub ParticipantRepo {}
        #[async_trait]
        impl MatchParticipantRepo for ParticipantRepo {
            async fn find_by_match(&self, match_id: Uuid) -> RepoResult<Vec<MatchParticipant>>;
            async fn insert(&self, match_id: Uuid, input: &ParticipantInput) -> RepoResult<MatchParticipant>;
            async fn update_result(&self, id: Uuid, result: ParticipantResult, final_stats: Option<serde_json::Value>) -> RepoResult<()>;
            async fn update_entity_id(&self, id: Uuid, bevy_entity_id: i64) -> RepoResult<()>;
            async fn update_result_by_team(&self, match_id: Uuid, team: Team, result: ParticipantResult) -> RepoResult<u64>;
        }
    }

    mock! {
        pub EventRepo {}
        #[async_trait]
        impl MatchEventRepo for EventRepo {
            async fn append(&self, match_id: Uuid, event: &MatchEventInput) -> RepoResult<MatchEvent>;
            async fn list_by_match(&self, match_id: Uuid, from_seq: i32, limit: i64) -> RepoResult<Vec<MatchEvent>>;
        }
    }

    fn build_service(
        repo: MockMatchRepo,
        part: MockParticipantRepo,
        event: MockEventRepo,
    ) -> MatchServiceImpl {
        MatchServiceImpl {
            repo: Arc::new(repo),
            participant_repo: Arc::new(part),
            event_repo: Arc::new(event),
        }
    }

    fn sample_input() -> MatchInput {
        MatchInput {
            form: MatchForm::Local,
            room_id: None,
            mode: "1v1".into(),
            scenario_id: None,
            win_condition: None,
        }
    }

    fn sample_match(owner: i32, status: MatchStatus) -> Match {
        Match {
            id: Uuid::new_v4(),
            form: MatchForm::Local,
            room_id: None,
            owner_id: owner,
            mode: "1v1".into(),
            status,
            bevy_port: None,
            winner_team: None,
            abort_reason: None,
        }
    }

    #[tokio::test]
    async fn create_validates_empty_mode() {
        let mut input = sample_input();
        input.mode = "".into();
        let mut repo = MockMatchRepo::new();
        repo.expect_insert().times(0);
        let svc = build_service(repo, MockParticipantRepo::new(), MockEventRepo::new());
        assert!(matches!(
            svc.create(1, input).await.unwrap_err(),
            ServiceError::Validation(_)
        ));
    }

    #[tokio::test]
    async fn get_missing_not_found() {
        let mut repo = MockMatchRepo::new();
        repo.expect_find_by_id().returning(|_| Ok(None));
        let svc = build_service(repo, MockParticipantRepo::new(), MockEventRepo::new());
        assert!(matches!(
            svc.get(1, Uuid::new_v4()).await.unwrap_err(),
            ServiceError::NotFound
        ));
    }

    #[tokio::test]
    async fn start_when_finished_rejected() {
        let m = sample_match(1, MatchStatus::Finished);
        let mut repo = MockMatchRepo::new();
        repo.expect_find_by_id()
            .returning(move |_| Ok(Some(m.clone())));
        repo.expect_update_ports().times(0);
        repo.expect_update_status().times(0);
        let svc = build_service(repo, MockParticipantRepo::new(), MockEventRepo::new());
        assert!(matches!(
            svc.start(1, Uuid::new_v4(), 9100, 9101).await.unwrap_err(),
            ServiceError::Conflict(_)
        ));
    }

    #[tokio::test]
    async fn start_non_owner_forbidden() {
        let m = sample_match(1, MatchStatus::Pending);
        let mut repo = MockMatchRepo::new();
        repo.expect_find_by_id()
            .returning(move |_| Ok(Some(m.clone())));
        repo.expect_update_ports().times(0);
        let svc = build_service(repo, MockParticipantRepo::new(), MockEventRepo::new());
        assert!(matches!(
            svc.start(2, Uuid::new_v4(), 9100, 9101).await.unwrap_err(),
            ServiceError::Forbidden
        ));
    }

    #[tokio::test]
    async fn finish_when_pending_rejected() {
        let m = sample_match(1, MatchStatus::Pending);
        let mut repo = MockMatchRepo::new();
        repo.expect_find_by_id()
            .returning(move |_| Ok(Some(m.clone())));
        repo.expect_update_result().times(0);
        let mut part = MockParticipantRepo::new();
        part.expect_update_result_by_team().times(0);
        let svc = build_service(repo, part, MockEventRepo::new());
        assert!(matches!(
            svc.finish(1, Uuid::new_v4(), Winner::Order)
                .await
                .unwrap_err(),
            ServiceError::Conflict(_)
        ));
    }

    #[tokio::test]
    async fn abort_when_finished_rejected() {
        let m = sample_match(1, MatchStatus::Finished);
        let mut repo = MockMatchRepo::new();
        repo.expect_find_by_id()
            .returning(move |_| Ok(Some(m.clone())));
        repo.expect_update_abort().times(0);
        let svc = build_service(repo, MockParticipantRepo::new(), MockEventRepo::new());
        assert!(matches!(
            svc.abort(1, Uuid::new_v4(), "x".into()).await.unwrap_err(),
            ServiceError::Conflict(_)
        ));
    }

    #[tokio::test]
    async fn abort_empty_reason_rejected() {
        let m = sample_match(1, MatchStatus::Running);
        let mut repo = MockMatchRepo::new();
        repo.expect_find_by_id()
            .returning(move |_| Ok(Some(m.clone())));
        repo.expect_update_abort().times(0);
        let svc = build_service(repo, MockParticipantRepo::new(), MockEventRepo::new());
        assert!(matches!(
            svc.abort(1, Uuid::new_v4(), "   ".into())
                .await
                .unwrap_err(),
            ServiceError::Validation(_)
        ));
    }

    #[tokio::test]
    async fn append_event_non_owner_forbidden() {
        let m = sample_match(1, MatchStatus::Running);
        let mut repo = MockMatchRepo::new();
        repo.expect_find_by_id()
            .returning(move |_| Ok(Some(m.clone())));
        let mut event = MockEventRepo::new();
        event.expect_append().times(0);
        let svc = build_service(repo, MockParticipantRepo::new(), event);
        let ev = MatchEventInput {
            event_type: "move".into(),
            agent_id: None,
            payload: serde_json::json!({}),
            game_time_ms: 0,
        };
        assert!(matches!(
            svc.append_event(2, Uuid::new_v4(), ev).await.unwrap_err(),
            ServiceError::Forbidden
        ));
    }
}
