use std::cell::RefCell;

use gpui::prelude::*;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::scroll::ScrollableElement;
use gpui_component::{h_flex, v_flex, ActiveTheme, Disableable, IconName, StyledExt};
use lol_web_protocol::agent::Agent;

use crate::components::sidebar::AppSidebar;
use crate::components::text_input::{render_edit_input, EditOptions};
use crate::services::provider;

const SORTS: &[(&str, &str)] = &[("recent", "最近"), ("popular", "热门"), ("elo", "ELO")];

// ── 页面本地状态 ──

thread_local! {
    static LOADING: RefCell<bool> = RefCell::new(false);
    static FORK_ERROR: RefCell<Option<String>> = RefCell::new(None);
}

// ── 数据加载 ──

fn spawn_load_agents(cx: &mut Context<AppSidebar>, sort: &str) {
    let sort = sort.to_string();
    let client = provider::cloud_client().clone();
    cx.spawn(
        move |weak: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
            let weak = weak.clone();
            let mut cx = cx.clone();
            let sort = sort.clone();
            let client = client.clone();
            async move {
                let agents = client
                    .browse_community_agents(&sort, 60)
                    .await
                    .unwrap_or_default();
                LOADING.with(|l| *l.borrow_mut() = false);
                if let Some(e) = weak.upgrade() {
                    let _ = e.update(&mut cx, |this, cx| {
                        this.community_agents = agents;
                        this.community_loaded = true;
                        cx.notify();
                    });
                }
            }
        },
    )
    .detach();
}

// ── 子区块 ──

fn render_sort_tabs(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) -> AnyElement {
    let current = sidebar.community_sort.clone();
    h_flex()
        .gap_1()
        .children(SORTS.iter().map(|(val, label)| {
            let selected = current == *val;
            let btn = Button::new(format!("comm-sort-{}", val)).label(label.to_string());
            if selected {
                btn.primary().into_any_element()
            } else {
                let v = val.to_string();
                btn.outline()
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.community_sort = v.clone();
                        LOADING.with(|l| *l.borrow_mut() = true);
                        spawn_load_agents(cx, &v);
                        cx.notify();
                    }))
                    .into_any_element()
            }
        }))
        .into_any_element()
}

fn render_search(
    sidebar: &mut AppSidebar,
    window: &mut Window,
    cx: &mut Context<AppSidebar>,
) -> AnyElement {
    let current = sidebar.community_search.clone();
    h_flex()
        .gap_1()
        .items_center()
        .child(div().w_64().child(render_edit_input(
            window,
            cx,
            "comm-search",
            "搜索 Agent / 英雄",
            EditOptions::default(),
            |this| this.community_search.clone(),
            |this, v| this.community_search = v,
        )))
        .child(if current.is_empty() {
            div().into_any_element()
        } else {
            Button::new("comm-search-clear")
                .outline()
                .label("x")
                .on_click(cx.listener(|this, _, _, cx| {
                    this.community_search.clear();
                    cx.notify();
                }))
                .into_any_element()
        })
        .into_any_element()
}

fn filtered_agents(agents: &[Agent], search: &str) -> Vec<Agent> {
    let q = search.trim().to_lowercase();
    if q.is_empty() {
        return agents.to_vec();
    }
    agents
        .iter()
        .filter(|a| a.name.to_lowercase().contains(&q) || a.champion.to_lowercase().contains(&q))
        .cloned()
        .collect()
}

fn render_cards(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) -> AnyElement {
    let loading = LOADING.with(|l| *l.borrow());
    let search = sidebar.community_search.clone();
    let agents = filtered_agents(&sidebar.community_agents, &search);

    if loading {
        return div()
            .text_sm()
            .text_color(cx.theme().muted_foreground)
            .py_16()
            .text_center()
            .child("加载中…")
            .into_any_element();
    }

    if agents.is_empty() {
        return div()
            .text_sm()
            .text_color(cx.theme().muted_foreground)
            .py_16()
            .text_center()
            .child("暂无公开 Agent")
            .into_any_element();
    }

    div()
        .grid()
        .grid_cols(3)
        .gap_4()
        .children(agents.into_iter().map(|a| {
            let aid = a.id.to_string();
            let name = a.name.clone();
            let champion = a.champion.clone();
            let created_date = &a.created_at[..10.min(a.created_at.len())];
            let is_fork = a.forked_from.is_some();
            let target = a.clone();

            v_flex()
                .gap_3()
                .p_4()
                .border_1()
                .border_color(cx.theme().border)
                .rounded_lg()
                .child(
                    h_flex()
                        .items_start()
                        .justify_between()
                        .child(
                            v_flex()
                                .gap_0p5()
                                .child(div().font_bold().text_sm().truncate().child(name))
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground)
                                        .child(format!("{} · 经理 #{}", champion, a.owner_id)),
                                ),
                        )
                        .child(if is_fork {
                            div()
                                .text_xs()
                                .px_1p5()
                                .py_0p5()
                                .border_1()
                                .border_color(cx.theme().border)
                                .rounded_sm()
                                .child(h_flex().gap_0p5().child(IconName::Plus).child("Fork"))
                                .into_any_element()
                        } else {
                            div().into_any_element()
                        }),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child(format!("创建于 {}", created_date)),
                )
                .child({
                    let target2 = target.clone();
                    Button::new(format!("comm-fork-btn-{}", aid))
                        .outline()
                        .w_full()
                        .label("Fork 到我的 Agent")
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.community_fork_target = Some(target2.clone());
                            this.community_fork_name = format!("{} · 副本", target2.name);
                            FORK_ERROR.with(|e| *e.borrow_mut() = None);
                            cx.notify();
                        }))
                        .into_any_element()
                })
                .into_any_element()
        }))
        .into_any_element()
}

