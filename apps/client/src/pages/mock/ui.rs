//! 子区块渲染：页头 / 列表态 / 会话态 / 调试器。

use gpui::prelude::*;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::scroll::ScrollableElement;
use gpui_component::{h_flex, v_flex, ActiveTheme, IconName, Sizable, StyledExt};

use crate::components::agent_chat_history::render_agent_chat_history;
use crate::components::sidebar::AppSidebar;

use super::input::render_edit_input;
use super::logic::{add_tool_result, inject_ai, inject_user, reset_all, simulate_decision};
use super::types::{current_round, update_state, with_state, AGENT_ID, CHAMPION, MockView};

// ── 子区块渲染 ──

/// 顶部：标题 + 数据源徽标 + 列表/会话切换。
pub(super) fn render_page_header(cx: &mut Context<AppSidebar>, view: &MockView) -> AnyElement {
    let view = *view;
    h_flex()
        .items_center()
        .justify_between()
        .w_full()
        .child(
            h_flex()
                .gap_3()
                .items_center()
                .child(
                    div()
                        .w(rems(2.5))
                        .h(rems(2.5))
                        .rounded_lg()
                        .bg(cx.theme().accent.opacity(0.12))
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_color(cx.theme().accent)
                        .child(IconName::Bot),
                )
                .child(
                    v_flex()
                        .gap_0p5()
                        .child(div().font_bold().text_lg().child("Mock 模拟测试沙盒"))
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child("在完全离线的环境下对 MoonLOL 桌面端的核心渲染与通信模块进行模拟调试与交互验证。"),
                        ),
                ),
        )
        .child(
            h_flex()
                .gap_3()
                .items_center()
                .child(render_view_toggle(cx, view))
                .child(
                    div()
                        .px_2()
                        .py_0p5()
                        .rounded_md()
                        .border_1()
                        .border_color(cx.theme().accent.opacity(0.3))
                        .bg(cx.theme().accent.opacity(0.08))
                        .text_xs()
                        .text_color(cx.theme().accent)
                        .child("数据源: mock.json"),
                ),
        )
        .into_any_element()
}

/// 列表/会话 切换按钮组。
fn render_view_toggle(cx: &mut Context<AppSidebar>, active: MockView) -> AnyElement {
    let list_active = active == MockView::List;
    let chat_active = active == MockView::Chat;
    h_flex()
        .gap_1()
        .child(
            Button::new("mock-view-list")
                .label("列表")
                .xsmall()
                .when(list_active, |b| b.primary())
                .when(!list_active, |b| b.outline())
                .on_click(cx.listener(|_, _, _, cx| {
                    update_state(|s| s.view = MockView::List);
                    cx.notify();
                })),
        )
        .child(
            Button::new("mock-view-chat")
                .label("会话")
                .xsmall()
                .when(chat_active, |b| b.primary())
                .when(!chat_active, |b| b.outline())
                .on_click(cx.listener(|_, _, _, cx| {
                    update_state(|s| s.view = MockView::Chat);
                    cx.notify();
                })),
        )
        .into_any_element()
}

/// 列表态：落地页（对应 index.vue）。
pub(super) fn render_list_view(cx: &mut Context<AppSidebar>) -> AnyElement {
    let accent = cx.theme().accent;
    v_flex()
        .size_full()
        .items_center()
        .justify_center()
        .gap_6()
        .child(
            v_flex()
                .items_center()
                .gap_2()
                .child(
                    div()
                        .text_3xl()
                        .font_bold()
                        .text_color(accent)
                        .child("Mock 模拟测试沙盒"),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child("在完全离线的环境下对 MoonLOL 桌面端的核心渲染和通信模块进行模拟调试与交互验证。"),
                ),
        )
        .child(
            div()
                .w(rems(32.))
                .border_1()
                .border_color(cx.theme().border)
                .rounded_lg()
                .p_6()
                .child(
                    v_flex()
                        .gap_3()
                        .child(
                            div()
                                .w(rems(3.))
                                .h(rems(3.))
                                .rounded_lg()
                                .border_1()
                                .border_color(cx.theme().accent.opacity(0.25))
                                .bg(cx.theme().accent.opacity(0.08))
                                .flex()
                                .items_center()
                                .justify_center()
                                .text_color(accent)
                                .child(IconName::Cpu),
                        )
                        .child(div().text_lg().font_bold().child("AI 决策流渲染模拟"))
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child("基于本地 mock.json 数据源，模拟 AI 代理的实时决策思维链生成。测试 Markdown 渲染格式、折叠面板及自定义工具调用展示的排版交互。"),
                        )
                        .child(
                            Button::new("mock-enter-chat")
                                .primary()
                                .icon(IconName::ArrowRight)
                                .label("进入测试床")
                                .on_click(cx.listener(|_, _, _, cx| {
                                    update_state(|s| s.view = MockView::Chat);
                                    cx.notify();
                                })),
                        ),
                ),
        )
        .into_any_element()
}

