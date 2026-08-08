use std::cell::RefCell;

use gpui::prelude::*;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::scroll::ScrollableElement;
use gpui_component::{h_flex, v_flex, ActiveTheme, Disableable, IconName, Sizable, StyledExt};
use lol_web_protocol::history::{GameHistorySummary, SavedAgentHistory};

use crate::components::agent_chat_history::{render_agent_chat_history, AgentChatMessage};
use crate::components::sidebar::AppSidebar;
use crate::services::provider::cloud_client;

// ── 页面本地状态 ──

struct HistoryPageState {
    /// 列表首次加载是否已触发
    loaded: bool,
    /// 列表加载中
    loading: bool,
    histories: Vec<GameHistorySummary>,
    /// 当前选中的对局 id
    selected_id: Option<String>,
    /// 选中对局的 Agent 详情
    detail: Vec<SavedAgentHistory>,
    /// 详情加载中
    detail_loading: bool,
    /// 详情中选中的 agent_id
    selected_agent_id: Option<String>,
    /// 删除进行中
    deleting: bool,
    error: Option<String>,
}

impl Default for HistoryPageState {
    fn default() -> Self {
        Self {
            loaded: false,
            loading: false,
            histories: Vec::new(),
            selected_id: None,
            detail: Vec::new(),
            detail_loading: false,
            selected_agent_id: None,
            deleting: false,
            error: None,
        }
    }
}

thread_local! {
    static STATE: RefCell<HistoryPageState> = RefCell::new(HistoryPageState::default());
}

fn with_state<R>(f: impl FnOnce(&HistoryPageState) -> R) -> R {
    STATE.with(|s| f(&s.borrow()))
}

fn update_state(f: impl FnOnce(&mut HistoryPageState)) {
    STATE.with(|s| f(&mut s.borrow_mut()));
}

// ── 格式化辅助 ──

/// RFC3339 时间转展示文本："2025-08-01T00:00:00Z" → "2025-08-01 00:00"
fn fmt_datetime(iso: &str) -> String {
    iso.replace('T', " ").chars().take(16).collect()
}

/// 时长（秒）转 "Xm Ys"
fn fmt_duration(secs: i64) -> String {
    if secs <= 0 {
        return "—".to_string();
    }
    let (m, s) = (secs / 60, secs % 60);
    if m == 0 {
        format!("{}s", s)
    } else {
        format!("{}m {}s", m, s)
    }
}

/// 阵营 → 中文标签
fn team_label(team: &str) -> &'static str {
    match team.to_ascii_lowercase().as_str() {
        "order" | "blue" => "秩序",
        "chaos" | "red" => "混沌",
        _ => "中立",
    }
}

/// 阵营 → 颜色（秩序蓝 / 混沌红）
fn team_color(team: &str, cx: &mut Context<AppSidebar>) -> Hsla {
    match team.to_ascii_lowercase().as_str() {
        "order" | "blue" => gpui::hsla(0.58, 0.85, 0.55, 1.0),
        "chaos" | "red" => gpui::hsla(0.0, 0.85, 0.55, 1.0),
        _ => cx.theme().accent,
    }
}

/// serde_json::Value → 展示文本：字符串直接取，数组取各元素 text 字段拼接，其余转 JSON。
fn json_content(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Array(items) => items
            .iter()
            .filter_map(|i| i.get("text").and_then(|t| t.as_str()))
            .collect::<Vec<_>>()
            .join("\n"),
        other => other.to_string(),
    }
}

