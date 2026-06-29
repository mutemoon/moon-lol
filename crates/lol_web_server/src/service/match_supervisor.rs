//! Match Supervisor：每个托管对局一个 tokio task，订阅 Bevy 子进程的 WS，
//! 将引擎产出的对局事件套用 SOLO 胜负规则，落库对局结果并追加事件流。
//!
//! 设计（见 docs/product/match/arch.md）：
//! - 胜负判定在 web server 侧，不在 Bevy 进程内。
//! - Bevy 经 WS 推送 `match_event` 事件（champion_kill / turret_destroyed /
//!   cs_threshold / time_progress），本 supervisor 维护 [`SoloState`]，
//!   每条事件后调 [`solo_rules::evaluate`] 判定。
//! - 命中胜负 → 调 [`MatchService::finish_internal`] 落库；同时把每条事件
//!   [`MatchService::append_event_internal`] 写入 match_events 供 observe 轮询。
//! - Bevy 进程退出（WS 断开）→ task 结束。

use std::sync::Arc;

use futures_util::StreamExt;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::domain::match_::Winner;
use crate::domain::solo_rules::{SoloRule, SoloState, SoloVerdict, evaluate};
use crate::repository::match_repo::MatchEventInput;
use crate::service::match_service::MatchService;

/// WS 推送的 WsEvent 反序列化结构（只关心 match_event）。
#[derive(serde::Deserialize, Debug)]
struct WsEventEnvelope {
    #[serde(rename = "type")]
    msg_type: String,
    event: String,
    data: serde_json::Value,
}

/// 引擎产出的 match_event.data 反序列化结构（对应 lol_core::MatchEventOut）。
#[derive(serde::Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
enum MatchEventPayload {
    ChampionKill { killer_team: TeamSer },
    TurretDestroyed { killer_team: TeamSer },
    CsThreshold { team: TeamSer, cs: u32 },
    TimeProgress { elapsed_secs: f64 },
}

/// Team 的宽松解析：兼容 "Order"/"order" 等大小写。
#[derive(serde::Deserialize, Debug)]
#[serde(untagged)]
enum TeamSer {
    Named(String),
}

