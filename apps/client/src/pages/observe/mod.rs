//! 观战/回放页 — 用 sidebar.current_match_id 决定观战对局，定时轮询增量事件。
//!
//! 拆分：`types.rs` 页面状态、`logic.rs` 数据加载/轮询、`display.rs` 展示辅助。

mod display;
mod logic;
mod types;

use gpui::prelude::*;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::scroll::ScrollableElement;
use gpui_component::{h_flex, v_flex, ActiveTheme, Disableable, IconName, Sizable, StyledExt};
use lol_web_protocol::match_::MatchStatus;

use self::display::{build_rosters, event_label, fmt_date, info_row, short_id};
use self::logic::{spawn_load, spawn_poll};
use self::types::{reset_state_for, update_state, with_state, RosterAgent};
use crate::components::sidebar::AppSidebar;
use crate::services::provider;
use crate::types::ActiveView;

/// 观战/回放页（对应 client `pages/observe/[id].vue`，用 sidebar.current_match_id）。
pub fn render_observe(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) -> AnyElement {
    let match_id = sidebar.current_match_id;
    reset_state_for(match_id);

    // ── 空态：未选中对局 ──
    let Some(id) = match_id else {
        return v_flex()
            .size_full()
            .flex_1()
            .items_center()
            .justify_center()
            .gap_3()
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child("尚未选择对局，去游戏页选择一场正在进行的对局进行观战。"),
            )
            .child(
                Button::new("observe-empty-back-btn")
                    .primary()
                    .label("返回游戏页")
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.navigate_to(ActiveView::Games);
                        cx.notify();
                    })),
            )
            .into_any_element();
    };
    let id_str = id.to_string();
    let short = short_id(&id_str);

    // 首次加载 + 轮询（防重复 spawn）
    let (inited, polling) = with_state(|s| (s.inited, s.polling));
    if !inited {
        update_state(|s| {
            s.inited = true;
            s.loading = true;
        });
        spawn_load(id, cx);
    }
    if !polling {
        update_state(|s| s.polling = true);
        spawn_poll(cx);
    }

    let (match_info, loading, error, paused, confirming_stop, stopping, stop_error, events_count) =
        with_state(|s| {
            (
                s.match_info.clone(),
                s.loading,
                s.error.clone(),
                s.paused,
                s.confirming_stop,
                s.stopping,
                s.stop_error.clone(),
                s.events.len(),
            )
        });

    let warning = cx.theme().warning;
    let foreground = cx.theme().foreground;
    let muted = cx.theme().muted_foreground;
    let border = cx.theme().border;
    let accent = cx.theme().accent;
    let danger = cx.theme().danger;
    let success = cx.theme().success;

    // 时间线行（倒序展示）
    let lines: Vec<(String, String, Hsla)> = with_state(|s| {
        s.events
            .iter()
            .rev()
            .map(|ev| {
                let et = ev
                    .payload
                    .get("event_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let tone = match et {
                    "agent_stalled" => warning,
                    "champion_kill" | "turret_destroyed" | "match_finished" => foreground,
                    _ => muted,
                };
                (format!("#{:04}", ev.seq), event_label(ev), tone)
            })
            .collect()
    });
    let lines_empty = lines.is_empty();
    let (order_agents, chaos_agents, stalled_agents) = with_state(|s| build_rosters(&s.events));
    let stalled_text = stalled_agents
        .iter()
        .map(|s| short_id(s))
        .collect::<Vec<_>>()
        .join("、");

    let mode = match_info
        .as_ref()
        .map_or("—".to_string(), |m| m.mode.clone());
    let created_at = match_info
        .as_ref()
        .map_or_else(String::new, |m| fmt_date(&m.created_at));
    let is_running = match_info
        .as_ref()
        .map_or(false, |m| m.status == MatchStatus::Running);

    // ── 头部徽章 ──
    let mode_pill = h_flex()
        .items_center()
        .gap_1p5()
        .rounded_md()
        .px_2()
        .py_0p5()
        .text_xs()
        .font_bold()
        .bg(accent.opacity(0.15))
        .text_color(accent)
        .child(mode.clone())
        .into_any_element();

    let status_pill: Option<AnyElement> = match_info.as_ref().map(|m| {
        let (label, running) = match m.status {
            MatchStatus::Running => ("直播中", true),
            MatchStatus::Pending => ("等待开始", false),
            MatchStatus::Paused => ("已暂停", false),
            MatchStatus::Finished => ("已结束", false),
            MatchStatus::Aborted => ("已中止", false),
        };
        let color = match m.status {
            MatchStatus::Running => success,
            MatchStatus::Paused => warning,
            MatchStatus::Aborted => danger,
            _ => muted,
        };
        h_flex()
            .items_center()
            .gap_1p5()
            .rounded_full()
            .px_2()
            .py_0p5()
            .text_xs()
            .font_bold()
            .bg(color.opacity(0.15))
            .text_color(color)
            .when(running, |d| {
                d.child(div().w_2().h_2().rounded_full().bg(color))
            })
            .child(label.to_string())
            .into_any_element()
    });

    // ── 右侧操作：暂停刷新 + 结束对局（含二次确认） ──
    let right_actions: AnyElement = if confirming_stop && is_running {
        h_flex()
            .gap_2()
            .child(
                Button::new("observe-confirm-stop-btn")
                    .danger()
                    .label(if stopping {
                        "结束中…"
                    } else {
                        "确认结束"
                    })
                    .loading(stopping)
                    .disabled(stopping)
                    .on_click(cx.listener(move |_this, _, _, cx| {
                        update_state(|s| {
                            s.stopping = true;
                            s.stop_error = None;
                        });
                        let client = provider::cloud_client().clone();
                        let id_str = id_str.clone();
                        cx.spawn(
                            move |weak: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
                                let weak = weak.clone();
                                let mut cx = cx.clone();
                                let client = client.clone();
                                let id_str = id_str.clone();
                                async move {
                                    match client.stop_match(&id_str).await {
                                        Ok(()) => {
                                            update_state(|s| {
                                                s.match_id = None;
                                                s.stopping = false;
                                                s.confirming_stop = false;
                                            });
                                            if let Some(entity) = weak.upgrade() {
                                                let _ = entity.update(&mut cx, |this, cx| {
                                                    this.current_match_id = None;
                                                    this.navigate_to(ActiveView::Games);
                                                    cx.notify();
                                                });
                                            }
                                        }
                                        Err(e) => {
                                            update_state(|s| {
                                                s.stopping = false;
                                                s.stop_error = Some(e.to_string());
                                            });
                                            if let Some(entity) = weak.upgrade() {
                                                let _ = entity.update(&mut cx, |_, cx| cx.notify());
                                            }
                                        }
                                    }
                                }
                            },
                        )
                        .detach();
                    })),
            )
            .child(
                Button::new("observe-cancel-stop-btn")
                    .ghost()
                    .label("取消")
                    .disabled(stopping)
                    .on_click(cx.listener(move |_this, _, _, cx| {
                        update_state(|s| s.confirming_stop = false);
                        cx.notify();
                    })),
            )
            .into_any_element()
    } else if is_running {
        Button::new("observe-stop-btn")
            .outline()
            .danger()
            .icon(IconName::CircleX)
            .label("结束对局")
            .small()
            .on_click(cx.listener(move |_this, _, _, cx| {
                update_state(|s| s.confirming_stop = true);
                cx.notify();
            }))
            .into_any_element()
    } else {
        div().into_any_element()
    };

    let header = h_flex()
        .items_center()
        .justify_between()
        .child(
            h_flex()
                .items_center()
                .gap_2()
                .child(
                    Button::new("observe-back-btn")
                        .ghost()
                        .icon(IconName::ArrowLeft)
                        .tooltip("返回")
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.current_match_id = None;
                            this.navigate_to(ActiveView::Games);
                            update_state(|s| s.match_id = None);
                            cx.notify();
                        })),
                )
                .child(div().text_lg().font_bold().child("观战"))
                .child(
                    div()
                        .text_xs()
                        .text_color(muted)
                        .child(format!("#{}", short)),
                )
                .child(mode_pill)
                .when_some(status_pill, |d, pill| d.child(pill)),
        )
        .child(
            h_flex()
                .gap_2()
                .child(
                    Button::new("observe-pause-btn")
                        .outline()
                        .icon(if paused {
                            IconName::Play
                        } else {
                            IconName::Pause
                        })
                        .label(if paused {
                            "继续刷新"
                        } else {
                            "暂停刷新"
                        })
                        .small()
                        .on_click(cx.listener(move |_this, _, _, cx| {
                            update_state(|s| s.paused = !s.paused);
                            cx.notify();
                        })),
                )
                .child(right_actions),
        );

    // ── 左栏：对局信息 + 双方阵容 ──
    let info_card = v_flex()
        .gap_2()
        .child(div().text_sm().font_bold().child("对局信息"))
        .child(info_row(
            "对局 ID",
            format!("#{}", short),
            muted,
            foreground,
        ))
        .child(info_row("模式", mode, muted, foreground))
        .child(info_row("创建时间", created_at, muted, foreground))
        .when_some(match_info.as_ref(), |d, m| {
            let label = match m.status {
                MatchStatus::Running => "直播中",
                MatchStatus::Pending => "等待开始",
                MatchStatus::Paused => "已暂停",
                MatchStatus::Finished => "已结束",
                MatchStatus::Aborted => "已中止",
            };
            d.child(info_row("状态", label.to_string(), muted, foreground))
        });

    let team_col = |title: &str, agents: &[RosterAgent], stalled: &[String]| -> AnyElement {
        v_flex()
            .flex_1()
            .gap_1p5()
            .child(div().text_xs().text_color(muted).child(title.to_string()))
            .when(agents.is_empty(), |d| {
                d.child(
                    div()
                        .py_3()
                        .text_center()
                        .text_xs()
                        .text_color(muted)
                        .child("等待数据…"),
                )
            })
            .when(!agents.is_empty(), |d| {
                d.children(agents.iter().map(|a| {
                    let dot = if stalled.iter().any(|s| s == &a.id) {
                        warning
                    } else {
                        success
                    };
                    h_flex()
                        .items_center()
                        .justify_between()
                        .gap_2()
                        .rounded_md()
                        .border_1()
                        .border_color(border.opacity(0.5))
                        .px_3()
                        .py_2()
                        .text_xs()
                        .child(
                            div()
                                .min_w_0()
                                .child(div().truncate().font_bold().child(a.name.clone()))
                                .child(
                                    div().truncate().text_color(muted).child(a.champion.clone()),
                                ),
                        )
                        .child(div().w_2().h_2().flex_shrink_0().rounded_full().bg(dot))
                        .into_any_element()
                }))
            })
            .into_any_element()
    };

    let rosters_card = v_flex()
        .gap_2()
        .child(div().text_sm().font_bold().child("双方阵容"))
        .child(
            h_flex()
                .gap_4()
                .child(team_col("Order · 蓝方", &order_agents, &stalled_agents))
                .child(team_col("Chaos · 红方", &chaos_agents, &stalled_agents)),
        );

    let left_col = v_flex()
        .flex_1()
        .min_h_0()
        .gap_4()
        .overflow_hidden()
        .child(info_card)
        .child(rosters_card);

    // ── 右栏：事件时间线 ──
    let timeline = v_flex()
        .w(rems(24.))
        .min_h_0()
        .gap_2()
        .overflow_hidden()
        .child(
            h_flex()
                .items_center()
                .justify_between()
                .child(div().text_sm().font_bold().child("事件时间线"))
                .child(
                    div()
                        .px_2()
                        .py_0p5()
                        .rounded_md()
                        .text_xs()
                        .font_bold()
                        .bg(accent.opacity(0.15))
                        .text_color(accent)
                        .child(events_count.to_string()),
                ),
        )
        .child(
            div()
                .flex_1()
                .min_h_0()
                .rounded_lg()
                .border_1()
                .border_color(border)
                .overflow_hidden()
                .child(
                    div()
                        .size_full()
                        .overflow_y_scrollbar()
                        .py_1()
                        .when(lines_empty, |d| {
                            d.flex().items_center().justify_center().child(
                                div().text_xs().text_color(muted).child(if loading {
                                    "加载中…".to_string()
                                } else {
                                    "等待事件…".to_string()
                                }),
                            )
                        })
                        .when(!lines_empty, |d| {
                            d.children(lines.into_iter().map(|(seq, label, tone)| {
                                h_flex()
                                    .items_start()
                                    .gap_2()
                                    .px_3()
                                    .py_1()
                                    .text_xs()
                                    .child(div().flex_shrink_0().text_color(muted).child(seq))
                                    .child(div().text_color(tone).child(label))
                                    .into_any_element()
                            }))
                        }),
                ),
        );

    let body_row = div()
        .flex_1()
        .min_h_0()
        .flex()
        .flex_row()
        .gap_6()
        .child(left_col)
        .child(timeline);

    v_flex()
        .size_full()
        .flex_1()
        .overflow_hidden()
        .child(header)
        .child(div().w_full().h_px().bg(border))
        .when(!stalled_text.is_empty(), |d| {
            d.child(
                h_flex()
                    .gap_2()
                    .items_start()
                    .rounded_md()
                    .border_1()
                    .border_color(warning.opacity(0.4))
                    .bg(warning.opacity(0.1))
                    .px_3()
                    .py_2()
                    .text_xs()
                    .child(
                        div()
                            .font_bold()
                            .text_color(warning)
                            .child("部分 Agent 动力源失联，对局已暂停等待恢复"),
                    )
                    .child(
                        div()
                            .text_color(warning)
                            .child(format!("失联 Agent：{}", stalled_text)),
                    ),
            )
        })
        .when_some(error.as_ref().or(stop_error.as_ref()), |d, err| {
            d.child(
                div()
                    .px_3()
                    .py_2()
                    .rounded_md()
                    .bg(danger.opacity(0.1))
                    .text_color(danger)
                    .text_xs()
                    .child(err.clone()),
            )
        })
        .child(body_row)
        .into_any_element()
}