/// 把 SavedAgentHistory 的对话历史映射为 AgentChatMessage 列表。
///
/// 与 client history.vue 一致：`{User:{content}}` → 观测/指令，`{Assistant:{content}}` → 公开决策；
/// 否则回退到 `{role, content}`。轮次按 (user, assistant) 成对计算。
fn agent_messages(agent: &SavedAgentHistory) -> Vec<AgentChatMessage> {
    agent
        .history
        .iter()
        .enumerate()
        .filter_map(|(idx, v)| {
            let obj = v.as_object()?;
            let (role, content) = if let Some(user) = obj.get("User") {
                ("user", json_content(user))
            } else if let Some(assistant) = obj.get("Assistant") {
                ("assistant", json_content(assistant))
            } else {
                let role = obj.get("role").and_then(|r| r.as_str()).unwrap_or("user");
                let content = obj.get("content").map(json_content).unwrap_or_default();
                (role, content)
            };
            let kind = if role == "assistant" {
                "public_decision"
            } else {
                "observation"
            };
            Some(AgentChatMessage {
                agent_id: agent.agent_id.clone(),
                role: role.to_string(),
                kind: kind.to_string(),
                content,
                round: Some(idx as u32 / 2 + 1),
            })
        })
        .collect()
}

// ── 异步加载 ──

/// 拉取对局列表并写回状态（同时清理已失效的选中项）。
async fn fetch_list() {
    match cloud_client().list_game_histories().await {
        Ok(list) => update_state(|s| {
            s.histories = list;
            s.loading = false;
            if let Some(sid) = &s.selected_id {
                let exists = s
                    .histories
                    .iter()
                    .any(|h| h.id.as_deref() == Some(sid.as_str()));
                if !exists {
                    s.selected_id = None;
                    s.detail.clear();
                    s.selected_agent_id = None;
                }
            }
        }),
        Err(e) => update_state(|s| {
            s.loading = false;
            s.error = Some(format!("加载对局列表失败: {}", e));
        }),
    }
}

fn spawn_refresh_list(cx: &mut Context<AppSidebar>) {
    cx.spawn(
        move |weak: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
            let weak = weak.clone();
            let mut cx = cx.clone();
            async move {
                update_state(|s| {
                    s.loading = true;
                    s.error = None;
                });
                fetch_list().await;
                if let Some(entity) = weak.upgrade() {
                    let _ = entity.update(&mut cx, |_, cx| cx.notify());
                }
            }
        },
    )
    .detach();
}

fn spawn_select_game(cx: &mut Context<AppSidebar>, id: &str) {
    let id = id.to_string();
    cx.spawn(
        move |weak: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
            let weak = weak.clone();
            let mut cx = cx.clone();
            let id = id.clone();
            async move {
                update_state(|s| {
                    s.selected_id = Some(id.clone());
                    s.detail.clear();
                    s.selected_agent_id = None;
                    s.detail_loading = true;
                    s.error = None;
                });
                match cloud_client().get_game_history_detail(&id).await {
                    Ok(detail) => update_state(|s| {
                        s.detail = detail;
                        s.detail_loading = false;
                        if let Some(first) = s.detail.first() {
                            s.selected_agent_id = Some(first.agent_id.clone());
                        }
                    }),
                    Err(e) => update_state(|s| {
                        s.detail_loading = false;
                        s.error = Some(format!("加载对局详情失败: {}", e));
                    }),
                }
                if let Some(entity) = weak.upgrade() {
                    let _ = entity.update(&mut cx, |_, cx| cx.notify());
                }
            }
        },
    )
    .detach();
}

fn spawn_delete_game(cx: &mut Context<AppSidebar>) {
    let Some(id) = with_state(|s| s.selected_id.clone()) else {
        return;
    };
    cx.spawn(
        move |weak: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
            let weak = weak.clone();
            let mut cx = cx.clone();
            let id = id.clone();
            async move {
                update_state(|s| s.deleting = true);
                match cloud_client().delete_game_history(&id).await {
                    Ok(()) => {
                        update_state(|s| {
                            s.deleting = false;
                            if s.selected_id.as_deref() == Some(id.as_str()) {
                                s.selected_id = None;
                                s.detail.clear();
                                s.selected_agent_id = None;
                            }
                        });
                        update_state(|s| {
                            s.loading = true;
                            s.error = None;
                        });
                        fetch_list().await;
                    }
                    Err(e) => update_state(|s| {
                        s.deleting = false;
                        s.error = Some(format!("删除失败: {}", e));
                    }),
                }
                if let Some(entity) = weak.upgrade() {
                    let _ = entity.update(&mut cx, |_, cx| cx.notify());
                }
            }
        },
    )
    .detach();
}

