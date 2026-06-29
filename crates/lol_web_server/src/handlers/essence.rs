//! Essence 路由：余额 / 签到 / 交易流水。

use axum::extract::{Query, State};
use serde::{Deserialize, Serialize};

use super::response::ApiResponse;
use super::{AppState, AuthUser};

pub async fn essence_balance(auth: AuthUser, State(s): State<AppState>) -> ApiResponse<i64> {
    match s.essence_service.get_balance(auth.user_id).await {
        Ok(b) => ApiResponse::ok(b),
        Err(e) => ApiResponse::from_error(e),
    }
}

#[derive(Serialize)]
pub struct CheckInDto {
    pub already_checked_in: bool,
    pub granted: i64,
    pub balance: i64,
}

pub async fn essence_check_in(
    auth: AuthUser,
    State(s): State<AppState>,
) -> ApiResponse<CheckInDto> {
    let date = chrono::Utc::now().format("%Y-%m-%d").to_string();
    match s.essence_service.check_in(auth.user_id, &date).await {
        Ok(r) => ApiResponse::ok(CheckInDto {
            already_checked_in: r.already_checked_in,
            granted: r.granted,
            balance: r.balance,
        }),
        Err(e) => ApiResponse::from_error(e),
    }
}

#[derive(Deserialize)]
pub struct TransactionsQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn essence_transactions(
    auth: AuthUser,
    State(s): State<AppState>,
    Query(q): Query<TransactionsQuery>,
) -> ApiResponse<Vec<crate::domain::essence::EssenceTransaction>> {
    match s
        .essence_service
        .get_transactions(auth.user_id, q.limit.unwrap_or(50), q.offset.unwrap_or(0))
        .await
    {
        Ok(list) => ApiResponse::ok(list),
        Err(e) => ApiResponse::from_error(e),
    }
}
