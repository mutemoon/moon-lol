//! 房间页视图：Tab 按钮 / 房间卡片 / 创建房间对话框。

use gpui::prelude::*;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::checkbox::Checkbox;
use gpui_component::menu::{DropdownMenu, PopupMenuItem};
use gpui_component::{h_flex, v_flex, ActiveTheme, Disableable, IconName, StyledExt};
use lol_web_protocol::room::{Room, RoomStatus, TeamPolicy};

use super::input::render_state_input;
use super::logic::{spawn_join_or_enter_room, try_create_room};
use super::types::{update_state, with_state};
use crate::components::sidebar::AppSidebar;

pub(super) fn tab_button(
    cx: &mut Context<AppSidebar>,
    label: &str,
    count: usize,
    active: bool,
    on_click: impl Fn(&mut AppSidebar, &ClickEvent, &mut Window, &mut Context<AppSidebar>) + 'static,
) -> AnyElement {
    let label_owned = label.to_string();
    Button::new(format!("rooms-tab-{}", label.to_lowercase()))
        .when(active, |b| b.primary())
        .when(!active, |b| b.outline())
        .child(
            h_flex().gap_1().items_center().child(label_owned).child(
                div()
                    .px_1p5()
                    .py_0p5()
                    .rounded_full()
                    .text_xs()
                    .font_bold()
                    .bg(if active {
                        gpui::white()
                    } else {
                        cx.theme().accent.opacity(0.2)
                    })
                    .child(format!("{}", count)),
            ),
        )
        .on_click(cx.listener(on_click))
        .into_any_element()
}

pub(super) fn room_card(cx: &mut Context<AppSidebar>, room: &Room, _logged_in: bool) -> AnyElement {
    let status_label = match room.status {
        RoomStatus::Lobby => "待开始",
        RoomStatus::Running => "对局中",
        RoomStatus::Closed => "已结束",
    };
    let status_color = match room.status {
        RoomStatus::Lobby => cx.theme().accent,
        RoomStatus::Running => gpui::hsla(0.4, 0.8, 0.5, 1.0),
        RoomStatus::Closed => cx.theme().muted_foreground,
    };
    let team_policy_label = match room.constraints.team_policy {
        TeamPolicy::SingleTeam => "单阵营",
        TeamPolicy::Free => "自由阵营",
    };

    let room_id = room.id.to_string();
    let is_my_room = with_state(|s| s.my_rooms.iter().any(|r| r.id == room.id));

    div()
        .rounded_lg()
        .border_1()
        .border_color(cx.theme().border)
        .p_4()
        .flex()
        .flex_col()
        .gap_2()
        .w_64()
        .hover(|s| s.bg(cx.theme().accent.opacity(0.03)))
        .child(
            h_flex()
                .items_start()
                .justify_between()
                .child(div().font_bold().text_sm().child(room.name.clone()))
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
            h_flex()
                .gap_3()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child(format!(
                    "{} / {} 人",
                    room.member_count.unwrap_or(0),
                    room.constraints.max_members
                ))
                .child(team_policy_label),
        )
        .child(
            div()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child(format!("邀请码: {}", room.invite_code)),
        )
        .when(room.status == RoomStatus::Running, |d| {
            d.child(
                h_flex()
                    .gap_1()
                    .items_center()
                    .child(
                        div()
                            .w_1p5()
                            .h_1p5()
                            .rounded_full()
                            .bg(gpui::hsla(0.4, 0.8, 0.5, 1.0)),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().accent)
                            .child("对局进行中"),
                    ),
            )
        })
        .child(
            h_flex().gap_1().pt_1().child(
                Button::new(format!("room-join-{}", room_id))
                    .when(is_my_room, |b| b.outline().label("进入"))
                    .when(!is_my_room, |b| b.primary().label("加入"))
                    .on_click({
                        let rid = room_id.clone();
                        cx.listener(move |_, _, _, cx| {
                            let room_id = rid.clone();
                            spawn_join_or_enter_room(cx, &room_id);
                        })
                    }),
            ),
        )
        .into_any_element()
}

