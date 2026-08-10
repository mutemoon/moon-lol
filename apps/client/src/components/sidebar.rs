use std::cell::Cell;
use std::collections::HashMap;
use std::sync::Arc;

use gpui::*;
use gpui_component::table::TableState;
use gpui_component::{h_flex, v_flex, ActiveTheme, Root};
use lol_rl_protocol::{InFrame, TaskOverviewItem, VisualInFrame, VisualObsFrame};
use lol_web_protocol::agent::Agent;
use lol_web_protocol::agent_snapshot::AgentSnapshot;
use lol_web_protocol::rank::{EloRating, RankQueueEntry, Season};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::components::auth_dialog::render_auth_dialog;
use crate::components::navigation::{render_sidebar_menu, render_topbar};
use crate::components::tasks_table::TaskTableDelegate;
use crate::components::{render_running_visual, render_task_detail, render_tasks_table};
use crate::pages::heroes::HeroesState;
use crate::pages::settings::SettingsState;
use crate::pages::{
    render_admin, render_billing, render_blog, render_community, render_debug, render_extractor,
    render_games, render_hero, render_heroes, render_history, render_home, render_launcher,
    render_leaderboard, render_logs_archive, render_mock, render_observe, render_particles,
    render_rank, render_room_detail, render_rooms, render_settings, render_wad_browser,
};
use crate::services::cloud::CloudClient;
use crate::services::provider;
use crate::services::ws::spawn_ws_service;
use crate::types::{
    ActiveView, HeroPreset, LocalTaskDetail, ModelProviderInfo, RunningGameInfo, SpawnPreset,
    TaskDetailTab, UserInfo,
};

// 401 回调标记：回调是无参 Fn()，拿不到 &mut App 句柄直接更新实体，
// 只能置位该标记，由 render 帧消费后弹出登录框。
thread_local! {
    static UNAUTHORIZED_PENDING: Cell<bool> = Cell::new(false);
}

pub struct VisualSession {
    pub child: Option<tokio::process::Child>,
    pub port: u16,
    pub cmd_tx: Option<mpsc::UnboundedSender<VisualInFrame>>,
}

pub struct AppSidebar {
    pub active_view: ActiveView,
    pub sidebar_collapsed: bool,
    pub locale: String,

    // ── WebSocket 通信与任务数据 ──
    pub ws_connected: bool,
    pub tx: Option<mpsc::UnboundedSender<InFrame>>,
    pub task_list: Vec<TaskOverviewItem>,
    pub selected_task_id: Option<String>,
    pub task_details: HashMap<String, LocalTaskDetail>,
    pub running_visual_model: Option<String>,
    pub task_detail_tab: TaskDetailTab,

    // ── Visual subprocess session（RL 并行团队维护，不可删除） ──
    pub visual_session: Option<VisualSession>,
    pub visual_ws_connected: bool,
    pub visual_paused: bool,
    pub latest_visual_frame: Option<VisualObsFrame>,
    pub visual_in_tx: Option<mpsc::UnboundedSender<VisualInFrame>>,
    pub visual_error: Option<String>,
    /// 当前可视化子进程所属的任务 id（用于删除任务时联动关闭）
    pub visual_task_id: Option<String>,
    /// 任务概览 DataTable 状态（惰性创建）
    pub table_state: Option<Entity<TableState<TaskTableDelegate>>>,
    /// 新建 RL 训练任务 Modal 弹窗状态与表单
    pub create_task_modal_open: bool,
    pub create_task_form: lol_rl_protocol::TaskConfigPayload,

    // ── 全局状态：Auth（M3/M4 对接 cloud REST 客户端后填充） ──
    pub auth_token: Option<String>,
    pub current_user: Option<UserInfo>,
    pub auth_loading: bool,
    pub show_auth_dialog: bool,

    // ── 全局状态：Game / Match ──
    pub champion: String,
    pub game_mode: String,
    pub launch_error: Option<String>,
    pub is_starting_game: bool,
    pub running_games: Vec<RunningGameInfo>,
    pub current_game_id: Option<String>,
    pub current_match_id: Option<Uuid>,
    pub current_room_id: Option<Uuid>,
    pub champions_list: Vec<String>,
    pub spawn_presets: Vec<SpawnPreset>,
    pub hero_presets: Vec<HeroPreset>,

