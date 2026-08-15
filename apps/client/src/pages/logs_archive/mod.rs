//! 日志工具：
//! 1) 日志归档（在线）：我的对局（24h）列表 + 下载 SQLite DB + 按 game_id 查询日志；
//! 2) 日志浏览（离线）：加载本地 .sqlite 校验大小（本地回放浏览后续提供）。

mod input;
mod logic;
mod types;

use gpui::prelude::*;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::scroll::ScrollableElement;
use gpui_component::{h_flex, v_flex, ActiveTheme, Disableable, IconName, StyledExt};
use lol_web_protocol::match_::Match;
use rust_i18n::t;
pub use types::LogsArchiveState;

use self::input::render_text_input;
use self::logic::{
    do_load_logs, do_query, download_match_db, fmt_date, load_local_sqlite, load_matches,
    status_label,
};
use crate::components::sidebar::AppSidebar;
use crate::services::log_service;
use crate::services::types::{LogCategory, LogEntity, LogRow};

/// 日志归档（在线）：我的对局（24h）列表 + 下载 SQLite DB + 按 game_id 查询日志。
pub fn render_logs_archive(
    sidebar: &mut AppSidebar,
    window: &mut Window,
    cx: &mut Context<AppSidebar>,
) -> AnyElement {
    // 首次渲染触发拉取对局列表
    if !sidebar.logs_archive.matches_loaded && !sidebar.logs_archive.matches_loading {
        load_matches(sidebar, cx);
    }

    // Read individual fields without cloning the whole state
    let game_id = sidebar.logs_archive.game_id.clone();
    let game_id_empty = game_id.is_empty();
    let levels = sidebar.logs_archive.levels.clone();
    let entities = sidebar.logs_archive.entities.len();
    let entities_empty = entities == 0;
    let categories = sidebar.logs_archive.categories.len();
    let categories_empty = categories == 0;
    let loading = sidebar.logs_archive.loading;
    let error = sidebar.logs_archive.error.clone();
    let has_results = sidebar.logs_archive.results.is_some();
    let total_count = sidebar
        .logs_archive
        .results
        .as_ref()
        .map_or(0, |r| r.total_count);
    let total_pages = {
        let total = sidebar
            .logs_archive
            .results
            .as_ref()
            .map_or(0, |r| r.total_count);
        if sidebar.logs_archive.limit > 0 {
            ((total as f64) / (sidebar.logs_archive.limit as f64)).ceil() as i64
        } else {
            0
        }
    };
    let current_page = {
        if sidebar.logs_archive.limit > 0 {
            sidebar.logs_archive.offset / sidebar.logs_archive.limit + 1
        } else {
            1
        }
    };

    let matches = sidebar.logs_archive.matches.clone();
    let matches_loading = sidebar.logs_archive.matches_loading;
    let matches_error = sidebar.logs_archive.matches_error.clone();
    let downloading = sidebar.logs_archive.downloading.clone();
    let download_msg = sidebar.logs_archive.download_msg.clone();

    let entity_list: Vec<LogEntity> = sidebar
        .logs_archive
        .entities
        .iter()
        .map(|e| LogEntity {
            entity_id: e.entity_id,
            entity_name: e.entity_name.clone(),
        })
        .collect();
    let category_list: Vec<LogCategory> = sidebar
        .logs_archive
        .categories
        .iter()
        .map(|c| LogCategory {
            category: c.category.clone(),
        })
        .collect();
    let rows_data: Vec<LogRow> = sidebar
        .logs_archive
        .results
        .as_ref()
        .map_or(Vec::new(), |r| {
            r.rows
                .iter()
                .map(|row| LogRow {
                    id: row.id,
                    timestamp: row.timestamp,
                    level: row.level.clone(),
                    file: row.file.clone(),
                    line: row.line,
                    entity_id: row.entity_id,
                    entity_name: row.entity_name.clone(),
                    category: row.category.clone(),
                    message: row.message.clone(),
                })
                .collect()
        });

    div()
        .size_full()
        .flex_1()
        .overflow_hidden()
        .child(
            div()
                .size_full()
                .flex()
                .flex_col()
                .gap_4()
                .overflow_y_scrollbar()
                .p_6()
                // ── 标题行 ──
                .child(
                    v_flex()
                        .gap_1()
                        .child(
                            h_flex()
                                .gap_2()
                                .items_center()
                                .child(IconName::HardDrive)
                                .child(
                                    div()
                                        .font_bold()
                                        .text_lg()
                                        .child(t!("app.nav.title_logs_archive")),
                                ),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child(
                                "服务器保留最近 24 小时对局日志，可下载为 SQLite 库离线分析，或按级别/实体/类别筛选查询。",
                            ),
                        ),
                )
                // ── 错误提示 ──
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
                // ── 我的对局（24h）──
                .child(
                    v_flex()
                        .gap_3()
                        .child(
                            h_flex()
                                .items_center()
                                .justify_between()
                                .child(
                                    h_flex()
                                        .gap_1p5()
                                        .items_center()
                                        .child(IconName::Inbox)
                                        .child(div().text_sm().font_bold().child("我的对局（24h）"))
                                        .child(
                                            div()
                                                .px_2()
                                                .py_0p5()
                                                .rounded_md()
                                                .text_xs()
                                                .font_bold()
                                                .bg(cx.theme().accent.opacity(0.15))
                                                .text_color(cx.theme().accent)
                                                .child(format!("{}", matches.len())),
                                        ),
                                )
                                .child(
                                    Button::new("logs-matches-refresh")
                                        .ghost()
                                        .icon(IconName::Redo2)
                                        .label("刷新")
                                        .when(matches_loading, |b| b.disabled(true))
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            load_matches(this, cx)
                                        })),
                                ),
                        )
                        .when_some(matches_error.as_ref(), |d, err| {
                            d.child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().danger)
                                    .child(err.clone()),
                            )
                        })
                        .when(matches_loading, |d| {
                            d.child(
                                div()
                                    .py_6()
                                    .w_full()
                                    .text_center()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("加载对局列表…"),
                            )
                        })
                        .when(!matches_loading && matches.is_empty(), |d| {
                            d.child(
                                div()
                                    .py_6()
                                    .w_full()
                                    .text_center()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("近 24 小时没有对局记录"),
                            )
                        })
                        .when(!matches.is_empty(), |d| {
                            d.child(
                                div()
                                    .rounded_lg()
                                    .border_1()
                                    .border_color(cx.theme().border)
                                    .overflow_hidden()
                                    .child(
                                        v_flex()
                                            .child(
                                                h_flex()
                                                    .px_4()
                                                    .py_2()
                                                    .border_b_1()
                                                    .border_color(cx.theme().border)
                                                    .bg(cx.theme().background)
                                                    .child(
                                                        div()
                                                            .w(rems(9.))
                                                            .text_xs()
                                                            .font_bold()
                                                            .child("对局"),
                                                    )
                                                    .child(
                                                        div()
                                                            .w(rems(7.))
                                                            .text_xs()
                                                            .font_bold()
                                                            .child("模式"),
                                                    )
                                                    .child(
                                                        div()
                                                            .w(rems(6.))
                                                            .text_xs()
                                                            .font_bold()
                                                            .child("状态"),
                                                    )
                                                    .child(
                                                        div()
                                                            .w(rems(14.))
                                                            .text_xs()
                                                            .font_bold()
                                                            .child("开始"),
                                                    )
                                                    .child(
                                                        div()
                                                            .flex_1()
                                                            .text_xs()
                                                            .font_bold()
                                                            .text_right()
                                                            .child("操作"),
                                                    ),
                                            )
                                            .children(
                                                matches.iter().map(|m| {
                                                    render_match_row(m, &downloading, cx)
                                                }),
                                            ),
                                    ),
                            )
                        })
                        .when_some(download_msg.as_ref(), |d, msg| {
                            d.child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().accent)
                                    .child(msg.clone()),
                            )
                        }),
                )
                // ── Game ID 输入与操作行 ──
                .child(
                    h_flex()
                        .gap_3()
                        .items_center()
                        .flex_wrap()
                        .child(
                            h_flex()
                                .gap_1p5()
                                .items_center()
                                .child(div().text_xs().font_bold().child("对局 ID"))
                                .child(
                                    div().w(rems(24.)).child(render_text_input(
                                        window,
                                        cx,
                                        &*sidebar,
                                        "logs-game-id",
                                        "输入对局 ID（或从上方列表点击「查询」）",
                                        |s: &AppSidebar| s.logs_archive.game_id.clone(),
                                        |s: &mut AppSidebar, v: String| {
                                            s.logs_archive.game_id = v
                                        },
                                    )),
                                ),
                        )
                        .child({
                            let gid = game_id.clone();
                            Button::new("logs-load-btn")
                                .icon(IconName::Search)
                                .label("加载日志")
                                .when(gid.is_empty(), |b| b.disabled(true))
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    if gid.is_empty() {
                                        return;
                                    }
                                    this.logs_archive.loading = true;
                                    this.logs_archive.error = None;
                                    this.logs_archive.results = None;
                                    this.logs_archive.entities.clear();
                                    this.logs_archive.categories.clear();
                                    do_load_logs(this, cx);
                                }))
                        }),
                )
                // ── 筛选控制行 ──
                .child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .flex_wrap()
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child("级别:"),
                        )
                        .children(["DEBUG", "INFO", "WARN", "ERROR"].iter().map(|lv| {
                            let lvl = lv.to_string();
                            let selected = levels.contains(&lvl);
                            let btn = Button::new(format!("logs-level-{}", lvl)).label(lvl.clone());
                            if selected {
                                let lvl2 = lvl.clone();
                                btn.on_click(cx.listener(move |this, _, _, cx| {
                                    this.logs_archive.levels.retain(|l| l != &lvl2);
                                    cx.notify();
                                }))
                                .into_any_element()
                            } else {
                                let lvl2 = lvl.clone();
                                btn.ghost()
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        if !this.logs_archive.levels.contains(&lvl2) {
                                            this.logs_archive.levels.push(lvl2.clone());
                                        }
                                        cx.notify();
                                    }))
                                    .into_any_element()
                            }
                        })),
                )
                // Entity / Category 过滤器
                .when(!entities_empty || !categories_empty, |d| {
                    d.child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .flex_wrap()
                            .when(!entities_empty, |d2| {
                                d2.child(
                                    div()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground)
                                        .child("实体:"),
                                )
                                .children(entity_list.iter().map(|e| {
                                    let eid = e.entity_id;
                                    let label = e.entity_name.clone().unwrap_or_else(|| {
                                        format!("Entity {}", e.entity_id.unwrap_or(0))
                                    });
                                    let is_selected = sidebar.logs_archive.entity_id == eid;
                                    let btn =
                                        Button::new(format!("logs-entity-{}", eid.unwrap_or(0)))
                                            .label(label);
                                    if is_selected {
                                        btn.on_click(cx.listener(|this, _, _, cx| {
                                            this.logs_archive.entity_id = None;
                                            cx.notify();
                                        }))
                                        .into_any_element()
                                    } else {
                                        btn.ghost()
                                            .on_click(cx.listener(move |this, _, _, cx| {
                                                this.logs_archive.entity_id = eid;
                                                cx.notify();
                                            }))
                                            .into_any_element()
                                    }
                                }))
                            })
                            .when(!categories_empty, |d2| {
                                d2.child(
                                    div()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground)
                                        .child("类别:"),
                                )
                                .children(
                                    category_list.iter().map(|c| {
                                        let cat = c.category.clone().unwrap_or_default();
                                        let is_selected = sidebar
                                            .logs_archive
                                            .category
                                            .as_ref()
                                            .map_or(false, |sc| sc == &cat);
                                        let btn = Button::new(format!("logs-cat-{}", cat))
                                            .label(cat.clone());
                                        if is_selected {
                                            btn.on_click(cx.listener(|this, _, _, cx| {
                                                this.logs_archive.category = None;
                                                cx.notify();
                                            }))
                                            .into_any_element()
                                        } else {
                                            let cat2 = cat.clone();
                                            btn.ghost()
                                                .on_click(cx.listener(move |this, _, _, cx| {
                                                    this.logs_archive.category = Some(cat2.clone());
                                                    cx.notify();
                                                }))
                                                .into_any_element()
                                        }
                                    }),
                                )
                            }),
                    )
                })
                // ── 操作按钮行 ──
                .child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .justify_between()
                        .child(
                            h_flex()
                                .gap_2()
                                .child(
                                    Button::new("logs-search-btn")
                                        .icon(IconName::Search)
                                        .label("应用筛选")
                                        .when(game_id_empty, |b| b.disabled(true))
                                        .on_click(cx.listener(move |this, _, _, cx| {
                                            this.logs_archive.offset = 0;
                                            this.logs_archive.loading = true;
                                            this.logs_archive.error = None;
                                            do_query(this, cx);
                                        })),
                                )
                                .child(
                                    Button::new("logs-clear-btn")
                                        .ghost()
                                        .icon(IconName::Delete)
                                        .label("清空日志")
                                        .when(game_id_empty, |b| b.disabled(true))
                                        .on_click(cx.listener(move |this, _, _, cx| {
                                            let gid2 = this.logs_archive.game_id.clone();
                                            if gid2.is_empty() {
                                                return;
                                            }
                                            let gid3 = gid2.clone();
                                            cx.spawn(
                                                move |weak: gpui::WeakEntity<AppSidebar>,
                                                 cx: &mut gpui::AsyncApp| {
                                                    let weak = weak.clone();
                                                    let mut cx = cx.clone();
                                                    let gid4 = gid3.clone();
                                                    async move {
                                                        let result =
                                                            log_service::clear_logs(&gid4).await;
                                                        weak.update(&mut cx, |this, cx| {
                                                            match result {
                                                                Ok(()) => {
                                                                    this.logs_archive.results = None;
                                                                    this
                                                                        .logs_archive
                                                                        .entities
                                                                        .clear();
                                                                    this
                                                                        .logs_archive
                                                                        .categories
                                                                        .clear();
                                                                }
                                                                Err(e) => {
                                                                    this.logs_archive.error =
                                                                        Some(e)
                                                                }
                                                            }
                                                            cx.notify();
                                                        })
                                                        .ok();
                                                    }
                                                },
                                            )
                                            .detach();
                                        })),
                                ),
                        )
                        .when(has_results, |d| {
                            d.child(
                                div()
                                    .px_2()
                                    .py_0p5()
                                    .rounded_md()
                                    .text_xs()
                                    .font_bold()
                                    .bg(cx.theme().accent.opacity(0.15))
                                    .text_color(cx.theme().accent)
                                    .child(format!("{} 条", total_count)),
                            )
                        }),
                )
                // ── Loading ──
                .when(loading, |d| {
                    d.child(
                        div()
                            .py_12()
                            .w_full()
                            .text_center()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child("加载中…"),
                    )
                })
                // ── 日志表格 ──
                .when(has_results, |d| {
                    d.child(
                        div()
                            .rounded_lg()
                            .border_1()
                            .border_color(cx.theme().border)
                            .overflow_hidden()
                            .flex_1()
                            .child(
                                v_flex()
                                    .size_full()
                                    .child(
                                        h_flex()
                                            .px_4()
                                            .py_2()
                                            .border_b_1()
                                            .border_color(cx.theme().border)
                                            .bg(cx.theme().background)
                                            .child(
                                                div().w(rems(4.)).text_xs().font_bold().child("ID"),
                                            )
                                            .child(
                                                div()
                                                    .w(rems(5.))
                                                    .text_xs()
                                                    .font_bold()
                                                    .child("级别"),
                                            )
                                            .child(
                                                div()
                                                    .w(rems(10.))
                                                    .text_xs()
                                                    .font_bold()
                                                    .child("时间"),
                                            )
                                            .child(
                                                div()
                                                    .w(rems(8.))
                                                    .text_xs()
                                                    .font_bold()
                                                    .child("实体"),
                                            )
                                            .child(
                                                div()
                                                    .w(rems(6.))
                                                    .text_xs()
                                                    .font_bold()
                                                    .child("类别"),
                                            )
                                            .child(
                                                div().flex_1().text_xs().font_bold().child("消息"),
                                            ),
                                    )
                                    .child(div().flex_1().overflow_y_scrollbar().children(
                                        rows_data.iter().map(|row| render_log_row(cx, row)),
                                    )),
                            ),
                    )
                })
                // ── 分页 ──
                .when(total_pages > 1, |d| {
                    d.child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .justify_center()
                            .child(
                                Button::new("logs-prev-page")
                                    .ghost()
                                    .label("上一页")
                                    .when(current_page <= 1, |b| b.disabled(true))
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.logs_archive.offset = (this.logs_archive.offset
                                            - this.logs_archive.limit)
                                            .max(0);
                                        this.logs_archive.loading = true;
                                        do_query(this, cx);
                                    })),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(format!("第 {} / {} 页", current_page, total_pages)),
                            )
                            .child(
                                Button::new("logs-next-page")
                                    .ghost()
                                    .label("下一页")
                                    .when(current_page >= total_pages, |b| b.disabled(true))
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.logs_archive.offset += this.logs_archive.limit;
                                        this.logs_archive.loading = true;
                                        do_query(this, cx);
                                    })),
                            ),
                    )
                })
                // ── 未加载占位 ──
                .when(!loading && !has_results && error.is_none(), |d| {
                    d.child(
                        div()
                            .flex_1()
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child("需要先选择一个对局 ID，点击「加载日志」开始查询"),
                    )
                })
                .into_any_element(),
        )
        .into_any_element()
}

