//! 房间详情页 UI 片段：槽位行 / 阵营列 / 添加槽位对话框。

use gpui::prelude::*;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::menu::{DropdownMenu, PopupMenuItem};
use gpui_component::{
    h_flex, v_flex, ActiveTheme, Disableable, IconName, Sizable, StyledExt, WindowExt as _,
};
use lol_web_protocol::agent::Agent;
use lol_web_protocol::room::RoomAgentSlot;
use lol_web_protocol::spawn_preset::Team;
use uuid::Uuid;

use super::logic::{spawn_add_slot, spawn_remove_slot};
use crate::components::dialog::open_form_dialog;
use crate::components::sidebar::AppSidebar;

// ── 展示辅助 ──

fn agent_name(agents: &[Agent], id: Uuid) -> String {
    agents
        .iter()
        .find(|a| a.id == id)
        .map(|a| a.name.clone())
        .unwrap_or_else(|| id.to_string())
}

fn agent_champion(agents: &[Agent], id: Uuid) -> String {
    agents
        .iter()
        .find(|a| a.id == id)
        .map(|a| a.champion.clone())
        .unwrap_or_else(|| "—".to_string())
}

// ── 槽位行 ──

fn slot_row(
    cx: &mut Context<AppSidebar>,
    room_id: Uuid,
    slot: &RoomAgentSlot,
    agents: &[Agent],
) -> AnyElement {
    let name = agent_name(agents, slot.agent_id);
    let subtitle = format!(
        "{} · 成员 #{}",
        agent_champion(agents, slot.agent_id),
        slot.member_user_id
    );
    h_flex()
        .items_center()
        .justify_between()
        .gap_2()
        .rounded_md()
        .border_1()
        .border_color(cx.theme().border.opacity(0.5))
        .px_3()
        .py_2()
        .text_xs()
        .child(
            div()
                .min_w_0()
                .child(div().truncate().font_bold().child(name))
                .child(
                    div()
                        .truncate()
                        .text_color(cx.theme().muted_foreground)
                        .child(subtitle),
                ),
        )
        .child(remove_slot_btn(cx, room_id, slot.id))
        .into_any_element()
}

fn remove_slot_btn(cx: &mut Context<AppSidebar>, room_id: Uuid, slot_id: Uuid) -> AnyElement {
    Button::new(format!("remove-slot-{}", slot_id))
        .ghost()
        .xsmall()
        .icon(IconName::Delete)
        .tooltip("删除槽位")
        .on_click(cx.listener(move |_, _, _, cx| {
            spawn_remove_slot(cx, room_id, slot_id);
        }))
        .into_any_element()
}

// ── 阵营列 ──

pub(super) fn render_team_column(
    cx: &mut Context<AppSidebar>,
    room_id: Uuid,
    team: Team,
    label: &str,
    color: Hsla,
    slots: &[RoomAgentSlot],
    agents: &[Agent],
) -> AnyElement {
    let team_slots: Vec<&RoomAgentSlot> = slots.iter().filter(|s| s.team == team).collect();
    let team_str = team.as_str();
    v_flex()
        .flex_1()
        .rounded_lg()
        .border_1()
        .border_color(cx.theme().border)
        .overflow_hidden()
        .child(
            h_flex()
                .items_center()
                .justify_between()
                .px_3()
                .py_2()
                .border_b_1()
                .border_color(cx.theme().border)
                .child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .child(div().w_2().h_2().rounded_full().bg(color))
                        .child(div().text_xs().font_bold().child(label.to_string()))
                        .child(
                            div()
                                .px_1p5()
                                .py_0p5()
                                .rounded_md()
                                .bg(cx.theme().accent.opacity(0.15))
                                .text_xs()
                                .font_bold()
                                .text_color(cx.theme().accent)
                                .child(team_slots.len().to_string()),
                        ),
                )
                .child(
                    Button::new(format!("add-{team_str}-slot"))
                        .outline()
                        .xsmall()
                        .icon(IconName::Plus)
                        .label("添加槽位")
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.room_detail.show_add_team = Some(team);
                            this.room_detail.add_agent_id = None;
                            this.room_detail.add_error.clear();
                            cx.notify();
                            open_add_slot_dialog(window, cx, room_id);
                        })),
                ),
        )
        .child(
            v_flex()
                .gap_2()
                .p_2()
                .when(team_slots.is_empty(), |d| {
                    d.child(
                        div()
                            .py_6()
                            .w_full()
                            .text_center()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child("暂无 Agent"),
                    )
                })
                .children(team_slots.iter().map(|s| slot_row(cx, room_id, *s, agents))),
        )
        .into_any_element()
}

