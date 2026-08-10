//! 日志归档页状态：thread_local 单例 + 读写访问器。

use std::cell::RefCell;

use lol_web_protocol::match_::Match;

use crate::services::types::{LogCategory, LogEntity, QueryLogsResult};

pub(super) struct LogsArchiveState {
    // 查询面板
    pub(super) game_id: String,
    pub(super) levels: Vec<String>,
    pub(super) entity_id: Option<i64>,
    pub(super) category: Option<String>,
    pub(super) search_text: Option<String>,
    pub(super) offset: i64,
    pub(super) limit: i64,
    pub(super) results: Option<QueryLogsResult>,
    pub(super) entities: Vec<LogEntity>,
    pub(super) categories: Vec<LogCategory>,
    pub(super) loading: bool,
    pub(super) error: Option<String>,
    // 我的对局（24h）
    pub(super) matches: Vec<Match>,
    pub(super) matches_loaded: bool,
    pub(super) matches_loading: bool,
    pub(super) matches_error: Option<String>,
    pub(super) downloading: Option<String>,
    pub(super) download_msg: Option<String>,
    // 加载本地 SQLite
    pub(super) local_path: String,
    pub(super) local_size: Option<u64>,
    pub(super) local_msg: Option<String>,
}

impl Default for LogsArchiveState {
    fn default() -> Self {
        Self {
            game_id: String::new(),
            levels: Vec::new(),
            entity_id: None,
            category: None,
            search_text: None,
            offset: 0,
            limit: 50,
            results: None,
            entities: Vec::new(),
            categories: Vec::new(),
            loading: false,
            error: None,
            matches: Vec::new(),
            matches_loaded: false,
            matches_loading: false,
            matches_error: None,
            downloading: None,
            download_msg: None,
            local_path: String::new(),
            local_size: None,
            local_msg: None,
        }
    }
}

thread_local! {
    pub(super) static STATE: RefCell<LogsArchiveState> = RefCell::new(LogsArchiveState::default());
}

pub(super) fn with_state<R>(f: impl FnOnce(&LogsArchiveState) -> R) -> R {
    STATE.with(|s| f(&s.borrow()))
}

pub(super) fn update_state(f: impl FnOnce(&mut LogsArchiveState)) {
    STATE.with(|s| f(&mut s.borrow_mut()));
}