// ── 渲染 ──

/// 对局历史详情 + Agent 对话回放（对应 client `pages/history.vue`）。
pub fn render_history(_sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) -> AnyElement {
    // 首次渲染自动加载对局列表
    if !with_state(|s| s.loaded) {
        update_state(|s| s.loaded = true);
        spawn_refresh_list(cx);
    }

    let (
        loading,
        histories,
        selected_id,
        detail_loading,
        detail,
        selected_agent_id,
        deleting,
        error,
    ) = with_state(|s| {
        (
            s.loading,
            s.histories.clone(),
            s.selected_id.clone(),
            s.detail_loading,
            s.detail.clone(),
            s.selected_agent_id.clone(),
            s.deleting,
            s.error.clone(),
        )
    });

    let has_selection = selected_id.is_some();
    let count = histories.len();
    let selected_summary = histories
        .iter()
        .find(|h| h.id.as_deref() == selected_id.as_deref());
    let selected_dt = selected_summary
        .map(|h| fmt_datetime(&h.datetime))
        .unwrap_or_default();
    let selected_dur = selected_summary
        .map(|h| fmt_duration(h.duration))
        .unwrap_or_default();

    let list_items: Vec<AnyElement> = histories
        .iter()
        .map(|h| history_list_item(cx, h, selected_id.as_deref() == h.id.as_deref()))
        .collect();

    v_flex()
        .size_full()
        .flex_1()
        .gap_4()
        .overflow_hidden()
        // ── 标题行：标题 + 删除/刷新 ──
        .child(
            h_flex()
                .items_center()
                .justify_between()
                .child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .child(div().font_bold().text_lg().child("对局历史"))
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child(format!("共 {} 条记录", count)),
                        ),
                )
                .child(
                    h_flex()
                        .gap_2()
                        .when(has_selection, |d| {
                            d.child(
                                Button::new("history-delete-btn")
                                    .outline()
                                    .danger()
                                    .icon(IconName::Delete)
                                    .label(if deleting { "删除中…" } else { "删除" })
                                    .disabled(deleting)
                                    .on_click(cx.listener(|_, _, _, cx| {
                                        spawn_delete_game(cx);
                                    })),
                            )
                        })
                        .child(
                            Button::new("history-refresh-btn")
                                .outline()
                                .icon(IconName::Loader)
                                .label("刷新列表")
                                .disabled(loading)
                                .on_click(cx.listener(|_, _, _, cx| {
                                    spawn_refresh_list(cx);
                                })),
                        ),
                ),
        )
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
        // ── 主体：左列表 + 右详情 ──
        .child(
            h_flex()
                .flex_1()
                .min_h_0()
                .gap_4()
                .overflow_hidden()
                // 左：对局列表
                .child(
                    v_flex()
                        .w_80()
                        .flex_none()
                        .h_full()
                        .rounded_lg()
                        .border_1()
                        .border_color(cx.theme().border)
                        .overflow_hidden()
                        .child(
                            h_flex()
                                .px_3()
                                .py_2()
                                .border_b_1()
                                .border_color(cx.theme().border)
                                .items_center()
                                .justify_between()
                                .child(
                                    div()
                                        .text_xs()
                                        .font_bold()
                                        .text_color(cx.theme().muted_foreground)
                                        .child("对局记录"),
                                )
                                .child(
                                    div()
                                        .px_2()
                                        .py_0p5()
                                        .rounded_md()
                                        .text_xs()
                                        .font_bold()
                                        .bg(cx.theme().accent.opacity(0.15))
                                        .text_color(cx.theme().accent)
                                        .child(format!("{}", count)),
                                ),
                        )
                        .child(
                            div()
                                .flex_1()
                                .min_h_0()
                                .overflow_y_scrollbar()
                                .p_2()
                                .flex()
                                .flex_col()
                                .gap_2()
                                .when(loading && count == 0, |d| {
                                    d.flex().items_center().justify_center().child(
                                        div()
                                            .text_xs()
                                            .text_color(cx.theme().muted_foreground)
                                            .child("加载中…"),
                                    )
                                })
                                .when(!loading && count == 0, |d| {
                                    d.flex().items_center().justify_center().child(
                                        v_flex()
                                            .items_center()
                                            .gap_2()
                                            .text_color(cx.theme().muted_foreground)
                                            .text_xs()
                                            .child(IconName::Inbox)
                                            .child("暂无对局记录"),
                                    )
                                })
                                .children(list_items),
                        ),
                )
                // 右：详情
                .child(
                    v_flex()
                        .flex_1()
                        .min_h_0()
                        .h_full()
                        .rounded_lg()
                        .border_1()
                        .border_color(cx.theme().border)
                        .overflow_hidden()
                        .child(render_detail_panel(
                            cx,
                            has_selection,
                            detail_loading,
                            &detail,
                            &selected_agent_id,
                            &selected_dt,
                            &selected_dur,
                        )),
                ),
        )
        .into_any_element()
}

