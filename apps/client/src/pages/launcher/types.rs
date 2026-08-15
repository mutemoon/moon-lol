//! 启动器页本地状态：结构体 + 纯辅助，状态由 AppSidebar.launcher 持有。

use lol_web_protocol::agent::Agent;
use lol_web_protocol::scenario::Scenario;
use lol_web_protocol::spawn_preset::SpawnPreset as ProtoSpawnPreset;

/// 单个阵营槽位：选手预设名 + 出生点预设名，二者相互独立。
#[derive(Debug, Clone, Default)]
pub(super) struct LauncherSlot {
    pub(super) hero_name: String,
    pub(super) spawn_name: String,
}

/// 启动游戏页顶层视图：模式卡片选择 / 自定义编排。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(super) enum LauncherView {
    /// 模式卡片选择页
    #[default]
    Modes,
    /// 自定义对局编排页
    Custom,
}

pub struct LauncherPageState {
    pub(super) view: LauncherView,
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
            view: LauncherView::default(),
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
