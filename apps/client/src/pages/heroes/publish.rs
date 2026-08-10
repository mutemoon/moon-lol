//! Publish Tab：上游同步 / 可见性 / 发布快照 / 历史快照 / 状态栏。

use gpui::prelude::*;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::scroll::ScrollableElement;
use gpui_component::separator::Separator;
use gpui_component::{h_flex, v_flex, ActiveTheme, IconName, StyledExt};
use lol_web_protocol::agent::Agent;
use lol_web_protocol::agent_snapshot::AgentSnapshot;
use lol_web_protocol::spawn_preset::Visibility;

use super::types::HeroesMode;
use super::utils::{ago, has_unpublished_changes, pretty_agent, visibility_label};
use super::{handle_publish, handle_pull_upstream, handle_visibility_change};
use crate::components::sidebar::AppSidebar;

pub(super) fn render_publish_tab(
    sidebar: &mut AppSidebar,
    cx: &mut Context<AppSidebar>,
) -> AnyElement {
    let editing_id = match &sidebar.heroes.mode {
        HeroesMode::Edit { editing_id } => *editing_id,
        _ => None,
    };
    let current_agent = editing_id
        .and_then(|id| sidebar.heroes.agents.iter().find(|a| a.id == id))
        .cloned();
    let snaps = editing_id
        .map(|id| {
            sidebar
                .heroes
                .snapshots
                .get(&id)
                .cloned()
                .unwrap_or_default()
        })
        .unwrap_or_default();
    let upstream_id = current_agent
        .as_ref()
        .and_then(|a| a.upstream_agent_id.or(a.forked_from));

    v_flex()
        .gap_6()
        .child(if upstream_id.is_some() {
            render_upstream_sync(sidebar, cx, &current_agent)
        } else {
            div().into_any_element()
        })
        .child(render_visibility_section(sidebar, cx))
        .child(Separator::horizontal())
        .child(render_publish_section(sidebar, cx, &snaps))
        .child(render_snapshot_list(sidebar, cx, &snaps))
        .into_any_element()
}

/// 上游同步 + Fork diff 两栏对照 + 应用上游。
fn render_upstream_sync(
    sidebar: &mut AppSidebar,
    cx: &mut Context<AppSidebar>,
    current_agent: &Option<Agent>,
) -> AnyElement {
    let up = sidebar.heroes.upstream_agent.clone();
    let owner_id = up.as_ref().map_or(0, |a| a.owner_id);
    let up_name = up.as_ref().map_or("…".to_string(), |a| a.name.clone());
    let show_diff = up.is_some();
    let current_text = current_agent.as_ref().map(pretty_agent).unwrap_or_default();
    let upstream_text = up.as_ref().map(pretty_agent).unwrap_or_default();

    v_flex()
        .gap_3()
        .child(div().text_sm().font_bold().child("上游同步"))
        .child(
            div()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child("该选手 Fork 自上游公开选手。可对比差异并拉取上游最新策略覆盖当前编辑态。"),
        )
        .child(
            h_flex()
                .items_center()
                .justify_between()
                .rounded_md()
                .border_1()
                .border_color(cx.theme().border)
                .px_4()
                .py_3()
                .child(
                    div()
                        .text_sm()
                        .child(format!("Fork 自「{}」· 经理 #{}", up_name, owner_id)),
                )
                .child(if show_diff {
                    Button::new("pull-btn")
                        .outline()
                        .label("应用上游（覆盖当前）")
                        .on_click(cx.listener(|this, _, _, cx| {
                            handle_pull_upstream(this, cx);
                        }))
                        .into_any_element()
                } else {
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child("加载上游中…")
                        .into_any_element()
                }),
        )
        .child(if show_diff {
            h_flex()
                .gap_3()
                .items_start()
                .child(diff_column("当前", &current_text, cx))
                .child(diff_column("上游", &upstream_text, cx))
                .into_any_element()
        } else {
            div().into_any_element()
        })
        .into_any_element()
}

fn diff_column(title: &str, text: &str, cx: &mut Context<AppSidebar>) -> AnyElement {
    v_flex()
        .flex_1()
        .gap_1()
        .child(
            div()
                .text_xs()
                .font_bold()
                .text_color(cx.theme().muted_foreground)
                .child(title.to_string()),
        )
        .child(
            div()
                .h(px(260.))
                .overflow_y_scrollbar()
                .rounded_md()
                .border_1()
                .border_color(cx.theme().border)
                .px_3()
                .py_2()
                .text_xs()
                .child(text.to_string()),
        )
        .into_any_element()
}

