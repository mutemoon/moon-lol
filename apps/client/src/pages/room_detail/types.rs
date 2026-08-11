//! 房间详情页本地状态（存储在 `AppSidebar.room_detail`）。

use lol_web_protocol::agent::Agent;
use lol_web_protocol::room::{Room, RoomAgentSlot};
use lol_web_protocol::spawn_preset::Team;
use uuid::Uuid;

use crate::components::sidebar::AppSidebar;

// ── 页面本地状态 ──

pub struct RoomDetailPageState {
    /// 状态绑定的房间 id；与 sidebar.current_room_id 不一致时重置
    pub(super) room_id: Option<Uuid>,
    /// 是否已触发首次加载
    pub(super) inited: bool,
    /// 是否已启动轮询循环（防重复 spawn）
    pub(super) polling: bool,
    /// 首次加载中
    pub(super) loading: bool,
    pub(super) room: Option<Room>,
    pub(super) slots: Vec<RoomAgentSlot>,
    /// agent 列表（用于槽位名称解析与「添加槽位」下拉）
    pub(super) agents: Vec<Agent>,
    pub(super) error: Option<String>,
    /// 非 None 表示「添加槽位」对话框打开，值为目标阵营
    pub(super) show_add_team: Option<Team>,
    pub(super) add_agent_id: Option<String>,
    pub(super) adding: bool,
    pub(super) add_error: String,
    /// 开始对局请求进行中
    pub(super) starting: bool,
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

/// 将状态绑定到当前房间；id 变化时清空旧数据。
pub(super) fn reset_state_for(sidebar: &mut AppSidebar, room_id: Option<Uuid>) {
    if sidebar.room_detail.room_id != room_id {
        sidebar.room_detail = RoomDetailPageState {
            room_id,
            ..RoomDetailPageState::default()
        };
    }
}
