use std::collections::HashMap;

use gpui::prelude::*;
use gpui::*;
use gpui_component::button::Button;
use gpui_component::scroll::ScrollableElement;
use gpui_component::{h_flex, v_flex, ActiveTheme, Disableable, IconName, Sizable, StyledExt};
use lol_web_protocol::RunningGame;

use crate::components::sidebar::AppSidebar;
use crate::services::provider;
use crate::services::runtime::run_on_tokio;
use crate::types::{ActiveView, RunningGameInfo};

// ── 页面本地状态 ──

pub struct GamesPageState {
    /// 是否已触发首次自动加载
    inited: bool,
    /// 是否已启动 5s 轮询（防重复 spawn）
    polling: bool,
    /// 加载中
    loading: bool,
    /// 加载错误
    error: Option<String>,
    /// 运行中对局列表（来源 `process_service().list()`）
    games: Vec<RunningGame>,
    /// 正在停止的对局 id
    stopping: Option<String>,
}

impl Default for GamesPageState {
    fn default() -> Self {
        Self {
            inited: false,
            polling: false,
            loading: false,
            error: None,
            games: Vec::new(),
            stopping: None,
        }
    }
}

// ── 数据加载 ──

/// 把进程列表同步到 `sidebar.running_games`，保留已知的 champion/mode。
fn sync_sidebar_games(sidebar: &mut AppSidebar, games: Vec<RunningGame>) {
    let known: HashMap<String, RunningGameInfo> = sidebar
        .running_games
        .iter()
        .map(|g| (g.id.clone(), g.clone()))
        .collect();
    sidebar.running_games = games
        .into_iter()
        .map(|g| {
            let prev = known.get(&g.id);
            RunningGameInfo {
                id: g.id,
                mode: prev.map_or_else(String::new, |p| p.mode.clone()),
                champion: prev.map_or_else(String::new, |p| p.champion.clone()),
                port: g.port as u16,
            }
        })
        .collect();
}

/// 拉取运行中对局并写回页面状态与 `sidebar.running_games`。
async fn fetch_games(weak: &gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp) {
    match provider::process_service().list().await {
        Ok(games) => {
            weak.update(cx, |s, cx| {
                s.games.games = games.clone();
                s.games.loading = false;
                s.games.error = None;
                sync_sidebar_games(s, games);
                cx.notify();
            })
            .ok();
        }
        Err(err) => {
            weak.update(cx, |s, cx| {
                s.games.loading = false;
                s.games.error = Some(err);
                cx.notify();
            })
            .ok();
        }
    }
}

/// 异步加载（首次渲染与手动刷新共用）。
fn spawn_load(cx: &mut Context<AppSidebar>) {
    cx.spawn(
        move |weak: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
            let weak = weak.clone();
            let mut cx = cx.clone();
            async move {
                fetch_games(&weak, &mut cx).await;
            }
        },
    )
    .detach();
}

/// 5s 自动轮询运行中对局。
fn spawn_poll(cx: &mut Context<AppSidebar>) {
    cx.spawn(
        move |weak: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
            let weak = weak.clone();
            let mut cx = cx.clone();
            async move {
                loop {
                    // 通过 tokio runtime 桥接 sleep，避免在 gpui executor 直接调用 panic
                    if run_on_tokio(|| async {
                        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                        Ok::<(), String>(())
                    })
                    .await
                    .is_err()
                    {
                        break;
                    }
                    fetch_games(&weak, &mut cx).await;
                }
            }
        },
    )
    .detach();
}

// ── 表格 ──

/// 表格行：进程列表字段 + sidebar 补充的英雄/模式。
struct GameRow {
    id: String,
    port: i32,
    status: String,
    champion: String,
    mode: String,
}

fn short_id(id: &str) -> String {
    if id.len() > 8 {
        id[..8].to_string()
    } else {
        id.to_string()
    }
}

fn status_pill(status: &str, cx: &mut Context<AppSidebar>) -> AnyElement {
    h_flex()
        .gap_1p5()
        .items_center()
        .rounded_full()
        .px_2()
        .py_0p5()
        .text_xs()
        .font_bold()
        .bg(cx.theme().success.opacity(0.15))
        .text_color(cx.theme().success)
        .child(div().w_2().h_2().rounded_full().bg(cx.theme().success))
        .child(status.to_string())
        .into_any_element()
}

fn header_row(cx: &mut Context<AppSidebar>) -> AnyElement {
    h_flex()
        .px_4()
        .py_2()
        .border_b_1()
        .border_color(cx.theme().border)
        .bg(cx.theme().background)
        .child(
            div()
                .flex_1()
                .text_xs()
                .font_bold()
                .text_color(cx.theme().muted_foreground)
                .child("对局 ID"),
        )
        .child(
            div()
                .w(rems(7.))
                .text_xs()
                .font_bold()
                .text_color(cx.theme().muted_foreground)
                .child("通信端口"),
        )
        .child(
            div()
                .w(rems(7.))
                .text_xs()
                .font_bold()
                .text_color(cx.theme().muted_foreground)
                .child("状态"),
        )
        .child(
            div()
                .w(rems(8.))
                .text_xs()
                .font_bold()
                .text_color(cx.theme().muted_foreground)
                .child("英雄"),
        )
        .child(
            div()
                .w(rems(8.))
                .text_xs()
                .font_bold()
                .text_color(cx.theme().muted_foreground)
                .child("模式"),
        )
        .child(
            div()
                .w(rems(18.))
                .text_xs()
                .font_bold()
                .text_color(cx.theme().muted_foreground)
                .text_right()
                .child("操作"),
        )
        .into_any_element()
}

