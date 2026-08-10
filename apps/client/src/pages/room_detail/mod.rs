//! 房间详情页（对应 client `pages/rooms/[id].vue`，用 sidebar.current_room_id）。
//!
//! 子模块：types（thread_local 状态）、logic（拉取 / 轮询 / 房间操作）、ui（渲染片段）。

mod logic;
mod types;
mod ui;

use gpui::prelude::*;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::clipboard::Clipboard;
use gpui_component::scroll::ScrollableElement;
use gpui_component::{h_flex, v_flex, ActiveTheme, Disableable, IconName, StyledExt};
use lol_web_protocol::room::{RoomStatus, TeamPolicy};
use lol_web_protocol::spawn_preset::Team;

use self::logic::{spawn_dissolve_room, spawn_leave_room, spawn_load, spawn_poll, spawn_start_match};
use self::types::{reset_state_for, update_state, with_state};
use self::ui::{render_add_dialog, render_team_column};
use crate::components::sidebar::AppSidebar;
use crate::types::ActiveView;

// ── 页面入口 ──

/// 房间详情（对应 client `pages/rooms/[id].vue`，用 sidebar.current_room_id）。
pub fn render_room_detail(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) -> AnyElement {
    let room_id = sidebar.current_room_id;
    reset_state_for(room_id);

    // ── 空态：未选中房间 ──
    let Some(id) = room_id else {
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
                    .child("尚未进入任何房间，去房间列表选择一个房间查看详情。"),
            )
            .child(
                Button::new("room-detail-empty-back-btn")
                    .primary()
                    .label("返回房间列表")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.current_room_id = None;
                        this.navigate_to(ActiveView::Rooms);
                        cx.notify();
                    })),
            )
            .into_any_element();
    };

    // 首次加载 + 5s 轮询（防重复 spawn）
    let (inited, polling) = with_state(|s| (s.inited, s.polling));
    if !inited {
        update_state(|s| {
            s.inited = true;
            s.loading = true;
        });
        spawn_load(cx, id);
    }
    if !polling {
        update_state(|s| s.polling = true);
        spawn_poll(cx, id);
    }

    let (loading, room, slots, agents, error, starting) = with_state(|s| {
        (
            s.loading,
            s.room.clone(),
            s.slots.clone(),
            s.agents.clone(),
            s.error.clone(),
            s.starting,
        )
    });

    let muted = cx.theme().muted_foreground;
    let border = cx.theme().border;
    let danger = cx.theme().danger;
    let accent = cx.theme().accent;

    // 加载中
    if loading && room.is_none() {
        return v_flex()
            .size_full()
            .flex_1()
            .items_center()
            .justify_center()
            .child(div().text_sm().text_color(muted).child("加载中…"))
            .into_any_element();
    }
    // 加载失败且无数据
    let Some(room) = room else {
        return v_flex()
            .size_full()
            .flex_1()
            .items_center()
            .justify_center()
            .gap_3()
            .when_some(error.as_ref(), |d, err| {
                d.child(div().text_sm().text_color(danger).child(err.clone()))
            })
            .child(
                Button::new("room-detail-error-back-btn")
                    .primary()
                    .label("返回房间列表")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.current_room_id = None;
                        this.navigate_to(ActiveView::Rooms);
                        cx.notify();
                    })),
            )
            .into_any_element();
    };

    let is_owner = sidebar
        .current_user
        .as_ref()
        .map_or(false, |u| u.id as i32 == room.owner_id);
    let status_label = match room.status {
        RoomStatus::Lobby => "待开始",
        RoomStatus::Running => "对局中",
        RoomStatus::Closed => "已结束",
    };
    let status_color = match room.status {
        RoomStatus::Lobby => accent,
        RoomStatus::Running => gpui::hsla(0.4, 0.8, 0.5, 1.0),
        RoomStatus::Closed => muted,
    };
    let c = room.constraints;
    let team_policy_label = match c.team_policy {
        TeamPolicy::SingleTeam => "单阵营策略",
        TeamPolicy::Free => "自由阵营",
    };

    // ── 顶部：返回 + 标题 + 状态徽章 + 刷新 ──
    let header = h_flex()
        .items_center()
        .justify_between()
        .child(
            h_flex()
                .items_center()
                .gap_2()
                .child(
                    Button::new("room-detail-back-btn")
                        .ghost()
                        .icon(IconName::ArrowLeft)
                        .tooltip("返回房间列表")
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.current_room_id = None;
                            this.navigate_to(ActiveView::Rooms);
                            cx.notify();
                        })),
                )
                .child(div().text_lg().font_bold().child(room.name.clone()))
                .child(
                    div()
                        .px_2()
                        .py_0p5()
                        .rounded_md()
                        .text_xs()
                        .font_bold()
                        .bg(status_color.opacity(0.15))
                        .text_color(status_color)
                        .child(status_label),
                ),
        )
        .child(
            Button::new("room-detail-refresh-btn")
                .outline()
                .icon(IconName::Redo)
                .label("刷新")
                .on_click(cx.listener(move |_, _, _, cx| {
                    spawn_load(cx, id);
                })),
        );

    // ── 一行约束 chips + 邀请码 ──
    let chips = h_flex()
        .flex_wrap()
        .gap_4()
        .text_xs()
        .text_color(muted)
        .child(format!(
            "{} / {} 成员",
            room.member_count.unwrap_or(0),
            c.max_members
        ))
        .child(format!("每人最多 {} 个 Agent", c.max_agents_per_member))
        .child(team_policy_label.to_string())
        .child(if c.lobby_visible {
            "大厅公开"
        } else {
            "邀请码加入"
        })
        .child(if c.prompt_visible {
            "Prompt 公开"
        } else {
            "Prompt 隐藏"
        })
        .child(
            h_flex()
                .gap_1()
                .items_center()
                .child(
                    div()
                        .font_family("monospace")
                        .font_bold()
                        .text_color(accent)
                        .child(room.invite_code.clone()),
                )
                .child(
                    Clipboard::new("room-invite-copy")
                        .value(room.invite_code.clone())
                        .tooltip("复制邀请码"),
                ),
        );

    let divider = || div().w_full().h_px().bg(border);

    // ── 双阵营槽位列 ──
    let columns = h_flex()
        .gap_4()
        .items_start()
        .child(render_team_column(
            cx,
            id,
            Team::Order,
            "Order · 蓝色方",
            gpui::hsla(0.6, 0.7, 0.5, 1.0),
            &slots,
            &agents,
        ))
        .child(render_team_column(
            cx,
            id,
            Team::Chaos,
            "Chaos · 红色方",
            gpui::hsla(0.0, 0.7, 0.5, 1.0),
            &slots,
            &agents,
        ));

    // ── 底部操作：离开 / 解散 / 开始对局 ──
    let footer = h_flex()
        .items_center()
        .justify_between()
        .child(
            h_flex()
                .gap_2()
                .child(
                    Button::new("room-leave-btn")
                        .ghost()
                        .label("离开房间")
                        .on_click(cx.listener(move |_, _, _, cx| {
                            spawn_leave_room(cx, id);
                        })),
                )
                .when(is_owner, |d| {
                    d.child(
                        Button::new("room-dissolve-btn")
                            .ghost()
                            .danger()
                            .label("解散房间")
                            .on_click(cx.listener(move |_, _, _, cx| {
                                spawn_dissolve_room(cx, id);
                            })),
                    )
                }),
        )
        .child(
            Button::new("room-start-match-btn")
                .primary()
                .icon(if starting {
                    IconName::Loader
                } else {
                    IconName::Play
                })
                .label(if starting {
                    "启动中…"
                } else {
                    "开始对局"
                })
                .disabled(slots.is_empty() || starting)
                .on_click(cx.listener(move |_, _, _, cx| {
                    spawn_start_match(cx, id);
                })),
        );

    v_flex()
        .size_full()
        .flex_1()
        .gap_5()
        .overflow_hidden()
        .when_some(error.as_ref(), |d, err| {
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
        .child(header)
        .child(chips)
        .child(divider())
        .child(
            div()
                .flex_1()
                .min_h_0()
                .overflow_y_scrollbar()
                .child(columns),
        )
        .child(divider())
        .child(footer)
        .child(render_add_dialog(cx, id, &agents))
        .into_any_element()
}
