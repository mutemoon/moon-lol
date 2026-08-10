//! Browse：Agent 卡片网格。

use gpui::prelude::*;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::scroll::ScrollableElement;
use gpui_component::separator::Separator;
use gpui_component::{h_flex, v_flex, ActiveTheme, IconName, StyledExt};
use lol_web_protocol::agent::Agent;
use lol_web_protocol::agent_snapshot::AgentSnapshot;

use super::utils::{
    champion_display, has_unpublished_changes, latest_snapshot_label, visibility_label,
};
use super::{enter_edit, start_new};
use crate::components::sidebar::AppSidebar;

pub(super) fn render_browse(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) -> AnyElement {
    let agents = sidebar.heroes.agents.clone();
    let snaps_map = sidebar.heroes.snapshots.clone();

    v_flex()
        .size_full()
        .overflow_hidden()
        .child(
            h_flex()
                .items_center()
                .justify_between()
                .p_4()
                .child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .child(IconName::LayoutDashboard)
                        .child(div().font_bold().text_lg().child("我的选手"))
                        .child(
                            div()
                                .text_sm()
                                .text_color(cx.theme().muted_foreground)
                                .child(agents.len().to_string()),
                        ),
                )
                .child(
                    h_flex().gap_2().child(
                        Button::new("new-hero-btn")
                            .primary()
                            .icon(IconName::Plus)
                            .label("新建选手")
                            .on_click(cx.listener(|this, _, _, cx| {
                                start_new(this, cx);
                            })),
                    ),
                ),
        )
        .child(Separator::horizontal())
        .child(
            div()
                .flex_1()
                .w_full()
                .overflow_y_scrollbar()
                .child(if agents.is_empty() {
                    v_flex()
                        .items_center()
                        .justify_center()
                        .py_24()
                        .gap_4()
                        .child(
                            div()
                                .text_color(cx.theme().muted_foreground)
                                .text_sm()
                                .child("还没有选手，先建一个吧"),
                        )
                        .child(
                            Button::new("new-hero-empty-btn")
                                .outline()
                                .icon(IconName::Plus)
                                .label("新建选手")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    start_new(this, cx);
                                })),
                        )
                        .into_any_element()
                } else {
                    div()
                        .grid()
                        .grid_cols(3)
                        .gap_4()
                        .children(agents.iter().map(|a| {
                            let snaps = snaps_map.get(&a.id).cloned().unwrap_or_default();
                            render_agent_card(a, &snaps, sidebar, cx)
                        }))
                        .into_any_element()
                }),
        )
        .into_any_element()
}

fn render_agent_card(
    agent: &Agent,
    snaps: &[AgentSnapshot],
    _sidebar: &mut AppSidebar,
    cx: &mut Context<AppSidebar>,
) -> AnyElement {
    let agent_clone = agent.clone();
    let name = agent.name.clone();
    let champion = agent.champion.clone();
    let agent_type = agent.agent_type;
    let visibility = agent.visibility;
    let dirty = has_unpublished_changes(agent, snaps);
    let snap_label = latest_snapshot_label(snaps);

    div()
        .rounded_lg()
        .border_1()
        .border_color(cx.theme().border)
        .p_4()
        .cursor_pointer()
        .id(format!("agent-card-{}", agent.id))
        .on_click(cx.listener(move |this, _, _, cx| {
            enter_edit(this, cx, &agent_clone);
        }))
        .child(
            v_flex()
                .gap_2()
                .child(
                    h_flex()
                        .justify_between()
                        .items_center()
                        .child(div().font_bold().text_sm().child(name.clone()))
                        .child(
                            div()
                                .px_2()
                                .py_0p5()
                                .rounded_md()
                                .bg(cx.theme().muted)
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child(agent_type.as_str().to_uppercase()),
                        ),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child(champion_display(&champion)),
                )
                .child(
                    h_flex()
                        .justify_between()
                        .items_center()
                        .child(
                            div()
                                .px_2()
                                .py_0p5()
                                .rounded_md()
                                .bg(cx.theme().muted)
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child(visibility_label(visibility)),
                        )
                        .child(
                            h_flex()
                                .gap_2()
                                .items_center()
                                .child(if dirty {
                                    div()
                                        .text_xs()
                                        .text_color(rgb(0xd97706))
                                        .child("未发布改动")
                                        .into_any_element()
                                } else {
                                    div().into_any_element()
                                })
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground)
                                        .child(snap_label),
                                ),
                        ),
                ),
        )
        .into_any_element()
}
