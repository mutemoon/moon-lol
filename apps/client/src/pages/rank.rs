use gpui::prelude::*;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::scroll::ScrollableElement;
use gpui_component::{h_flex, v_flex, ActiveTheme, IconName, StyledExt};
use rust_i18n::t;

use crate::components::sidebar::AppSidebar;
use crate::services::provider;
use crate::types::ActiveView;

// ── 数据加载辅助 ──

/// 拉取当前选中 Agent 的参赛快照，写入 sidebar.rank_snapshots。
fn load_snapshots(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) {
    let agent_id = sidebar.rank_selected_agent_id.clone();
    if agent_id.is_empty() {
        sidebar.rank_snapshots.clear();
        sidebar.rank_selected_snapshot_id.clear();
        cx.notify();
        return;
    }
    let cloud = provider::cloud_client().clone();
    cx.spawn(
        move |weak: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
            let weak = weak.clone();
            let mut cx = cx.clone();
            async move {
                let snapshots = cloud.list_snapshots(&agent_id).await.unwrap_or_default();
                weak.update(&mut cx, |this, cx| {
                    // 防止切换 Agent 后旧结果覆盖新选择
                    if this.rank_selected_agent_id == agent_id {
                        this.rank_snapshots = snapshots.clone();
                        this.rank_selected_snapshot_id = snapshots
                            .first()
                            .map(|s| s.id.to_string())
                            .unwrap_or_default();
                        cx.notify();
                    }
                })
                .ok();
            }
        },
    )
    .detach();
}

/// 重新拉取当前排队状态，写入 sidebar.rank_queue。
fn refresh_queue(_sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) {
    let cloud = provider::cloud_client().clone();
    cx.spawn(
        move |weak: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
            let weak = weak.clone();
            let mut cx = cx.clone();
            async move {
                let queue = cloud.get_rank_status().await.unwrap_or_default();
                weak.update(&mut cx, |this, cx| {
                    this.rank_queue = queue;
                    cx.notify();
                })
                .ok();
            }
        },
    )
    .detach();
}
fn render_mode_selector(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) -> AnyElement {
    let selected = sidebar.rank_mode.clone();
    v_flex()
        .gap_1()
        .child(div().text_xs().font_bold().child(t!("app.rank.mode_label")))
        .child(
            h_flex()
                .gap_1()
                .children([("top_solo", "上单 SOLO")].map(|(val, label)| {
                    let btn = Button::new(format!("rank-mode-{}", val)).label(label.to_string());
                    if selected == val {
                        btn.primary().into_any_element()
                    } else {
                        let v = val.to_string();
                        btn.outline()
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.rank_mode = v.clone();
                                cx.notify();
                            }))
                            .into_any_element()
                    }
                })),
        )
        .into_any_element()
}

fn render_agent_selector(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) -> AnyElement {
    let agents = sidebar.rank_agents.clone();
    v_flex()
        .gap_1()
        .child(
            div()
                .text_xs()
                .font_bold()
                .child(t!("app.rank.agent_label")),
        )
        .child(if agents.is_empty() {
            div()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child("-")
                .into_any_element()
        } else {
            h_flex()
                .gap_1()
                .flex_wrap()
                .children(agents.into_iter().map(|a| {
                    let aid = a.id.to_string();
                    let label = format!("{} · {}", a.name, a.champion);
                    let selected = sidebar.rank_selected_agent_id == aid;
                    let btn = Button::new(format!("rank-agent-{}", aid)).label(label);
                    if selected {
                        btn.primary().into_any_element()
                    } else {
                        let aid2 = aid.clone();
                        btn.outline()
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.rank_selected_agent_id = aid2.clone();
                                this.rank_selected_snapshot_id.clear();
                                this.rank_snapshots.clear();
                                cx.notify();
                                load_snapshots(this, cx);
                            }))
                            .into_any_element()
                    }
                }))
                .into_any_element()
        })
        .into_any_element()
}

fn render_snapshot_selector(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) -> AnyElement {
    let snapshots = sidebar.rank_snapshots.clone();
    v_flex()
        .gap_1()
        .child(
            div()
                .text_xs()
                .font_bold()
                .child(t!("app.rank.snapshot_label")),
        )
        .child(if snapshots.is_empty() {
            v_flex()
                .gap_1()
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child(t!("app.rank.no_snapshot")),
                )
                .when(!sidebar.rank_selected_agent_id.is_empty(), |d| {
                    d.child(
                        Button::new("rank-go-publish")
                            .ghost()
                            .label(t!("app.rank.go_publish"))
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.navigate_to(ActiveView::Heroes);
                                cx.notify();
                            })),
                    )
                })
                .into_any_element()
        } else {
            h_flex()
                .gap_1()
                .flex_wrap()
                .children(snapshots.into_iter().map(|s| {
                    let sid = s.id.to_string();
                    let label = format!("v{}", s.version);
                    let selected = sidebar.rank_selected_snapshot_id == sid;
                    let btn = Button::new(format!("rank-snap-{}", sid)).label(label);
                    if selected {
                        btn.primary().into_any_element()
                    } else {
                        let sid2 = sid.clone();
                        btn.outline()
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.rank_selected_snapshot_id = sid2.clone();
                                cx.notify();
                            }))
                            .into_any_element()
                    }
                }))
                .into_any_element()
        })
        .into_any_element()
}