/// 「我的对局」单行：id / 模式 / 状态 / 开始时间 / 操作。
fn render_match_row(
    m: &Match,
    downloading: &Option<String>,
    cx: &mut Context<AppSidebar>,
) -> AnyElement {
    let mid = m.id.to_string();
    let short: String = mid.chars().take(8).collect();
    let mode = m.mode.clone();
    let status = status_label(&m.status).to_string();
    let created = fmt_date(&m.created_at);
    let is_downloading = downloading.as_ref() == Some(&mid);
    let border = cx.theme().border;
    let muted = cx.theme().muted_foreground;
    let accent = cx.theme().accent;
    h_flex()
        .px_4()
        .py_2()
        .border_b_1()
        .border_color(border.opacity(0.3))
        .items_center()
        .child(div().w(rems(9.)).text_xs().child(short))
        .child(
            div().w(rems(7.)).child(
                div()
                    .px_1p5()
                    .py_0p5()
                    .rounded_md()
                    .bg(accent.opacity(0.15))
                    .text_color(accent)
                    .text_xs()
                    .font_bold()
                    .child(mode),
            ),
        )
        .child(div().w(rems(6.)).text_xs().child(status))
        .child(
            div()
                .w(rems(14.))
                .text_xs()
                .text_color(muted)
                .child(created),
        )
        .child(
            h_flex()
                .flex_1()
                .gap_1p5()
                .justify_end()
                .items_center()
                .child({
                    let mid2 = mid.clone();
                    Button::new(format!("logs-use-{}", mid))
                        .ghost()
                        .label("查询")
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.logs_archive.game_id = mid2.clone();
                            cx.notify();
                        }))
                })
                .child({
                    let mid2 = mid.clone();
                    Button::new(format!("logs-dl-{}", mid))
                        .outline()
                        .icon(IconName::ArrowDown)
                        .label(if is_downloading {
                            "下载中…".to_string()
                        } else {
                            "下载 DB".to_string()
                        })
                        .when(is_downloading, |b| b.disabled(true))
                        .on_click(cx.listener(move |this, _, _, cx| {
                            download_match_db(this, cx, &mid2);
                        }))
                }),
        )
        .into_any_element()
}