// ── Fork ──

fn confirm_fork(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) {
    let Some(target) = sidebar.community_fork_target.clone() else {
        return;
    };
    let name = sidebar.community_fork_name.trim().to_string();
    if name.is_empty() {
        FORK_ERROR.with(|e| *e.borrow_mut() = Some("请输入新 Agent 名称".to_string()));
        cx.notify();
        return;
    }
    sidebar.community_forking = true;
    FORK_ERROR.with(|e| *e.borrow_mut() = None);
    let sort = sidebar.community_sort.clone();
    let client = provider::cloud_client().clone();
    let agent_id = target.id.to_string();
    cx.spawn(
        move |weak: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
            let weak = weak.clone();
            let mut cx = cx.clone();
            let client = client.clone();
            let agent_id = agent_id.clone();
            let name = name.clone();
            let sort = sort.clone();
            async move {
                match client.fork_agent(&agent_id, Some(&name)).await {
                    Ok(_) => {
                        LOADING.with(|l| *l.borrow_mut() = true);
                        let agents = client
                            .browse_community_agents(&sort, 60)
                            .await
                            .unwrap_or_default();
                        LOADING.with(|l| *l.borrow_mut() = false);
                        if let Some(e) = weak.upgrade() {
                            let _ = e.update(&mut cx, |this, cx| {
                                this.community_forking = false;
                                this.community_fork_target = None;
                                this.community_fork_name.clear();
                                this.community_agents = agents;
                                this.community_loaded = true;
                                cx.notify();
                            });
                        }
                    }
                    Err(err) => {
                        FORK_ERROR.with(|e| {
                            *e.borrow_mut() = Some(format!("Fork 失败：{}", err));
                        });
                        if let Some(e) = weak.upgrade() {
                            let _ = e.update(&mut cx, |this, cx| {
                                this.community_forking = false;
                                cx.notify();
                            });
                        }
                    }
                }
            }
        },
    )
    .detach();
}

fn render_fork_dialog(
    sidebar: &mut AppSidebar,
    window: &mut Window,
    cx: &mut Context<AppSidebar>,
) -> AnyElement {
    if sidebar.community_fork_target.is_none() {
        return div().into_any_element();
    }
    let forking = sidebar.community_forking;
    let fork_error = FORK_ERROR.with(|e| e.borrow().clone()).unwrap_or_default();

    v_flex()
        .gap_4()
        .p_5()
        .border_1()
        .border_color(cx.theme().border)
        .rounded_lg()
        .child(div().font_bold().text_lg().child("Fork Agent"))
        .child(
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child("将创建一个属于你的副本，可继续编辑配置或上游同步。"),
        )
        .child(
            h_flex()
                .gap_2()
                .items_center()
                .child(div().text_xs().child("名称"))
                .child(div().flex_1().child(render_edit_input(
                    window,
                    cx,
                    "comm-fork-name",
                    "新 Agent 名称",
                    EditOptions::default(),
                    |this| this.community_fork_name.clone(),
                    |this, v| this.community_fork_name = v,
                ))),
        )
        .when(!fork_error.is_empty(), |d| {
            d.child(
                div()
                    .text_xs()
                    .text_color(cx.theme().danger)
                    .child(fork_error),
            )
        })
        .child(
            h_flex()
                .gap_2()
                .justify_end()
                .child(
                    Button::new("comm-fork-cancel")
                        .outline()
                        .label("取消")
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.community_fork_target = None;
                            this.community_fork_name.clear();
                            FORK_ERROR.with(|e| *e.borrow_mut() = None);
                            cx.notify();
                        })),
                )
                .child(
                    Button::new("comm-fork-confirm")
                        .primary()
                        .label(if forking {
                            "处理中…"
                        } else {
                            "确认 Fork"
                        })
                        .disabled(forking)
                        .on_click(cx.listener(|this, _, _, cx| {
                            confirm_fork(this, cx);
                        })),
                ),
        )
        .into_any_element()
}

// ── 入口 ──

pub fn render_community(
    sidebar: &mut AppSidebar,
    window: &mut Window,
    cx: &mut Context<AppSidebar>,
) -> AnyElement {
    // 首次渲染加载社区列表
    if !sidebar.community_loaded {
        sidebar.community_loaded = true;
        LOADING.with(|l| *l.borrow_mut() = true);
        let sort = sidebar.community_sort.clone();
        spawn_load_agents(cx, &sort);
    }

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
                        .child(IconName::Globe)
                        .child(div().font_bold().text_lg().child("Agent 社区")),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child("浏览其他电竞经理公开的 Agent，Fork 到本地继续训练或参赛。"),
                ),
        )
        .child(
            h_flex()
                .items_center()
                .justify_between()
                .child(render_sort_tabs(sidebar, cx))
                .child(render_search(sidebar, window, cx)),
        )
        .child(div().h_0p5().bg(cx.theme().border))
        .child(if sidebar.community_fork_target.is_some() {
            render_fork_dialog(sidebar, window, cx)
        } else {
            div().into_any_element()
        })
        .child(
            div()
                .flex_1()
                .w_full()
                .overflow_y_scrollbar()
                .child(render_cards(sidebar, cx)),
        )
        .into_any_element()
}
