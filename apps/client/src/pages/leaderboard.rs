use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::scroll::ScrollableElement;
use gpui_component::table::{Table, TableBody, TableCell, TableHead, TableHeader, TableRow};
use gpui_component::{h_flex, v_flex, ActiveTheme, IconName, StyledExt};
use rust_i18n::t;

use crate::components::sidebar::AppSidebar;
use crate::services::provider;

// ── 子视图 ──

fn render_view_tabs(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) -> AnyElement {
    let is_total = sidebar.leaderboard_view == "total";
    h_flex()
        .gap_1()
        .child({
            let btn = Button::new("lb-tab-total").label(t!("app.leaderboard.tab_total"));
            if is_total {
                btn.primary().into_any_element()
            } else {
                btn.outline()
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.leaderboard_view = "total".into();
                        cx.notify();
                    }))
                    .into_any_element()
            }
        })
        .child({
            let btn = Button::new("lb-tab-daily").label(t!("app.leaderboard.tab_daily"));
            if !is_total {
                btn.primary().into_any_element()
            } else {
                btn.outline()
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.leaderboard_view = "daily".into();
                        cx.notify();
                    }))
                    .into_any_element()
            }
        })
        .into_any_element()
}

fn render_mode_select(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) -> AnyElement {
    h_flex()
        .gap_1()
        .children([("top_solo", "上单 SOLO")].map(|(val, label)| {
            let selected = sidebar.leaderboard_mode == val;
            let btn = Button::new(format!("lb-mode-{}", val)).label(label.to_string());
            if selected {
                btn.primary().into_any_element()
            } else {
                let v = val.to_string();
                btn.outline()
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.leaderboard_mode = v.clone();
                        this.leaderboard_loaded = false;
                        this.leaderboard_loading = false;
                        cx.notify();
                    }))
                    .into_any_element()
            }
        }))
        .into_any_element()
}

fn render_table(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) -> AnyElement {
    let is_daily = sidebar.leaderboard_view == "daily";
    let mut data = sidebar.leaderboard_data.clone();
    if is_daily {
        data.sort_by(|a, b| {
            b.daily_delta
                .partial_cmp(&a.daily_delta)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    } else {
        data.sort_by(|a, b| {
            b.rating
                .partial_cmp(&a.rating)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    if data.is_empty() {
        return div()
            .text_sm()
            .text_color(cx.theme().muted_foreground)
            .py_12()
            .text_center()
            .child(t!("app.leaderboard.empty"))
            .into_any_element();
    }

    let mut rows = Vec::new();
    for (idx, r) in data.iter().enumerate() {
        let rank: AnyElement = if idx == 0 {
            div()
                .child(IconName::Star)
                .text_color(gpui::hsla(44.0 / 360.0, 1.0, 0.5, 1.0))
                .into_any_element()
        } else if idx == 1 {
            div()
                .child(IconName::Star)
                .text_color(gpui::hsla(0.0, 0.0, 0.6, 1.0))
                .into_any_element()
        } else if idx == 2 {
            div()
                .child(IconName::Star)
                .text_color(gpui::hsla(30.0 / 360.0, 0.8, 0.4, 1.0))
                .into_any_element()
        } else {
            div()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child(format!("{}", idx + 1))
                .into_any_element()
        };

        let val: AnyElement = if is_daily {
            let d = r.daily_delta;
            let sign = if d >= 0.0 { "+" } else { "" };
            let c = if d >= 0.0 {
                gpui::hsla(160.0 / 360.0, 0.6, 0.4, 1.0)
            } else {
                cx.theme().danger
            };
            div()
                .text_color(c)
                .child(format!("{}{}", sign, d))
                .into_any_element()
        } else {
            div().child(format!("{}", r.rating)).into_any_element()
        };

        let wr = if r.games_played > 0 {
            format!("{:.1}%", (r.wins as f64 / r.games_played as f64) * 100.0)
        } else {
            "-".into()
        };

        rows.push(
            TableRow::new().children([
                TableCell::new().child(rank),
                TableCell::new().child(
                    h_flex()
                        .gap_1()
                        .items_center()
                        .child(div().text_sm().font_bold().child(r.agent_name.clone()))
                        .child(
                            div()
                                .text_xs()
                                .px_1()
                                .py_0p5()
                                .bg(cx.theme().secondary)
                                .rounded_sm()
                                .child(r.mode.clone()),
                        ),
                ),
                TableCell::new().child(val),
                TableCell::new().child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child(format!("{} / {}", r.wins, r.losses)),
                ),
                TableCell::new().child(div().text_sm().child(wr)),
            ]),
        );
    }

    Table::new()
        .child(TableHeader::new().child(TableRow::new().children([
            TableHead::new().child(div().w_8().child("#")),
            TableHead::new().child(t!("app.leaderboard.col_agent")),
            TableHead::new().child(if is_daily {
                t!("app.leaderboard.col_daily_delta")
            } else {
                t!("app.leaderboard.col_elo")
            }),
            TableHead::new().child(t!("app.leaderboard.col_record")),
            TableHead::new().child(t!("app.leaderboard.col_winrate")),
        ])))
        .child(TableBody::new().children(rows))
        .into_any_element()
}

pub fn render_leaderboard(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) -> AnyElement {
    // 首次渲染或 mode 切换后拉取数据（与 client 一致：mode 传给 API，view 仅本地排序切换）
    if !sidebar.leaderboard_loaded && !sidebar.leaderboard_loading {
        let client = provider::cloud_client().clone();
        let mode = sidebar.leaderboard_mode.clone();
        sidebar.leaderboard_loading = true;
        cx.spawn(
            move |weak: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
                let weak = weak.clone();
                let mut cx = cx.clone();
                let client = client.clone();
                async move {
                    let data = client.get_leaderboard(&mode, 100).await;
                    weak.update(&mut cx, |this, cx| {
                        this.leaderboard_loading = false;
                        // mode 已切换时丢弃过期响应，避免覆盖新 mode 数据
                        if this.leaderboard_mode == mode {
                            this.leaderboard_data = data.unwrap_or_default();
                            this.leaderboard_loaded = true;
                            cx.notify();
                        }
                    })
                    .ok();
                }
            },
        )
        .detach();
    }

    let table_content = if sidebar.leaderboard_loaded {
        render_table(sidebar, cx)
    } else {
        div()
            .text_sm()
            .text_color(cx.theme().muted_foreground)
            .py_12()
            .text_center()
            .child("加载中…")
            .into_any_element()
    };

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
                        .child(
                            div()
                                .font_bold()
                                .text_lg()
                                .child(t!("app.nav.title_leaderboard")),
                        ),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child(t!("app.leaderboard.desc")),
                ),
        )
        .child(
            h_flex()
                .items_center()
                .justify_between()
                .child(render_view_tabs(sidebar, cx))
                .child(render_mode_select(sidebar, cx)),
        )
        .child(
            div()
                .flex_1()
                .w_full()
                .overflow_y_scrollbar()
                .child(table_content),
        )
        .into_any_element()
}