fn render_queue_section(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) -> AnyElement {
    let queue = sidebar.rank_queue.clone();
    v_flex()
        .gap_2()
        .child(
            h_flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .font_bold()
                        .text_sm()
                        .child(t!("app.rank.queue_title")),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child(format!("{}", queue.len())),
                ),
        )
        .child(if queue.is_empty() {
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .py_8()
                .text_center()
                .border_1()
                .border_color(cx.theme().border)
                .rounded_lg()
                .child(t!("app.rank.queue_empty"))
                .into_any_element()
        } else {
            v_flex()
                .gap_2()
                .children(queue.into_iter().map(|q| {
                    let mode = q.mode.clone();
                    let rating = q.rating;
                    let ts = &q.enqueued_at[..19.min(q.enqueued_at.len())];
                    let aid = q.agent_id.to_string();
                    let aid_short = &aid[..8.min(aid.len())];
                    let dequeue_aid = aid.clone();
                    let dequeue_mode = mode.clone();
                    h_flex()
                        .gap_2()
                        .items_center()
                        .justify_between()
                        .p_3()
                        .border_1()
                        .border_color(cx.theme().border)
                        .rounded_lg()
                        .child(
                            v_flex()
                                .gap_1()
                                .child(
                                    h_flex()
                                        .gap_2()
                                        .items_center()
                                        .child(IconName::Play)
                                        .child(div().text_sm().child(mode))
                                        .child(
                                            div()
                                                .text_xs()
                                                .px_1p5()
                                                .py_0p5()
                                                .bg(cx.theme().secondary)
                                                .rounded_sm()
                                                .child(format!("{} ELO", rating)),
                                        ),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground)
                                        .child(format!("{} · Agent {}", ts, aid_short)),
                                ),
                        )
                        .child(
                            Button::new(format!("rank-dequeue-{}", q.agent_snapshot_id))
                                .ghost()
                                .label("退出")
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.rank_queue.retain(|e| {
                                        !(e.agent_id.to_string() == dequeue_aid
                                            && e.mode == dequeue_mode)
                                    });
                                    cx.notify();
                                    refresh_queue(this, cx);
                                })),
                        )
                        .into_any_element()
                }))
                .into_any_element()
        })
        .into_any_element()
}

