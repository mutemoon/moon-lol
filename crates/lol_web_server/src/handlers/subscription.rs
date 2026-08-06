//! Subscription 路由：当前套餐 / 订阅 / 套餐列表。

use axum::Json;
use axum::extract::State;
use lol_web_protocol::essence::{BillingPlan, SubscribeRequest};

use super::response::{ApiResponse, api_error};
use super::{AppState, AuthUser};

pub async fn get_subscription(
    auth: AuthUser,
    State(s): State<AppState>,
) -> ApiResponse<BillingPlan> {
    match s.subscription_service.get_active_plan(auth.user_id).await {
        Ok(plan) => ApiResponse::ok(plan.into()),
        Err(e) => api_error(e),
    }
}

pub async fn subscribe(
    auth: AuthUser,
    State(s): State<AppState>,
    Json(req): Json<SubscribeRequest>,
) -> ApiResponse<lol_web_protocol::essence::EssenceTransaction> {
    match s
        .subscription_service
        .subscribe(auth.user_id, &req.plan_id)
        .await
    {
        Ok(sub) => ApiResponse::ok(lol_web_protocol::essence::EssenceTransaction {
            id: 0,
            user_id: sub.user_id,
            amount: 0,
            reason: format!("subscribed {}", sub.plan_id),
            created_at: sub.period_start.to_rfc3339(),
        }),
        Err(e) => api_error(e),
    }
}

pub async fn list_plans(_auth: AuthUser) -> ApiResponse<Vec<BillingPlan>> {
    let plans: Vec<BillingPlan> = crate::domain::essence::BillingPlan::all()
        .into_iter()
        .map(Into::into)
        .collect();
    ApiResponse::ok(plans)
}
