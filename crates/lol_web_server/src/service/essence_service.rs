//! Essence + Subscription 子系统的 service 层。

use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};

use crate::domain::essence::{
    BillingPlan, DAILY_CHECKIN_REWARD, EssenceError, EssenceReason, EssenceTransaction, PLAN_FREE,
    can_deduct,
};
use crate::domain::{ServiceError, ServiceResult};
use crate::repository::essence_repo::{EssenceRepo, Subscription, SubscriptionRepo};
use crate::service::agent_service::AgentLimitProvider;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckInResult {
    pub already_checked_in: bool,
    pub granted: i64,
    pub balance: i64,
}

#[async_trait]
pub trait EssenceService: Send + Sync {
    async fn get_balance(&self, user_id: i32) -> ServiceResult<i64>;
    async fn check_in(&self, user_id: i32, date: &str) -> ServiceResult<CheckInResult>;
    async fn deduct(&self, user_id: i32, amount: i64, reason: &str) -> ServiceResult<i64>;
    async fn grant(&self, user_id: i32, amount: i64, reason: &str) -> ServiceResult<i64>;
    async fn get_transactions(
        &self,
        user_id: i32,
        limit: i64,
        offset: i64,
    ) -> ServiceResult<Vec<EssenceTransaction>>;
}

pub struct EssenceServiceImpl {
    pub repo: Arc<dyn EssenceRepo>,
}

