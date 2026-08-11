//! 对局调试台 — 对应 client `pages/debug/[id].vue`，用 sidebar.current_game_id 定位对局。
//!
//! 页面状态在 `types.rs`，事件解析 / 对局控制逻辑在 `logic.rs`，
//! 本文件只保留渲染函数与公开入口 `render_debug`。

mod logic;
mod types;

use gpui::prelude::*;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::menu::{DropdownMenu, PopupMenuItem};
use gpui_component::scroll::ScrollableElement;
use gpui_component::{h_flex, v_flex, ActiveTheme, Disableable, IconName, Sizable, StyledExt};
pub use types::DebugPageState;
use uuid::Uuid;

use self::logic::{run_match_cmd, spawn_init};
use self::types::{DebugTab, MatchCmd};
use crate::components::agent_chat_history::render_agent_chat_history;
use crate::components::game_console_logs::render_game_console_logs;
use crate::components::sidebar::AppSidebar;
use crate::services::provider;
use crate::types::ActiveView;

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
    ) = (
        sidebar.debug.error.clone(),
        sidebar.debug.logs.clone(),
        sidebar.debug.messages.clone(),
        sidebar.debug.active_tab,
        sidebar.debug.god_mode,
        sidebar.debug.cooldown_disabled,
        sidebar.debug.paused,
        sidebar.debug.switch_target.clone(),
        sidebar.debug.stopping,
        sidebar.debug.stream_alive,
    );

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
            this.debug.god_mode = enabled;
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
            this.debug.cooldown_disabled = enabled;
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
            this.debug.paused = !paused;
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
                        weak.update(cx, |this, cx| {
                            this.debug.switch_target = name.clone();
                            cx.notify();
                        })
                        .ok();
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
            let target = this.debug.switch_target.clone();
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
            this.debug.stopping = true;
            let gid = this.current_game_id.clone().unwrap_or_default();
            cx.spawn(
                move |weak: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
                    let weak = weak.clone();
                    let mut cx = cx.clone();
                    let gid = gid.clone();
                    async move {
                        let _ = provider::process_service().stop(&gid).await;
                        weak.update(&mut cx, |this, cx| {
                            this.debug.stopping = false;
                            this.debug.stream_alive = false;
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
        .on_click(cx.listener(|this, _, _, cx| {
            this.debug.active_tab = DebugTab::Logs;
            cx.notify();
        }));

    let agents_tab_btn = Button::new("debug-tab-agents")
        .small()
        .icon(IconName::Bot)
        .label("AI 思维链")
        .when(active_tab == DebugTab::Agents, |b| b.primary())
        .when(active_tab != DebugTab::Agents, |b| b.ghost())
        .on_click(cx.listener(|this, _, _, cx| {
            this.debug.active_tab = DebugTab::Agents;
            cx.notify();
        }));

    let tab_content = match active_tab {
        DebugTab::Logs => render_game_console_logs(&logs, &*sidebar, cx),
        DebugTab::Agents => render_agent_chat_history(&messages, &*sidebar, cx),
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
    let is_current = sidebar.debug.current_game.as_deref() == Some(game_id.as_str());
    if !is_current {
        let gen = sidebar.debug.generation + 1;
        let first_champ = sidebar
            .champions_list
            .first()
            .cloned()
            .unwrap_or_else(|| "Riven".to_string());
        sidebar.debug = DebugPageState {
            current_game: Some(game_id.clone()),
            generation: gen,
            switch_target: first_champ,
            ..Default::default()
        };
        spawn_init(game_id.clone(), gen, cx);
    }

    render_content(sidebar, &game_id, cx)
}
