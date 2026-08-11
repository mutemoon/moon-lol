use gpui::prelude::*;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::scroll::ScrollableElement;
use gpui_component::{h_flex, v_flex, ActiveTheme, IconName, StyledExt};
use lol_web_protocol::history::GameHistorySummary;
use lol_web_protocol::scenario::Scenario;
use rust_i18n::t;

use crate::components::sidebar::AppSidebar;
use crate::services::provider;

// ── 页面本地状态：对局历史与场景模板（运行中对局存于 sidebar.running_games） ──

pub struct HomePageState {
    /// 首次渲染是否已触发加载
    inited: bool,
    histories: Vec<GameHistorySummary>,
    scenarios: Vec<Scenario>,
}

impl Default for HomePageState {
    fn default() -> Self {
        Self {
            inited: false,
            histories: Vec::new(),
            scenarios: Vec::new(),
        }
    }
}

/// 拉取运行中对局、对局历史与场景模板，写回 sidebar 与页面本地状态。
async fn refresh_home_data(sidebar_weak: &gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp) {
    let client = provider::cloud_client().clone();
    let (games, histories, scenarios) = tokio::join!(
        provider::process_service().list(),
        async { client.list_game_histories().await },
        async { client.list_scenarios().await },
    );
    if let Some(entity) = sidebar_weak.upgrade() {
        let _ = entity.update(cx, |sidebar, cx| {
            if let Ok(h) = histories {
                sidebar.home.histories = h;
            }
            if let Ok(sc) = scenarios {
                sidebar.home.scenarios = sc;
            }
            if let Ok(games) = games {
                sidebar.running_games = games
                    .into_iter()
                    .map(|g| crate::types::RunningGameInfo {
                        id: g.id,
                        mode: String::new(),
                        champion: String::new(),
                        port: g.port as u16,
                    })
                    .collect();
            }
            cx.notify();
        });
    }
}

/// 异步刷新首页数据（首次渲染与手动刷新共用）。
fn spawn_refresh(cx: &mut Context<AppSidebar>) {
    cx.spawn(
        move |weak: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
            let weak = weak.clone();
            let mut cx = cx.clone();
            async move {
                refresh_home_data(&weak, &mut cx).await;
            }
        },
    )
    .detach();
}

