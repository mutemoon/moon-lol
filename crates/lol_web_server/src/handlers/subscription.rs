//! Subscription 路由：当前套餐 / 订阅 / 套餐列表。

use axum::extract::State;
use axum::Json;
use serde::Deserialize;

use super::response::ApiResponse;
use super::{AppState, AuthUser};

pub async fn get_subscription(
    auth: AuthUser,
    State(s): State<AppState>,
) -> ApiResponse<crate::domain::essence::BillingPlan> {
    match s.subscription_service.get_active_plan(auth.user_id).await {
        Ok(plan) => ApiResponse::ok(plan),
        Err(e) => ApiResponse::from_error(e),
    }
}

#[derive(Deserialize)]
pub struct SubscribeRequest {
    pub plan_id: String,
}

pub async fn subscribe(
    auth: AuthUser,
    State(s): State<AppState>,
    Json(req): Json<SubscribeRequest>,
) -> ApiResponse<crate::repository::essence_repo::Subscription> {
    match s
        .subscription_service
        .subscribe(auth.user_id, &req.plan_id)
        .await
    {
        Ok(sub) => ApiResponse::ok(sub),
        Err(e) => ApiResponse::from_error(e),
    }
}

pub async fn list_plans(_auth: AuthUser) -> ApiResponse<Vec<crate::domain::essence::BillingPlan>> {
    ApiResponse::ok(crate::domain::essence::BillingPlan::all())
}
