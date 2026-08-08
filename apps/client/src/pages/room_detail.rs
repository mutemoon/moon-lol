use std::cell::RefCell;
use std::time::Duration;

use gpui::prelude::*;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::clipboard::Clipboard;
use gpui_component::menu::{DropdownMenu, PopupMenuItem};
use gpui_component::scroll::ScrollableElement;
use gpui_component::{h_flex, v_flex, ActiveTheme, Disableable, IconName, Sizable, StyledExt};
use lol_web_protocol::agent::Agent;
use lol_web_protocol::room::{Room, RoomAgentSlot, RoomStatus, TeamPolicy};
use lol_web_protocol::spawn_preset::Team;
use uuid::Uuid;

use crate::components::sidebar::AppSidebar;
use crate::services::provider::cloud_client;
use crate::services::runtime::run_on_tokio;
use crate::types::ActiveView;

/// 轮询间隔（秒）
const POLL_INTERVAL_SECS: u64 = 5;

// ── 页面本地状态 ──

struct RoomDetailPageState {
    /// 状态绑定的房间 id；与 sidebar.current_room_id 不一致时重置
    room_id: Option<Uuid>,
    /// 是否已触发首次加载
    inited: bool,
    /// 是否已启动轮询循环（防重复 spawn）
    polling: bool,
    /// 首次加载中
    loading: bool,
    room: Option<Room>,
    slots: Vec<RoomAgentSlot>,
    /// agent 列表（用于槽位名称解析与「添加槽位」下拉）
    agents: Vec<Agent>,
    error: Option<String>,
    /// 非 None 表示「添加槽位」对话框打开，值为目标阵营
    show_add_team: Option<Team>,
    add_agent_id: Option<String>,
    adding: bool,
    add_error: String,
    /// 开始对局请求进行中
    starting: bool,
}

impl Default for RoomDetailPageState {
    fn default() -> Self {
        Self {
            room_id: None,
            inited: false,
            polling: false,
            loading: false,
            room: None,
            slots: Vec::new(),
            agents: Vec::new(),
            error: None,
            show_add_team: None,
            add_agent_id: None,
            adding: false,
            add_error: String::new(),
            starting: false,
        }
    }
}

thread_local! {
    static STATE: RefCell<RoomDetailPageState> = RefCell::new(RoomDetailPageState::default());
}

fn with_state<R>(f: impl FnOnce(&RoomDetailPageState) -> R) -> R {
    STATE.with(|s| f(&s.borrow()))
}

fn update_state(f: impl FnOnce(&mut RoomDetailPageState)) {
    STATE.with(|s| f(&mut s.borrow_mut()));
}

/// 将状态绑定到当前房间；id 变化时清空旧数据。
fn reset_state_for(room_id: Option<Uuid>) {
    update_state(|s| {
        if s.room_id != room_id {
            *s = RoomDetailPageState {
                room_id,
                ..RoomDetailPageState::default()
            };
        }
    });
}

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

// ── 数据加载 ──

/// 并行拉取房间信息 + 槽位 + agent 列表并写回状态。
async fn fetch_room_data(id: Uuid, weak: &gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp) {
    // 中途已切换房间则丢弃本次结果
    if with_state(|s| s.room_id) != Some(id) {
        return;
    }
    let client = cloud_client().clone();
    let id_a = id.to_string();
    let id_b = id_a.clone();
    let (r, slots, agents) = tokio::join!(
        async { client.get_room(&id_a).await },
        async { client.list_room_slots(&id_b).await.unwrap_or_default() },
        async { client.list_agents().await.unwrap_or_default() },
    );
    update_state(|s| {
        s.loading = false;
        match r {
            Ok(room) => {
                s.room = Some(room);
                s.error = None;
            }
            Err(e) => s.error = Some(e.to_string()),
        }
        s.slots = slots;
        s.agents = agents;
    });
    if let Some(entity) = weak.upgrade() {
        let _ = entity.update(cx, |_, cx| cx.notify());
    }
}

fn spawn_load(cx: &mut Context<AppSidebar>, id: Uuid) {
    cx.spawn(
        move |weak: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
            let weak = weak.clone();
            let mut cx = cx.clone();
            async move {
                fetch_room_data(id, &weak, &mut cx).await;
            }
        },
    )
    .detach();
}