pub(super) fn create_room_dialog(
    window: &mut Window,
    cx: &mut Context<AppSidebar>,
    create_error: String,
    creating: bool,
) -> AnyElement {
    let (draft_team_policy, draft_lobby_visible) =
        with_state(|s| (s.draft_team_policy.clone(), s.draft_lobby_visible));
    let team_policy_free = draft_team_policy != "single_team";
    let team_policy_label = if team_policy_free {
        "自由（红蓝皆可）"
    } else {
        "单阵营（每人只能在一方）"
    };
    let weak = cx.entity().downgrade();
    let free_weak = weak.clone();
    let single_weak = weak.clone();
    let checkbox_weak = weak.clone();

    // 遮罩 + 居中对话框
    div()
        .absolute()
        .inset_0()
        .bg(gpui::black().opacity(0.4))
        .flex()
        .items_center()
        .justify_center()
        .on_any_mouse_down(cx.listener(|_, _, _, cx| {
            update_state(|s| s.show_create = false);
            cx.notify();
        }))
        .child(
            div()
                .rounded_lg()
                .border_1()
                .border_color(cx.theme().border)
                .bg(cx.theme().background)
                .p_6()
                .w_96()
                .flex()
                .flex_col()
                .gap_4()
                .on_any_mouse_down(|_, _, _| {}) // 阻止冒泡
                .child(
                    h_flex()
                        .items_center()
                        .justify_between()
                        .child(div().font_bold().text_sm().child("创建房间"))
                        .child(
                            Button::new("close-create-room")
                                .ghost()
                                .icon(IconName::Close)
                                .on_click(cx.listener(|_, _, _, cx| {
                                    update_state(|s| s.show_create = false);
                                    cx.notify();
                                })),
                        ),
                )
                .child(
                    v_flex()
                        .gap_3()
                        // 房间名称
                        .child(
                            v_flex()
                                .gap_1()
                                .child(div().text_xs().child("房间名称"))
                                .child(render_state_input(
                                    window,
                                    cx,
                                    "create-room-name",
                                    "周末野队挑战",
                                    || with_state(|s| s.draft_name.clone()),
                                    |v| update_state(|s| s.draft_name = v),
                                    None,
                                )),
                        )
                        // 最大人数 / 每人 Agent 上限
                        .child(
                            h_flex()
                                .gap_3()
                                .child(
                                    v_flex()
                                        .gap_1()
                                        .flex_1()
                                        .child(div().text_xs().child("最大人数"))
                                        .child(render_state_input(
                                            window,
                                            cx,
                                            "create-max-members",
                                            "10",
                                            || with_state(|s| s.draft_max_members.clone()),
                                            |v| {
                                                update_state(|s| {
                                                    s.draft_max_members = v
                                                        .chars()
                                                        .filter(|c| c.is_ascii_digit())
                                                        .collect();
                                                })
                                            },
                                            None,
                                        )),
                                )
                                .child(
                                    v_flex()
                                        .gap_1()
                                        .flex_1()
                                        .child(div().text_xs().child("每人 Agent 上限"))
                                        .child(render_state_input(
                                            window,
                                            cx,
                                            "create-max-agents",
                                            "3",
                                            || with_state(|s| s.draft_max_agents.clone()),
                                            |v| {
                                                update_state(|s| {
                                                    s.draft_max_agents = v
                                                        .chars()
                                                        .filter(|c| c.is_ascii_digit())
                                                        .collect();
                                                })
                                            },
                                            None,
                                        )),
                                ),
                        )
                        // 阵营策略
                        .child(
                            v_flex()
                                .gap_1()
                                .child(div().text_xs().child("阵营策略"))
                                .child(
                                    Button::new("create-team-policy")
                                        .label(team_policy_label)
                                        .icon(IconName::ChevronDown)
                                        .outline()
                                        .w_full()
                                        .dropdown_menu(move |menu, _window, _cx| {
                                            let free_weak = free_weak.clone();
                                            let single_weak = single_weak.clone();
                                            menu.item(
                                                PopupMenuItem::new("自由（红蓝皆可）")
                                                    .checked(team_policy_free)
                                                    .on_click(move |_, _, cx| {
                                                        update_state(|s| {
                                                            s.draft_team_policy = "free".into();
                                                        });
                                                        let _ = free_weak
                                                            .update(cx, |_, cx| cx.notify());
                                                    }),
                                            )
                                            .item(
                                                PopupMenuItem::new("单阵营（每人只能在一方）")
                                                    .checked(!team_policy_free)
                                                    .on_click(move |_, _, cx| {
                                                        update_state(|s| {
                                                            s.draft_team_policy =
                                                                "single_team".into();
                                                        });
                                                        let _ = single_weak
                                                            .update(cx, |_, cx| cx.notify());
                                                    }),
                                            )
                                        }),
                                ),
                        )
                        // 公开到大厅
                        .child(
                            Checkbox::new("create-lobby-visible")
                                .checked(draft_lobby_visible)
                                .label("公开到大厅")
                                .on_click(move |new_checked, _, cx| {
                                    update_state(|s| s.draft_lobby_visible = *new_checked);
                                    let _ = checkbox_weak.update(cx, |_, cx| cx.notify());
                                }),
                        ),
                )
                .when(!create_error.is_empty(), |d| {
                    d.child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().danger)
                            .child(create_error),
                    )
                })
                .child(
                    h_flex()
                        .gap_2()
                        .justify_end()
                        .child(
                            Button::new("cancel-create-room")
                                .ghost()
                                .label("取消")
                                .on_click(cx.listener(|_, _, _, cx| {
                                    update_state(|s| s.show_create = false);
                                    cx.notify();
                                })),
                        )
                        .child(
                            Button::new("confirm-create-room")
                                .primary()
                                .label("创建")
                                .disabled(creating)
                                .on_click(cx.listener(|_, _, _, cx| {
                                    try_create_room(cx);
                                })),
                        ),
                ),
        )
        .into_any_element()
}