    // ── 全局状态：Provider ──
    pub model_providers: Vec<ModelProviderInfo>,
    pub providers_loading: bool,

    // ── Rank 页面 ──
    pub rank_agents: Vec<Agent>,
    pub rank_selected_agent_id: String,
    pub rank_snapshots: Vec<AgentSnapshot>,
    pub rank_selected_snapshot_id: String,
    pub rank_mode: String,
    pub rank_queue: Vec<RankQueueEntry>,
    pub rank_season: Option<Season>,
    pub rank_enqueueing: bool,
    pub rank_error: String,
    pub rank_loaded: bool,

    // ── Leaderboard 页面 ──
    pub leaderboard_data: Vec<EloRating>,
    pub leaderboard_mode: String,
    pub leaderboard_view: String,
    pub leaderboard_loaded: bool,

    // ── Community 页面 ──
    pub community_agents: Vec<Agent>,
    pub community_sort: String,
    pub community_search: String,
    pub community_loaded: bool,
    pub community_fork_target: Option<Agent>,
    pub community_fork_name: String,
    pub community_forking: bool,

    pub history: Vec<ActiveView>,
    pub history_index: usize,

    // ── Heroes / Settings 页面 ──
    pub cloud: CloudClient,
    pub heroes: HeroesState,
    pub settings: SettingsState,
}

impl AppSidebar {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let (tx, rx) = mpsc::unbounded_channel::<InFrame>();

        let initial_details = HashMap::new();
        let initial_task_list = Vec::new();

        // 后台启动 WebSocket 服务
        let entity_weak = cx.entity().downgrade();
        spawn_ws_service(entity_weak, cx, rx);

        let mut this = Self {
            active_view: ActiveView::RlTraining,
            sidebar_collapsed: false,
            locale: crate::i18n::read_persisted_locale(),
            ws_connected: false,
            tx: Some(tx),
            task_list: initial_task_list,
            selected_task_id: None,
            task_details: initial_details,
            running_visual_model: None,
            task_detail_tab: TaskDetailTab::Metrics,
            // Visual subprocess
            visual_session: None,
            visual_ws_connected: false,
            visual_paused: false,
            latest_visual_frame: None,
            visual_in_tx: None,
            visual_error: None,
            visual_task_id: None,
            table_state: None,
            create_task_modal_open: false,
            create_task_form: lol_rl_protocol::TaskConfigPayload::default(),
            // Auth
            auth_token: None,
            current_user: None,
            auth_loading: false,
            show_auth_dialog: false,
            // Game
            champion: String::new(),
            game_mode: String::from("sandbox"),
            launch_error: None,
            is_starting_game: false,
            running_games: Vec::new(),
            current_game_id: None,
            current_match_id: None,
            current_room_id: None,
            champions_list: vec!["Riven".into(), "Fiora".into()],
            spawn_presets: Vec::new(),
            hero_presets: Vec::new(),
            // Provider
            model_providers: Vec::new(),
            providers_loading: false,
            // Rank
            rank_agents: Vec::new(),
            rank_selected_agent_id: String::new(),
            rank_snapshots: Vec::new(),
            rank_selected_snapshot_id: String::new(),
            rank_mode: String::from("top_solo"),
            rank_queue: Vec::new(),
            rank_season: None,
            rank_enqueueing: false,
            rank_error: String::new(),
            rank_loaded: false,
            // Leaderboard
            leaderboard_data: Vec::new(),
            leaderboard_mode: String::from("top_solo"),
            leaderboard_view: String::from("total"),
            leaderboard_loaded: false,
            // Community
            community_agents: Vec::new(),
            community_sort: String::from("recent"),
            community_search: String::new(),
            community_loaded: false,
            community_fork_target: None,
            community_fork_name: String::new(),
            community_forking: false,
            history: vec![ActiveView::RlTraining],
            history_index: 0,
            // Heroes / Settings
            // 与 provider::cloud_client() 共享同一 token 存储，登录态全应用一致
            cloud: provider::cloud_client().clone(),
            heroes: HeroesState::default(),
            settings: SettingsState::default(),
        };