/// 5 秒轮询。睡眠必须走 run_on_tokio（gpui executor 非 tokio runtime，直接 sleep 会 panic）。
fn spawn_poll(cx: &mut Context<AppSidebar>, id: Uuid) {
    cx.spawn(
        move |weak: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
            let weak = weak.clone();
            let mut cx = cx.clone();
            async move {
                loop {
                    if run_on_tokio(|| async {
                        tokio::time::sleep(Duration::from_secs(POLL_INTERVAL_SECS)).await;
                        Ok::<(), String>(())
                    })
                    .await
                    .is_err()
                    {
                        break;
                    }
                    if with_state(|s| s.room_id) != Some(id) {
                        break;
                    }
                    // 已离开详情页则停止轮询
                    let still_detail = weak.upgrade().map_or(false, |e| {
                        e.read_with(&cx, |s, _| s.active_view == ActiveView::RoomDetail)
                    });
                    if !still_detail {
                        break;
                    }
                    fetch_room_data(id, &weak, &mut cx).await;
                }
            }
        },
    )
    .detach();
}

// ── 异步动作 ──

/// 添加槽位：校验已选 agent 后调用 add_room_slot。
fn spawn_add_slot(cx: &mut Context<AppSidebar>, room_id: Uuid) {
    let (agent_id, team) = with_state(|s| (s.add_agent_id.clone(), s.show_add_team));
    let Some(agent_id) = agent_id else {
        update_state(|s| s.add_error = "请选择 Agent".into());
        cx.notify();
        return;
    };
    let Some(team) = team else {
        return;
    };
    update_state(|s| {
        s.adding = true;
        s.add_error.clear();
    });
    let id_str = room_id.to_string();
    cx.spawn(
        move |weak: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
            let weak = weak.clone();
            let mut cx = cx.clone();
            let id_str = id_str.clone();
            let agent_id = agent_id.clone();
            async move {
                match cloud_client().add_room_slot(&id_str, &agent_id, team).await {
                    Ok(_) => {
                        update_state(|s| {
                            s.adding = false;
                            s.show_add_team = None;
                            s.add_agent_id = None;
                            s.add_error.clear();
                        });
                        fetch_room_data(room_id, &weak, &mut cx).await;
                    }
                    Err(e) => update_state(|s| {
                        s.adding = false;
                        s.add_error = format!("添加失败: {}", e);
                    }),
                }
                if let Some(entity) = weak.upgrade() {
                    let _ = entity.update(&mut cx, |_, cx| cx.notify());
                }
            }
        },
    )
    .detach();
}

fn spawn_remove_slot(cx: &mut Context<AppSidebar>, room_id: Uuid, slot_id: Uuid) {
    let id_str = room_id.to_string();
    let slot_id_str = slot_id.to_string();
    cx.spawn(
        move |weak: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
            let weak = weak.clone();
            let mut cx = cx.clone();
            let id_str = id_str.clone();
            let slot_id_str = slot_id_str.clone();
            async move {
                if let Err(e) = cloud_client().remove_room_slot(&id_str, &slot_id_str).await {
                    update_state(|s| s.error = Some(e.to_string()));
                }
                fetch_room_data(room_id, &weak, &mut cx).await;
            }
        },
    )
    .detach();
}

fn spawn_start_match(cx: &mut Context<AppSidebar>, room_id: Uuid) {
    update_state(|s| {
        s.starting = true;
        s.error = None;
    });
    let id_str = room_id.to_string();
    cx.spawn(
        move |weak: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
            let weak = weak.clone();
            let mut cx = cx.clone();
            let id_str = id_str.clone();
            async move {
                match cloud_client().start_room_match(&id_str).await {
                    Ok(res) => {
                        update_state(|s| s.starting = false);
                        if let Some(entity) = weak.upgrade() {
                            let _ = entity.update(&mut cx, |this, cx| {
                                this.current_room_id = None;
                                this.current_match_id = Some(res.match_id);
                                this.navigate_to(ActiveView::Observe);
                                cx.notify();
                            });
                        }
                    }
                    Err(e) => {
                        update_state(|s| {
                            s.starting = false;
                            s.error = Some(format!("启动失败: {}", e));
                        });
                        if let Some(entity) = weak.upgrade() {
                            let _ = entity.update(&mut cx, |_, cx| cx.notify());
                        }
                    }
                }
            }
        },
    )
    .detach();
}

