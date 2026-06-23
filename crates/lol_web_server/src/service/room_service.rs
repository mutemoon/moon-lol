//! Room 子系统的 service 层。

use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::room::{
    Room, RoomAgentSlot, RoomConstraints, RoomStatus, RoomValidationError, validate_add_slot,
    validate_join, validate_room_name,
};
use crate::domain::spawn_preset::Team;
use crate::domain::{ServiceError, ServiceResult};
use crate::repository::room_repo::{CreateRoomInput, RoomRepo};

#[async_trait]
pub trait RoomService: Send + Sync {
    async fn create(
        &self,
        owner_id: i32,
        name: String,
        constraints: RoomConstraints,
    ) -> ServiceResult<Room>;
    async fn get(&self, requester_id: i32, room_id: Uuid) -> ServiceResult<Room>;
    async fn list_mine(&self, user_id: i32) -> ServiceResult<Vec<Room>>;
    async fn list_lobby(&self) -> ServiceResult<Vec<Room>>;
    async fn join_by_code(&self, user_id: i32, code: &str) -> ServiceResult<Room>;
    async fn join(&self, user_id: i32, room_id: Uuid) -> ServiceResult<()>;
    async fn leave(&self, user_id: i32, room_id: Uuid) -> ServiceResult<()>;
    async fn dissolve(&self, owner_id: i32, room_id: Uuid) -> ServiceResult<()>;
    async fn update_constraints(
        &self,
        owner_id: i32,
        room_id: Uuid,
        constraints: RoomConstraints,
    ) -> ServiceResult<()>;
    async fn add_slot(
        &self,
        user_id: i32,
        room_id: Uuid,
        agent_id: Uuid,
        team: Team,
    ) -> ServiceResult<RoomAgentSlot>;
    async fn remove_slot(
        &self,
        requester_id: i32,
        room_id: Uuid,
        slot_id: Uuid,
    ) -> ServiceResult<()>;
    async fn list_slots(
        &self,
        requester_id: i32,
        room_id: Uuid,
    ) -> ServiceResult<Vec<RoomAgentSlot>>;
}

pub struct RoomServiceImpl {
    pub repo: Arc<dyn RoomRepo>,
}

impl RoomServiceImpl {
    pub fn new(repo: Arc<dyn RoomRepo>) -> Self {
        Self { repo }
    }

    fn map_validation_err(e: RoomValidationError) -> ServiceError {
        match e {
            RoomValidationError::NotInLobby => ServiceError::Conflict("房间不在大厅状态".into()),
            RoomValidationError::AgentLimitExceeded { current, limit } => {
                ServiceError::Conflict(format!("Agent 槽位已达上限 ({current}/{limit})"))
            }
            RoomValidationError::MemberLimitExceeded { current, limit } => {
                ServiceError::Conflict(format!("房间人数已达上限 ({current}/{limit})"))
            }
            RoomValidationError::TeamPolicyViolation {
                existing,
                requested,
            } => ServiceError::Conflict(format!(
                "单阵营策略：已有 {:?} 槽位，不能加 {:?}",
                existing, requested
            )),
        }
    }
}

#[async_trait]
impl RoomService for RoomServiceImpl {
    async fn create(
        &self,
        owner_id: i32,
        name: String,
        constraints: RoomConstraints,
    ) -> ServiceResult<Room> {
        if !validate_room_name(&name) {
            return Err(ServiceError::Validation(
                "房间名不能为空且不超过 64 字符".into(),
            ));
        }
        Ok(self
            .repo
            .insert(owner_id, &CreateRoomInput { name, constraints })
            .await?)
    }

    async fn get(&self, requester_id: i32, room_id: Uuid) -> ServiceResult<Room> {
        let room = self
            .repo
            .find_by_id(room_id)
            .await?
            .ok_or(ServiceError::NotFound)?;
        if !self.repo.is_member(room_id, requester_id).await? && room.owner_id != requester_id {
            return Err(ServiceError::Forbidden);
        }
        Ok(room)
    }

