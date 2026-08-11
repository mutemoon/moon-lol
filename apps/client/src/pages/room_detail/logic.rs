//! 房间详情页数据加载与房间操作（拉取 / 轮询 / 添加移除槽位 / 开始对局 / 离开解散）。

use std::time::Duration;

use gpui::*;
use uuid::Uuid;

use crate::components::sidebar::AppSidebar;
use crate::services::provider::cloud_client;
use crate::services::runtime::run_on_tokio;
use crate::types::ActiveView;

/// 轮询间隔（秒）
const POLL_INTERVAL_SECS: u64 = 5;

// ── 数据加载 ──

/// 并行拉取房间信息 + 槽位 + agent 列表并写回状态。
async fn fetch_room_data(id: Uuid, weak: &gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp) {
    // 中途已切换房间则丢弃本次结果
    let still_this_room = weak.upgrade().map_or(false, |e| {
        e.read_with(cx, |s, _| s.room_detail.room_id) == Some(id)
    });
    if !still_this_room {
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
    if let Some(entity) = weak.upgrade() {
        let _ = entity.update(cx, |s, cx| {
            s.room_detail.loading = false;
            match r {
                Ok(room) => {
                    s.room_detail.room = Some(room);
                    s.room_detail.error = None;
                }
                Err(e) => s.room_detail.error = Some(e.to_string()),
            }
            s.room_detail.slots = slots;
            s.room_detail.agents = agents;
            cx.notify();
        });
    }
}

pub(super) fn spawn_load(cx: &mut Context<AppSidebar>, id: Uuid) {
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
pub(super) fn spawn_poll(cx: &mut Context<AppSidebar>, id: Uuid) {
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
                    let still_this_room = weak.upgrade().map_or(false, |e| {
                        e.read_with(&cx, |s, _| s.room_detail.room_id) == Some(id)
                    });
                    if !still_this_room {
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
pub(super) fn spawn_add_slot(
    sidebar: &mut AppSidebar,
    cx: &mut Context<AppSidebar>,
    room_id: Uuid,
) {
    let agent_id = sidebar.room_detail.add_agent_id.clone();
    let team = sidebar.room_detail.show_add_team;
    let Some(agent_id) = agent_id else {
        sidebar.room_detail.add_error = "请选择 Agent".into();
        cx.notify();
        return;
    };
    let Some(team) = team else {
        return;
    };
    sidebar.room_detail.adding = true;
    sidebar.room_detail.add_error.clear();
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
                        if let Some(entity) = weak.upgrade() {
                            let _ = entity.update(&mut cx, |s, cx| {
                                s.room_detail.adding = false;
                                s.room_detail.show_add_team = None;
                                s.room_detail.add_agent_id = None;
                                s.room_detail.add_error.clear();
                                cx.notify();
                            });
                        }
                        fetch_room_data(room_id, &weak, &mut cx).await;
                    }
                    Err(e) => {
                        if let Some(entity) = weak.upgrade() {
                            let _ = entity.update(&mut cx, |s, cx| {
                                s.room_detail.adding = false;
                                s.room_detail.add_error = format!("添加失败: {}", e);
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

pub(super) fn spawn_remove_slot(cx: &mut Context<AppSidebar>, room_id: Uuid, slot_id: Uuid) {
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
                    if let Some(entity) = weak.upgrade() {
                        let _ = entity.update(&mut cx, |s, cx| {
                            s.room_detail.error = Some(e.to_string());
                            cx.notify();
                        });
                    }
                }
                fetch_room_data(room_id, &weak, &mut cx).await;
            }
        },
    )
    .detach();
}

pub(super) fn spawn_start_match(
    sidebar: &mut AppSidebar,
    cx: &mut Context<AppSidebar>,
    room_id: Uuid,
) {
    sidebar.room_detail.starting = true;
    sidebar.room_detail.error = None;
    let id_str = room_id.to_string();
    cx.spawn(
        move |weak: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
            let weak = weak.clone();
            let mut cx = cx.clone();
            let id_str = id_str.clone();
            async move {
                match cloud_client().start_room_match(&id_str).await {
                    Ok(res) => {
                        if let Some(entity) = weak.upgrade() {
                            let _ = entity.update(&mut cx, |this, cx| {
                                this.room_detail.starting = false;
                                this.current_room_id = None;
                                this.current_match_id = Some(res.match_id);
                                this.navigate_to(ActiveView::Observe);
                                cx.notify();
                            });
                        }
                    }
                    Err(e) => {
                        if let Some(entity) = weak.upgrade() {
                            let _ = entity.update(&mut cx, |s, cx| {
                                s.room_detail.starting = false;
                                s.room_detail.error = Some(format!("启动失败: {}", e));
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

pub(super) fn spawn_leave_room(cx: &mut Context<AppSidebar>, room_id: Uuid) {
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
                        if let Some(entity) = weak.upgrade() {
                            let _ = entity.update(&mut cx, |s, cx| {
                                s.room_detail.error = Some(format!("离开失败: {}", e));
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

pub(super) fn spawn_dissolve_room(cx: &mut Context<AppSidebar>, room_id: Uuid) {
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
                        if let Some(entity) = weak.upgrade() {
                            let _ = entity.update(&mut cx, |s, cx| {
                                s.room_detail.error = Some(format!("解散失败: {}", e));
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