fn render_visibility_section(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) -> AnyElement {
    let current = sidebar.heroes.draft_visibility;
    let vis_btns: Vec<AnyElement> = [Visibility::Private, Visibility::Friends, Visibility::Public]
        .iter()
        .map(|&v| {
            let active = current == v;
            let btn = Button::new(format!("vis-{:?}", v)).label(visibility_label(v));
            let btn = if active { btn.primary() } else { btn.outline() };
            btn.on_click(cx.listener(move |this, _, _, cx| {
                handle_visibility_change(this, v, cx);
            }))
            .into_any_element()
        })
        .collect();

    v_flex()
        .gap_2()
        .child(div().text_sm().font_bold().child("可见性"))
        .child(
            div()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child("公开后他人可在社区 Fork；提示词与模型配置等敏感字段是否暴露由可见性决定。"),
        )
        .child(h_flex().gap_2().children(vis_btns))
        .into_any_element()
}

fn render_publish_section(
    sidebar: &mut AppSidebar,
    cx: &mut Context<AppSidebar>,
    snaps: &[AgentSnapshot],
) -> AnyElement {
    let editing = matches!(
        sidebar.heroes.mode,
        HeroesMode::Edit {
            editing_id: Some(_)
        }
    );
    let editing_id = match &sidebar.heroes.mode {
        HeroesMode::Edit { editing_id } => *editing_id,
        _ => None,
    };
    let dirty = editing_id
        .and_then(|id| sidebar.heroes.agents.iter().find(|a| a.id == id))
        .map(|a| has_unpublished_changes(a, snaps))
        .unwrap_or(false);
    let publishing = sidebar.heroes.publishing;

    v_flex()
        .gap_3()
        .child(div().text_sm().font_bold().child("发布参赛快照"))
        .child(
            div()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child("Rank 队列始终取用该选手最新发布的快照。改完配置后需要再发布一次才会进入下一局；进行中的对局不受影响。"),
        )
        .child(if dirty {
            div()
                .text_xs()
                .text_color(rgb(0xd97706))
                .child("当前配置晚于最新快照，需重新发布才会在 Rank 生效。")
                .into_any_element()
        } else {
            div().into_any_element()
        })
        .child(if !editing {
            div()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child("该选手尚未在云端注册，无法发布快照。请先保存完成同步。")
                .into_any_element()
        } else {
            div().into_any_element()
        })
        .child(
            Button::new("publish-btn")
                .primary()
                .icon(IconName::Play)
                .label(if publishing { "发布中…" } else { "发布快照" })
                .on_click(cx.listener(|this, _, _, cx| {
                    handle_publish(this, cx);
                })),
        )
        .into_any_element()
}

fn render_snapshot_list(
    _sidebar: &mut AppSidebar,
    cx: &mut Context<AppSidebar>,
    snaps: &[AgentSnapshot],
) -> AnyElement {
    v_flex()
        .gap_2()
        .child(
            div()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .font_bold()
                .child("历史快照"),
        )
        .child(if snaps.is_empty() {
            div()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child("尚无历史快照")
                .into_any_element()
        } else {
            v_flex()
                .rounded_md()
                .border_1()
                .border_color(cx.theme().border)
                .children(snaps.iter().enumerate().map(|(i, s)| {
                    let row = v_flex().px_4().py_2();
                    let row = if i > 0 {
                        row.border_t_1().border_color(cx.theme().border)
                    } else {
                        row
                    };
                    row.child(
                        h_flex()
                            .justify_between()
                            .items_center()
                            .child(
                                h_flex()
                                    .gap_2()
                                    .items_center()
                                    .child(div().text_sm().child(format!("v{}", s.version)))
                                    .child(if i == 0 {
                                        div()
                                            .px_2()
                                            .py_0p5()
                                            .rounded_md()
                                            .bg(cx.theme().muted)
                                            .text_xs()
                                            .child("当前最新")
                                            .into_any_element()
                                    } else {
                                        div().into_any_element()
                                    }),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(format!("{} 前", ago(&s.created_at))),
                            ),
                    )
                    .into_any_element()
                }))
                .into_any_element()
        })
        .into_any_element()
}

pub(super) fn render_status_bar(
    sidebar: &mut AppSidebar,
    cx: &mut Context<AppSidebar>,
) -> AnyElement {
    let editing = matches!(
        sidebar.heroes.mode,
        HeroesMode::Edit {
            editing_id: Some(_)
        }
    );
    let error = sidebar.heroes.error_msg.clone();
    let success = sidebar.heroes.success_msg.clone();

    h_flex()
        .px_4()
        .py_2()
        .gap_2()
        .items_center()
        .child(if !error.is_empty() {
            div()
                .text_sm()
                .text_color(cx.theme().danger)
                .child(error)
                .into_any_element()
        } else if !success.is_empty() {
            div().text_sm().child(success).into_any_element()
        } else {
            div().into_any_element()
        })
        .child(if editing {
            Button::new("delete-btn")
                .ghost()
                .label("删除")
                .on_click(cx.listener(|this, _, _, cx| {
                    this.heroes.show_delete_confirm = true;
                    cx.notify();
                }))
                .into_any_element()
        } else {
            div().into_any_element()
        })
        .into_any_element()
}
