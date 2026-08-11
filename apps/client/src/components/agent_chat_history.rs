//! AI 决策历史 / 对话时间线共享组件。
//!
//! 供 history / debug / mock / observe 页面复用：渲染思维链、工具调用、公开决策、
//! 环境观测四类消息流，支持按 kind 过滤、按轮次折叠，代码段以等宽底色块展示，
//! 空态显示「暂无消息」。
//!
//! 文案内联中文（未接入 i18n），本文件自包含、不依赖其它页面私有状态；
//! 组件注册（mod.rs）与页面接入由主会话处理。

use std::collections::{HashMap, HashSet};

use gpui::prelude::*;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::scroll::ScrollableElement;
use gpui_component::{h_flex, v_flex, ActiveTheme, IconName, Sizable, StyledExt};

use crate::components::sidebar::AppSidebar;

// ── 数据模型 ──

/// 单条 AI 决策历史消息。
#[derive(Debug, Clone)]
pub struct AgentChatMessage {
    pub agent_id: String,
    /// 如 "assistant" / "user"
    pub role: String,
    /// 如 "think" / "tool_call" / "public_decision" / "observation" / "message"
    pub kind: String,
    pub content: String,
    /// 轮次，可选
    pub round: Option<u32>,
}

// ── 消息类型分类 ──

#[derive(Clone, Copy, PartialEq, Eq)]
enum KindClass {
    Think,
    Tool,
    Decision,
    Observation,
    Message,
}

fn classify_kind(kind: &str) -> KindClass {
    match kind {
        "think" | "thought" | "thinking" | "thought_reasoning" => KindClass::Think,
        "tool_call" | "tool" | "toolresult" | "tool_result" | "tool_result_output" => {
            KindClass::Tool
        }
        "public_decision" | "decision" | "reply" | "public" => KindClass::Decision,
        "observation" | "obs" | "observe" | "environment" => KindClass::Observation,
        _ => KindClass::Message,
    }
}

fn kind_label(kind: &str) -> &'static str {
    match classify_kind(kind) {
        KindClass::Think => "思维",
        KindClass::Tool => "工具",
        KindClass::Decision => "决策",
        KindClass::Observation => "观测",
        KindClass::Message => "消息",
    }
}

fn kind_accent(kind: &str, cx: &Context<AppSidebar>) -> Hsla {
    match classify_kind(kind) {
        KindClass::Think => cx.theme().accent,
        KindClass::Tool => cx.theme().info,
        KindClass::Decision => cx.theme().success,
        KindClass::Observation => cx.theme().muted_foreground,
        KindClass::Message => cx.theme().foreground,
    }
}

// ── 筛选 / 折叠状态（存于 AppSidebar.agent_chat_filters） ──

/// 消息流筛选与折叠状态。多个页面复用一个组件时共享此状态，
/// 简化起见不做按页面隔离。
#[derive(Clone)]
pub struct ChatHistoryFilters {
    pub show_think: bool,
    pub show_tool: bool,
    pub show_decision: bool,
    pub show_observation: bool,
    pub show_message: bool,
    pub collapsed_rounds: HashSet<u32>,
}

impl Default for ChatHistoryFilters {
    fn default() -> Self {
        Self {
            show_think: true,
            show_tool: true,
            show_decision: true,
            show_observation: true,
            show_message: true,
            collapsed_rounds: HashSet::new(),
        }
    }
}

fn is_kind_shown(kind: &str, filters: &ChatHistoryFilters) -> bool {
    match classify_kind(kind) {
        KindClass::Think => filters.show_think,
        KindClass::Tool => filters.show_tool,
        KindClass::Decision => filters.show_decision,
        KindClass::Observation => filters.show_observation,
        KindClass::Message => filters.show_message,
    }
}

fn toggle_kind(sidebar: &mut AppSidebar, kind: KindClass) {
    let f = &mut sidebar.agent_chat_filters;
    match kind {
        KindClass::Think => f.show_think = !f.show_think,
        KindClass::Tool => f.show_tool = !f.show_tool,
        KindClass::Decision => f.show_decision = !f.show_decision,
        KindClass::Observation => f.show_observation = !f.show_observation,
        KindClass::Message => f.show_message = !f.show_message,
    }
}

fn toggle_round_collapse(sidebar: &mut AppSidebar, round: u32) {
    if !sidebar.agent_chat_filters.collapsed_rounds.remove(&round) {
        sidebar.agent_chat_filters.collapsed_rounds.insert(round);
    }
}

