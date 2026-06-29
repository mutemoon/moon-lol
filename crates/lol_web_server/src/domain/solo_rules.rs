//! 上单 SOLO 模式胜负判定规则（纯函数裁决器，无 IO）。
//!
//! 胜负判定由 web server 的 match supervisor 执行（不在 Bevy 进程内）：
//! supervisor 订阅 Bevy 产出的结构化事件（champion_kill / turret_destroyed /
//! cs_threshold），维护 [`SoloState`]，每收到一个事件就调 [`evaluate`] 判定。
//!
//! 规则（与 docs/product/match/product.md §3.C 一致）：
//! 先达成任一即胜——拿一血 / 推掉对方一塔 / 补刀满 100；
//! 若游戏超过 15 分钟仍未分胜负，则按补刀数判定胜负（多者胜，相等为平局）。
//!
//! "先到先得"语义由调用方按事件到达顺序逐个喂入实现：每条事件只可能让状态前进，
//! [`evaluate`] 在状态首次满足任一胜利条件时返回 [`SoloVerdict`]，之后调用方应停止。

use serde::{Deserialize, Serialize};

use super::match_::Winner;

/// SOLO 规则参数。默认值：100 刀、15 分钟（900 秒）。
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SoloRule {
    /// 补刀获胜阈值。
    pub cs_target: u32,
    /// 超时阈值（秒）。超时后按补刀数判胜负。
    pub time_limit_secs: f64,
}

impl Default for SoloRule {
    fn default() -> Self {
        Self {
            cs_target: 100,
            time_limit_secs: 15.0 * 60.0,
        }
    }
}

/// SOLO 对局运行时状态，由事件流累积维护。
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct SoloState {
    /// Order 方英雄击杀数（用于判定一血：首次 0→1）。
    pub order_kills: u32,
    pub chaos_kills: u32,
    /// Order 方已摧毁的对方防御塔数（首次 0→1 即一塔）。
    pub order_towers: u32,
    pub chaos_towers: u32,
    /// 补刀数（Creep Score）。
    pub order_cs: u32,
    pub chaos_cs: u32,
    /// 对局已进行秒数。
    pub elapsed_secs: u32,
    /// 一旦命中胜负条件，置为 true；调用方应停止喂事件。
    pub decided: bool,
}

/// 胜负原因。
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SoloWinReason {
    /// 一血。
    FirstBlood,
    /// 一塔。
    FirstTower,
    /// 补刀达到阈值。
    CsTarget,
    /// 超时后按补刀数判定。
    CsDecided,
}

/// 裁决结果。
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct SoloVerdict {
    pub winner: Winner,
    pub reason: SoloWinReason,
}

