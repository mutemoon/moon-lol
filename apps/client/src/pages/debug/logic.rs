//! 对局调试台逻辑：事件解析 / 时间格式化 / 初始化订阅与对局控制命令。

use gpui::*;

use super::types::{update_state, with_state, MatchCmd};
use crate::components::agent_chat_history::AgentChatMessage;
use crate::components::game_console_logs::ConsoleLogRow;
use crate::components::sidebar::AppSidebar;
use crate::services::types::LogQueryParams;
use crate::services::{match_ws, provider};

// ── 事件解析 ──

/// 事件名转中文标签。
fn event_label(event: &str) -> &'static str {
    match event {
        "game_loaded" => "对局加载完成",
        "game_paused" => "对局暂停",
        "game_close" => "对局连接关闭",
        "champion_changed" => "调试英雄切换",
        "entity_selected" => "实体选中",
        "match_event" => "对局事件",
        "champion_kill" => "英雄击杀",
        "turret_destroyed" => "防御塔被摧毁",
        "cs_threshold" => "补刀里程碑",
        "time_progress" => "对局时间推进",
        _ => "事件",
    }
}

/// 把 data 对象里的关键字段拼成可读文本（简化实现，未知字段忽略）。
fn format_event_data(data: Option<&serde_json::Value>) -> String {
    let Some(obj) = data.and_then(|d| d.as_object()) else {
        return String::new();
    };
    let mut parts: Vec<String> = Vec::new();
    for key in [
        "name",
        "entity_id",
        "kind",
        "reason",
        "paused",
        "killer_team",
        "team",
        "cs",
        "elapsed_secs",
    ] {
        if let Some(v) = obj.get(key) {
            let text = match v {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Bool(b) => b.to_string(),
                serde_json::Value::Number(n) => n.to_string(),
                _ => continue,
            };
            parts.push(format!("{key}={text}"));
        }
    }
    parts.join(" ")
}

/// 事件 JSON → 控制台日志行。简化：把事件名 + data 关键字段拼成文本。
fn event_to_log(val: &serde_json::Value) -> Option<ConsoleLogRow> {
    let obj = val.as_object()?;
    let event = obj.get("event").and_then(|v| v.as_str()).unwrap_or("");
    let msg_type = obj.get("type").and_then(|v| v.as_str()).unwrap_or("");

    let level = match msg_type.to_lowercase().as_str() {
        "log" | "game_log" | "info" => "INFO",
        "warn" | "warning" => "WARN",
        "error" | "fatal" => "ERROR",
        "debug" => "DEBUG",
        _ => "INFO",
    }
    .to_string();

    let data_text = format_event_data(obj.get("data"));
    let message = if !event.is_empty() && !data_text.is_empty() {
        format!("{} · {}", event_label(event), data_text)
    } else if !event.is_empty() {
        event_label(event).to_string()
    } else if !data_text.is_empty() {
        data_text
    } else {
        val.to_string()
    };

    let entity = obj
        .get("entity_name")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    Some(ConsoleLogRow {
        level,
        category: event.to_string(),
        entity,
        message,
        timestamp: Some(now_hms()),
    })
}

/// 事件 JSON → AI 决策消息（事件名/类型含 agent/decision/finished/think/tool 时）。
fn event_to_agent(val: &serde_json::Value) -> Option<AgentChatMessage> {
    let obj = val.as_object()?;
    let event = obj.get("event").and_then(|v| v.as_str()).unwrap_or("");
    let msg_type = obj.get("type").and_then(|v| v.as_str()).unwrap_or("");
    let hay = format!("{} {}", msg_type, event).to_lowercase();
    const KEYWORDS: [&str; 6] = ["agent", "decision", "finished", "think", "thought", "tool"];
    if !KEYWORDS.iter().any(|k| hay.contains(k)) {
        return None;
    }

    let agent_id = obj
        .get("agent_id")
        .and_then(|v| v.as_str())
        .unwrap_or(event)
        .to_string();
    let role = obj
        .get("role")
        .and_then(|v| v.as_str())
        .unwrap_or("assistant")
        .to_string();
    let kind = if hay.contains("think") || hay.contains("thought") {
        "think"
    } else if hay.contains("tool") {
        "tool_call"
    } else if hay.contains("decision") {
        "public_decision"
    } else {
        "message"
    }
    .to_string();
    let content = obj
        .get("content")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| format_event_data(obj.get("data")));
    let content = if content.is_empty() {
        val.to_string()
    } else {
        content
    };

    Some(AgentChatMessage {
        agent_id,
        role,
        kind,
        content,
        round: None,
    })
}

fn is_game_close(val: &serde_json::Value) -> bool {
    val.get("event").and_then(|v| v.as_str()) == Some("game_close")
}

// ── 时间格式化 ──

fn fmt_epoch_ms(ms: i64) -> String {
    let secs = ms.div_euclid(1000);
    let h = (secs / 3600) % 24;
    let m = (secs / 60) % 60;
    let s = secs % 60;
    format!("{h:02}:{m:02}:{s:02}")
}

fn now_hms() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    fmt_epoch_ms((secs * 1000) as i64)
}