impl TeamSer {
    fn to_winner_side(&self) -> Option<TeamSide> {
        match self {
            TeamSer::Named(s) => match s.to_lowercase().as_str() {
                "order" => Some(TeamSide::Order),
                "chaos" => Some(TeamSide::Chaos),
                _ => None,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TeamSide {
    Order,
    Chaos,
}

impl TeamSide {
    fn winner(self) -> Winner {
        match self {
            TeamSide::Order => Winner::Order,
            TeamSide::Chaos => Winner::Chaos,
        }
    }
}

/// 启动一个对局的 supervisor task。
///
/// 连接 `ws://127.0.0.1:<ws_port>`，循环读取事件直到 WS 关闭或胜负落库。
/// 由 [`LocalGameService`](crate::service::local_game_service) 在启动子进程后 spawn。
pub async fn run_supervisor(
    match_id: Uuid,
    ws_port: i32,
    match_service: Arc<dyn MatchService>,
) {
    info!(
        "[supervisor] 启动 match {} supervisor，连接 ws://127.0.0.1:{}",
        match_id, ws_port
    );

    let url = format!("ws://127.0.0.1:{ws_port}");
    let mut state = SoloState::default();
    let rule = SoloRule::default();
    let mut decided = false;

    // 重试连接：Bevy 进程刚启动，WS 可能尚未就绪。
    let ws_stream = match connect_with_retry(&url).await {
        Some(s) => s,
        None => {
            warn!("[supervisor] match {} 无法连接 Bevy WS，supervisor 退出", match_id);
            return;
        }
    };
    let (_write, mut read) = ws_stream.split();

    while let Some(msg) = read.next().await {
        let text = match msg {
            Ok(Message::Text(t)) => t.to_string(),
            Ok(Message::Close(_)) | Err(_) => break,
            Ok(_) => continue,
        };

        let env: WsEventEnvelope = match serde_json::from_str(&text) {
            Ok(e) => e,
            Err(_) => continue,
        };

        if env.event != "match_event" {
            // 其他事件（game_loaded 等）忽略。
            continue;
        }

        let raw_data = env.data.clone();
        let payload: MatchEventPayload = match serde_json::from_value(raw_data.clone()) {
            Ok(p) => p,
            Err(e) => {
                debug!("[supervisor] match {} 解析 match_event 失败: {}", match_id, e);
                continue;
            }
        };

        // 推进 SoloState 并落库事件（落库用原始 data，保留引擎原文）。
        let (event_type, game_time_ms) = advance_state(&mut state, &payload);
        let _ = match_service
            .append_event_internal(
                match_id,
                MatchEventInput {
                    event_type,
                    agent_id: None,
                    payload: raw_data,
                    game_time_ms,
                },
            )
            .await;

        if decided {
            continue;
        }

        if let Some(SoloVerdict { winner, reason }) = evaluate(&state, &rule) {
            info!(
                "[supervisor] match {} 判定胜负: winner={:?} reason={:?}",
                match_id, winner, reason
            );
            match match_service.finish_internal(match_id, winner).await {
                Ok(_) => {
                    decided = true;
                    state.decided = true;
                }
                Err(e) => {
                    warn!("[supervisor] match {} finish_internal 失败: {:?}", match_id, e);
                }
            }
        }
    }

    info!("[supervisor] match {} supervisor 结束", match_id);
}

/// 根据事件推进 SoloState，返回 (event_type, game_time_ms) 用于落库。
fn advance_state(state: &mut SoloState, payload: &MatchEventPayload) -> (String, i64) {
    match payload {
        MatchEventPayload::ChampionKill { killer_team } => {
            if let Some(side) = killer_team.to_winner_side() {
                match side {
                    TeamSide::Order => state.order_kills += 1,
                    TeamSide::Chaos => state.chaos_kills += 1,
                }
            }
            ("champion_kill".to_string(), (state.elapsed_secs as i64) * 1000)
        }
        MatchEventPayload::TurretDestroyed { killer_team } => {
            if let Some(side) = killer_team.to_winner_side() {
                match side {
                    TeamSide::Order => state.order_towers += 1,
                    TeamSide::Chaos => state.chaos_towers += 1,
                }
            }
            ("turret_destroyed".to_string(), (state.elapsed_secs as i64) * 1000)
        }
        MatchEventPayload::CsThreshold { team, cs } => {
            if let Some(side) = team.to_winner_side() {
                match side {
                    TeamSide::Order => state.order_cs = state.order_cs.max(*cs),
                    TeamSide::Chaos => state.chaos_cs = state.chaos_cs.max(*cs),
                }
            }
            ("cs_threshold".to_string(), (state.elapsed_secs as i64) * 1000)
        }
        MatchEventPayload::TimeProgress { elapsed_secs } => {
            state.elapsed_secs = *elapsed_secs as u32;
            ("time_progress".to_string(), (*elapsed_secs * 1000.0) as i64)
        }
    }
}

/// 重试连接 Bevy WS：最多 30 秒，每 500ms 一次。
async fn connect_with_retry(
    url: &str,
) -> Option<tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>>
{
    for _ in 0..60 {
        match connect_async(url).await {
            Ok((stream, _)) => return Some(stream),
            Err(_) => tokio::time::sleep(std::time::Duration::from_millis(500)).await,
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn advance_champion_kill_order() {
        let mut s = SoloState::default();
        let (et, _) = advance_state(
            &mut s,
            &MatchEventPayload::ChampionKill {
                killer_team: TeamSer::Named("Order".into()),
            },
        );
        assert_eq!(et, "champion_kill");
        assert_eq!(s.order_kills, 1);
        assert_eq!(s.chaos_kills, 0);
    }

    #[test]
    fn advance_turret_destroyed_chaos() {
        let mut s = SoloState::default();
        advance_state(
            &mut s,
            &MatchEventPayload::TurretDestroyed {
                killer_team: TeamSer::Named("chaos".into()),
            },
        );
        assert_eq!(s.chaos_towers, 1);
    }

    #[test]
    fn advance_cs_threshold_takes_max() {
        let mut s = SoloState::default();
        s.order_cs = 100;
        advance_state(
            &mut s,
            &MatchEventPayload::CsThreshold {
                team: TeamSer::Named("order".into()),
                cs: 100,
            },
        );
        assert_eq!(s.order_cs, 100);
    }

    #[test]
    fn advance_time_progress_sets_elapsed() {
        let mut s = SoloState::default();
        let (_, ms) = advance_state(
            &mut s,
            &MatchEventPayload::TimeProgress { elapsed_secs: 900.0 },
        );
        assert_eq!(s.elapsed_secs, 900);
        assert_eq!(ms, 900000);
    }

    #[test]
    fn first_blood_after_champion_kill() {
        let mut s = SoloState::default();
        advance_state(
            &mut s,
            &MatchEventPayload::ChampionKill {
                killer_team: TeamSer::Named("Order".into()),
            },
        );
        let v = evaluate(&s, &SoloRule::default()).unwrap();
        assert_eq!(v.winner, Winner::Order);
    }
}