fn spawn_leave_room(cx: &mut Context<AppSidebar>, room_id: Uuid) {
    let id_str = room_id.to_string();
    cx.spawn(
        move |weak: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
            let weak = weak.clone();
            let mut cx = cx.clone();
            let id_str = id_str.clone();
            async move {
                match cloud_client().leave_room(&id_str).await {
                    Ok(()) => {
                        if let Some(entity) = weak.upgrade() {
                            let _ = entity.update(&mut cx, |this, cx| {
                                this.current_room_id = None;
                                this.navigate_to(ActiveView::Rooms);
                                cx.notify();
                            });
                        }
                    }
                    Err(e) => {
                        update_state(|s| s.error = Some(format!("离开失败: {}", e)));
                        if let Some(entity) = weak.upgrade() {
                            let _ = entity.update(&mut cx, |_, cx| cx.notify());
                        }
                    }
                }
            }
        },
    )
    .detach();
}

fn spawn_dissolve_room(cx: &mut Context<AppSidebar>, room_id: Uuid) {
    let id_str = room_id.to_string();
    cx.spawn(
        move |weak: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
            let weak = weak.clone();
            let mut cx = cx.clone();
            let id_str = id_str.clone();
            async move {
                match cloud_client().dissolve_room(&id_str).await {
                    Ok(()) => {
                        if let Some(entity) = weak.upgrade() {
                            let _ = entity.update(&mut cx, |this, cx| {
                                this.current_room_id = None;
                                this.navigate_to(ActiveView::Rooms);
                                cx.notify();
                            });
                        }
                    }
                    Err(e) => {
                        update_state(|s| s.error = Some(format!("解散失败: {}", e)));
                        if let Some(entity) = weak.upgrade() {
                            let _ = entity.update(&mut cx, |_, cx| cx.notify());
                        }
                    }
                }
            }
        },
    )
    .detach();
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

fn render_team_column(
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
                        .on_click(cx.listener(move |_, _, _, cx| {
                            update_state(|s| {
                                s.show_add_team = Some(team);
                                s.add_agent_id = None;
                                s.add_error.clear();
                            });
                            cx.notify();
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

fn render_add_dialog(cx: &mut Context<AppSidebar>, room_id: Uuid, agents: &[Agent]) -> AnyElement {
    let (show_team, add_agent_id, add_error, adding) = with_state(|s| {
        (
            s.show_add_team,
            s.add_agent_id.clone(),
            s.add_error.clone(),
            s.adding,
        )
    });
    let Some(team) = show_team else {
        return div().into_any_element();
    };
    let title = match team {
        Team::Order => "添加到 Order（蓝方）".to_string(),
        Team::Chaos => "添加到 Chaos（红方）".to_string(),
    };
    let agent_label = add_agent_id
        .as_deref()
        .and_then(|aid| agents.iter().find(|a| a.id.to_string() == aid))
        .map(|a| format!("{} · {}", a.name, a.champion))
        .unwrap_or_else(|| "选择 Agent…".to_string());
    let weak = cx.entity().downgrade();
    let agents_owned = agents.to_vec();

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
                            update_state(|s| s.add_agent_id = Some(aid.clone()));
                            let _ = weak.update(cx, |_, cx| cx.notify());
                        },
                    ));
                }
                m
            });

    div()
        .absolute()
        .inset_0()
        .bg(gpui::black().opacity(0.4))
        .flex()
        .items_center()
        .justify_center()
        .on_any_mouse_down(cx.listener(|_, _, _, cx| {
            update_state(|s| s.show_add_team = None);
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
                        .child(div().font_bold().text_sm().child(title))
                        .child(
                            Button::new("close-add-slot")
                                .ghost()
                                .icon(IconName::Close)
                                .on_click(cx.listener(|_, _, _, cx| {
                                    update_state(|s| s.show_add_team = None);
                                    cx.notify();
                                })),
                        ),
                )
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
                                .on_click(cx.listener(|_, _, _, cx| {
                                    update_state(|s| s.show_add_team = None);
                                    cx.notify();
                                })),
                        )
                        .child(
                            Button::new("confirm-add-slot")
                                .primary()
                                .label(if adding { "添加中…" } else { "添加" })
                                .disabled(adding)
                                .on_click(cx.listener(move |_, _, _, cx| {
                                    spawn_add_slot(cx, room_id);
                                })),
                        ),
                ),
        )
        .into_any_element()
}

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