pub fn render_rank(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) -> AnyElement {
    // 首次渲染：并行加载 Agent 列表、排队状态、当前赛季
    if !sidebar.rank_loaded {
        sidebar.rank_loaded = true;
        let cloud = provider::cloud_client().clone();
        cx.spawn(
            move |weak: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
                let weak = weak.clone();
                let mut cx = cx.clone();
                async move {
                    let (agents, queue, season) = tokio::join!(
                        async { cloud.list_agents().await },
                        async { cloud.get_rank_status().await },
                        async { cloud.get_current_season().await },
                    );
                    let agents = agents.unwrap_or_default();
                    let queue = queue.unwrap_or_default();
                    let season = season.ok();
                    let selected = weak
                        .update(&mut cx, |this, cx| {
                            this.rank_agents = agents.clone();
                            this.rank_queue = queue;
                            this.rank_season = season;
                            if this.rank_selected_agent_id.is_empty() {
                                if let Some(a) = agents.first() {
                                    this.rank_selected_agent_id = a.id.to_string();
                                }
                            }
                            cx.notify();
                            this.rank_selected_agent_id.clone()
                        })
                        .unwrap_or_default();
                    // 自动加载首个 Agent 的参赛快照
                    if !selected.is_empty() {
                        let snapshots = cloud.list_snapshots(&selected).await.unwrap_or_default();
                        weak.update(&mut cx, |this, cx| {
                            if this.rank_selected_agent_id == selected {
                                this.rank_snapshots = snapshots.clone();
                                this.rank_selected_snapshot_id = snapshots
                                    .first()
                                    .map(|s| s.id.to_string())
                                    .unwrap_or_default();
                                cx.notify();
                            }
                        })
                        .ok();
                    }
                }
            },
        )
        .detach();
    }

    let season_text = sidebar
        .rank_season
        .as_ref()
        .map(|s| s.starts_at[..10.min(s.starts_at.len())].to_string())
        .unwrap_or_else(|| "-".into());

    let error_msg = sidebar.rank_error.clone();
    let has_error = !error_msg.is_empty();
    let can_enqueue = !sidebar.rank_selected_agent_id.is_empty()
        && !sidebar.rank_selected_snapshot_id.is_empty()
        && !sidebar.rank_enqueueing;

    v_flex()
        .size_full()
        .flex_1()
        .gap_4()
        .overflow_hidden()
        .child(
            h_flex()
                .items_center()
                .justify_between()
                .child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .child(IconName::ChartPie)
                        .child(div().font_bold().text_lg().child(t!("app.nav.title_rank"))),
                )
                .child(
                    h_flex()
                        .gap_3()
                        .items_center()
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child(t!("app.rank.desc")),
                        )
                        .child(
                            Button::new("rank-view-leaderboard")
                                .outline()
                                .icon(IconName::Star)
                                .label("查看排行榜")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.navigate_to(ActiveView::Leaderboard);
                                    cx.notify();
                                })),
                        ),
                ),
        )
        .child(
            div().flex_1().w_full().overflow_y_scrollbar().child(
                v_flex()
                    .gap_4()
                    .child(
                        v_flex()
                            .gap_3()
                            .p_4()
                            .border_1()
                            .border_color(cx.theme().border)
                            .rounded_lg()
                            .child(
                                div()
                                    .font_bold()
                                    .text_sm()
                                    .child(t!("app.rank.enroll_title")),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(t!("app.rank.enroll_desc")),
                            )
                            .child(div().h_0p5().bg(cx.theme().border))
                            .child(render_mode_selector(sidebar, cx))
                            .child(render_agent_selector(sidebar, cx))
                            .child(render_snapshot_selector(sidebar, cx))
                            .child(
                                h_flex()
                                    .items_center()
                                    .justify_between()
                                    .pt_2()
                                    .child(if has_error {
                                        div()
                                            .text_xs()
                                            .text_color(cx.theme().danger)
                                            .child(error_msg)
                                            .into_any_element()
                                    } else {
                                        div()
                                            .text_xs()
                                            .text_color(cx.theme().muted_foreground)
                                            .child(format!(
                                                "{}: {}",
                                                t!("app.rank.season_label").as_ref(),
                                                season_text
                                            ))
                                            .into_any_element()
                                    })
                                    .child(if can_enqueue {
                                        let label: gpui::SharedString = if sidebar.rank_enqueueing {
                                            t!("app.rank.enqueueing").into()
                                        } else {
                                            t!("app.rank.enqueue_btn").into()
                                        };
                                        Button::new("rank-enqueue")
                                            .primary()
                                            .label(label)
                                            .on_click(cx.listener(move |this, _, _, cx| {
                                                if this.rank_selected_agent_id.is_empty()
                                                    || this.rank_selected_snapshot_id.is_empty()
                                                {
                                                    this.rank_error =
                                                        t!("app.rank.error_select").into();
                                                    cx.notify();
                                                    return;
                                                }
                                                this.rank_error.clear();
                                                this.rank_enqueueing = true;
                                                cx.notify();
                                                let cloud = provider::cloud_client().clone();
                                                let agent_id = this.rank_selected_agent_id.clone();
                                                let snapshot_id =
                                                    this.rank_selected_snapshot_id.clone();
                                                let mode = this.rank_mode.clone();
                                                cx.spawn(
                                                    move |weak: gpui::WeakEntity<AppSidebar>,
                                                     cx: &mut gpui::AsyncApp| {
                                                        let weak = weak.clone();
                                                        let mut cx = cx.clone();
                                                        async move {
                                                            match cloud
                                                                .enqueue_rank(
                                                                    &agent_id,
                                                                    &snapshot_id,
                                                                    &mode,
                                                                )
                                                                .await
                                                            {
                                                                Ok(_) => {
                                                                    let queue = cloud
                                                                        .get_rank_status()
                                                                        .await
                                                                        .unwrap_or_default();
                                                                    weak.update(
                                                                        &mut cx,
                                                                        |this, cx| {
                                                                            this.rank_enqueueing =
                                                                                false;
                                                                            this.rank_error.clear();
                                                                            this.rank_queue = queue;
                                                                            cx.notify();
                                                                        },
                                                                    )
                                                                    .ok();
                                                                }
                                                                Err(e) => {
                                                                    weak.update(
                                                                        &mut cx,
                                                                        |this, cx| {
                                                                            this.rank_enqueueing =
                                                                                false;
                                                                            this.rank_error =
                                                                                e.to_string();
                                                                            cx.notify();
                                                                        },
                                                                    )
                                                                    .ok();
                                                                }
                                                            }
                                                        }
                                                    },
                                                )
                                                .detach();
                                            }))
                                            .into_any_element()
                                    } else {
                                        let label: gpui::SharedString = if sidebar.rank_enqueueing {
                                            t!("app.rank.enqueueing").into()
                                        } else {
                                            t!("app.rank.enqueue_btn").into()
                                        };
                                        Button::new("rank-enqueue")
                                            .primary()
                                            .label(label)
                                            .into_any_element()
                                    }),
                            ),
                    )
                    .child(render_queue_section(sidebar, cx)),
            ),
        )
        .into_any_element()
}