/// 左侧对局列表项：英雄徽标 + 时间 + 时长。
fn history_list_item(
    cx: &mut Context<AppSidebar>,
    summary: &GameHistorySummary,
    selected: bool,
) -> AnyElement {
    let id = summary.id.clone().unwrap_or_default();
    let dt = fmt_datetime(&summary.datetime);
    let dur = fmt_duration(summary.duration);
    let accent = cx.theme().accent;

    let champs: Vec<AnyElement> = summary
        .agents
        .iter()
        .map(|a| {
            let color = team_color(&a.team, cx);
            h_flex()
                .gap_1()
                .items_center()
                .child(div().w_1p5().h_1p5().rounded_full().bg(color))
                .child(
                    div()
                        .text_xs()
                        .font_semibold()
                        .text_color(cx.theme().foreground)
                        .child(a.champion.clone()),
                )
                .into_any_element()
        })
        .collect();

    let click_id = id.clone();
    div()
        .rounded_md()
        .border_1()
        .px_2()
        .py_2()
        .flex()
        .flex_col()
        .gap_1()
        .cursor_pointer()
        .border_color(if selected {
            accent
        } else {
            cx.theme().border.opacity(0.6)
        })
        .when(selected, |d| d.bg(accent.opacity(0.08)))
        .on_any_mouse_down(cx.listener(move |_this, _e, _window, cx| {
            spawn_select_game(cx, &click_id);
        }))
        .child(h_flex().gap_2().flex_wrap().children(champs))
        .child(
            h_flex()
                .items_center()
                .justify_between()
                .gap_2()
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child(dt),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child(dur),
                ),
        )
        .into_any_element()
}