impl EssenceServiceImpl {
    pub fn new(repo: Arc<dyn EssenceRepo>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl EssenceService for EssenceServiceImpl {
    async fn get_balance(&self, user_id: i32) -> ServiceResult<i64> {
        Ok(self.repo.get_balance(user_id).await?)
    }

    async fn check_in(&self, user_id: i32, date: &str) -> ServiceResult<CheckInResult> {
        let reference = format!("checkin_{date}");
        if let Some(_existing) = self.repo.find_by_reference(user_id, &reference).await? {
            let balance = self.repo.get_balance(user_id).await?;
            return Ok(CheckInResult {
                already_checked_in: true,
                granted: 0,
                balance,
            });
        }
        let balance = self
            .repo
            .add_transaction(
                user_id,
                DAILY_CHECKIN_REWARD,
                EssenceReason::Checkin.as_str().to_string(),
                Some(reference),
            )
            .await?;
        Ok(CheckInResult {
            already_checked_in: false,
            granted: DAILY_CHECKIN_REWARD,
            balance,
        })
    }

    async fn deduct(&self, user_id: i32, amount: i64, reason: &str) -> ServiceResult<i64> {
        let balance = self.repo.get_balance(user_id).await?;
        let new_balance = match can_deduct(balance, amount) {
            Ok(v) => v,
            Err(EssenceError::Insufficient { balance, required }) => {
                return Err(ServiceError::InsufficientEssence { required, balance });
            }
            Err(EssenceError::InvalidAmount) => {
                return Err(ServiceError::Validation("扣款金额必须非负".into()));
            }
        };
        self.repo
            .add_transaction(user_id, -amount, reason.to_string(), None)
            .await?;
        Ok(new_balance)
    }

    async fn grant(&self, user_id: i32, amount: i64, reason: &str) -> ServiceResult<i64> {
        if amount <= 0 {
            return Err(ServiceError::Validation("发放金额必须为正".into()));
        }
        Ok(self
            .repo
            .add_transaction(user_id, amount, reason.to_string(), None)
            .await?)
    }

    async fn get_transactions(
        &self,
        user_id: i32,
        limit: i64,
        offset: i64,
    ) -> ServiceResult<Vec<EssenceTransaction>> {
        Ok(self.repo.get_transactions(user_id, limit, offset).await?)
    }
}

#[async_trait]
pub trait SubscriptionService: Send + Sync {
    async fn get_active_plan(&self, user_id: i32) -> ServiceResult<BillingPlan>;
    async fn subscribe(&self, user_id: i32, plan_id: &str) -> ServiceResult<Subscription>;
    async fn deactivate(&self, user_id: i32) -> ServiceResult<()>;
    async fn get_agent_limit(&self, user_id: i32) -> ServiceResult<usize>;
}

pub struct SubscriptionServiceImpl {
    pub sub_repo: Arc<dyn SubscriptionRepo>,
    pub period_days: i64,
}

impl SubscriptionServiceImpl {
    pub fn new(sub_repo: Arc<dyn SubscriptionRepo>) -> Self {
        Self {
            sub_repo,
            period_days: 30,
        }
    }
}

#[async_trait]
impl SubscriptionService for SubscriptionServiceImpl {
    async fn get_active_plan(&self, user_id: i32) -> ServiceResult<BillingPlan> {
        let active = self.sub_repo.get_active(user_id).await?;
        let plan_id = match active {
            Some(s) => s.plan_id,
            None => return Ok(BillingPlan::free()),
        };
        match self.sub_repo.get_plan(&plan_id).await? {
            Some(p) => Ok(p),
            None => Ok(BillingPlan::free()),
        }
    }

    async fn subscribe(&self, user_id: i32, plan_id: &str) -> ServiceResult<Subscription> {
        let plan = self
            .sub_repo
            .get_plan(plan_id)
            .await?
            .ok_or_else(|| ServiceError::Validation(format!("未知套餐: {plan_id}")))?;
        if plan.id == PLAN_FREE {
            return Err(ServiceError::Validation("免费档无需订阅".into()));
        }
        self.sub_repo.deactivate(user_id).await?;
        let now: DateTime<Utc> = Utc::now();
        let end = now + Duration::days(self.period_days);
        Ok(self
            .sub_repo
            .insert(user_id, &plan.id, now, end, false)
            .await?)
    }

    async fn deactivate(&self, user_id: i32) -> ServiceResult<()> {
        self.sub_repo.deactivate(user_id).await?;
        Ok(())
    }

    async fn get_agent_limit(&self, user_id: i32) -> ServiceResult<usize> {
        let plan = self.get_active_plan(user_id).await?;
        Ok(plan.max_agents.max(0) as usize)
    }
}

#[async_trait]
impl AgentLimitProvider for SubscriptionServiceImpl {
    async fn get_agent_limit(&self, user_id: i32) -> ServiceResult<usize> {
        SubscriptionService::get_agent_limit(self, user_id).await
    }
}

#[cfg(test)]
mod tests {
    use mockall::mock;
    use uuid::Uuid;

    use super::*;
    use crate::domain::RepoResult;

    mock! {
        pub EssenceRepo {}
        #[async_trait]
        impl EssenceRepo for EssenceRepo {
            async fn get_balance(&self, user_id: i32) -> RepoResult<i64>;
            async fn add_transaction(&self, user_id: i32, delta: i64, reason: String, reference: Option<String>) -> RepoResult<i64>;
            async fn get_transactions(&self, user_id: i32, limit: i64, offset: i64) -> RepoResult<Vec<EssenceTransaction>>;
            async fn find_by_reference(&self, user_id: i32, reference: &str) -> RepoResult<Option<EssenceTransaction>>;
        }
    }

    mock! {
        pub SubscriptionRepo {}
        #[async_trait]
        impl SubscriptionRepo for SubscriptionRepo {
            async fn get_active(&self, user_id: i32) -> RepoResult<Option<Subscription>>;
            async fn insert(&self, user_id: i32, plan_id: &str, period_start: DateTime<Utc>, period_end: DateTime<Utc>, auto_renew: bool) -> RepoResult<Subscription>;
            async fn deactivate(&self, user_id: i32) -> RepoResult<()>;
            async fn get_plan(&self, plan_id: &str) -> RepoResult<Option<BillingPlan>>;
        }
    }

    #[tokio::test]
    async fn check_in_grants_first_time() {
        let mut repo = MockEssenceRepo::new();
        repo.expect_find_by_reference().returning(|_, _| Ok(None));
        repo.expect_add_transaction()
            .returning(|_, _, _, _| Ok(DAILY_CHECKIN_REWARD));
        let svc = EssenceServiceImpl::new(Arc::new(repo));
        let r = svc.check_in(1, "2026-06-23").await.unwrap();
        assert!(!r.already_checked_in);
        assert_eq!(r.granted, DAILY_CHECKIN_REWARD);
    }

    #[tokio::test]
    async fn check_in_idempotent_same_day() {
        let mut repo = MockEssenceRepo::new();
        repo.expect_find_by_reference().returning(|_, _| {
            Ok(Some(EssenceTransaction {
                id: 1,
                user_id: 1,
                delta: DAILY_CHECKIN_REWARD,
                reason: "checkin".into(),
                reference: Some("checkin_2026-06-23".into()),
                balance_after: DAILY_CHECKIN_REWARD,
            }))
        });
        repo.expect_add_transaction().times(0);
        repo.expect_get_balance()
            .returning(|_| Ok(DAILY_CHECKIN_REWARD));
        let svc = EssenceServiceImpl::new(Arc::new(repo));
        let r = svc.check_in(1, "2026-06-23").await.unwrap();
        assert!(r.already_checked_in);
        assert_eq!(r.granted, 0);
    }

    #[tokio::test]
    async fn deduct_insufficient_rejected() {
        let mut repo = MockEssenceRepo::new();
        repo.expect_get_balance().returning(|_| Ok(50));
        repo.expect_add_transaction().times(0);
        let svc = EssenceServiceImpl::new(Arc::new(repo));
        let err = svc.deduct(1, 300, "x").await.unwrap_err();
        match err {
            ServiceError::InsufficientEssence { required, balance } => {
                assert_eq!(required, 300);
                assert_eq!(balance, 50);
            }
            other => panic!("期望 InsufficientEssence，实际 {other:?}"),
        }
    }

    #[tokio::test]
    async fn deduct_negative_rejected() {
        let mut repo = MockEssenceRepo::new();
        repo.expect_get_balance().returning(|_| Ok(1000));
        repo.expect_add_transaction().times(0);
        let svc = EssenceServiceImpl::new(Arc::new(repo));
        assert!(matches!(
            svc.deduct(1, -5, "x").await.unwrap_err(),
            ServiceError::Validation(_)
        ));
    }

    #[tokio::test]
    async fn grant_non_positive_rejected() {
        let mut repo = MockEssenceRepo::new();
        repo.expect_add_transaction().times(0);
        let svc = EssenceServiceImpl::new(Arc::new(repo));
        assert!(svc.grant(1, 0, "x").await.is_err());
    }

    #[tokio::test]
    async fn no_subscription_returns_free() {
        let mut repo = MockSubscriptionRepo::new();
        repo.expect_get_active().returning(|_| Ok(None));
        let svc = SubscriptionServiceImpl::new(Arc::new(repo));
        assert_eq!(
            SubscriptionService::get_agent_limit(&svc, 1).await.unwrap(),
            5
        );
    }

    #[tokio::test]
    async fn subscribe_unknown_plan_rejected() {
        let mut repo = MockSubscriptionRepo::new();
        repo.expect_get_plan().returning(|_| Ok(None));
        repo.expect_deactivate().times(0);
        let svc = SubscriptionServiceImpl::new(Arc::new(repo));
        assert!(matches!(
            svc.subscribe(1, "ghost").await.unwrap_err(),
            ServiceError::Validation(_)
        ));
    }

    #[tokio::test]
    async fn subscribe_free_plan_rejected() {
        let mut repo = MockSubscriptionRepo::new();
        repo.expect_get_plan()
            .returning(|_| Ok(Some(BillingPlan::free())));
        repo.expect_deactivate().times(0);
        let svc = SubscriptionServiceImpl::new(Arc::new(repo));
        assert!(matches!(
            svc.subscribe(1, "free").await.unwrap_err(),
            ServiceError::Validation(_)
        ));
    }

    #[tokio::test]
    async fn agent_limit_pro_is_20() {
        let mut repo = MockSubscriptionRepo::new();
        repo.expect_get_active().returning(|_| {
            Ok(Some(Subscription {
                id: Uuid::new_v4(),
                user_id: 1,
                plan_id: "pro".into(),
                status: crate::domain::essence::SubscriptionStatus::Active,
                period_start: Utc::now(),
                period_end: Utc::now() + Duration::days(30),
                auto_renew: false,
            }))
        });
        repo.expect_get_plan()
            .returning(|_| Ok(Some(BillingPlan::pro())));
        let svc = SubscriptionServiceImpl::new(Arc::new(repo));
        assert_eq!(
            SubscriptionService::get_agent_limit(&svc, 1).await.unwrap(),
            20
        );
    }
}
