//! 对局调试台页面状态：选项卡 / 控制命令 / 本地乐观状态 + thread_local 存储。

use std::cell::RefCell;

use crate::components::agent_chat_history::AgentChatMessage;
use crate::components::game_console_logs::ConsoleLogRow;

// ── 页面本地状态 ──

/// 右侧工作区选项卡。
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum DebugTab {
    Logs,
    Agents,
}

/// 对局控制命令（对应 `apps/client/src/pages/debug/[id].vue` 的按钮组）。
#[derive(Clone)]
pub(super) enum MatchCmd {
    GodMode(bool),
    Cooldown(bool),
    Pause,
    Resume,
    ResetPosition,
    SwitchChampion(String),
}

pub(super) struct DebugPageState {
    /// 当前调试的对局 id（与 sidebar.current_game_id 联动）。
    pub(super) current_game: Option<String>,
    /// 事件循环代际：每次进入新对局自增，旧事件循环靠它识别自己已过期。
    pub(super) generation: u64,
    /// 是否已订阅并正在消费实时事件流。
    pub(super) stream_alive: bool,
    pub(super) error: Option<String>,
    /// 控制台日志行（历史 + 实时）。
    pub(super) logs: Vec<ConsoleLogRow>,
    /// AI 决策消息流。
    pub(super) messages: Vec<AgentChatMessage>,
    pub(super) active_tab: DebugTab,
    /// 本地乐观状态（与游戏端可能短暂不同步，失败时回滚）。
    pub(super) god_mode: bool,
    pub(super) cooldown_disabled: bool,
    pub(super) paused: bool,
    pub(super) switch_target: String,
    pub(super) stopping: bool,
}

impl Default for DebugPageState {
    fn default() -> Self {
        Self {
            current_game: None,
            generation: 0,
            stream_alive: false,
            error: None,
            logs: Vec::new(),
            messages: Vec::new(),
            active_tab: DebugTab::Logs,
            god_mode: false,
            cooldown_disabled: false,
            paused: false,
            switch_target: "Riven".to_string(),
            stopping: false,
        }
    }
}

thread_local! {
    pub(super) static STATE: RefCell<DebugPageState> = RefCell::new(DebugPageState::default());
}

pub(super) fn with_state<R>(f: impl FnOnce(&DebugPageState) -> R) -> R {
    STATE.with(|s| f(&s.borrow()))
}

pub(super) fn update_state(f: impl FnOnce(&mut DebugPageState)) {
    STATE.with(|s| f(&mut s.borrow_mut()));
}
