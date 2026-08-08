use std::cell::RefCell;
use std::collections::HashMap;

use gpui::prelude::*;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::checkbox::Checkbox;
use gpui_component::menu::{DropdownMenu, PopupMenuItem};
use gpui_component::scroll::ScrollableElement;
use gpui_component::{h_flex, v_flex, ActiveTheme, Disableable, IconName, StyledExt};
use lol_web_protocol::room::{Room, RoomConstraints, RoomStatus, TeamPolicy};

use crate::components::sidebar::AppSidebar;
use crate::services::provider::cloud_client;

// ── 房间页面临时状态 ──

#[derive(Debug, Clone, Default)]
struct RoomsPageState {
    /// 是否已触发首次自动加载
    loaded: bool,
    lobby_rooms: Vec<Room>,
    my_rooms: Vec<Room>,
    loading: bool,
    active_tab: RoomsTab,
    // 加入码
    join_code: String,
    join_error: String,
    joining: bool,
    // 创建房间
    show_create: bool,
    creating: bool,
    create_error: String,
    draft_name: String,
    draft_max_members: String,
    draft_max_agents: String,
    draft_team_policy: String,
    draft_lobby_visible: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RoomsTab {
    Lobby,
    Mine,
}

impl Default for RoomsTab {
    fn default() -> Self {
        RoomsTab::Lobby
    }
}

thread_local! {
    static STATE: RefCell<RoomsPageState> = RefCell::new(RoomsPageState {
        draft_max_members: "10".into(),
        draft_max_agents: "3".into(),
        draft_team_policy: "free".into(),
        draft_lobby_visible: true,
        ..Default::default()
    });
}

fn with_state<R>(f: impl FnOnce(&RoomsPageState) -> R) -> R {
    STATE.with(|s| f(&s.borrow()))
}

fn update_state(f: impl FnOnce(&mut RoomsPageState)) {
    STATE.with(|s| f(&mut s.borrow_mut()));
}

// ── 可编辑文本输入（焦点/光标跨渲染保持） ──

#[derive(Clone)]
struct EditMeta {
    cursor: usize,
    focus: FocusHandle,
}

thread_local! {
    static EDITS: RefCell<HashMap<String, EditMeta>> = RefCell::new(HashMap::new());
}

fn edit_meta(id: &str, cx: &App) -> EditMeta {
    EDITS.with(|m| {
        let mut m = m.borrow_mut();
        if let Some(meta) = m.get(id) {
            return meta.clone();
        }
        let meta = EditMeta {
            cursor: 0,
            focus: cx.focus_handle(),
        };
        m.insert(id.to_string(), meta.clone());
        meta
    })
}

fn edit_cursor(id: &str) -> usize {
    EDITS.with(|m| m.borrow().get(id).map_or(0, |e| e.cursor))
}

fn set_edit_cursor(id: &str, cursor: usize) {
    EDITS.with(|m| {
        if let Some(e) = m.borrow_mut().get_mut(id) {
            e.cursor = cursor;
        }
    })
}

/// 处理单个按键，返回（新文本，新光标）。无变化返回 None。
fn apply_key(value: &str, cursor: usize, event: &KeyDownEvent) -> Option<(String, usize)> {
    let ks = &event.keystroke;
    let mods = &ks.modifiers;
    let mut chars: Vec<char> = value.chars().collect();
    let cursor = cursor.min(chars.len());

    if mods.control || mods.platform {
        return None;
    }

    if let Some(ch) = ks.key_char.as_deref() {
        let insert_chars: Vec<char> = ch.chars().collect();
        if !mods.alt && !insert_chars.is_empty() && !insert_chars.iter().any(|c| c.is_control()) {
            for (i, c) in insert_chars.iter().enumerate() {
                chars.insert(cursor + i, *c);
            }
            return Some((chars.into_iter().collect(), cursor + insert_chars.len()));
        }
    }

    match ks.key.as_str() {
        "backspace" => {
            if cursor > 0 {
                chars.remove(cursor - 1);
                Some((chars.into_iter().collect(), cursor - 1))
            } else {
                None
            }
        }
        "delete" => {
            if cursor < chars.len() {
                chars.remove(cursor);
                Some((chars.into_iter().collect(), cursor))
            } else {
                None
            }
        }
        "left" => Some((value.to_string(), cursor.saturating_sub(1))),
        "right" => Some((value.to_string(), (cursor + 1).min(chars.len()))),
        "home" => Some((value.to_string(), 0)),
        "end" => Some((value.to_string(), chars.len())),
        "space" => {
            chars.insert(cursor, ' ');
            Some((chars.into_iter().collect(), cursor + 1))
        }
        _ => None,
    }
}

/// 可聚焦、可键盘编辑的文本输入框，读写页面 thread_local 状态。
/// 复用 community.rs 的手写输入框手法（gpui_component Input 需要 &mut Window）。
fn render_state_input(
    cx: &mut Context<AppSidebar>,
    id: &str,
    placeholder: &str,
    get_value: impl Fn() -> String + 'static,
    set_value: impl Fn(String) + 'static,
    on_enter: Option<Box<dyn Fn(&mut Context<AppSidebar>)>>,
) -> AnyElement {
    let value = get_value();
    let meta = edit_meta(id, cx);
    let focus_handle = meta.focus.clone();
    let empty = value.is_empty();
    let chars: Vec<char> = value.chars().collect();
    let cursor = meta.cursor.min(chars.len());
    let before: String = chars[..cursor].iter().collect();
    let after: String = chars[cursor..].iter().collect();
    let accent = cx.theme().accent;
    let muted = cx.theme().muted_foreground;
    let id_owned = id.to_string();

    let listener = cx.listener(move |_this, event: &KeyDownEvent, _window, cx| {
        if event.keystroke.key.as_str() == "enter" {
            if let Some(f) = on_enter.as_ref() {
                f(cx);
            }
            return;
        }
        let live = get_value();
        let cur = edit_cursor(&id_owned);
        if let Some((nv, nc)) = apply_key(&live, cur, event) {
            set_value(nv);
            set_edit_cursor(&id_owned, nc);
            cx.notify();
        }
    });

    div()
        .track_focus(&focus_handle)
        .px_2()
        .py_1()
        .w_full()
        .border_1()
        .border_color(cx.theme().border)
        .rounded_md()
        .bg(cx.theme().background)
        .text_sm()
        .on_key_down(listener)
        .child(
            h_flex()
                .items_center()
                .when(empty, |d| {
                    d.text_color(muted).child(placeholder.to_string())
                })
                .when(!empty, |d| {
                    d.child(before)
                        .child(div().w(px(1.)).h(rems(1.)).bg(accent))
                        .child(after)
                }),
        )
        .into_any_element()
}

/// 房间列表/创建/加入/解散页面。
pub fn render_rooms(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) -> AnyElement {
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
            d.child(create_room_dialog(cx, create_error, creating))
        })
        .into_any_element()
}

