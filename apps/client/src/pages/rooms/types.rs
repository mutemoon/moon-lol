//! 房间页状态：RoomsPageState + thread_local 全局状态 + with_state/update_state 访问器。

use std::cell::RefCell;

use lol_web_protocol::room::Room;

#[derive(Debug, Clone, Default)]
pub(super) struct RoomsPageState {
    /// 是否已触发首次自动加载
    pub(super) loaded: bool,
    pub(super) lobby_rooms: Vec<Room>,
    pub(super) my_rooms: Vec<Room>,
    pub(super) loading: bool,
    pub(super) active_tab: RoomsTab,
    // 加入码
    pub(super) join_code: String,
    pub(super) join_error: String,
    pub(super) joining: bool,
    // 创建房间
    pub(super) show_create: bool,
    pub(super) creating: bool,
    pub(super) create_error: String,
    pub(super) draft_name: String,
    pub(super) draft_max_members: String,
    pub(super) draft_max_agents: String,
    pub(super) draft_team_policy: String,
    pub(super) draft_lobby_visible: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RoomsTab {
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

pub(super) fn with_state<R>(f: impl FnOnce(&RoomsPageState) -> R) -> R {
    STATE.with(|s| f(&s.borrow()))
}

pub(super) fn update_state(f: impl FnOnce(&mut RoomsPageState)) {
    STATE.with(|s| f(&mut s.borrow_mut()));
}
