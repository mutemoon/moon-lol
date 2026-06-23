//! Essence（精粹）+ Subscription（订阅）子系统的领域层。

use serde::{Deserialize, Serialize};

/// 精粹交易原因。
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EssenceReason {
    Checkin,
    Recharge,
    TokenDeduction,
    AgentSlot,
}

impl EssenceReason {
    pub fn as_str(&self) -> &'static str {
        match self {
            EssenceReason::Checkin => "checkin",
            EssenceReason::Recharge => "recharge",
            EssenceReason::TokenDeduction => "token_deduction",
            EssenceReason::AgentSlot => "agent_slot",
        }
    }
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "checkin" => Some(EssenceReason::Checkin),
            "recharge" => Some(EssenceReason::Recharge),
            "token_deduction" => Some(EssenceReason::TokenDeduction),
            "agent_slot" => Some(EssenceReason::AgentSlot),
            _ => None,
        }
    }
}

/// 精粹流水记录。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EssenceTransaction {
    pub id: i64,
    pub user_id: i32,
    pub delta: i64,
    pub reason: String,
    pub reference: Option<String>,
    pub balance_after: i64,
}

/// 每日签到奖励（固定值）。
pub const DAILY_CHECKIN_REWARD: i64 = 100;

/// 校验扣款：余额是否足够。
pub fn can_deduct(balance: i64, amount: i64) -> Result<i64, EssenceError> {
    if amount < 0 {
        return Err(EssenceError::InvalidAmount);
    }
    let new_balance = balance - amount;
    if new_balance < 0 {
        return Err(EssenceError::Insufficient {
            balance,
            required: amount,
        });
    }
    Ok(new_balance)
}

#[derive(Debug, Clone, PartialEq)]
pub enum EssenceError {
    InvalidAmount,
    Insufficient { balance: i64, required: i64 },
}

/// 订阅档位 ID。
pub const PLAN_FREE: &str = "free";
pub const PLAN_PRO: &str = "pro";
pub const PLAN_ELITE: &str = "elite";

/// 订阅档位定义。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BillingPlan {
    pub id: String,
    pub name: String,
    pub price_cents: i32,
    pub essence_per_month: i64,
    pub max_agents: i32,
}

impl BillingPlan {
    /// 免费档。
    pub fn free() -> Self {
        BillingPlan {
            id: PLAN_FREE.into(),
            name: "免费版".into(),
            price_cents: 0,
            essence_per_month: 0,
            max_agents: 5,
        }
    }
    pub fn pro() -> Self {
        BillingPlan {
            id: PLAN_PRO.into(),
            name: "专业版".into(),
            price_cents: 2900,
            essence_per_month: 3000,
            max_agents: 20,
        }
    }
    pub fn elite() -> Self {
        BillingPlan {
            id: PLAN_ELITE.into(),
            name: "精英版".into(),
            price_cents: 9900,
            essence_per_month: 12000,
            max_agents: 100,
        }
    }

    pub fn all() -> Vec<BillingPlan> {
        vec![Self::free(), Self::pro(), Self::elite()]
    }
}

/// 订阅状态。
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SubscriptionStatus {
    Active,
    Expired,
    Cancelled,
}

impl SubscriptionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            SubscriptionStatus::Active => "active",
            SubscriptionStatus::Expired => "expired",
            SubscriptionStatus::Cancelled => "cancelled",
        }
    }
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "active" => Some(SubscriptionStatus::Active),
            "expired" => Some(SubscriptionStatus::Expired),
            "cancelled" => Some(SubscriptionStatus::Cancelled),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_deduct_sufficient_balance() {
        assert_eq!(can_deduct(1000, 300), Ok(700));
        assert_eq!(can_deduct(300, 300), Ok(0)); // 刚好
    }

    #[test]
    fn can_deduct_insufficient_rejected() {
        let err = can_deduct(100, 300).unwrap_err();
        assert_eq!(
            err,
            EssenceError::Insufficient {
                balance: 100,
                required: 300
            }
        );
    }

    #[test]
    fn can_deduct_negative_amount_rejected() {
        assert_eq!(
            can_deduct(1000, -1).unwrap_err(),
            EssenceError::InvalidAmount
        );
    }

    #[test]
    fn can_deduct_zero_allowed() {
        assert_eq!(can_deduct(0, 0), Ok(0));
    }

    #[test]
    fn free_plan_max_agents_5() {
        assert_eq!(BillingPlan::free().max_agents, 5);
        assert_eq!(BillingPlan::free().price_cents, 0);
    }

    #[test]
    fn pro_plan_more_agents() {
        assert!(BillingPlan::pro().max_agents > BillingPlan::free().max_agents);
        assert!(BillingPlan::pro().essence_per_month > 0);
    }

    #[test]
    fn all_plans_sorted_by_price() {
        let plans = BillingPlan::all();
        assert!(plans[0].price_cents <= plans[1].price_cents);
        assert!(plans[1].price_cents <= plans[2].price_cents);
    }

    #[test]
    fn essence_reason_roundtrip() {
        assert_eq!(
            EssenceReason::from_str("checkin"),
            Some(EssenceReason::Checkin)
        );
        assert_eq!(
            EssenceReason::from_str("token_deduction"),
            Some(EssenceReason::TokenDeduction)
        );
        assert_eq!(EssenceReason::from_str("x"), None);
    }

    #[test]
    fn subscription_status_roundtrip() {
        assert_eq!(
            SubscriptionStatus::from_str("active"),
            Some(SubscriptionStatus::Active)
        );
        assert_eq!(SubscriptionStatus::from_str("x"), None);
    }

    #[test]
    fn daily_checkin_reward_positive() {
        assert!(DAILY_CHECKIN_REWARD > 0);
    }
}