fn build_row(row: GameRow, stopping: &Option<String>, cx: &mut Context<AppSidebar>) -> AnyElement {
    let is_stopping = stopping.as_deref() == Some(row.id.as_str());
    let debug_id = format!("debug-{}", row.id);
    let stop_id = format!("stop-{}", row.id);
    let gid_debug = row.id.clone();
    let gid_stop = row.id.clone();

    h_flex()
        .px_4()
        .py_2()
        .border_b_1()
        .border_color(cx.theme().border.opacity(0.5))
        .child(
            div()
                .flex_1()
                .text_xs()
                .text_color(cx.theme().foreground)
                .child(short_id(&row.id)),
        )
        .child(
            div()
                .w(rems(7.))
                .text_xs()
                .text_color(cx.theme().accent)
                .child(row.port.to_string()),
        )
        .child(div().w(rems(7.)).child(status_pill(&row.status, cx)))
        .child(
            div()
                .w(rems(8.))
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child(row.champion),
        )
        .child(
            div()
                .w(rems(8.))
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child(row.mode),
        )
        .child(
            h_flex()
                .w(rems(18.))
                .gap_2()
                .justify_end()
                .child(
                    Button::new(debug_id)
                        .outline()
                        .icon(IconName::SquareTerminal)
                        .label("进入调试")
                        .small()
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.current_game_id = Some(gid_debug.clone());
                            this.navigate_to(ActiveView::Debug);
                            cx.notify();
                        })),
                )
                .child(
                    Button::new(stop_id)
                        .outline()
                        .icon(IconName::CircleX)
                        .label(if is_stopping {
                            "停止中…"
                        } else {
                            "停止"
                        })
                        .disabled(is_stopping)
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.games.stopping = Some(gid_stop.clone());
                            cx.notify();
                            let gid = gid_stop.clone();
                            cx.spawn(
                                move |weak: gpui::WeakEntity<AppSidebar>,
                                      cx: &mut gpui::AsyncApp| {
                                    let weak = weak.clone();
                                    let mut cx = cx.clone();
                                    let gid = gid.clone();
                                    async move {
                                        let _ = provider::process_service().stop(&gid).await;
                                        fetch_games(&weak, &mut cx).await;
                                        weak.update(&mut cx, |s, cx| {
                                            s.games.stopping = None;
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
        .into_any_element()
}

// ── 公开入口 ──

/// 运行中对局管理（对应 client `pages/games.vue`）。
pub fn render_games(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) -> AnyElement {
    let (inited, polling, loading, error) = {
        let s = &sidebar.games;
        (s.inited, s.polling, s.loading, s.error.clone())
    };

    // 首次渲染自动加载
    if !inited {
        sidebar.games.inited = true;
        sidebar.games.loading = true;
        spawn_load(cx);
    }
    // 5s 自动轮询（防重复 spawn）
    if !polling {
        sidebar.games.polling = true;
        spawn_poll(cx);
    }

    // 用 sidebar.running_games 补充英雄/模式展示（进程列表本身不含这些字段）
    let known: HashMap<String, (String, String)> = sidebar
        .running_games
        .iter()
        .map(|g| (g.id.clone(), (g.champion.clone(), g.mode.clone())))
        .collect();

    let rows: Vec<GameRow> = sidebar
        .games
        .games
        .iter()
        .map(|g| {
            let (champion, mode) = known.get(&g.id).cloned().unwrap_or_default();
            GameRow {
                id: g.id.clone(),
                port: g.port,
                status: g.status.clone(),
                champion: if champion.is_empty() {
                    "—".into()
                } else {
                    champion
                },
                mode: if mode.is_empty() { "—".into() } else { mode },
            }
        })
        .collect();
    let rows_empty = rows.is_empty();
    let stopping = sidebar.games.stopping.clone();

    v_flex()
        .size_full()
        .flex_1()
        .gap_6()
        .overflow_hidden()
        // ── 标题行：标题 + 刷新按钮 ──
        .child(
            h_flex()
                .items_center()
                .justify_between()
                .child(
                    v_flex()
                        .gap_1()
                        .child(div().font_bold().text_lg().child("运行中对局"))
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child("查看并管理当前在本地计算机上运行的 Bevy 游戏仿真对局。"),
                        ),
                )
                .child(
                    Button::new("games-refresh-btn")
                        .outline()
                        .icon(IconName::Redo)
                        .label("刷新")
                        .disabled(loading)
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.games.loading = true;
                            spawn_load(cx);
                        })),
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
        // ── 对局表格 ──
        .child(
            div()
                .flex_1()
                .rounded_lg()
                .border_1()
                .border_color(cx.theme().border)
                .overflow_hidden()
                .child(
                    v_flex().size_full().child(header_row(cx)).child(
                        div()
                            .flex_1()
                            .overflow_y_scrollbar()
                            .when(rows_empty, |d| {
                                d.flex().items_center().justify_center().child(
                                    div()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground)
                                        .child(if loading {
                                            "加载中…".to_string()
                                        } else {
                                            "暂无运行中对局".to_string()
                                        }),
                                )
                            })
                            .when(!rows_empty, |d| {
                                d.children(
                                    rows.into_iter().map(|row| build_row(row, &stopping, cx)),
                                )
                            }),
                    ),
                ),
        )
        .into_any_element()
}