fn reset_filters(sidebar: &mut AppSidebar) {
    sidebar.agent_chat_filters = ChatHistoryFilters::default();
}

// ── 渲染 ──

/// 渲染可滚动的 AI 决策历史流（思维 / 工具 / 决策 / 观测 / 消息）。
pub fn render_agent_chat_history(
    messages: &[AgentChatMessage],
    sidebar: &AppSidebar,
    cx: &mut Context<AppSidebar>,
) -> AnyElement {
    let filters = sidebar.agent_chat_filters.clone();

    let total = messages.len();
    let visible: Vec<&AgentChatMessage> = messages
        .iter()
        .filter(|m| is_kind_shown(&m.kind, &filters))
        .collect();
    let shown = visible.len();

    // 空态：无任何消息
    if total == 0 {
        return v_flex()
            .size_full()
            .flex_1()
            .items_center()
            .justify_center()
            .text_color(cx.theme().muted_foreground)
            .text_xs()
            .child("暂无消息")
            .into_any_element();
    }

    // 各轮次可见消息数（轮次头部展示用）
    let mut round_counts: HashMap<u32, usize> = HashMap::new();
    for m in &visible {
        if let Some(r) = m.round {
            *round_counts.entry(r).or_insert(0) += 1;
        }
    }

    // ── 顶部筛选栏 ──
    let mut kind_buttons: Vec<AnyElement> = Vec::new();
    kind_buttons.push(render_kind_filter_button(
        "agent-chat-filter-think",
        KindClass::Think,
        "思维",
        filters.show_think,
        IconName::Cpu,
        cx,
    ));
    kind_buttons.push(render_kind_filter_button(
        "agent-chat-filter-tool",
        KindClass::Tool,
        "工具",
        filters.show_tool,
        IconName::SquareTerminal,
        cx,
    ));
    kind_buttons.push(render_kind_filter_button(
        "agent-chat-filter-decision",
        KindClass::Decision,
        "决策",
        filters.show_decision,
        IconName::CircleCheck,
        cx,
    ));
    kind_buttons.push(render_kind_filter_button(
        "agent-chat-filter-observation",
        KindClass::Observation,
        "观测",
        filters.show_observation,
        IconName::Eye,
        cx,
    ));
    kind_buttons.push(render_kind_filter_button(
        "agent-chat-filter-message",
        KindClass::Message,
        "消息",
        filters.show_message,
        IconName::Info,
        cx,
    ));
    kind_buttons.push(
        Button::new("agent-chat-filter-reset")
            .icon(IconName::Loader)
            .label("重置")
            .xsmall()
            .ghost()
            .on_click(cx.listener(|this, _, _, cx| {
                reset_filters(this);
                cx.notify();
            }))
            .into_any_element(),
    );

    let filter_bar = v_flex()
        .flex_shrink_0()
        .gap_2()
        .pb_3()
        .border_b_1()
        .border_color(cx.theme().border)
        .child(h_flex().gap_2().flex_wrap().children(kind_buttons))
        .child(
            div()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child(format!("显示 {} / {} 条", shown, total)),
        );

    // ── 消息流（按轮次分组 + 折叠） ──
    let mut list_children: Vec<AnyElement> = Vec::new();
    let mut current_round: Option<u32> = None;
    for m in &visible {
        if m.round != current_round {
            current_round = m.round;
            if let Some(r) = m.round {
                let count = round_counts.get(&r).copied().unwrap_or(0);
                let collapsed = filters.collapsed_rounds.contains(&r);
                list_children.push(render_round_header(r, count, collapsed, cx));
            }
        }
        if let Some(r) = m.round {
            if filters.collapsed_rounds.contains(&r) {
                continue;
            }
        }
        list_children.push(render_message_row(m, cx));
    }

    // 过滤 / 折叠后为空
    if list_children.is_empty() {
        return v_flex()
            .size_full()
            .flex_1()
            .items_center()
            .justify_center()
            .gap_3()
            .text_color(cx.theme().muted_foreground)
            .text_xs()
            .child(
                v_flex()
                    .items_center()
                    .gap_2()
                    .child(div().child("没有符合当前过滤条件的记录"))
                    .child(
                        Button::new("agent-chat-filter-reset-empty")
                            .label("重置过滤条件")
                            .xsmall()
                            .outline()
                            .on_click(cx.listener(|this, _, _, cx| {
                                reset_filters(this);
                                cx.notify();
                            })),
                    ),
            )
            .into_any_element();
    }

    div()
        .size_full()
        .flex_1()
        .flex()
        .flex_col()
        .overflow_hidden()
        .child(filter_bar)
        .child(
            div()
                .id("agent-chat-history-scroll")
                .flex_1()
                .w_full()
                .overflow_y_scrollbar()
                .pt_3()
                .child(v_flex().gap_3().children(list_children)),
        )
        .into_any_element()
}