// ── 异步逻辑 ──

/// 首次进入对局：校验对局存在 → 拉历史日志 → 订阅实时事件 → 消费事件流。
pub(super) fn spawn_init(game_id: String, gen: u64, cx: &mut Context<AppSidebar>) {
    let state = provider::process_service().state.clone();
    cx.spawn(
        move |weak: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
            let weak = weak.clone();
            let mut cx = cx.clone();
            let state = state.clone();
            async move {
                // 1. 校验对局存在
                match provider::process_service().get(&game_id).await {
                    Ok(Some(_)) => {}
                    Ok(None) => {
                        update_state(|s| s.error = Some("对局未运行或不存在".to_string()));
                        weak.update(&mut cx, |_, cx| cx.notify()).ok();
                        return;
                    }
                    Err(e) => {
                        update_state(|s| s.error = Some(e));
                        weak.update(&mut cx, |_, cx| cx.notify()).ok();
                        return;
                    }
                }

                // 2. 拉历史日志（SQLite 只读查询，可选增强）
                let params = LogQueryParams {
                    offset: 0,
                    limit: 200,
                    levels: None,
                    entity_id: None,
                    category: None,
                    search_text: None,
                };
                if let Ok(res) = crate::services::log_service::query_logs(&game_id, &params).await {
                    let rows: Vec<ConsoleLogRow> = res
                        .rows
                        .into_iter()
                        .map(|r| ConsoleLogRow {
                            level: r.level,
                            category: r.category.unwrap_or_default(),
                            entity: r.entity_name.unwrap_or_default(),
                            message: r.message,
                            timestamp: Some(fmt_epoch_ms(r.timestamp)),
                        })
                        .collect();
                    update_state(|s| s.logs = rows);
                }

                // 3. 订阅实时事件
                let mut rx = match match_ws::subscribe_match_events(&state, &game_id) {
                    Ok(rx) => rx,
                    Err(e) => {
                        update_state(|s| s.error = Some(e));
                        weak.update(&mut cx, |_, cx| cx.notify()).ok();
                        return;
                    }
                };
                update_state(|s| s.stream_alive = true);

                // 4. 消费事件流
                while let Some(val) = rx.recv().await {
                    // 该对局不再是调试焦点（已导航离开或重新进入新对局）则退出
                    let owned = with_state(|s| {
                        s.current_game.as_deref() == Some(game_id.as_str()) && s.generation == gen
                    });
                    if !owned {
                        break;
                    }
                    // 对局连接关闭：标记断开并退出（不重置 current_game，避免重复初始化）
                    if is_game_close(&val) {
                        update_state(|s| {
                            s.stream_alive = false;
                            s.error = Some("对局连接已关闭（可能已停止）".to_string());
                        });
                        weak.update(&mut cx, |_, cx| cx.notify()).ok();
                        break;
                    }
                    if let Some(row) = event_to_log(&val) {
                        update_state(|s| s.logs.push(row));
                    }
                    if let Some(msg) = event_to_agent(&val) {
                        update_state(|s| s.messages.push(msg));
                    }
                    weak.update(&mut cx, |_, cx| cx.notify()).ok();
                }
                update_state(|s| s.stream_alive = false);
            }
        },
    )
    .detach();
}

/// 发送一条对局控制命令，把结果反馈到页面错误横幅（失败时回滚乐观状态）。
pub(super) fn run_match_cmd(game_id: String, cmd: MatchCmd, cx: &mut Context<AppSidebar>) {
    let state = provider::process_service().state.clone();
    cx.spawn(
        move |weak: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
            let weak = weak.clone();
            let mut cx = cx.clone();
            let state = state.clone();
            let game_id = game_id.clone();
            let cmd = cmd.clone();
            async move {
                let result: Result<(), String> = match &cmd {
                    MatchCmd::GodMode(enabled) => {
                        match_ws::set_god_mode(&state, &game_id, *enabled).await
                    }
                    MatchCmd::Cooldown(enabled) => {
                        match_ws::toggle_cooldown(&state, &game_id, *enabled).await
                    }
                    MatchCmd::Pause => match_ws::pause_match(&state, &game_id).await.map(|_| ()),
                    MatchCmd::Resume => match_ws::resume_match(&state, &game_id).await.map(|_| ()),
                    MatchCmd::ResetPosition => match_ws::reset_position(&state, &game_id).await,
                    MatchCmd::SwitchChampion(name) => {
                        match_ws::switch_champion(&state, &game_id, name).await
                    }
                };
                match result {
                    Ok(()) => update_state(|s| s.error = None),
                    Err(e) => update_state(|s| {
                        s.error = Some(e);
                        match &cmd {
                            MatchCmd::GodMode(_) => s.god_mode = !s.god_mode,
                            MatchCmd::Cooldown(_) => s.cooldown_disabled = !s.cooldown_disabled,
                            MatchCmd::Pause | MatchCmd::Resume => s.paused = !s.paused,
                            _ => {}
                        }
                    }),
                }
                weak.update(&mut cx, |_, cx| cx.notify()).ok();
            }
        },
    )
    .detach();
}