    async fn list_mine(&self, user_id: i32) -> ServiceResult<Vec<Room>> {
        Ok(self.repo.list_by_member(user_id).await?)
    }

    async fn list_lobby(&self) -> ServiceResult<Vec<Room>> {
        Ok(self.repo.list_lobby().await?)
    }

    async fn join_by_code(&self, user_id: i32, code: &str) -> ServiceResult<Room> {
        let room = self
            .repo
            .find_by_invite_code(code)
            .await?
            .ok_or(ServiceError::NotFound)?;
        let member_count = self.repo.count_members(room.id).await?;
        validate_join(room.status, room.constraints, member_count as i32)
            .map_err(Self::map_validation_err)?;
        self.repo.add_member(room.id, user_id).await?;
        Ok(room)
    }

    async fn join(&self, user_id: i32, room_id: Uuid) -> ServiceResult<()> {
        let room = self
            .repo
            .find_by_id(room_id)
            .await?
            .ok_or(ServiceError::NotFound)?;
        if !room.constraints.lobby_visible {
            return Err(ServiceError::Forbidden);
        }
        let member_count = self.repo.count_members(room.id).await?;
        validate_join(room.status, room.constraints, member_count as i32)
            .map_err(Self::map_validation_err)?;
        self.repo.add_member(room.id, user_id).await?;
        Ok(())
    }

    async fn leave(&self, user_id: i32, room_id: Uuid) -> ServiceResult<()> {
        let room = self
            .repo
            .find_by_id(room_id)
            .await?
            .ok_or(ServiceError::NotFound)?;
        if room.owner_id == user_id {
            // 房主离开 = 解散
            self.repo.delete(room_id).await?;
            return Ok(());
        }
        self.repo.remove_member(room_id, user_id).await?;
        Ok(())
    }

    async fn dissolve(&self, owner_id: i32, room_id: Uuid) -> ServiceResult<()> {
        let room = self
            .repo
            .find_by_id(room_id)
            .await?
            .ok_or(ServiceError::NotFound)?;
        if room.owner_id != owner_id {
            return Err(ServiceError::Forbidden);
        }
        self.repo.delete(room_id).await?;
        Ok(())
    }

    async fn update_constraints(
        &self,
        owner_id: i32,
        room_id: Uuid,
        constraints: RoomConstraints,
    ) -> ServiceResult<()> {
        let room = self
            .repo
            .find_by_id(room_id)
            .await?
            .ok_or(ServiceError::NotFound)?;
        if room.owner_id != owner_id {
            return Err(ServiceError::Forbidden);
        }
        if room.status != RoomStatus::Lobby {
            return Err(ServiceError::Conflict("房间不在大厅状态".into()));
        }
        self.repo.update_constraints(room_id, constraints).await?;
        Ok(())
    }

    async fn add_slot(
        &self,
        user_id: i32,
        room_id: Uuid,
        agent_id: Uuid,
        team: Team,
    ) -> ServiceResult<RoomAgentSlot> {
        let room = self
            .repo
            .find_by_id(room_id)
            .await?
            .ok_or(ServiceError::NotFound)?;
        if !self.repo.is_member(room_id, user_id).await? {
            return Err(ServiceError::Forbidden);
        }
        // 先校验状态（避免在非 lobby 时仍查询 slots）
        use crate::domain::room::RoomValidationError;
        if let Err(e) = crate::domain::room::validate_add_slot(
            room.status,
            room.constraints,
            0, // 占位，状态校验优先；下面的完整校验会用真实 slot 数
            None,
            team,
        ) {
            if matches!(e, RoomValidationError::NotInLobby) {
                return Err(Self::map_validation_err(e));
            }
        }
        let member_slots = self.repo.count_slots_by_member(room_id, user_id).await? as i32;
        let existing_team = self.repo.member_existing_team(room_id, user_id).await?;
        validate_add_slot(
            room.status,
            room.constraints,
            member_slots,
            existing_team,
            team,
        )
        .map_err(Self::map_validation_err)?;
        Ok(self.repo.add_slot(room_id, user_id, agent_id, team).await?)
    }