/// 单个 kind 的过滤切换按钮（激活态 = primary，未激活 = outline）。
fn render_kind_filter_button(
    id: &'static str,
    kind: KindClass,
    label: &'static str,
    active: bool,
    icon: IconName,
    cx: &mut Context<AppSidebar>,
) -> AnyElement {
    Button::new(id)
        .icon(icon)
        .label(label)
        .xsmall()
        .when(active, |b| b.primary())
        .when(!active, |b| b.outline())
        .on_click(cx.listener(move |this, _, _, cx| {
            toggle_kind(this, kind);
            cx.notify();
        }))
        .into_any_element()
}

/// 轮次头部：居中徽标 + 展开/折叠按钮。
fn render_round_header(
    round: u32,
    count: usize,
    collapsed: bool,
    cx: &mut Context<AppSidebar>,
) -> AnyElement {
    h_flex()
        .items_center()
        .gap_2()
        .w_full()
        .child(div().flex_1().h_px().bg(cx.theme().border))
        .child(
            div()
                .px_3()
                .py_0p5()
                .rounded_full()
                .border_1()
                .border_color(cx.theme().accent.opacity(0.25))
                .bg(cx.theme().accent.opacity(0.08))
                .text_color(cx.theme().accent)
                .text_xs()
                .font_semibold()
                .child(format!("第 {} 轮 · {} 条", round, count)),
        )
        .child(
            Button::new(format!("agent-chat-round-{}", round))
                .icon(if collapsed {
                    IconName::ChevronDown
                } else {
                    IconName::ChevronUp
                })
                .label(if collapsed { "展开" } else { "折叠" })
                .xsmall()
                .ghost()
                .on_click(cx.listener(move |this, _, _, cx| {
                    toggle_round_collapse(this, round);
                    cx.notify();
                })),
        )
        .child(div().flex_1().h_px().bg(cx.theme().border))
        .into_any_element()
}

/// 单条消息：左侧 kind 徽标 + agent_id + content（含代码段等宽底色块）。
fn render_message_row(msg: &AgentChatMessage, cx: &mut Context<AppSidebar>) -> AnyElement {
    let accent = kind_accent(&msg.kind, cx);
    let label = kind_label(&msg.kind);

    let byline = if msg.role.is_empty() {
        msg.agent_id.clone()
    } else {
        format!("{} · {}", msg.agent_id, msg.role)
    };

    h_flex()
        .items_start()
        .gap_2()
        .w_full()
        .child(
            div()
                .flex_none()
                .px_2()
                .py_0p5()
                .rounded_md()
                .bg(accent.opacity(0.15))
                .text_color(accent)
                .text_xs()
                .font_semibold()
                .child(label),
        )
        .child(
            v_flex()
                .flex_1()
                .min_w_0()
                .gap_1()
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child(byline),
                )
                .children(render_content_segments(&msg.content, cx)),
        )
        .into_any_element()
}

/// 将 content 拆成普通文本段与 ```fenced``` 代码段（代码段 = 等宽 + 底色块）。
fn render_content_segments(content: &str, cx: &mut Context<AppSidebar>) -> Vec<AnyElement> {
    let mut children: Vec<AnyElement> = Vec::new();
    for (idx, part) in content.split("```").enumerate() {
        let text = part.trim();
        if text.is_empty() {
            continue;
        }
        if idx % 2 == 1 {
            children.push(
                div()
                    .w_full()
                    .rounded_md()
                    .px_2()
                    .py_1()
                    .mt_1()
                    .mb_1()
                    .bg(cx.theme().muted_foreground.opacity(0.08))
                    .font_family("monospace")
                    .text_xs()
                    .child(text.to_string())
                    .into_any_element(),
            );
        } else {
            children.push(
                div()
                    .text_xs()
                    .text_color(cx.theme().foreground)
                    .child(text.to_string())
                    .into_any_element(),
            );
        }
    }
    children
}
