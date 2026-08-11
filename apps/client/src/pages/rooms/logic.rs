//! 房间页操作逻辑：加入 / 创建 / 刷新房间的 spawn 任务与调用入口。

use gpui::prelude::*;
use gpui::*;
use lol_web_protocol::room::{RoomConstraints, TeamPolicy};

use super::types::RoomsTab;
use crate::components::sidebar::AppSidebar;
use crate::services::provider::cloud_client;

// ── 辅助：加入 / 创建 ──

pub(super) fn try_join_by_code(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) {
    let code = sidebar.rooms.join_code.trim().to_uppercase();
    if code.is_empty() {
        sidebar.rooms.join_error = "请输入邀请码".into();
        cx.notify();
        return;
    }
    sidebar.rooms.join_error.clear();
    sidebar.rooms.joining = true;
    spawn_join_room_by_code(cx, &code);
}

pub(super) fn try_create_room(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) {
    let name = sidebar.rooms.draft_name.trim().to_string();
    let max_members = sidebar
        .rooms
        .draft_max_members
        .trim()
        .parse::<i32>()
        .unwrap_or(10)
        .clamp(2, 20);
    let max_agents = sidebar
        .rooms
        .draft_max_agents
        .trim()
        .parse::<i32>()
        .unwrap_or(3)
        .clamp(1, 10);
    let team_policy = if sidebar.rooms.draft_team_policy == "single_team" {
        TeamPolicy::SingleTeam
    } else {
        TeamPolicy::Free
    };
    let constraints = RoomConstraints {
        max_members,
        max_agents_per_member: max_agents,
        team_policy,
        lobby_visible: sidebar.rooms.draft_lobby_visible,
        prompt_visible: false,
    };
    if name.is_empty() {
        sidebar.rooms.create_error = "请填写房间名称".into();
        cx.notify();
        return;
    }
    sidebar.rooms.creating = true;
    spawn_create_room(cx, &name, constraints);
}

// ── 辅助：spawn 刷新房间列表 ──

pub(super) fn spawn_refresh_rooms(cx: &mut Context<AppSidebar>) {
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
                if let Some(entity) = weak.upgrade() {
                    let _ = entity.update(&mut cx, |s, _| s.rooms.joining = false);
                }
                match result {
                    Ok(_room) => {
                        if let Some(entity) = weak.upgrade() {
                            refresh_rooms(&mut cx, &entity).await;
                        }
                    }
                    Err(e) => {
                        if let Some(entity) = weak.upgrade() {
                            let _ = entity.update(&mut cx, |s, _| {
                                s.rooms.join_error = format!("加入失败: {}", e);
                            });
                        }
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

pub(super) fn spawn_join_or_enter_room(cx: &mut Context<AppSidebar>, room_id: &str) {
    let room_id = room_id.to_string();
    cx.spawn(
        move |weak: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
            let weak = weak.clone();
            let mut cx = cx.clone();
            let room_id = room_id.clone();
            async move {
                let is_member = weak.upgrade().map_or(false, |e| {
                    e.read_with(&cx, |s, _| {
                        s.rooms.my_rooms.iter().any(|r| r.id.to_string() == room_id)
                    })
                });
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
                if let Some(entity) = weak.upgrade() {
                    let _ = entity.update(&mut cx, |s, _| s.rooms.creating = false);
                }
                match result {
                    Ok(_) => {
                        if let Some(entity) = weak.upgrade() {
                            let _ = entity.update(&mut cx, |s, _| {
                                s.rooms.show_create = false;
                                s.rooms.draft_name.clear();
                                s.rooms.active_tab = RoomsTab::Mine;
                            });
                            refresh_rooms(&mut cx, &entity).await;
                        }
                    }
                    Err(e) => {
                        if let Some(entity) = weak.upgrade() {
                            let _ = entity.update(&mut cx, |s, _| {
                                s.rooms.create_error = format!("创建失败: {}", e);
                            });
                        }
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
    let _ = entity.update(cx, |s, _| s.rooms.loading = true);

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

    let _ = entity.update(cx, |s, _| {
        s.rooms.my_rooms = mine;
        s.rooms.lobby_rooms = lobby;
        s.rooms.loading = false;
    });

    entity.update(cx, |_, cx| cx.notify());
}