fn tab_button(
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

fn room_card(cx: &mut Context<AppSidebar>, room: &Room, _logged_in: bool) -> AnyElement {
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

fn create_room_dialog(
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

// ── 辅助：加入 / 创建 ──

fn try_join_by_code(cx: &mut Context<AppSidebar>) {
    let code = with_state(|s| s.join_code.trim().to_uppercase());
    if code.is_empty() {
        update_state(|s| s.join_error = "请输入邀请码".into());
        cx.notify();
        return;
    }
    update_state(|s| {
        s.join_error.clear();
        s.joining = true;
    });
    spawn_join_room_by_code(cx, &code);
}

fn try_create_room(cx: &mut Context<AppSidebar>) {
    let (name, constraints) = with_state(|s| {
        let name = s.draft_name.trim().to_string();
        let max_members = s
            .draft_max_members
            .trim()
            .parse::<i32>()
            .unwrap_or(10)
            .clamp(2, 20);
        let max_agents = s
            .draft_max_agents
            .trim()
            .parse::<i32>()
            .unwrap_or(3)
            .clamp(1, 10);
        let team_policy = if s.draft_team_policy == "single_team" {
            TeamPolicy::SingleTeam
        } else {
            TeamPolicy::Free
        };
        let constraints = RoomConstraints {
            max_members,
            max_agents_per_member: max_agents,
            team_policy,
            lobby_visible: s.draft_lobby_visible,
            prompt_visible: false,
        };
        (name, constraints)
    });
    if name.is_empty() {
        update_state(|s| s.create_error = "请填写房间名称".into());
        cx.notify();
        return;
    }
    update_state(|s| s.creating = true);
    spawn_create_room(cx, &name, constraints);
}

// ── 辅助：spawn 刷新房间列表 ──

fn spawn_refresh_rooms(cx: &mut Context<AppSidebar>) {
    cx.spawn(
        move |weak: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
            let weak = weak.clone();
            let mut cx = cx.clone();
            async move {
                if let Some(entity) = weak.upgrade() {
                    refresh_rooms(&mut cx, &entity).await;
                }
            }
        },
    )
    .detach();
}

fn spawn_join_room_by_code(cx: &mut Context<AppSidebar>, code: &str) {
    let code = code.to_string();
    cx.spawn(
        move |weak: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
            let weak = weak.clone();
            let mut cx = cx.clone();
            let code = code.clone();
            async move {
                let result = cloud_client().join_room_by_code(&code).await;
                update_state(|s| s.joining = false);
                match result {
                    Ok(_room) => {
                        if let Some(entity) = weak.upgrade() {
                            refresh_rooms(&mut cx, &entity).await;
                        }
                    }
                    Err(e) => {
                        update_state(|s| s.join_error = format!("加入失败: {}", e));
                    }
                }
                if let Some(entity) = weak.upgrade() {
                    entity.update(&mut cx, |_, cx| cx.notify());
                }
            }
        },
    )
    .detach();
}

fn spawn_join_or_enter_room(cx: &mut Context<AppSidebar>, room_id: &str) {
    let room_id = room_id.to_string();
    cx.spawn(
        move |weak: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
            let weak = weak.clone();
            let mut cx = cx.clone();
            let room_id = room_id.clone();
            async move {
                let is_member =
                    with_state(|s| s.my_rooms.iter().any(|r| r.id.to_string() == room_id));
                if !is_member {
                    let _ = cloud_client().join_room(&room_id).await;
                }
                if let Some(entity) = weak.upgrade() {
                    refresh_rooms(&mut cx, &entity).await;
                    entity.update(&mut cx, |_, cx| cx.notify());
                }
            }
        },
    )
    .detach();
}

