//! 观战/回放页状态类型与页面级 thread_local 状态。

use std::cell::RefCell;

use lol_web_protocol::match_::{Match, MatchEvent};
use uuid::Uuid;

pub(super) struct ObservePageState {
    /// 状态对应的对局 id；与 sidebar.current_match_id 不一致时重置
    pub(super) match_id: Option<Uuid>,
    /// 是否已触发首次加载
    pub(super) inited: bool,
    /// 是否已启动轮询循环（防重复 spawn）
    pub(super) polling: bool,
    /// 首次加载中
    pub(super) loading: bool,
    /// 拉取错误
    pub(super) error: Option<String>,
    /// 对局信息
    pub(super) match_info: Option<Match>,
    /// 事件时间线（按 seq 升序）
    pub(super) events: Vec<MatchEvent>,
    /// 下次拉取的起始 seq
    pub(super) last_seq: u32,
    /// 暂停自动刷新
    pub(super) paused: bool,
    /// 是否处于「结束对局」确认态
    pub(super) confirming_stop: bool,
    /// 结束对局请求进行中
    pub(super) stopping: bool,
    /// 结束对局失败信息
    pub(super) stop_error: Option<String>,
}

impl Default for ObservePageState {
    fn default() -> Self {
        Self {
            match_id: None,
            inited: false,
            polling: false,
            loading: false,
            error: None,
            match_info: None,
            events: Vec::new(),
            last_seq: 0,
            paused: false,
            confirming_stop: false,
            stopping: false,
            stop_error: None,
        }
    }
}

thread_local! {
    pub(super) static STATE: RefCell<ObservePageState> = RefCell::new(ObservePageState::default());
}

pub(super) fn with_state<R>(f: impl FnOnce(&ObservePageState) -> R) -> R {
    STATE.with(|s| f(&s.borrow()))
}

pub(super) fn update_state(f: impl FnOnce(&mut ObservePageState)) {
    STATE.with(|s| f(&mut s.borrow_mut()));
}

/// 将状态绑定到当前对局；id 变化时清空旧数据（也用于离开页面时的重置）。
pub(super) fn reset_state_for(match_id: Option<Uuid>) {
    update_state(|s| {
        if s.match_id != match_id {
            *s = ObservePageState {
                match_id,
                ..ObservePageState::default()
            };
        }
    });
}

/// 阵容 Agent 摘要（从事件时间线回填）。
pub(super) struct RosterAgent {
    pub(super) id: String,
    pub(super) name: String,
    pub(super) champion: String,
}