/// 会话态整体（对应 chat.vue）：左侧控制面板 + 右侧消息流。
pub(super) fn render_chat_view(
    window: &mut Window,
    cx: &mut Context<AppSidebar>,
) -> AnyElement {
    let messages = with_state(|s| s.messages.clone());
    let turns = current_round(&messages);
    let count = messages.len();

    // 先构建消息流元素（内部带独立滚动区与筛选栏）
    let history_element = render_agent_chat_history(&messages, cx);

    h_flex()
        .size_full()
        .gap_4()
        .overflow_hidden()
        .child(
            v_flex()
                .w(rems(20.))
                .flex_shrink_0()
                .gap_4()
                .overflow_y_scrollbar()
                .child(render_agent_card(cx, turns, count))
                .child(render_debugger(window, cx)),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .rounded_lg()
                .border_1()
                .border_color(cx.theme().border)
                .overflow_hidden()
                .child(render_chat_panel_header(cx, count))
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .overflow_hidden()
                        .p_4()
                        .child(history_element),
                ),
        )
        .into_any_element()
}

/// 会话态右侧面板头部。
fn render_chat_panel_header(cx: &mut Context<AppSidebar>, count: usize) -> AnyElement {
    h_flex()
        .items_center()
        .justify_between()
        .px_4()
        .py_2()
        .border_b_1()
        .border_color(cx.theme().border)
        .bg(cx.theme().background)
        .child(
            h_flex()
                .gap_2()
                .items_center()
                .child(
                    div()
                        .w(px(8.))
                        .h(px(8.))
                        .rounded_full()
                        .bg(cx.theme().success),
                )
                .child(div().text_xs().font_bold().child("AI 决策流实时渲染")),
        )
        .child(
            h_flex()
                .gap_2()
                .items_center()
                .child(
                    div()
                        .px_2()
                        .py_0p5()
                        .rounded_md()
                        .border_1()
                        .border_color(cx.theme().accent.opacity(0.25))
                        .bg(cx.theme().accent.opacity(0.08))
                        .text_xs()
                        .text_color(cx.theme().accent)
                        .child("AgentChatHistory"),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child(format!("{} 条消息", count)),
                ),
        )
        .into_any_element()
}