fn render_log_row(cx: &mut Context<AppSidebar>, row: &LogRow) -> AnyElement {
    let level = row.level.clone();
    let level_hsla = match level.as_str() {
        "DEBUG" => cx.theme().muted_foreground,
        "INFO" => cx.theme().accent,
        "WARN" => hsla(40.0 / 360.0, 0.9, 0.5, 1.0),
        "ERROR" => cx.theme().danger,
        _ => cx.theme().muted_foreground,
    };
    h_flex()
        .px_4()
        .py_1p5()
        .border_b_1()
        .border_color(cx.theme().border.opacity(0.3))
        .child(
            div()
                .w(rems(4.))
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child(format!("{}", row.id)),
        )
        .child(
            div().w(rems(5.)).child(
                div()
                    .px_1p5()
                    .py_0p5()
                    .rounded_md()
                    .text_xs()
                    .font_bold()
                    .bg(level_hsla.opacity(0.15))
                    .text_color(level_hsla)
                    .child(level),
            ),
        )
        .child(
            div()
                .w(rems(10.))
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child(format!("{}s", row.timestamp)),
        )
        .child(
            div()
                .w(rems(8.))
                .text_xs()
                .child(row.entity_name.clone().unwrap_or_else(|| {
                    row.entity_id
                        .map(|id| format!("#{}", id))
                        .unwrap_or_else(|| "—".to_string())
                })),
        )
        .child(
            div()
                .w(rems(6.))
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child(row.category.clone().unwrap_or_else(|| "—".to_string())),
        )
        .child(div().flex_1().text_xs().child(row.message.clone()))
        .into_any_element()
}