fn spawn_create_room(cx: &mut Context<AppSidebar>, room_name: &str, constraints: RoomConstraints) {
    let room_name = room_name.to_string();
    cx.spawn(
        move |weak: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
            let weak = weak.clone();
            let mut cx = cx.clone();
            let room_name = room_name.clone();
            async move {
                let result = cloud_client().create_room(&room_name, &constraints).await;
                update_state(|s| s.creating = false);
                match result {
                    Ok(_) => {
                        update_state(|s| {
                            s.show_create = false;
                            s.draft_name.clear();
                            s.active_tab = RoomsTab::Mine;
                        });
                        if let Some(entity) = weak.upgrade() {
                            refresh_rooms(&mut cx, &entity).await;
                        }
                    }
                    Err(e) => {
                        update_state(|s| s.create_error = format!("创建失败: {}", e));
                    }
                }
                if let Some(entity) = weak.upgrade() {
                    entity.update(&mut cx, |_, cx| cx.notify());
                }
            }
        },
    )
    .detach();
}

// ── 辅助：刷新房间列表 ──

async fn refresh_rooms(cx: &mut AsyncApp, entity: &Entity<AppSidebar>) {
    update_state(|s| s.loading = true);

    let (mine, lobby) = tokio::join!(
        async {
            cloud_client()
                .list_my_rooms()
                .await
                .unwrap_or_else(|_| Vec::new())
        },
        async {
            cloud_client()
                .list_lobby_rooms()
                .await
                .unwrap_or_else(|_| Vec::new())
        },
    );

    update_state(|s| {
        s.my_rooms = mine;
        s.lobby_rooms = lobby;
        s.loading = false;
    });

    entity.update(cx, |_, cx| cx.notify());
}
