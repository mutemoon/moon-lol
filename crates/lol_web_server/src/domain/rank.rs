//! Rank 子系统的领域层（ELO 计算 + 赛季 + 匹配配对）。

use serde::{Deserialize, Serialize};

/// ELO 初始值。
pub const ELO_INITIAL: f64 = 1200.0;

/// ELO K 因子。
pub const ELO_K_FACTOR: f64 = 32.0;

/// 赛季状态。
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SeasonStatus {
    Scheduled,
    Active,
    Concluded,
}

impl SeasonStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            SeasonStatus::Scheduled => "scheduled",
            SeasonStatus::Active => "active",
            SeasonStatus::Concluded => "concluded",
        }
    }
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "scheduled" => Some(SeasonStatus::Scheduled),
            "active" => Some(SeasonStatus::Active),
            "concluded" => Some(SeasonStatus::Concluded),
            _ => None,
        }
    }
}

/// 队列状态。
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum QueueStatus {
    Queued,
    Matching,
    InMatch,
    Paused,
    Removed,
}

impl QueueStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            QueueStatus::Queued => "queued",
            QueueStatus::Matching => "matching",
            QueueStatus::InMatch => "in_match",
            QueueStatus::Paused => "paused",
            QueueStatus::Removed => "removed",
        }
    }
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "queued" => Some(QueueStatus::Queued),
            "matching" => Some(QueueStatus::Matching),
            "in_match" => Some(QueueStatus::InMatch),
            "paused" => Some(QueueStatus::Paused),
            "removed" => Some(QueueStatus::Removed),
            _ => None,
        }
    }
}

/// 对局结局（用于 ELO 更新）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Outcome {
    Win,
    Loss,
    Draw,
}

impl Outcome {
    /// 实际得分：胜=1, 平=0.5, 负=0。
    pub fn score(&self) -> f64 {
        match self {
            Outcome::Win => 1.0,
            Outcome::Draw => 0.5,
            Outcome::Loss => 0.0,
        }
    }
}

/// 预期胜率：E = 1 / (1 + 10^((R_opp - R_self) / 400))。
pub fn expected_score(rating_self: f64, rating_opponent: f64) -> f64 {
    1.0 / (1.0 + 10f64.powf((rating_opponent - rating_self) / 400.0))
}

/// 更新后的评分：R_new = R_old + K * (S - E)。
pub fn updated_rating(rating: f64, outcome: Outcome, expected: f64) -> f64 {
    rating + ELO_K_FACTOR * (outcome.score() - expected)
}

/// 一对选手对局后的 ELO 更新（双方同时更新）。
pub fn elo_exchange(winner_rating: f64, loser_rating: f64, outcome: Outcome) -> (f64, f64) {
    let exp_winner = expected_score(winner_rating, loser_rating);
    let exp_loser = expected_score(loser_rating, winner_rating);
    let (outcome_winner, outcome_loser) = match outcome {
        Outcome::Win => (Outcome::Win, Outcome::Loss),
        Outcome::Loss => (Outcome::Loss, Outcome::Win),
        Outcome::Draw => (Outcome::Draw, Outcome::Draw),
    };
    (
        updated_rating(winner_rating, outcome_winner, exp_winner),
        updated_rating(loser_rating, outcome_loser, exp_loser),
    )
}

/// 匹配配对：两个评分是否在匹配窗口内。
pub fn is_within_match_window(rating_a: f64, rating_b: f64, window: f64) -> bool {
    (rating_a - rating_b).abs() <= window
}

/// 匹配窗口随等待时间扩大（初始 50，每 10 秒 +10，上限 300）。
pub fn match_window_after_wait(wait_seconds: i64) -> f64 {
    let base = 50.0;
    let expansion = (wait_seconds / 10) as f64 * 10.0;
    (base + expansion).min(300.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expected_score_equal_ratings() {
        let e = expected_score(1200.0, 1200.0);
        assert!((e - 0.5).abs() < 0.001, "等分应预期 0.5");
    }

    #[test]
    fn expected_score_higher_rated_favored() {
        let e = expected_score(1800.0, 1500.0);
        assert!(e > 0.8, "高分方预期胜率应 >0.8, 实际 {e}");
    }

    #[test]
    fn expected_score_symmetric() {
        let e_high = expected_score(1800.0, 1500.0);
        let e_low = expected_score(1500.0, 1800.0);
        assert!(
            (e_high + e_low - 1.0).abs() < 0.001,
            "双方预期胜率之和应为 1"
        );
    }

    #[test]
    fn win_increases_rating() {
        let new_r = updated_rating(1200.0, Outcome::Win, expected_score(1200.0, 1200.0));
        assert!(new_r > 1200.0, "获胜应涨分");
    }

    #[test]
    fn loss_decreases_rating() {
        let new_r = updated_rating(1200.0, Outcome::Loss, expected_score(1200.0, 1200.0));
        assert!(new_r < 1200.0, "失败应掉分");
    }

    #[test]
    fn upset_gives_larger_delta() {
        // 1200 击败 1800：低分方涨分应多于 1200 击败 1200
        let (winner_new_upset, _) = elo_exchange(1200.0, 1800.0, Outcome::Win);
        let (winner_new_even, _) = elo_exchange(1200.0, 1200.0, Outcome::Win);
        assert!(
            winner_new_upset - 1200.0 > winner_new_even - 1200.0,
            "爆冷涨分应多于稳赢"
        );
    }

    #[test]
    fn draw_favors_lower_rated() {
        // 1800 vs 1500 平局：高分方掉分，低分方涨分
        let (high, low) = elo_exchange(1800.0, 1500.0, Outcome::Draw);
        assert!(high < 1800.0, "高分方平局应掉分");
        assert!(low > 1500.0, "低分方平局应涨分");
    }

    #[test]
    fn elo_exchange_sum_preserved() {
        // 双方评分变化之和应近似为 0（零和博弈）
        let (w, l) = elo_exchange(1200.0, 1300.0, Outcome::Win);
        let delta = (w - 1200.0) + (l - 1300.0);
        assert!(delta.abs() < 0.001, "ELO 变化应零和, delta={delta}");
    }

    #[test]
    fn match_window_expands_with_wait() {
        assert_eq!(match_window_after_wait(0), 50.0);
        assert_eq!(match_window_after_wait(10), 60.0);
        assert_eq!(match_window_after_wait(60), 110.0);
        assert_eq!(match_window_after_wait(1000), 300.0, "上限 300");
    }

    #[test]
    fn is_within_match_window_boundary() {
        assert!(is_within_match_window(1200.0, 1250.0, 50.0));
        assert!(is_within_match_window(1200.0, 1250.0, 50.0)); // 边界
        assert!(!is_within_match_window(1200.0, 1300.0, 50.0));
    }

    #[test]
    fn season_status_roundtrip() {
        assert_eq!(SeasonStatus::from_str("active"), Some(SeasonStatus::Active));
        assert_eq!(SeasonStatus::from_str("x"), None);
    }

    #[test]
    fn queue_status_roundtrip() {
        assert_eq!(QueueStatus::from_str("queued"), Some(QueueStatus::Queued));
        assert_eq!(
            QueueStatus::from_str("in_match"),
            Some(QueueStatus::InMatch)
        );
        assert_eq!(QueueStatus::from_str("x"), None);
    }

    #[test]
    fn outcome_score() {
        assert_eq!(Outcome::Win.score(), 1.0);
        assert_eq!(Outcome::Draw.score(), 0.5);
        assert_eq!(Outcome::Loss.score(), 0.0);
    }
}
