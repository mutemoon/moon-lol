//! 房间页状态：RoomsPageState（存储在 `AppSidebar.rooms`）。

use lol_web_protocol::room::Room;

#[derive(Debug, Clone)]
pub struct RoomsPageState {
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

impl Default for RoomsPageState {
    fn default() -> Self {
        Self {
            loaded: false,
            lobby_rooms: Vec::new(),
            my_rooms: Vec::new(),
            loading: false,
            active_tab: RoomsTab::Lobby,
            join_code: String::new(),
            join_error: String::new(),
            joining: false,
            show_create: false,
            creating: false,
            create_error: String::new(),
            draft_name: String::new(),
            draft_max_members: "10".into(),
            draft_max_agents: "3".into(),
            draft_team_policy: "free".into(),
            draft_lobby_visible: true,
        }
    }
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