/// 右侧详情面板：Agent 选择 + 全局/英雄 Prompt + 对话回放。
fn render_detail_panel(
    cx: &mut Context<AppSidebar>,
    has_selection: bool,
    detail_loading: bool,
    detail: &[SavedAgentHistory],
    selected_agent_id: &Option<String>,
    game_dt: &str,
    game_dur: &str,
) -> AnyElement {
    let muted = cx.theme().muted_foreground;

    // 无选中 → 空态
    if !has_selection {
        return v_flex()
            .size_full()
            .flex_1()
            .items_center()
            .justify_center()
            .gap_2()
            .text_color(muted)
            .text_sm()
            .child(IconName::Inbox)
            .child("选择左侧记录查看详情")
            .into_any_element();
    }

    // 详情加载中
    if detail_loading {
        return v_flex()
            .size_full()
            .flex_1()
            .items_center()
            .justify_center()
            .text_color(muted)
            .text_xs()
            .child("加载中…")
            .into_any_element();
    }

    // 详情为空
    if detail.is_empty() {
        return v_flex()
            .size_full()
            .flex_1()
            .items_center()
            .justify_center()
            .text_color(muted)
            .text_xs()
            .child("该局暂无 Agent 记录")
            .into_any_element();
    }

    let selected_agent = detail
        .iter()
        .find(|a| Some(&a.agent_id) == selected_agent_id.as_ref())
        .unwrap_or(&detail[0]);

    // Agent 选择按钮
    let agent_buttons: Vec<AnyElement> = detail
        .iter()
        .map(|a| {
            let is_active = Some(&a.agent_id) == selected_agent_id.as_ref();
            let aid = a.agent_id.clone();
            let champion = a.champion.clone();
            let team = a.team.clone();
            Button::new(format!("history-agent-{}", aid))
                .when(is_active, |b| b.primary())
                .when(!is_active, |b| b.outline())
                .label(format!("{} · {}", champion, team_label(&team)))
                .xsmall()
                .on_click(cx.listener(move |_, _, _, cx| {
                    update_state(|s| s.selected_agent_id = Some(aid.clone()));
                    cx.notify();
                }))
                .into_any_element()
        })
        .collect();

    // 对话回放（映射为 AgentChatMessage）
    let msgs = agent_messages(selected_agent);
    let chat = render_agent_chat_history(&msgs, cx);

    v_flex()
        .size_full()
        .flex_1()
        .gap_3()
        .overflow_hidden()
        .p_4()
        // ── 顶部：对局信息 + Agent 选择 ──
        .child(
            v_flex()
                .flex_shrink_0()
                .gap_2()
                .child(
                    h_flex()
                        .items_center()
                        .gap_3()
                        .child(
                            h_flex()
                                .gap_1p5()
                                .items_center()
                                .text_xs()
                                .text_color(cx.theme().accent)
                                .child(IconName::Calendar)
                                .child(game_dt.to_string()),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(muted)
                                .child(format!("时长 {}", game_dur)),
                        ),
                )
                .child(h_flex().gap_2().flex_wrap().children(agent_buttons)),
        )
        // ── 全局 Prompt + 英雄 Prompt ──
        .child(
            v_flex()
                .flex_shrink_0()
                .gap_3()
                .child(prompt_block(
                    cx,
                    "全局 Prompt",
                    &selected_agent.system_prompt,
                ))
                .child(prompt_block(cx, "英雄 Prompt", &selected_agent.prompt)),
        )
        // ── 分隔线 ──
        .child(div().w_full().h_px().bg(cx.theme().border))
        // ── 对话回放 ──
        .child(div().flex_1().min_h_0().overflow_hidden().child(chat))
        .into_any_element()
}

/// Prompt 文本块（可滚动，空内容显示「无」）。
fn prompt_block(cx: &mut Context<AppSidebar>, title: &str, content: &str) -> AnyElement {
    let empty = content.trim().is_empty();
    v_flex()
        .gap_1()
        .child(
            div()
                .text_xs()
                .font_bold()
                .text_color(cx.theme().muted_foreground)
                .child(title.to_string()),
        )
        .child(
            div()
                .rounded_md()
                .border_1()
                .border_color(cx.theme().border)
                .bg(cx.theme().background.opacity(0.4))
                .max_h_32()
                .overflow_y_scrollbar()
                .p_2()
                .w_full()
                .child(
                    div()
                        .text_xs()
                        .text_color(if empty {
                            cx.theme().muted_foreground
                        } else {
                            cx.theme().foreground
                        })
                        .child(if empty {
                            "无".to_string()
                        } else {
                            content.to_string()
                        }),
                ),
        )
        .into_any_element()
}
