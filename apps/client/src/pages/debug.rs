use std::cell::RefCell;

use gpui::prelude::*;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::menu::{DropdownMenu, PopupMenuItem};
use gpui_component::scroll::ScrollableElement;
use gpui_component::{h_flex, v_flex, ActiveTheme, Disableable, IconName, Sizable, StyledExt};
use uuid::Uuid;

use crate::components::agent_chat_history::{render_agent_chat_history, AgentChatMessage};
use crate::components::game_console_logs::{render_game_console_logs, ConsoleLogRow};
use crate::components::sidebar::AppSidebar;
use crate::services::types::LogQueryParams;
use crate::services::{match_ws, provider};
use crate::types::ActiveView;

// ── 页面本地状态 ──

/// 右侧工作区选项卡。
#[derive(Clone, Copy, PartialEq, Eq)]
enum DebugTab {
    Logs,
    Agents,
}

/// 对局控制命令（对应 `apps/client/src/pages/debug/[id].vue` 的按钮组）。
#[derive(Clone)]
enum MatchCmd {
    GodMode(bool),
    Cooldown(bool),
    Pause,
    Resume,
    ResetPosition,
    SwitchChampion(String),
}

struct DebugPageState {
    /// 当前调试的对局 id（与 sidebar.current_game_id 联动）。
    current_game: Option<String>,
    /// 事件循环代际：每次进入新对局自增，旧事件循环靠它识别自己已过期。
    generation: u64,
    /// 是否已订阅并正在消费实时事件流。
    stream_alive: bool,
    error: Option<String>,
    /// 控制台日志行（历史 + 实时）。
    logs: Vec<ConsoleLogRow>,
    /// AI 决策消息流。
    messages: Vec<AgentChatMessage>,
    active_tab: DebugTab,
    /// 本地乐观状态（与游戏端可能短暂不同步，失败时回滚）。
    god_mode: bool,
    cooldown_disabled: bool,
    paused: bool,
    switch_target: String,
    stopping: bool,
}

impl Default for DebugPageState {
    fn default() -> Self {
        Self {
            current_game: None,
            generation: 0,
            stream_alive: false,
            error: None,
            logs: Vec::new(),
            messages: Vec::new(),
            active_tab: DebugTab::Logs,
            god_mode: false,
            cooldown_disabled: false,
            paused: false,
            switch_target: "Riven".to_string(),
            stopping: false,
        }
    }
}

thread_local! {
    static STATE: RefCell<DebugPageState> = RefCell::new(DebugPageState::default());
}

fn with_state<R>(f: impl FnOnce(&DebugPageState) -> R) -> R {
    STATE.with(|s| f(&s.borrow()))
}