    async fn remove_slot(
        &self,
        requester_id: i32,
        room_id: Uuid,
        slot_id: Uuid,
    ) -> ServiceResult<()> {
        let room = self
            .repo
            .find_by_id(room_id)
            .await?
            .ok_or(ServiceError::NotFound)?;
        let slots = self.repo.list_slots(room_id).await?;
        let slot = slots
            .iter()
            .find(|s| s.id == slot_id)
            .ok_or(ServiceError::NotFound)?;
        // 房主或槽位拥有者可移除
        if room.owner_id != requester_id && slot.user_id != requester_id {
            return Err(ServiceError::Forbidden);
        }
        self.repo.remove_slot(slot_id).await?;
        Ok(())
    }

    async fn list_slots(
        &self,
        requester_id: i32,
        room_id: Uuid,
    ) -> ServiceResult<Vec<RoomAgentSlot>> {
        // 必须是成员才能看槽位
        if !self.repo.is_member(room_id, requester_id).await? {
            let room = self
                .repo
                .find_by_id(room_id)
                .await?
                .ok_or(ServiceError::NotFound)?;
            if room.owner_id != requester_id {
                return Err(ServiceError::Forbidden);
            }
        }
        Ok(self.repo.list_slots(room_id).await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::room::{Room, RoomConstraints, RoomStatus, TeamPolicy};
    use crate::domain::{RepoError, RepoResult};
    use mockall::mock;
    use mockall::predicate::*;

    mock! {
        pub RoomRepo {}
        #[async_trait]
        impl RoomRepo for RoomRepo {
            async fn find_by_id(&self, id: Uuid) -> RepoResult<Option<Room>>;
            async fn find_by_invite_code(&self, code: &str) -> RepoResult<Option<Room>>;
            async fn list_by_member(&self, user_id: i32) -> RepoResult<Vec<Room>>;
            async fn list_lobby(&self) -> RepoResult<Vec<Room>>;
            async fn insert(&self, owner_id: i32, input: &CreateRoomInput) -> RepoResult<Room>;
            async fn update_status(&self, id: Uuid, status: RoomStatus) -> RepoResult<()>;
            async fn update_constraints(&self, id: Uuid, constraints: RoomConstraints) -> RepoResult<()>;
            async fn delete(&self, id: Uuid) -> RepoResult<()>;
            async fn count_members(&self, room_id: Uuid) -> RepoResult<i64>;
            async fn add_member(&self, room_id: Uuid, user_id: i32) -> RepoResult<()>;
            async fn remove_member(&self, room_id: Uuid, user_id: i32) -> RepoResult<()>;
            async fn is_member(&self, room_id: Uuid, user_id: i32) -> RepoResult<bool>;
            async fn list_slots(&self, room_id: Uuid) -> RepoResult<Vec<RoomAgentSlot>>;
            async fn count_slots_by_member(&self, room_id: Uuid, user_id: i32) -> RepoResult<i64>;
            async fn member_existing_team(&self, room_id: Uuid, user_id: i32) -> RepoResult<Option<Team>>;
            async fn add_slot(&self, room_id: Uuid, user_id: i32, agent_id: Uuid, team: Team) -> RepoResult<RoomAgentSlot>;
            async fn remove_slot(&self, slot_id: Uuid) -> RepoResult<()>;
        }
    }

    fn build_service(repo: MockRoomRepo) -> RoomServiceImpl {
        RoomServiceImpl {
            repo: Arc::new(repo),
        }
    }

    fn sample_room(owner: i32, status: RoomStatus) -> Room {
        Room {
            id: Uuid::new_v4(),
            owner_id: owner,
            name: "test".into(),
            invite_code: "ABCDEF".into(),
            constraints: RoomConstraints::default(),
            status,
        }
    }

    #[tokio::test]
    async fn create_invalid_name_rejected() {
        let svc = build_service(MockRoomRepo::new());
        let err = svc
            .create(1, "".into(), RoomConstraints::default())
            .await
            .unwrap_err();
        assert!(matches!(err, ServiceError::Validation(_)));
    }

    #[tokio::test]
    async fn create_success() {
        let room = sample_room(1, RoomStatus::Lobby);
        let room_clone = room.clone();
        let mut repo = MockRoomRepo::new();
        repo.expect_insert()
            .returning(move |_, _| Ok(room_clone.clone()));
        let svc = build_service(repo);
        let result = svc
            .create(1, "valid".into(), RoomConstraints::default())
            .await
            .unwrap();
        assert_eq!(result.owner_id, 1);
    }

    #[tokio::test]
    async fn join_by_code_room_not_found() {
        let mut repo = MockRoomRepo::new();
        repo.expect_find_by_invite_code().returning(|_| Ok(None));
        let svc = build_service(repo);
        let err = svc.join_by_code(1, "BADCODE").await.unwrap_err();
        assert!(matches!(err, ServiceError::NotFound));
    }

    #[tokio::test]
    async fn join_by_code_success() {
        let room = sample_room(1, RoomStatus::Lobby);
        let room_clone = room.clone();
        let mut repo = MockRoomRepo::new();
        repo.expect_find_by_invite_code()
            .returning(move |_| Ok(Some(room_clone.clone())));
        repo.expect_count_members().returning(|_| Ok(1));
        repo.expect_add_member().returning(|_, _| Ok(()));
        let svc = build_service(repo);
        svc.join_by_code(2, "ABCDEF").await.unwrap();
    }

    #[tokio::test]
    async fn join_by_code_full_rejected() {
        let room = sample_room(1, RoomStatus::Lobby);
        let mut repo = MockRoomRepo::new();
        repo.expect_find_by_invite_code()
            .returning(move |_| Ok(Some(room.clone())));
        repo.expect_count_members().returning(|_| Ok(10)); // 满员
        repo.expect_add_member().times(0);
        let svc = build_service(repo);
        let err = svc.join_by_code(2, "ABCDEF").await.unwrap_err();
        assert!(matches!(err, ServiceError::Conflict(_)));
    }

    #[tokio::test]
    async fn leave_owner_dissolves_room() {
        let room = sample_room(1, RoomStatus::Lobby);
        let mut repo = MockRoomRepo::new();
        repo.expect_find_by_id()
            .returning(move |_| Ok(Some(room.clone())));
        repo.expect_delete().with(always()).returning(|_| Ok(()));
        let svc = build_service(repo);
        svc.leave(1, Uuid::new_v4()).await.unwrap();
    }

    #[tokio::test]
    async fn leave_non_owner_removes_member() {
        let room = sample_room(1, RoomStatus::Lobby);
        let mut repo = MockRoomRepo::new();
        repo.expect_find_by_id()
            .returning(move |_| Ok(Some(room.clone())));
        repo.expect_remove_member().returning(|_, _| Ok(()));
        let svc = build_service(repo);
        svc.leave(2, Uuid::new_v4()).await.unwrap();
    }

    #[tokio::test]
    async fn dissolve_non_owner_forbidden() {
        let room = sample_room(1, RoomStatus::Lobby);
        let mut repo = MockRoomRepo::new();
        repo.expect_find_by_id()
            .returning(move |_| Ok(Some(room.clone())));
        repo.expect_delete().times(0);
        let svc = build_service(repo);
        let err = svc.dissolve(2, Uuid::new_v4()).await.unwrap_err();
        assert!(matches!(err, ServiceError::Forbidden));
    }

    #[tokio::test]
    async fn add_slot_success() {
        let room = sample_room(1, RoomStatus::Lobby);
        let mut repo = MockRoomRepo::new();
        repo.expect_find_by_id()
            .returning(move |_| Ok(Some(room.clone())));
        repo.expect_is_member().returning(|_, _| Ok(true));
        repo.expect_count_slots_by_member().returning(|_, _| Ok(0));
        repo.expect_member_existing_team()
            .returning(|_, _| Ok(None));
        repo.expect_add_slot().returning(|_, _, _, team| {
            Ok(RoomAgentSlot {
                id: Uuid::new_v4(),
                room_id: Uuid::new_v4(),
                user_id: 1,
                agent_id: Uuid::new_v4(),
                team,
            })
        });
        let svc = build_service(repo);
        svc.add_slot(1, Uuid::new_v4(), Uuid::new_v4(), Team::Order)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn add_slot_non_member_forbidden() {
        let room = sample_room(1, RoomStatus::Lobby);
        let mut repo = MockRoomRepo::new();
        repo.expect_find_by_id()
            .returning(move |_| Ok(Some(room.clone())));
        repo.expect_is_member().returning(|_, _| Ok(false));
        repo.expect_add_slot().times(0);
        let svc = build_service(repo);
        let err = svc
            .add_slot(2, Uuid::new_v4(), Uuid::new_v4(), Team::Order)
            .await
            .unwrap_err();
        assert!(matches!(err, ServiceError::Forbidden));
    }

    #[tokio::test]
    async fn add_slot_at_limit_rejected() {
        let room = sample_room(1, RoomStatus::Lobby);
        let mut repo = MockRoomRepo::new();
        repo.expect_find_by_id()
            .returning(move |_| Ok(Some(room.clone())));
        repo.expect_is_member().returning(|_, _| Ok(true));
        repo.expect_count_slots_by_member().returning(|_, _| Ok(3)); // 满了
        repo.expect_member_existing_team()
            .returning(|_, _| Ok(Some(Team::Order)));
        repo.expect_add_slot().times(0);
        let svc = build_service(repo);
        let err = svc
            .add_slot(1, Uuid::new_v4(), Uuid::new_v4(), Team::Order)
            .await
            .unwrap_err();
        assert!(matches!(err, ServiceError::Conflict(_)));
    }

    #[tokio::test]
    async fn add_slot_not_in_lobby_rejected() {
        let room = sample_room(1, RoomStatus::Running);
        let mut repo = MockRoomRepo::new();
        repo.expect_find_by_id()
            .returning(move |_| Ok(Some(room.clone())));
        repo.expect_is_member().returning(|_, _| Ok(true));
        repo.expect_add_slot().times(0);
        let svc = build_service(repo);
        let err = svc
            .add_slot(1, Uuid::new_v4(), Uuid::new_v4(), Team::Order)
            .await
            .unwrap_err();
        assert!(matches!(err, ServiceError::Conflict(_)));
    }

    #[tokio::test]
    async fn remove_slot_owner_allowed() {
        let room = sample_room(1, RoomStatus::Lobby);
        let slot = RoomAgentSlot {
            id: Uuid::new_v4(),
            room_id: room.id,
            user_id: 2,
            agent_id: Uuid::new_v4(),
            team: Team::Order,
        };
        let slot_clone = slot.clone();
        let mut repo = MockRoomRepo::new();
        repo.expect_find_by_id()
            .returning(move |_| Ok(Some(room.clone())));
        repo.expect_list_slots()
            .returning(move |_| Ok(vec![slot_clone.clone()]));
        repo.expect_remove_slot().returning(|_| Ok(()));
        let svc = build_service(repo);
        svc.remove_slot(1, Uuid::new_v4(), slot.id).await.unwrap();
    }

    #[tokio::test]
    async fn remove_slot_non_owner_non_holder_forbidden() {
        let room = sample_room(1, RoomStatus::Lobby);
        let slot = RoomAgentSlot {
            id: Uuid::new_v4(),
            room_id: room.id,
            user_id: 2,
            agent_id: Uuid::new_v4(),
            team: Team::Order,
        };
        let slot_clone = slot.clone();
        let mut repo = MockRoomRepo::new();
        repo.expect_find_by_id()
            .returning(move |_| Ok(Some(room.clone())));
        repo.expect_list_slots()
            .returning(move |_| Ok(vec![slot_clone.clone()]));
        repo.expect_remove_slot().times(0);
        let svc = build_service(repo);
        let err = svc
            .remove_slot(3, Uuid::new_v4(), slot.id)
            .await
            .unwrap_err();
        assert!(matches!(err, ServiceError::Forbidden));
    }
}