/// 左侧「当前 Agent」信息卡。
fn render_agent_card(cx: &mut Context<AppSidebar>, turns: u32, count: usize) -> AnyElement {
    v_flex()
        .gap_2()
        .p_4()
        .border_1()
        .border_color(cx.theme().border)
        .rounded_lg()
        .child(
            div()
                .text_xs()
                .font_bold()
                .text_color(cx.theme().muted_foreground)
                .child("当前 Agent"),
        )
        .child(div().w_full().h_px().bg(cx.theme().border))
        .child(
            h_flex()
                .items_center()
                .justify_between()
                .px_3()
                .py_2()
                .border_1()
                .border_color(cx.theme().border)
                .rounded_md()
                .bg(cx.theme().muted_foreground.opacity(0.08))
                .child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .child(div().w(px(8.)).h(px(8.)).rounded_full().bg(cx.theme().info))
                        .child(div().text_xs().font_bold().child(CHAMPION)),
                )
                .child(
                    div()
                        .text_xs()
                        .font_family("monospace")
                        .text_color(cx.theme().muted_foreground)
                        .child(AGENT_ID),
                ),
        )
        .child(
            h_flex()
                .gap_2()
                .child(
                    div()
                        .flex_1()
                        .border_1()
                        .border_color(cx.theme().border)
                        .rounded_md()
                        .p_2()
                        .bg(cx.theme().muted_foreground.opacity(0.06))
                        .child(
                            v_flex()
                                .gap_0p5()
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground)
                                        .child("决策轮次"),
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .font_bold()
                                        .text_color(cx.theme().accent)
                                        .child(turns.to_string()),
                                ),
                        ),
                )
                .child(
                    div()
                        .flex_1()
                        .border_1()
                        .border_color(cx.theme().border)
                        .rounded_md()
                        .p_2()
                        .bg(cx.theme().muted_foreground.opacity(0.06))
                        .child(
                            v_flex()
                                .gap_0p5()
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground)
                                        .child("消息条数"),
                                )
                                .child(div().text_sm().font_bold().child(count.to_string())),
                        ),
                ),
        )
        .child(
            Button::new("mock-reset")
                .outline()
                .icon(IconName::Undo2)
                .w_full()
                .label("重置数据")
                .on_click(cx.listener(|_, _, _, cx| {
                    reset_all();
                    cx.notify();
                })),
        )
        .into_any_element()
}

/// 左侧「模拟调试器」：预设动作 + 手动注入。
fn render_debugger(
    window: &mut Window,
    cx: &mut Context<AppSidebar>,
) -> AnyElement {
    let get_user = || with_state(|s| s.user_input.clone());
    let set_user = |v: String| update_state(|s| s.user_input = v);
    let get_ai = || with_state(|s| s.assistant_input.clone());
    let set_ai = |v: String| update_state(|s| s.assistant_input = v);

    v_flex()
        .gap_3()
        .p_4()
        .border_1()
        .border_color(cx.theme().border)
        .rounded_lg()
        .child(
            div()
                .text_xs()
                .font_bold()
                .text_color(cx.theme().muted_foreground)
                .child("模拟调试器"),
        )
        .child(div().w_full().h_px().bg(cx.theme().border))
        .child(
            v_flex()
                .gap_2()
                .child(
                    div()
                        .text_xs()
                        .font_semibold()
                        .text_color(cx.theme().muted_foreground)
                        .child("预设复杂动作"),
                )
                .child(
                    Button::new("mock-simulate-decision")
                        .primary()
                        .icon(IconName::CircleCheck)
                        .w_full()
                        .label("模拟决策")
                        .on_click(cx.listener(|_, _, _, cx| simulate_decision(cx))),
                )
                .child(
                    Button::new("mock-simulate-result")
                        .outline()
                        .icon(IconName::SquareTerminal)
                        .w_full()
                        .label("模拟工具结果")
                        .on_click(cx.listener(|_, _, _, cx| add_tool_result(cx))),
                ),
        )
        .child(
            v_flex()
                .gap_3()
                .child(
                    div()
                        .text_xs()
                        .font_semibold()
                        .text_color(cx.theme().muted_foreground)
                        .child("手动注入（输入框支持 Enter）"),
                )
                .child(
                    v_flex()
                        .gap_1p5()
                        .child(render_edit_input(
                            window,
                            cx,
                            "mock-user-input",
                            "输入用户环境观测消息…",
                            get_user,
                            set_user,
                            Some(Box::new(inject_user)),
                        ))
                        .child(
                            Button::new("mock-inject-user")
                                .outline()
                                .icon(IconName::User)
                                .w_full()
                                .label("注入用户消息")
                                .on_click(cx.listener(|_, _, _, cx| inject_user(cx))),
                        ),
                )
                .child(
                    v_flex()
                        .gap_1p5()
                        .child(render_edit_input(
                            window,
                            cx,
                            "mock-assistant-input",
                            "输入 AI 回复消息…",
                            get_ai,
                            set_ai,
                            Some(Box::new(inject_ai)),
                        ))
                        .child(
                            Button::new("mock-inject-ai")
                                .outline()
                                .icon(IconName::Bot)
                                .w_full()
                                .label("注入 AI 消息")
                                .on_click(cx.listener(|_, _, _, cx| inject_ai(cx))),
                        ),
                ),
        )
        .into_any_element()
}
