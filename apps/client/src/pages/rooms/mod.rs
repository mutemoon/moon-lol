//! 房间页 — 大厅浏览 / 我的房间 / 邀请码加入 / 创建房间。

mod input;
mod logic;
mod types;
mod ui;

use gpui::prelude::*;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::scroll::ScrollableElement;
use gpui_component::{h_flex, v_flex, ActiveTheme, Disableable, IconName, StyledExt};

use self::input::render_state_input;
use self::logic::{spawn_refresh_rooms, try_join_by_code};
use self::types::{update_state, with_state, RoomsTab};
use self::ui::{create_room_dialog, room_card, tab_button};
use crate::components::sidebar::AppSidebar;

/// 房间列表/创建/加入/解散页面。
pub fn render_rooms(
    sidebar: &mut AppSidebar,
    window: &mut Window,
    cx: &mut Context<AppSidebar>,
) -> AnyElement {
    let logged_in = sidebar.auth_token.is_some();

    // 首次渲染自动加载大厅 + 我的房间
    if !with_state(|s| s.loaded) {
        update_state(|s| {
            s.loaded = true;
            s.loading = true;
        });
        spawn_refresh_rooms(cx);
    }

    let (
        lobby_count,
        my_count,
        loading,
        active_tab,
        join_error,
        joining,
        show_create,
        creating,
        create_error,
    ) = with_state(|s| {
        (
            s.lobby_rooms.len(),
            s.my_rooms.len(),
            s.loading,
            s.active_tab,
            s.join_error.clone(),
            s.joining,
            s.show_create,
            s.creating,
            s.create_error.clone(),
        )
    });
    let lobby_rooms = with_state(|s| s.lobby_rooms.clone());
    let my_rooms = with_state(|s| s.my_rooms.clone());

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
                        .child(IconName::User)
                        .child(div().font_bold().text_lg().child("房间")),
                )
                .child(
                    h_flex()
                        .gap_2()
                        .child(
                            Button::new("rooms-refresh-btn")
                                .outline()
                                .icon(IconName::Loader)
                                .label("刷新列表")
                                .disabled(!logged_in || loading)
                                .on_click({
                                    cx.listener(|_, _, _, cx| {
                                        spawn_refresh_rooms(cx);
                                    })
                                }),
                        )
                        .child(
                            Button::new("create-room-btn")
                                .primary()
                                .icon(IconName::Plus)
                                .label("创建房间")
                                .disabled(!logged_in)
                                .on_click(cx.listener(|_, _, _, cx| {
                                    update_state(|s| {
                                        s.show_create = true;
                                        s.create_error.clear();
                                    });
                                    cx.notify();
                                })),
                        ),
                ),
        )
        // ── 加入码输入 ──
        .child(
            h_flex().gap_3().items_end().child(
                v_flex()
                    .gap_1()
                    .w_64()
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child("邀请码加入"),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .child(div().flex_1().child(render_state_input(
                                window,
                                cx,
                                "rooms-join-code",
                                "ABCD1234",
                                || with_state(|s| s.join_code.clone()),
                                |v| update_state(|s| s.join_code = v),
                                Some(Box::new(|cx| try_join_by_code(cx))),
                            )))
                            .child(
                                Button::new("rooms-join-btn")
                                    .secondary()
                                    .label("加入")
                                    .disabled(!logged_in || joining)
                                    .on_click(cx.listener(|_, _, _, cx| {
                                        try_join_by_code(cx);
                                    })),
                            ),
                    )
                    .when(!join_error.is_empty(), |d| {
                        d.child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().danger)
                                .child(join_error),
                        )
                    }),
            ),
        )
        // ── Tab 栏 ──
        .child(
            h_flex()
                .gap_2()
                .child(tab_button(
                    cx,
                    "大厅",
                    lobby_count,
                    active_tab == RoomsTab::Lobby,
                    |_, _, _window, cx| {
                        update_state(|s| s.active_tab = RoomsTab::Lobby);
                        cx.notify();
                    },
                ))
                .child(tab_button(
                    cx,
                    "我的房间",
                    my_count,
                    active_tab == RoomsTab::Mine,
                    |_, _, _window, cx| {
                        update_state(|s| s.active_tab = RoomsTab::Mine);
                        cx.notify();
                    },
                )),
        )
        // ── 房间卡片网格 ──
        .child(
            div()
                .flex_1()
                .overflow_y_scrollbar()
                .when(loading, |d| {
                    d.flex().items_center().justify_center().child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child("加载中..."),
                    )
                })
                .when(
                    !loading && active_tab == RoomsTab::Lobby && lobby_count == 0,
                    |d| {
                        d.flex().items_center().justify_center().child(
                            div()
                                .text_sm()
                                .text_color(cx.theme().muted_foreground)
                                .child("大厅当前没有公开房间"),
                        )
                    },
                )
                .when(
                    !loading && active_tab == RoomsTab::Mine && my_count == 0,
                    |d| {
                        d.flex().items_center().justify_center().child(
                            div()
                                .text_sm()
                                .text_color(cx.theme().muted_foreground)
                                .child("你还没有加入任何房间"),
                        )
                    },
                )
                .when(!loading, |d| {
                    let rooms = if active_tab == RoomsTab::Lobby {
                        lobby_rooms
                    } else {
                        my_rooms
                    };
                    d.child(
                        h_flex()
                            .gap_3()
                            .flex_wrap()
                            .children(rooms.into_iter().map(|r| room_card(cx, &r, logged_in))),
                    )
                }),
        )
        // ── 创建房间对话框 ──
        .when(show_create, |d| {
            d.child(create_room_dialog(window, cx, create_error, creating))
        })
        .into_any_element()
}