// ── 日志浏览（离线）──

/// 日志浏览（离线）：加载本地 .sqlite 校验大小，本地回放浏览后续提供。
pub fn render_logs_browser(
    sidebar: &mut AppSidebar,
    window: &mut Window,
    cx: &mut Context<AppSidebar>,
) -> AnyElement {
    let local_size = sidebar.logs_archive.local_size;
    let local_msg = sidebar.logs_archive.local_msg.clone();

    div()
        .size_full()
        .flex_1()
        .overflow_hidden()
        .child(
            div()
                .size_full()
                .flex()
                .flex_col()
                .gap_4()
                .overflow_y_scrollbar()
                .p_6()
                .child(
                    v_flex()
                        .gap_1()
                        .child(
                            h_flex()
                                .gap_2()
                                .items_center()
                                .child(IconName::HardDrive)
                                .child(
                                    div()
                                        .font_bold()
                                        .text_lg()
                                        .child(t!("app.nav.title_logs_browser")),
                                ),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child(
                                "离线浏览本地 SQLite 日志库（从「日志归档」下载的 .sqlite 文件），无需连接服务器。",
                            ),
                        ),
                )
                // ── 加载本地 SQLite ──
                .child(
                    div()
                        .rounded_lg()
                        .border_1()
                        .border_color(cx.theme().border)
                        .p_4()
                        .child(
                            v_flex()
                                .gap_3()
                                .child(
                                    h_flex()
                                        .items_center()
                                        .justify_between()
                                        .flex_wrap()
                                        .child(
                                            v_flex()
                                                .gap_1()
                                                .child(
                                                    h_flex()
                                                        .gap_1p5()
                                                        .items_center()
                                                        .child(IconName::File)
                                                        .child(
                                                            div()
                                                                .text_sm()
                                                                .font_bold()
                                                                .child("加载本地 SQLite"),
                                                        ),
                                                )
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(cx.theme().muted_foreground)
                                                        .child(
                                                            if let Some(sz) = local_size {
                                                                format!(
                                                                    "已加载 {} bytes（{:.1} MB）",
                                                                    sz,
                                                                    sz as f64
                                                                        / 1024.0
                                                                        / 1024.0
                                                                )
                                                            } else {
                                                                "输入之前下载的 .sqlite 文件路径，校验文件存在并显示大小（本地回放浏览将在后续版本提供）".to_string()
                                                            },
                                                        ),
                                                ),
                                        )
                                        .child(
                                            h_flex()
                                                .gap_2()
                                                .items_center()
                                                .child(
                                                    div().w(rems(24.)).child(render_text_input(
                                                        window,
                                                        cx,
                                                        &*sidebar,
                                                        "logs-local-path",
                                                        "输入 .sqlite 文件路径…",
                                                        |s: &AppSidebar| {
                                                            s.logs_archive.local_path.clone()
                                                        },
                                                        |s: &mut AppSidebar, v: String| {
                                                            s.logs_archive.local_path = v;
                                                            s.logs_archive.local_size = None;
                                                            s.logs_archive.local_msg = None;
                                                        },
                                                    )),
                                                )
                                                .child(
                                                    Button::new("logs-local-load")
                                                        .icon(IconName::Play)
                                                        .label("加载")
                                                        .on_click(cx.listener(|this, _, _, cx| {
                                                            load_local_sqlite(this, cx);
                                                        })),
                                                )
                                                .child(
                                                    Button::new("logs-local-clear")
                                                        .ghost()
                                                        .label("清除")
                                                        .on_click(cx.listener(|this, _, _, cx| {
                                                            this.logs_archive.local_path.clear();
                                                            this.logs_archive.local_size = None;
                                                            this.logs_archive.local_msg = None;
                                                            cx.notify();
                                                        })),
                                                ),
                                        ),
                                )
                                .when_some(local_msg.as_ref(), |d, msg| {
                                    d.child(
                                        div()
                                            .text_xs()
                                            .text_color(cx.theme().muted_foreground)
                                            .child(msg.clone()),
                                    )
                                }),
                        ),
                ),
        )
        .into_any_element()
}