        setup_auth(&mut this, cx);

        this
    }

    pub fn navigate_to(&mut self, view: ActiveView) {
        if self.active_view == view {
            return;
        }
        if self.history_index + 1 < self.history.len() {
            self.history.truncate(self.history_index + 1);
        }
        self.history.push(view);
        self.history_index = self.history.len() - 1;
        self.active_view = view;
    }

    pub fn can_go_back(&self) -> bool {
        self.history_index > 0
    }

    pub fn can_go_forward(&self) -> bool {
        self.history_index + 1 < self.history.len()
    }

    pub fn go_back(&mut self) -> bool {
        if self.can_go_back() {
            self.history_index -= 1;
            self.active_view = self.history[self.history_index];
            true
        } else {
            false
        }
    }

    pub fn go_forward(&mut self) -> bool {
        if self.can_go_forward() {
            self.history_index += 1;
            self.active_view = self.history[self.history_index];
            true
        } else {
            false
        }
    }

    pub fn change_locale(&mut self, locale: &str, cx: &mut Context<Self>) {
        self.locale = locale.to_string();
        gpui_component::set_locale(locale);
        crate::i18n::persist_locale(locale);
        cx.notify();
    }

    pub fn send_in_frame(&self, frame: InFrame) {
        if let Some(tx) = &self.tx {
            let _ = tx.send(frame);
        }
    }

    pub fn send_visual_cmd(&mut self, cmd: VisualInFrame) {
        if matches!(cmd, VisualInFrame::Pause) {
            self.visual_paused = true;
        } else if matches!(cmd, VisualInFrame::Resume) {
            self.visual_paused = false;
        }
        if let Some(tx) = &self.visual_in_tx {
            let _ = tx.send(cmd);
        }
    }
}

/// Auth 初始化：注册 401 回调弹出登录框；启动时若已登录则恢复用户会话。
fn setup_auth(this: &mut AppSidebar, cx: &mut Context<AppSidebar>) {
    // 任何请求 401（token 失效/未登录）→ 弹出登录框。
    // 回调是无参 Fn()，无 &mut App 句柄，只能置位标记由 render 帧消费。
    this.cloud.set_on_unauthorized(Arc::new(|| {
        UNAUTHORIZED_PENDING.with(|f| f.set(true));
    }));

    if !this.cloud.is_authenticated() {
        return;
    }

    // 启动时拉取当前用户恢复会话；失败说明 token 失效，清掉登录态。
    let cloud = this.cloud.clone();
    let weak = cx.entity().downgrade();
    cx.spawn(
        move |_this: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
            let mut cx = cx.clone();
            let cloud = cloud.clone();
            async move {
                let user = cloud.get_current_user().await;
                weak.update(&mut cx, |this, cx| {
                    this.auth_loading = false;
                    match user {
                        Ok(u) => {
                            this.current_user = Some(crate::types::UserInfo {
                                id: u.id as i64,
                                phone: u.phone,
                            });
                            this.auth_token = cloud.get_token();
                        }
                        Err(_) => {
                            cloud.logout();
                            this.current_user = None;
                            this.auth_token = None;
                        }
                    }
                    cx.notify();
                })
                .ok();
            }
        },
    )
    .detach();
}

impl Render for AppSidebar {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // 401 回调置位后由渲染帧消费：弹出登录框并清空已失效的用户态
        if UNAUTHORIZED_PENDING.with(|f| f.replace(false)) {
            self.show_auth_dialog = true;
            self.current_user = None;
            self.auth_token = None;
        }

        // 惰性创建任务概览 DataTable 状态（需要 window）
        if self.table_state.is_none() {
            let weak = cx.entity().downgrade();
            let mut delegate = TaskTableDelegate::new(weak);
            delegate.set_tasks(self.task_list.clone());
            self.table_state = Some(cx.new(|cx| TableState::new(delegate, window, cx)));
        }

        let active = self.active_view;