/// 判定当前状态是否已产生胜负。
///
/// 检查顺序遵循"先到先得"：调用方逐事件推进状态后调用本函数，首个满足的条件即终局。
/// 注意：超时判定 (`CsDecided`) 要求 `elapsed_secs >= time_limit_secs`，由调用方在
/// 收到时间推进事件后调用。
pub fn evaluate(state: &SoloState, rule: &SoloRule) -> Option<SoloVerdict> {
    if state.decided {
        return None;
    }

    // 一血：任一方首次拿到击杀。
    if state.order_kills > 0 {
        return Some(SoloVerdict {
            winner: Winner::Order,
            reason: SoloWinReason::FirstBlood,
        });
    }
    if state.chaos_kills > 0 {
        return Some(SoloVerdict {
            winner: Winner::Chaos,
            reason: SoloWinReason::FirstBlood,
        });
    }

    // 一塔：任一方首次推掉对方塔。
    if state.order_towers > 0 {
        return Some(SoloVerdict {
            winner: Winner::Order,
            reason: SoloWinReason::FirstTower,
        });
    }
    if state.chaos_towers > 0 {
        return Some(SoloVerdict {
            winner: Winner::Chaos,
            reason: SoloWinReason::FirstTower,
        });
    }

    // 100 刀：补刀达到阈值。
    if state.order_cs >= rule.cs_target {
        return Some(SoloVerdict {
            winner: Winner::Order,
            reason: SoloWinReason::CsTarget,
        });
    }
    if state.chaos_cs >= rule.cs_target {
        return Some(SoloVerdict {
            winner: Winner::Chaos,
            reason: SoloWinReason::CsTarget,
        });
    }

    // 15 分钟超时：按补刀数判胜负，相等为平局。
    if state.elapsed_secs as f64 >= rule.time_limit_secs {
        let winner = match state.order_cs.cmp(&state.chaos_cs) {
            std::cmp::Ordering::Greater => Winner::Order,
            std::cmp::Ordering::Less => Winner::Chaos,
            std::cmp::Ordering::Equal => Winner::None,
        };
        return Some(SoloVerdict {
            winner,
            reason: SoloWinReason::CsDecided,
        });
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule() -> SoloRule {
        SoloRule::default()
    }

    fn state() -> SoloState {
        SoloState::default()
    }

    #[test]
    fn empty_state_no_verdict() {
        assert_eq!(evaluate(&state(), &rule()), None);
    }

    #[test]
    fn first_blood_order_wins() {
        let mut s = state();
        s.order_kills = 1;
        let v = evaluate(&s, &rule()).unwrap();
        assert_eq!(v.winner, Winner::Order);
        assert_eq!(v.reason, SoloWinReason::FirstBlood);
    }

    #[test]
    fn first_blood_chaos_wins() {
        let mut s = state();
        s.chaos_kills = 1;
        let v = evaluate(&s, &rule()).unwrap();
        assert_eq!(v.winner, Winner::Chaos);
        assert_eq!(v.reason, SoloWinReason::FirstBlood);
    }

    #[test]
    fn first_tower_order_wins() {
        let mut s = state();
        s.order_towers = 1;
        let v = evaluate(&s, &rule()).unwrap();
        assert_eq!(v.winner, Winner::Order);
        assert_eq!(v.reason, SoloWinReason::FirstTower);
    }

    #[test]
    fn first_tower_chaos_wins() {
        let mut s = state();
        s.chaos_towers = 1;
        let v = evaluate(&s, &rule()).unwrap();
        assert_eq!(v.winner, Winner::Chaos);
        assert_eq!(v.reason, SoloWinReason::FirstTower);
    }

    #[test]
    fn cs_target_order_wins() {
        let mut s = state();
        s.order_cs = 100;
        let v = evaluate(&s, &rule()).unwrap();
        assert_eq!(v.winner, Winner::Order);
        assert_eq!(v.reason, SoloWinReason::CsTarget);
    }

    #[test]
    fn cs_below_target_no_verdict() {
        let mut s = state();
        s.order_cs = 99;
        assert_eq!(evaluate(&s, &rule()), None);
    }

    #[test]
    fn timeout_cs_decided_order() {
        let mut s = state();
        s.elapsed_secs = 900;
        s.order_cs = 60;
        s.chaos_cs = 40;
        let v = evaluate(&s, &rule()).unwrap();
        assert_eq!(v.winner, Winner::Order);
        assert_eq!(v.reason, SoloWinReason::CsDecided);
    }

    #[test]
    fn timeout_cs_decided_chaos() {
        let mut s = state();
        s.elapsed_secs = 901;
        s.order_cs = 30;
        s.chaos_cs = 70;
        let v = evaluate(&s, &rule()).unwrap();
        assert_eq!(v.winner, Winner::Chaos);
        assert_eq!(v.reason, SoloWinReason::CsDecided);
    }

    #[test]
    fn timeout_cs_equal_is_draw() {
        let mut s = state();
        s.elapsed_secs = 900;
        s.order_cs = 50;
        s.chaos_cs = 50;
        let v = evaluate(&s, &rule()).unwrap();
        assert_eq!(v.winner, Winner::None);
        assert_eq!(v.reason, SoloWinReason::CsDecided);
    }

    #[test]
    fn below_timeout_no_cs_decided_even_if_cs_differs() {
        let mut s = state();
        s.elapsed_secs = 899;
        s.order_cs = 80;
        s.chaos_cs = 20;
        assert_eq!(evaluate(&s, &rule()), None);
    }

    #[test]
    fn first_blood_precedence_over_cs() {
        // 同时满足一血和补刀阈值，一血先判（因为先检查）。
        let mut s = state();
        s.order_kills = 1;
        s.order_cs = 100;
        let v = evaluate(&s, &rule()).unwrap();
        assert_eq!(v.reason, SoloWinReason::FirstBlood);
    }

    #[test]
    fn decided_state_returns_none() {
        let mut s = state();
        s.order_kills = 1;
        s.decided = true;
        assert_eq!(evaluate(&s, &rule()), None);
    }

    #[test]
    fn custom_rule_cs_target() {
        let rule = SoloRule {
            cs_target: 50,
            time_limit_secs: 600.0,
        };
        let mut s = state();
        s.order_cs = 50;
        let v = evaluate(&s, &rule).unwrap();
        assert_eq!(v.reason, SoloWinReason::CsTarget);
    }
}