fn update_state(f: impl FnOnce(&mut DebugPageState)) {
    STATE.with(|s| f(&mut s.borrow_mut()));
}

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
fn spawn_init(game_id: String, gen: u64, cx: &mut Context<AppSidebar>) {
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
fn run_match_cmd(game_id: String, cmd: MatchCmd, cx: &mut Context<AppSidebar>) {
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

// ── 空态 / 错误态 ──

fn back_to_games_button(id: &'static str, cx: &mut Context<AppSidebar>) -> Button {
    Button::new(id)
        .outline()
        .icon(IconName::ArrowLeft)
        .label("返回对局列表")
        .on_click(cx.listener(|this, _, _, cx| {
            this.navigate_to(ActiveView::Games);
            cx.notify();
        }))
}

fn render_no_game(cx: &mut Context<AppSidebar>) -> AnyElement {
    v_flex()
        .size_full()
        .flex()
        .items_center()
        .justify_center()
        .gap_3()
        .child(
            div()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child("未选择对局，请先从「运行中对局」进入调试。"),
        )
        .child(back_to_games_button("debug-back-empty", cx))
        .into_any_element()
}

fn render_invalid_id(game_id: &str, cx: &mut Context<AppSidebar>) -> AnyElement {
    v_flex()
        .size_full()
        .flex()
        .items_center()
        .justify_center()
        .gap_3()
        .child(
            div()
                .text_xs()
                .text_color(cx.theme().danger)
                .child(format!("对局 id 格式无效：{game_id}")),
        )
        .child(back_to_games_button("debug-back-invalid", cx))
        .into_any_element()
}

// ── 内容渲染 ──

fn short_id(id: &str) -> String {
    if id.len() > 8 {
        id[..8].to_string()
    } else {
        id.to_string()
    }
}

fn render_content(
    sidebar: &mut AppSidebar,
    game_id: &str,
    cx: &mut Context<AppSidebar>,
) -> AnyElement {
    let (
        error,
        logs,
        messages,
        active_tab,
        god_mode,
        cooldown_disabled,
        paused,
        switch_target,
        stopping,
        stream_alive,
    ) = with_state(|s| {
        (
            s.error.clone(),
            s.logs.clone(),
            s.messages.clone(),
            s.active_tab,
            s.god_mode,
            s.cooldown_disabled,
            s.paused,
            s.switch_target.clone(),
            s.stopping,
            s.stream_alive,
        )
    });

    // ── 对局控制按钮 ──
    let god_mode_btn = Button::new("debug-god-mode")
        .outline()
        .icon(IconName::Star)
        .label(if god_mode {
            "上帝模式：开".to_string()
        } else {
            "上帝模式：关".to_string()
        })
        .on_click(cx.listener(move |this, _, _, cx| {
            let enabled = !god_mode;
            update_state(|s| s.god_mode = enabled);
            let gid = this.current_game_id.clone().unwrap_or_default();
            run_match_cmd(gid, MatchCmd::GodMode(enabled), cx);
        }));

    let cooldown_btn = Button::new("debug-cooldown")
        .outline()
        .icon(IconName::Cpu)
        .label(if cooldown_disabled {
            "关闭冷却：开".to_string()
        } else {
            "关闭冷却：关".to_string()
        })
        .on_click(cx.listener(move |this, _, _, cx| {
            let enabled = !cooldown_disabled;
            update_state(|s| s.cooldown_disabled = enabled);
            let gid = this.current_game_id.clone().unwrap_or_default();
            run_match_cmd(gid, MatchCmd::Cooldown(enabled), cx);
        }));

    let pause_btn = Button::new("debug-pause")
        .outline()
        .icon(if paused {
            IconName::Play
        } else {
            IconName::Pause
        })
        .label(if paused {
            "恢复对局".to_string()
        } else {
            "暂停对局".to_string()
        })
        .on_click(cx.listener(move |this, _, _, cx| {
            let was_paused = paused;
            update_state(|s| s.paused = !paused);
            let gid = this.current_game_id.clone().unwrap_or_default();
            let cmd = if was_paused {
                MatchCmd::Resume
            } else {
                MatchCmd::Pause
            };
            run_match_cmd(gid, cmd, cx);
        }));

    let reset_btn = Button::new("debug-reset")
        .outline()
        .icon(IconName::Redo)
        .label("重置坐标")
        .on_click(cx.listener(move |this, _, _, cx| {
            let gid = this.current_game_id.clone().unwrap_or_default();
            run_match_cmd(gid, MatchCmd::ResetPosition, cx);
        }));

    // 英雄切换：下拉选择目标英雄
    let champions = sidebar.champions_list.clone();
    let weak = cx.entity().downgrade();
    let champ_dropdown = Button::new("debug-champion-select")
        .outline()
        .icon(IconName::User)
        .label(if switch_target.is_empty() {
            "选择英雄".to_string()
        } else {
            switch_target.clone()
        })
        .dropdown_menu(move |menu, _window, _cx| {
            let mut m = menu;
            for name in &champions {
                let name = name.clone();
                let checked = name == switch_target;
                let weak = weak.clone();
                m = m.item(PopupMenuItem::new(name.clone()).checked(checked).on_click(
                    move |_, _, cx| {
                        update_state(|s| s.switch_target = name.clone());
                        weak.update(cx, |_, cx| cx.notify()).ok();
                    },
                ));
            }
            m
        });

    let switch_btn = Button::new("debug-switch-submit")
        .outline()
        .icon(IconName::ChevronsUpDown)
        .label("切换英雄")
        .on_click(cx.listener(move |this, _, _, cx| {
            let target = with_state(|s| s.switch_target.clone());
            let gid = this.current_game_id.clone().unwrap_or_default();
            run_match_cmd(gid, MatchCmd::SwitchChampion(target), cx);
        }));

    // ── 停止对局 / 返回 ──
    let stop_btn = Button::new("debug-stop")
        .outline()
        .icon(IconName::CircleX)
        .label(if stopping {
            "停止中…".to_string()
        } else {
            "停止对局".to_string()
        })
        .disabled(stopping)
        .on_click(cx.listener(move |this, _, _, cx| {
            update_state(|s| s.stopping = true);
            let gid = this.current_game_id.clone().unwrap_or_default();
            cx.spawn(
                move |weak: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
                    let weak = weak.clone();
                    let mut cx = cx.clone();
                    let gid = gid.clone();
                    async move {
                        let _ = provider::process_service().stop(&gid).await;
                        update_state(|s| {
                            s.stopping = false;
                            s.stream_alive = false;
                        });
                        weak.update(&mut cx, |this, cx| {
                            this.current_game_id = None;
                            this.navigate_to(ActiveView::Games);
                            cx.notify();
                        })
                        .ok();
                    }
                },
            )
            .detach();
        }));

    let back_btn = Button::new("debug-back")
        .outline()
        .icon(IconName::ArrowLeft)
        .label("返回")
        .on_click(cx.listener(|this, _, _, cx| {
            this.navigate_to(ActiveView::Games);
            cx.notify();
        }));

    // ── 右侧选项卡 ──
    let logs_tab_btn = Button::new("debug-tab-logs")
        .small()
        .icon(IconName::SquareTerminal)
        .label("控制台日志")
        .when(active_tab == DebugTab::Logs, |b| b.primary())
        .when(active_tab != DebugTab::Logs, |b| b.ghost())
        .on_click(cx.listener(|_, _, _, cx| {
            update_state(|s| s.active_tab = DebugTab::Logs);
            cx.notify();
        }));

    let agents_tab_btn = Button::new("debug-tab-agents")
        .small()
        .icon(IconName::Bot)
        .label("AI 思维链")
        .when(active_tab == DebugTab::Agents, |b| b.primary())
        .when(active_tab != DebugTab::Agents, |b| b.ghost())
        .on_click(cx.listener(|_, _, _, cx| {
            update_state(|s| s.active_tab = DebugTab::Agents);
            cx.notify();
        }));

    let tab_content = match active_tab {
        DebugTab::Logs => render_game_console_logs(&logs, cx),
        DebugTab::Agents => render_agent_chat_history(&messages, cx),
    };

    let muted = cx.theme().muted_foreground;
    let border = cx.theme().border;
    let success = cx.theme().success;
    let warning = cx.theme().warning;

    v_flex()
        .size_full()
        .flex_1()
        .gap_3()
        .overflow_hidden()
        // ── 状态栏 ──
        .child(
            h_flex()
                .items_center()
                .justify_between()
                .child(
                    h_flex()
                        .gap_3()
                        .items_center()
                        .child(
                            h_flex()
                                .gap_1p5()
                                .items_center()
                                .child(div().w_2().h_2().rounded_full().bg(if stream_alive {
                                    success
                                } else {
                                    warning
                                }))
                                .child(div().text_xs().font_semibold().child(if stream_alive {
                                    "已连接".to_string()
                                } else {
                                    "连接中…".to_string()
                                })),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(muted)
                                .child(format!("对局 {}", short_id(game_id))),
                        ),
                )
                .child(h_flex().gap_2().child(back_btn).child(stop_btn)),
        )
        // ── 错误横幅 ──
        .when_some(error.as_ref(), |d, err| {
            d.child(
                div()
                    .px_3()
                    .py_2()
                    .rounded_md()
                    .bg(cx.theme().danger.opacity(0.1))
                    .text_color(cx.theme().danger)
                    .text_xs()
                    .child(err.clone()),
            )
        })
        // ── 主工作区 ──
        .child(
            h_flex()
                .flex_1()
                .gap_3()
                .overflow_hidden()
                // 左列：控制面板
                .child(
                    v_flex()
                        .w(rems(15.))
                        .flex_none()
                        .gap_3()
                        .overflow_y_scrollbar()
                        .child(
                            v_flex()
                                .gap_2()
                                .rounded_lg()
                                .border_1()
                                .border_color(border)
                                .px_3()
                                .py_2()
                                .child(
                                    div()
                                        .text_xs()
                                        .font_semibold()
                                        .text_color(muted)
                                        .child("对局控制"),
                                )
                                .child(god_mode_btn)
                                .child(cooldown_btn)
                                .child(pause_btn),
                        )
                        .child(
                            v_flex()
                                .gap_2()
                                .rounded_lg()
                                .border_1()
                                .border_color(border)
                                .px_3()
                                .py_2()
                                .child(
                                    div()
                                        .text_xs()
                                        .font_semibold()
                                        .text_color(muted)
                                        .child("英雄控制"),
                                )
                                .child(champ_dropdown)
                                .child(switch_btn),
                        )
                        .child(
                            v_flex()
                                .gap_2()
                                .rounded_lg()
                                .border_1()
                                .border_color(border)
                                .px_3()
                                .py_2()
                                .child(
                                    div()
                                        .text_xs()
                                        .font_semibold()
                                        .text_color(muted)
                                        .child("快捷操作"),
                                )
                                .child(reset_btn),
                        ),
                )
                // 右列：日志 / AI 思维链
                .child(
                    v_flex()
                        .flex_1()
                        .gap_2()
                        .overflow_hidden()
                        .child(
                            h_flex()
                                .gap_2()
                                .items_center()
                                .child(logs_tab_btn)
                                .child(agents_tab_btn)
                                .child(
                                    div()
                                        .px_2()
                                        .py_0p5()
                                        .rounded_md()
                                        .text_xs()
                                        .font_bold()
                                        .bg(cx.theme().accent.opacity(0.15))
                                        .text_color(cx.theme().accent)
                                        .child(match active_tab {
                                            DebugTab::Logs => format!("{} 条", logs.len()),
                                            DebugTab::Agents => format!("{} 条", messages.len()),
                                        }),
                                ),
                        )
                        .child(div().flex_1().overflow_hidden().child(tab_content)),
                ),
        )
        .into_any_element()
}

// ── 公开入口 ──

/// 对局调试台（对应 client `pages/debug/[id].vue`，用 sidebar.current_game_id）。
pub fn render_debug(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) -> AnyElement {
    let Some(game_id) = sidebar.current_game_id.clone() else {
        return render_no_game(cx);
    };

    // 对局 id 可能不是合法 Uuid（来自 sidebar.current_game_id），直接提示
    if Uuid::parse_str(&game_id).is_err() {
        return render_invalid_id(&game_id, cx);
    }

    // 首次进入该对局：重置页面状态并启动事件订阅 / 历史加载
    let is_current = with_state(|s| s.current_game.as_deref() == Some(game_id.as_str()));
    if !is_current {
        let gen = with_state(|s| s.generation + 1);
        update_state(|s| {
            *s = DebugPageState::default();
            s.current_game = Some(game_id.clone());
            s.generation = gen;
            s.switch_target = sidebar
                .champions_list
                .first()
                .cloned()
                .unwrap_or_else(|| "Riven".to_string());
        });
        spawn_init(game_id.clone(), gen, cx);
    }

    render_content(sidebar, &game_id, cx)
}