// ── 添加槽位对话框 ──

pub(super) fn open_add_slot_dialog(
    window: &mut Window,
    cx: &mut Context<AppSidebar>,
    room_id: Uuid,
) {
    let weak = cx.entity().downgrade();
    let team_title = match cx.entity().read(cx).room_detail.show_add_team {
        Some(Team::Order) => "添加到 Order（蓝方）",
        Some(Team::Chaos) => "添加到 Chaos（红方）",
        None => "添加 Agent 槽位",
    };
    open_form_dialog(
        window,
        cx,
        weak,
        move |sidebar, window, cx| build_add_slot_form(sidebar, window, cx, room_id),
        move |dialog, form| dialog.title(team_title).w(px(384.)).child(form),
    );
}

fn build_add_slot_form(
    sidebar: &AppSidebar,
    _window: &mut Window,
    cx: &mut Context<AppSidebar>,
    room_id: Uuid,
) -> AnyElement {
    let show_team = sidebar.room_detail.show_add_team;
    let add_agent_id = sidebar.room_detail.add_agent_id.clone();
    let add_error = sidebar.room_detail.add_error.clone();
    let adding = sidebar.room_detail.adding;
    let Some(_team) = show_team else {
        return div().into_any_element();
    };
    let agents = sidebar.room_detail.agents.clone();
    let agent_label = add_agent_id
        .as_deref()
        .and_then(|aid| agents.iter().find(|a| a.id.to_string() == aid))
        .map(|a| format!("{} · {}", a.name, a.champion))
        .unwrap_or_else(|| "选择 Agent…".to_string());
    let weak = cx.entity().downgrade();
    let agents_owned = agents.clone();

    let agent_dropdown =
        Button::new("room-add-agent-dropdown")
            .outline()
            .w_full()
            .icon(IconName::ChevronDown)
            .label(agent_label)
            .dropdown_menu(move |menu, _window, _cx| {
                let mut m = menu;
                if agents_owned.is_empty() {
                    m = m.item(PopupMenuItem::new("暂无 Agent").disabled(true));
                }
                for a in &agents_owned {
                    let aid = a.id.to_string();
                    let label = format!("{} · {}", a.name, a.champion);
                    let checked = Some(aid.clone()) == add_agent_id;
                    let weak = weak.clone();
                    m = m.item(PopupMenuItem::new(label).checked(checked).on_click(
                        move |_, _, cx| {
                            let _ = weak.update(cx, |s, cx| {
                                s.room_detail.add_agent_id = Some(aid.clone());
                                cx.notify();
                            });
                        },
                    ));
                }
                m
            });

    v_flex()
        .gap_4()
        .child(
            v_flex()
                .gap_3()
                .child(
                    v_flex()
                        .gap_1()
                        .child(div().text_xs().child("选择 Agent"))
                        .child(agent_dropdown),
                )
                .when(!add_error.is_empty(), |d| {
                    d.child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().danger)
                            .child(add_error),
                    )
                }),
        )
        .child(
            h_flex()
                .gap_2()
                .justify_end()
                .child(
                    Button::new("cancel-add-slot")
                        .ghost()
                        .label("取消")
                        .disabled(adding)
                        .on_click(cx.listener(|_, _, window, cx| {
                            window.close_dialog(cx);
                        })),
                )
                .child(
                    Button::new("confirm-add-slot")
                        .primary()
                        .label(if adding { "添加中…" } else { "添加" })
                        .disabled(adding)
                        .on_click(cx.listener(move |this, _, _, cx| {
                            spawn_add_slot(this, cx, room_id);
                        })),
                ),
        )
        .into_any_element()
}
