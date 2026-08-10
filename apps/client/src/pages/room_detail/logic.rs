//! 房间详情页数据加载与房间操作（拉取 / 轮询 / 添加移除槽位 / 开始对局 / 离开解散）。

use std::time::Duration;

use gpui::*;
use uuid::Uuid;

use super::types::{update_state, with_state};
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
pub(super) fn spawn_add_slot(cx: &mut Context<AppSidebar>, room_id: Uuid) {
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
                    update_state(|s| s.error = Some(e.to_string()));
                }
                fetch_room_data(room_id, &weak, &mut cx).await;
            }
        },
    )
    .detach();
}

pub(super) fn spawn_start_match(cx: &mut Context<AppSidebar>, room_id: Uuid) {
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
