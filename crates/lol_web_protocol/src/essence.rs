//! Essence wire DTO（精粹、订阅、套餐）。

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EssenceTransaction {
    pub id: i64,
    pub user_id: i32,
    pub amount: i64,
    pub reason: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckInResult {
    pub already_checked_in: bool,
    pub granted: i64,
    pub balance: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BillingPlan {
    pub id: String,
    pub name: String,
    pub monthly_essence: i64,
    pub agent_limit: i32,
    pub price_cents: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscribeRequest {
    pub plan_id: String,
}
