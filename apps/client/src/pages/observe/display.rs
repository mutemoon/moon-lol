//! 观战/回放页展示辅助：时间线文案、双方阵容回填、信息行。

use gpui::prelude::*;
use gpui::*;
use gpui_component::h_flex;
use lol_web_protocol::match_::MatchEvent;

use super::types::RosterAgent;

pub(super) fn short_id(id: &str) -> String {
    if id.len() > 8 {
        id[..8].to_string()
    } else {
        id.to_string()
    }
}

pub(super) fn fmt_date(iso: &str) -> String {
    if iso.len() >= 16 {
        iso[..16].replace('T', " ")
    } else {
        iso.to_string()
    }
}

fn team_label(team: Option<&str>) -> String {
    match team {
        Some("order") => "蓝方".to_string(),
        Some("chaos") => "红方".to_string(),
        Some(t) => t.to_string(),
        None => "未知".to_string(),
    }
}

/// 事件类型 → 时间线文案（payload 字段以 `lol_web_protocol::match_` 契约为准：
/// `event_type` / `agent_id` / `game_time_ms` 由服务端回填进 payload）。
pub(super) fn event_label(ev: &MatchEvent) -> String {
    let p = &ev.payload;
    let team = |key: &str| team_label(p.get(key).and_then(|v| v.as_str()));
    match p.get("event_type").and_then(|v| v.as_str()) {
        Some("champion_kill") => format!("{} 击杀一名英雄", team("killer_team")),
        Some("turret_destroyed") => format!("{} 摧毁防御塔", team("killer_team")),
        Some("cs_threshold") => format!(
            "{} 补刀达到 {} 触发阈值",
            team("team"),
            p.get("cs").and_then(|v| v.as_i64()).unwrap_or(0)
        ),
        Some("time_progress") => {
            let secs = p
                .get("elapsed_secs")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            format!("对局进行中 · {} 秒", secs.round() as i64)
        }
        Some("agent_join") => format!(
            "{}（{}）加入对局",
            p.get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("未知 Agent"),
            team("team")
        ),
        Some("agent_stalled") => format!(
            "{} 动力源失联，对局暂停等待恢复",
            p.get("agent_name")
                .and_then(|v| v.as_str())
                .unwrap_or("Agent")
        ),
        Some("agent_resumed") => format!(
            "{} 恢复连接",
            p.get("agent_name")
                .and_then(|v| v.as_str())
                .unwrap_or("Agent")
        ),
        Some("match_finished") => format!(
            "对局结束，胜方 {}",
            p.get("winner").and_then(|v| v.as_str()).unwrap_or("未知")
        ),
        _ => p
            .get("event_type")
            .and_then(|v| v.as_str())
            .unwrap_or("event")
            .to_string(),
    }
}

/// 从事件时间线回填双方阵容与失联 Agent。
/// 真实引擎事件没有 agent_join，阵容可能为空（与 client observe 页行为一致）。
pub(super) fn build_rosters(
    events: &[MatchEvent],
) -> (Vec<RosterAgent>, Vec<RosterAgent>, Vec<String>) {
    let mut order = Vec::new();
    let mut chaos = Vec::new();
    let mut stalled: Vec<String> = Vec::new();
    for ev in events {
        let Some(et) = ev.payload.get("event_type").and_then(|v| v.as_str()) else {
            continue;
        };
        match et {
            "agent_join" => {
                let agent = RosterAgent {
                    id: ev
                        .payload
                        .get("agent_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    name: ev
                        .payload
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("未知 Agent")
                        .to_string(),
                    champion: ev
                        .payload
                        .get("champion")
                        .and_then(|v| v.as_str())
                        .unwrap_or("—")
                        .to_string(),
                };
                match ev.payload.get("team").and_then(|v| v.as_str()) {
                    Some("order") => order.push(agent),
                    Some("chaos") => chaos.push(agent),
                    _ => {}
                }
            }
            "agent_stalled" => {
                if let Some(id) = ev.payload.get("agent_id").and_then(|v| v.as_str()) {
                    if !stalled.iter().any(|s| s == id) {
                        stalled.push(id.to_string());
                    }
                }
            }
            "agent_resumed" => {
                if let Some(id) = ev.payload.get("agent_id").and_then(|v| v.as_str()) {
                    stalled.retain(|s| s != id);
                }
            }
            _ => {}
        }
    }
    (order, chaos, stalled)
}

pub(super) fn info_row(label: &str, value: String, muted: Hsla, fg: Hsla) -> AnyElement {
    h_flex()
        .gap_2()
        .items_center()
        .text_xs()
        .child(
            div()
                .w(rems(5.))
                .flex_shrink_0()
                .text_color(muted)
                .child(label.to_string()),
        )
        .child(div().text_color(fg).child(value))
        .into_any_element()
}