        let main_view_content = match active {
            ActiveView::RlTraining => render_tasks_table(self, cx),
            ActiveView::RlTaskDetail => {
                if let Some(task_id) = self.selected_task_id.clone() {
                    render_task_detail(self, task_id, cx)
                } else {
                    render_tasks_table(self, cx)
                }
            }
            ActiveView::VisualEnv => render_running_visual(self, cx),
            ActiveView::Home => render_home(self, cx),
            ActiveView::Launcher => render_launcher(self, cx),
            ActiveView::Heroes => {
                if self.heroes.agents.is_empty() && !self.heroes.loading {
                    self.heroes.loading = true;
                    let cloud = self.cloud.clone();
                    cx.spawn(
                        |this: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
                            let this = this.clone();
                            let mut cx = cx.clone();
                            async move {
                                let agents = cloud.list_agents().await.unwrap_or_default();
                                use std::collections::HashMap;
                                let mut snapshots = HashMap::new();
                                for a in &agents {
                                    if let Ok(snaps) = cloud.list_snapshots(&a.id.to_string()).await
                                    {
                                        snapshots.insert(a.id, snaps);
                                    }
                                }
                                this.update(&mut cx, |this, ctx| {
                                    this.heroes.agents = agents;
                                    this.heroes.snapshots = snapshots;
                                    this.heroes.loading = false;
                                    ctx.notify();
                                })
                                .ok();
                            }
                        },
                    )
                    .detach();
                }
                render_heroes(self, cx)
            }
            ActiveView::Rooms => render_rooms(self, cx),
            ActiveView::Rank => render_rank(self, cx),
            ActiveView::Leaderboard => render_leaderboard(self, cx),
            ActiveView::Community => render_community(self, cx),
            ActiveView::Billing => render_billing(self, cx),
            ActiveView::Particles => render_particles(self, cx),
            ActiveView::LogsArchive => render_logs_archive(self, cx),
            ActiveView::Admin => render_admin(self, cx),
            ActiveView::Games => render_games(self, cx),
            ActiveView::History => render_history(self, cx),
            ActiveView::Blog => render_blog(self, cx),
            ActiveView::Debug => render_debug(self, cx),
            ActiveView::Mock => render_mock(self, cx),
            ActiveView::Observe => render_observe(self, cx),
            ActiveView::RoomDetail => render_room_detail(self, cx),
            ActiveView::Hero => render_hero(self, cx),
            ActiveView::WadBrowser => render_wad_browser(self, window, cx),
            ActiveView::Extractor => render_extractor(self, window, cx),

            ActiveView::Settings => {
                if self.settings.providers.is_empty() && !self.settings.loading {
                    self.settings.loading = true;
                    let cloud = self.cloud.clone();
                    cx.spawn(
                        |this: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
                            let this = this.clone();
                            let mut cx = cx.clone();
                            async move {
                                let providers =
                                    cloud.list_model_providers().await.unwrap_or_default();
                                this.update(&mut cx, |this, ctx| {
                                    this.settings.providers = providers;
                                    this.settings.loading = false;
                                    ctx.notify();
                                })
                                .ok();
                            }
                        },
                    )
                    .detach();
                }
                render_settings(self, cx)
            }
        };

        let content = div()
            .size_full()
            .flex_1()
            .p_4()
            .pt_0()
            .flex()
            .flex_col()
            .overflow_hidden()
            .child(render_topbar(self, window, cx))
            .child(
                div()
                    .flex_1()
                    .rounded_lg()
                    .border_1()
                    .border_color(cx.theme().border)
                    .p_6()
                    .overflow_hidden()
                    .child(main_view_content),
            );

        let body = h_flex()
            .size_full()
            .overflow_hidden()
            .child(render_sidebar_menu(self, cx))
            .child(content);

        let mut root = v_flex()
            .size_full()
            .relative()
            .child(div().flex_1().overflow_hidden().child(body))
            .children(Root::render_dialog_layer(window, cx))
            .children(Root::render_sheet_layer(window, cx))
            .children(Root::render_notification_layer(window, cx));

        // 登录弹窗覆盖层（绝对定位，盖在整个窗口之上）
        if let Some(dlg) = render_auth_dialog(self, cx) {
            root = root.child(dlg);
        }

        root
    }
}
