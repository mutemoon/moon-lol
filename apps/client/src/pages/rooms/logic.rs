//! 房间页操作逻辑：加入 / 创建 / 刷新房间的 spawn 任务与调用入口。

use gpui::prelude::*;
use gpui::*;

use super::types::{update_state, with_state, RoomsTab};
use crate::components::sidebar::AppSidebar;
use crate::services::provider::cloud_client;
use lol_web_protocol::room::{RoomConstraints, TeamPolicy};

// ── 辅助：加入 / 创建 ──

pub(super) fn try_join_by_code(cx: &mut Context<AppSidebar>) {
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

pub(super) fn try_create_room(cx: &mut Context<AppSidebar>) {
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

pub(super) fn spawn_join_or_enter_room(cx: &mut Context<AppSidebar>, room_id: &str) {
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
