//! 启动器页本地状态：thread_local 存储 + 快照访问辅助。

use std::cell::RefCell;

use lol_web_protocol::agent::Agent;
use lol_web_protocol::scenario::Scenario;
use lol_web_protocol::spawn_preset::SpawnPreset as ProtoSpawnPreset;

/// 单个阵营槽位：选手预设名 + 出生点预设名，二者相互独立。
#[derive(Debug, Clone, Default)]
pub(super) struct LauncherSlot {
    pub(super) hero_name: String,
    pub(super) spawn_name: String,
}

pub(super) struct LauncherPageState {
    pub(super) loaded: bool,
    pub(super) agents: Vec<Agent>,
    pub(super) spawns: Vec<ProtoSpawnPreset>,
    pub(super) scenarios: Vec<Scenario>,
    pub(super) blue_slots: Vec<LauncherSlot>,
    pub(super) red_slots: Vec<LauncherSlot>,
    pub(super) scene_name: String,
    pub(super) saving: bool,
    pub(super) loading_scenario: bool,
    pub(super) error: Option<String>,
    pub(super) message: Option<String>,
}

impl Default for LauncherPageState {
    fn default() -> Self {
        Self {
            loaded: false,
            agents: Vec::new(),
            spawns: Vec::new(),
            scenarios: Vec::new(),
            blue_slots: vec![LauncherSlot::default()],
            red_slots: vec![LauncherSlot::default()],
            scene_name: "default_scenario".into(),
            saving: false,
            loading_scenario: false,
            error: None,
            message: None,
        }
    }
}

thread_local! {
    static STATE: RefCell<LauncherPageState> = RefCell::new(LauncherPageState::default());
}

pub(super) fn with_state<R>(f: impl FnOnce(&LauncherPageState) -> R) -> R {
    STATE.with(|s| f(&s.borrow()))
}

pub(super) fn update_state(f: impl FnOnce(&mut LauncherPageState)) {
    STATE.with(|s| f(&mut s.borrow_mut()));
}

/// 渲染时快照，避免 borrow 逃逸。
pub(super) struct LauncherView {
    pub(super) loaded: bool,
    pub(super) scene_name: String,
    pub(super) scenarios: Vec<Scenario>,
    pub(super) blue_slots: Vec<LauncherSlot>,
    pub(super) red_slots: Vec<LauncherSlot>,
    pub(super) agents: Vec<Agent>,
    pub(super) spawns: Vec<ProtoSpawnPreset>,
    pub(super) saving: bool,
    pub(super) loading_scenario: bool,
    pub(super) error: Option<String>,
    pub(super) message: Option<String>,
}

pub(super) fn snapshot() -> LauncherView {
    with_state(|s| LauncherView {
        loaded: s.loaded,
        scene_name: s.scene_name.clone(),
        scenarios: s.scenarios.clone(),
        blue_slots: s.blue_slots.clone(),
        red_slots: s.red_slots.clone(),
        agents: s.agents.clone(),
        spawns: s.spawns.clone(),
        saving: s.saving,
        loading_scenario: s.loading_scenario,
        error: s.error.clone(),
        message: s.message.clone(),
    })
}