/// 工作台首页：运行中对局、场景模板、对局历史概览。
pub fn render_home(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) -> AnyElement {
    let running_count = sidebar.running_games.len();
    let (inited, history_count, scenarios) = {
        let s = &sidebar.home;
        (s.inited, s.histories.len(), s.scenarios.clone())
    };
    let scenario_count = scenarios.len();

    // 首次渲染自动拉取运行对局、对局历史与场景模板
    if !inited {
        sidebar.home.inited = true;
        spawn_refresh(cx);
    }

    v_flex()
        .size_full()
        .flex_1()
        .gap_6()
        .overflow_hidden()
        // ── 标题行 ──
        .child(
            h_flex()
                .items_center()
                .justify_between()
                .child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .child(IconName::LayoutDashboard)
                        .child(div().font_bold().text_lg().child(t!("app.nav.title_home"))),
                )
                .child(
                    Button::new("home-refresh-btn")
                        .outline()
                        .icon(IconName::Loader)
                        .label(t!("app.rl.refresh_list"))
                        .on_click(cx.listener(|_this, _, _, cx| {
                            spawn_refresh(cx);
                        })),
                ),
        )
        // ── 快捷入口卡片 ──
        .child(
            h_flex()
                .gap_4()
                .w_full()
                .child(quick_card(
                    cx,
                    IconName::Play,
                    t!("app.nav.menu_launcher"),
                    t!("app.sidebar.new_match"),
                    |this, _window, cx| {
                        this.navigate_to(crate::types::ActiveView::Launcher);
                        cx.notify();
                    },
                ))
                .child(quick_card(
                    cx,
                    IconName::SquareTerminal,
                    t!("app.sidebar.debug_sessions"),
                    format!("{} running", running_count),
                    |_this, _window, cx| {
                        spawn_refresh(cx);
                    },
                ))
                .child(quick_card(
                    cx,
                    IconName::User,
                    t!("app.nav.menu_rooms"),
                    t!("app.nav.title_rooms"),
                    |this, _window, cx| {
                        this.navigate_to(crate::types::ActiveView::Rooms);
                        cx.notify();
                    },
                ))
                .child(quick_card(
                    cx,
                    IconName::File,
                    "对局历史记录",
                    format!("{} 条对局记录", history_count),
                    |_this, _window, _cx| {
                        // gpui 暂无对局历史视图，暂不跳转
                    },
                )),
        )
        // ── 运行中对局 + 场景模板 双栏 ──
        .child(
            h_flex()
                .flex_1()
                .gap_4()
                .overflow_hidden()
                // 左栏：运行中对局
                .child(
                    div()
                        .flex_1()
                        .rounded_lg()
                        .border_1()
                        .border_color(cx.theme().border)
                        .overflow_hidden()
                        .child(
                            v_flex()
                                .size_full()
                                .child(
                                    h_flex()
                                        .px_4()
                                        .py_3()
                                        .border_b_1()
                                        .border_color(cx.theme().border)
                                        .items_center()
                                        .justify_between()
                                        .child(
                                            div()
                                                .font_bold()
                                                .text_sm()
                                                .child(t!("app.sidebar.debug_sessions")),
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
                                                .child(format!("{}", running_count)),
                                        ),
                                )
                                .child(
                                    div()
                                        .flex_1()
                                        .overflow_y_scrollbar()
                                        .p_4()
                                        .when(running_count == 0, |d| {
                                            d.flex().items_center().justify_center().child(
                                                div()
                                                    .text_xs()
                                                    .text_color(cx.theme().muted_foreground)
                                                    .child("暂无活跃运行中对局"),
                                            )
                                        })
                                        .children(sidebar.running_games.iter().map(|g| {
                                            let game_id = g.id.clone();
                                            let port = g.port;

                                            h_flex()
                                                .gap_3()
                                                .items_center()
                                                .px_3()
                                                .py_2()
                                                .rounded_md()
                                                .border_1()
                                                .border_color(cx.theme().border)
                                                .mb_2()
                                                .child(
                                                    div()
                                                        .w_2()
                                                        .h_2()
                                                        .rounded_full()
                                                        .bg(cx.theme().accent),
                                                )
                                                .child(
                                                    div()
                                                        .font_bold()
                                                        .text_xs()
                                                        .child(format!("Port: {}", port)),
                                                )
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(cx.theme().muted_foreground)
                                                        .child(format!(
                                                            "ID: {}",
                                                            &game_id[..8.min(game_id.len())]
                                                        )),
                                                )
                                                .child(
                                                    h_flex()
                                                        .flex_1()
                                                        .gap_1()
                                                        .justify_end()
                                                        .child(
                                                            Button::new(format!(
                                                                "stop-game-{}",
                                                                game_id
                                                            ))
                                                            .ghost()
                                                            .icon(IconName::CircleX)
                                                            .label(t!("app.rl.stop"))
                                                            .on_click({
                                                                let gid = game_id.clone();
                                                                cx.listener(move |_this, _, _, cx| {
                                                                    let eid = gid.clone();
                                                                    cx.spawn(
                                                                        move |weak: gpui::WeakEntity<
                                                                            AppSidebar,
                                                                        >,
                                                                         cx: &mut gpui::AsyncApp| {
                                                                            let weak = weak.clone();
                                                                            let mut cx = cx.clone();
                                                                            let eid = eid.clone();
                                                                            async move {
                                                                                if let Err(err) =
                                                                                    provider::process_service()
                                                                                        .stop(&eid)
                                                                                        .await
                                                                                {
                                                                                    tracing::warn!(
                                                                                        "停止对局失败: {}",
                                                                                        err
                                                                                    );
                                                                                }
                                                                                if let Some(entity) =
                                                                                    weak.upgrade()
                                                                                {
                                                                                    entity.update(
                                                                                        &mut cx,
                                                                                        |sidebar, cx| {
                                                                                            sidebar
                                                                                                .running_games
                                                                                                .retain(|g| {
                                                                                                    g.id != eid
                                                                                                });
                                                                                            cx.notify();
                                                                                        },
                                                                                    );
                                                                                }
                                                                            }
                                                                        },
                                                                    )
                                                                    .detach();
                                                                })
                                                            }),
                                                        ),
                                                )
                                        })),
                                ),
                        ),
                )
                // 右栏：场景配置模板
                .child(
                    div()
                        .flex_1()
                        .rounded_lg()
                        .border_1()
                        .border_color(cx.theme().border)
                        .overflow_hidden()
                        .child(
                            v_flex()
                                .size_full()
                                .child(
                                    h_flex()
                                        .px_4()
                                        .py_3()
                                        .border_b_1()
                                        .border_color(cx.theme().border)
                                        .items_center()
                                        .justify_between()
                                        .child(
                                            div()
                                                .font_bold()
                                                .text_sm()
                                                .child("场景配置模板"),
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
                                                .child(format!("{}", scenario_count)),
                                        ),
                                )
                                .child(
                                    div()
                                        .flex_1()
                                        .overflow_y_scrollbar()
                                        .p_4()
                                        .when(scenario_count == 0, |d| {
                                            d.flex().items_center().justify_center().child(
                                                div()
                                                    .text_xs()
                                                    .text_color(cx.theme().muted_foreground)
                                                    .child("暂无自定义场景配置模板"),
                                            )
                                        })
                                        .children(scenarios.iter().map(|s| {
                                            h_flex()
                                                .gap_3()
                                                .items_center()
                                                .px_3()
                                                .py_2()
                                                .rounded_md()
                                                .border_1()
                                                .border_color(cx.theme().border)
                                                .mb_2()
                                                .child(
                                                    div()
                                                        .text_color(cx.theme().muted_foreground)
                                                        .child(IconName::File),
                                                )
                                                .child(
                                                    div().flex_1().text_xs().child(s.name.clone()),
                                                )
                                        })),
                                ),
                        ),
                ),
        )
        .into_any_element()
}

fn quick_card(
    cx: &mut Context<AppSidebar>,
    icon: IconName,
    title: impl Into<SharedString>,
    desc: impl Into<SharedString>,
    on_click: impl Fn(&mut AppSidebar, &mut Window, &mut Context<AppSidebar>) + 'static,
) -> AnyElement {
    div()
        .flex_1()
        .rounded_lg()
        .border_1()
        .border_color(cx.theme().border)
        .p_5()
        .flex()
        .flex_col()
        .justify_between()
        .h_32()
        .hover(|s| s.bg(cx.theme().accent.opacity(0.05)))
        .cursor_pointer()
        .on_any_mouse_down(cx.listener(move |this, _e, window, cx| {
            on_click(this, window, cx);
        }))
        .child(
            h_flex()
                .items_start()
                .justify_between()
                .child(
                    v_flex()
                        .gap_1()
                        .child(div().font_bold().text_sm().child(title.into()))
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child(desc.into()),
                        ),
                )
                .child(
                    div()
                        .p_2()
                        .rounded_md()
                        .bg(cx.theme().accent.opacity(0.1))
                        .child(icon),
                ),
        )
        .into_any_element()
}
