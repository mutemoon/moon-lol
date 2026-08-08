//! 英雄/选手管理页 — 对应 apps/client/src/pages/heroes.vue
//!
//! 编辑态字段全部可编辑：名称/英雄/类型/提示词/模型（provider 下拉 + 手动模型）、
//! RL 配置（model_path / inference_endpoint / reward_shaper）、Script 脚本、
//! JSON 导入导出、删除确认弹窗、上游 Fork diff 预览与「应用上游」。

use std::cell::RefCell;
use std::collections::HashMap;

use gpui::prelude::*;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::menu::{DropdownMenu, PopupMenuItem};
use gpui_component::scroll::ScrollableElement;
use gpui_component::separator::Separator;
use gpui_component::{h_flex, v_flex, ActiveTheme, Disableable, IconName, Sizable, StyledExt};
use lol_web_protocol::agent::{Agent, AgentType, CreateAgentDto, UpdateAgentDto};
use lol_web_protocol::agent_snapshot::AgentSnapshot;
use lol_web_protocol::model_provider::ModelProvider;
use lol_web_protocol::spawn_preset::Visibility;
use uuid::Uuid;

use crate::components::sidebar::AppSidebar;

/// 平台模型供应商 id（对应 client 的 PLATFORM_PROVIDER_ID = "__platform__"）。
const PLATFORM_PROVIDER_ID: &str = "__platform__";

/// RL Reward Shaper 固定权重键（对应 heroes.vue 的 RL_REWARD_KEYS）。
const RL_REWARD_KEYS: [&str; 9] = [
    "last_hit",
    "kill",
    "death",
    "assist",
    "gold",
    "level",
    "health",
    "time",
    "proximity",
];

// ── 页面状态类型（存储在 AppSidebar） ──

#[derive(Clone, PartialEq)]
pub enum HeroesMode {
    Browse,
    Edit { editing_id: Option<Uuid> },
}

#[derive(Clone, Copy, PartialEq)]
pub enum HeroesTab {
    Config,
    Publish,
}

pub struct HeroesState {
    pub mode: HeroesMode,
    pub agents: Vec<Agent>,
    pub snapshots: HashMap<Uuid, Vec<AgentSnapshot>>,
    pub upstream_agent: Option<Agent>,
    pub loading: bool,
    pub error_msg: String,
    pub success_msg: String,
    pub show_delete_confirm: bool,
    pub deleting: bool,

    pub draft_name: String,
    pub draft_champion: String,
    pub draft_agent_type: AgentType,
    pub draft_prompt: String,
    pub draft_model: String,
    pub draft_config_json_str: String,
    pub draft_visibility: Visibility,
    pub draft_thinking_depth: u32,
    pub draft_provider_id: String,
    pub draft_manual_model: bool,
    pub draft_rl_model_path: String,
    pub draft_rl_endpoint: String,
    pub draft_rl_rewards: HashMap<String, f64>,
    pub draft_script: String,
    pub selected_tab: HeroesTab,
    pub publishing: bool,

    pub platform_models: Vec<String>,
    pub model_providers: Vec<ModelProvider>,
    pub providers_loaded: bool,
}

impl Default for HeroesState {
    fn default() -> Self {
        Self {
            mode: HeroesMode::Browse,
            agents: Vec::new(),
            snapshots: HashMap::new(),
            upstream_agent: None,
            loading: false,
            error_msg: String::new(),
            success_msg: String::new(),
            show_delete_confirm: false,
            deleting: false,
            draft_name: String::new(),
            draft_champion: "Riven".to_string(),
            draft_agent_type: AgentType::Llm,
            draft_prompt: String::new(),
            draft_model: String::new(),
            draft_config_json_str: String::new(),
            draft_visibility: Visibility::Private,
            draft_thinking_depth: 2,
            draft_provider_id: PLATFORM_PROVIDER_ID.to_string(),
            draft_manual_model: false,
            draft_rl_model_path: String::new(),
            draft_rl_endpoint: String::new(),
            draft_rl_rewards: default_rewards(),
            draft_script: String::new(),
            selected_tab: HeroesTab::Config,
            publishing: false,
            platform_models: Vec::new(),
            model_providers: Vec::new(),
            providers_loaded: false,
        }
    }
}

// ── 可编辑文本输入（焦点/光标跨渲染保持，参照 community.rs 手法） ──

#[derive(Clone)]
struct EditMeta {
    cursor: usize,
    focus: FocusHandle,
}

thread_local! {
    static EDITS: RefCell<HashMap<String, EditMeta>> = RefCell::new(HashMap::new());
}

fn edit_meta(id: &str, cx: &App) -> EditMeta {
    EDITS.with(|m| {
        let mut m = m.borrow_mut();
        if let Some(meta) = m.get(id) {
            return meta.clone();
        }
        let meta = EditMeta {
            cursor: 0,
            focus: cx.focus_handle(),
        };
        m.insert(id.to_string(), meta.clone());
        meta
    })
}

fn edit_cursor(id: &str) -> usize {
    EDITS.with(|m| m.borrow().get(id).map_or(0, |e| e.cursor))
}

fn set_edit_cursor(id: &str, cursor: usize) {
    EDITS.with(|m| {
        if let Some(e) = m.borrow_mut().get_mut(id) {
            e.cursor = cursor;
        }
    })
}

/// 处理单个按键，返回（新文本，新光标）。无变化返回 None。
/// `multiline` 为 true 时 Enter 换行，否则忽略。
fn apply_key(
    value: &str,
    cursor: usize,
    event: &KeyDownEvent,
    multiline: bool,
) -> Option<(String, usize)> {
    let ks = &event.keystroke;
    let mods = &ks.modifiers;
    let mut chars: Vec<char> = value.chars().collect();
    let cursor = cursor.min(chars.len());

    // ctrl / cmd 组合键不作为字符输入
    if mods.control || mods.platform {
        return None;
    }

    if let Some(ch) = ks.key_char.as_deref() {
        let insert_chars: Vec<char> = ch.chars().collect();
        if !mods.alt && !insert_chars.is_empty() && !insert_chars.iter().any(|c| c.is_control()) {
            for (i, c) in insert_chars.iter().enumerate() {
                chars.insert(cursor + i, *c);
            }
            return Some((chars.into_iter().collect(), cursor + insert_chars.len()));
        }
    }

    match ks.key.as_str() {
        "backspace" => {
            if cursor > 0 {
                chars.remove(cursor - 1);
                Some((chars.into_iter().collect(), cursor - 1))
            } else {
                None
            }
        }
        "delete" => {
            if cursor < chars.len() {
                chars.remove(cursor);
                Some((chars.into_iter().collect(), cursor))
            } else {
                None
            }
        }
        "left" => Some((value.to_string(), cursor.saturating_sub(1))),
        "right" => Some((value.to_string(), (cursor + 1).min(chars.len()))),
        "home" => Some((value.to_string(), 0)),
        "end" => Some((value.to_string(), chars.len())),
        "space" => {
            chars.insert(cursor, ' ');
            Some((chars.into_iter().collect(), cursor + 1))
        }
        "enter" => {
            if multiline {
                chars.insert(cursor, '\n');
                Some((chars.into_iter().collect(), cursor + 1))
            } else {
                None
            }
        }
        _ => None,
    }
}

/// 可聚焦、可键盘编辑的文本输入框。get_value 读 live 值，set_value 写回 sidebar 字段。
fn render_edit_input(
    sidebar: &AppSidebar,
    cx: &mut Context<AppSidebar>,
    id: &str,
    placeholder: &str,
    multiline: bool,
    get_value: impl Fn(&AppSidebar) -> String + 'static,
    set_value: impl Fn(&mut AppSidebar, String) + 'static,
) -> AnyElement {
    let value = get_value(sidebar);
    let meta = edit_meta(id, cx);
    let focus_handle = meta.focus.clone();
    let empty = value.is_empty();
    let chars: Vec<char> = value.chars().collect();
    let cursor = meta.cursor.min(chars.len());
    let before: String = chars[..cursor].iter().collect();
    let after: String = chars[cursor..].iter().collect();
    let accent = cx.theme().accent;
    let muted = cx.theme().muted_foreground;
    let id_owned = id.to_string();

    let listener = cx.listener(move |this, event: &KeyDownEvent, _window, cx| {
        let live = get_value(this);
        let cur = edit_cursor(&id_owned);
        if let Some((nv, nc)) = apply_key(&live, cur, event, multiline) {
            set_value(this, nv);
            set_edit_cursor(&id_owned, nc);
            cx.notify();
        }
    });

    div()
        .track_focus(&focus_handle)
        .px_3()
        .py_2()
        .w_full()
        .border_1()
        .border_color(cx.theme().border)
        .rounded_md()
        .bg(cx.theme().background)
        .text_sm()
        .when(multiline, |d| d.h(px(150.)).items_start())
        .on_key_down(listener)
        .child(
            h_flex()
                .items_center()
                .when(empty, |d| {
                    d.text_color(muted).child(placeholder.to_string())
                })
                .when(!empty, |d| {
                    d.child(before)
                        .child(div().w(px(1.)).h(rems(1.)).bg(accent))
                        .child(after)
                }),
        )
        .into_any_element()
}

/// 带标签的编辑区包装。
fn edit_field(label: &str, input: AnyElement) -> AnyElement {
    v_flex()
        .gap_1()
        .child(div().text_sm().font_bold().child(label.to_string()))
        .child(input)
        .into_any_element()
}

// ── 辅助函数 ──

fn visibility_label(v: Visibility) -> &'static str {
    match v {
        Visibility::Private => "私有",
        Visibility::Friends => "好友可见",
        Visibility::Public => "公开",
    }
}

fn latest_snapshot_label(snaps: &[AgentSnapshot]) -> String {
    snaps
        .first()
        .map_or_else(|| "未发布".to_string(), |s| format!("v{}", s.version))
}

fn has_unpublished_changes(agent: &Agent, snaps: &[AgentSnapshot]) -> bool {
    let latest = match snaps.first() {
        Some(s) => s,
        None => return true,
    };
    agent.updated_at.as_str() > latest.created_at.as_str()
}

fn ago(iso: &str) -> String {
    iso.chars().take(10).collect()
}

fn champion_display(name: &str) -> String {
    let key = format!("champions.{}", name);
    let localized = rust_i18n::t!(&key);
    if localized != key {
        localized.to_string()
    } else {
        name.to_string()
    }
}

fn default_rewards() -> HashMap<String, f64> {
    let mut m = HashMap::new();
    m.insert("last_hit".into(), 1.0);
    m.insert("kill".into(), 5.0);
    m.insert("death".into(), -5.0);
    m.insert("assist".into(), 2.0);
    m.insert("gold".into(), 0.0);
    m.insert("level".into(), 1.0);
    m.insert("health".into(), 1.0);
    m.insert("time".into(), -0.001);
    m.insert("proximity".into(), 0.0);
    m
}

fn cfg_str(cfg: &Option<serde_json::Value>, key: &str) -> String {
    cfg.as_ref()
        .and_then(|v| v.get(key))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_default()
}

fn pretty_config(cfg: &Option<serde_json::Value>) -> String {
    cfg.as_ref().map_or_else(
        || "{}".to_string(),
        |v| serde_json::to_string_pretty(v).unwrap_or_else(|_| "{}".to_string()),
    )
}

/// 当前草稿的 config_json（按类型组装，参照 heroes.vue handleSave）。
fn draft_config(state: &HeroesState) -> serde_json::Value {
    match state.draft_agent_type {
        AgentType::Llm => {
            let mut obj = serde_json::Map::new();
            obj.insert(
                "thinking_depth".into(),
                serde_json::json!(state.draft_thinking_depth),
            );
            if !state.draft_provider_id.is_empty()
                && state.draft_provider_id != PLATFORM_PROVIDER_ID
            {
                obj.insert(
                    "provider_id".into(),
                    serde_json::json!(state.draft_provider_id),
                );
            }
            serde_json::Value::Object(obj)
        }
        AgentType::Rl => {
            let mut obj = serde_json::Map::new();
            obj.insert(
                "model_path".into(),
                serde_json::json!(state.draft_rl_model_path),
            );
            obj.insert(
                "inference_endpoint".into(),
                serde_json::json!(state.draft_rl_endpoint),
            );
            let mut rs = serde_json::Map::new();
            for (k, v) in &state.draft_rl_rewards {
                rs.insert(k.clone(), serde_json::json!(v));
            }
            obj.insert("reward_shaper".into(), serde_json::Value::Object(rs));
            serde_json::Value::Object(obj)
        }
        AgentType::Script => serde_json::json!({ "script": state.draft_script }),
    }
}

/// 导出用整包 JSON（agent 配置），供展示/复制/导入。
fn export_json(state: &HeroesState) -> String {
    let model = if state.draft_model.is_empty() {
        serde_json::Value::Null
    } else {
        serde_json::Value::String(state.draft_model.clone())
    };
    serde_json::to_string_pretty(&serde_json::json!({
        "name": state.draft_name,
        "champion": state.draft_champion,
        "agent_type": state.draft_agent_type.as_str(),
        "prompt": state.draft_prompt,
        "model": model,
        "config_json": draft_config(state),
    }))
    .unwrap_or_default()
}

/// 把粘贴/输入的 JSON 填充进草稿字段。
fn apply_import_json(state: &mut HeroesState, s: &str) -> Result<(), String> {
    let v: serde_json::Value = serde_json::from_str(s).map_err(|e| e.to_string())?;
    let obj = v.as_array().and_then(|a| a.first()).unwrap_or(&v);
    let obj = obj
        .as_object()
        .ok_or_else(|| "期望 JSON 对象".to_string())?;

    if let Some(n) = obj.get("name").and_then(|v| v.as_str()) {
        state.draft_name = n.to_string();
    }
    if let Some(c) = obj.get("champion").and_then(|v| v.as_str()) {
        state.draft_champion = c.to_string();
    }
    if let Some(t) = obj
        .get("agent_type")
        .and_then(|v| v.as_str())
        .and_then(AgentType::from_str)
    {
        state.draft_agent_type = t;
    }
    if let Some(p) = obj.get("prompt").and_then(|v| v.as_str()) {
        state.draft_prompt = p.to_string();
    }
    if let Some(m) = obj.get("model").and_then(|v| v.as_str()) {
        state.draft_model = m.to_string();
    }
    if let Some(cfg) = obj.get("config_json").and_then(|v| v.as_object()) {
        if let Some(td) = cfg.get("thinking_depth").and_then(|v| v.as_u64()) {
            state.draft_thinking_depth = td as u32;
        }
        if let Some(pid) = cfg.get("provider_id").and_then(|v| v.as_str()) {
            state.draft_provider_id = pid.to_string();
        }
        if let Some(p) = cfg.get("model_path").and_then(|v| v.as_str()) {
            state.draft_rl_model_path = p.to_string();
        }
        if let Some(e) = cfg.get("inference_endpoint").and_then(|v| v.as_str()) {
            state.draft_rl_endpoint = e.to_string();
        }
        if let Some(rs) = cfg.get("reward_shaper").and_then(|v| v.as_object()) {
            for (k, v) in rs {
                if let Some(n) = v.as_f64() {
                    state.draft_rl_rewards.insert(k.clone(), n);
                }
            }
        }
        if let Some(sc) = cfg.get("script").and_then(|v| v.as_str()) {
            state.draft_script = sc.to_string();
        }
    }
    Ok(())
}

/// Agent 的文本快照（prompt + config），用于 Fork diff 两栏对照。
fn pretty_agent(a: &Agent) -> String {
    format!(
        "名称: {}\n英雄: {}\n类型: {}\n模型: {}\n\n【Prompt】\n{}\n\n【配置 JSON】\n{}",
        a.name,
        a.champion,
        a.agent_type.as_str(),
        a.model.as_deref().unwrap_or("(默认)"),
        a.prompt,
        pretty_config(&a.config_json),
    )
}

// ── 主渲染函数 ──

pub fn render_heroes(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) -> AnyElement {
    if sidebar.heroes.loading && sidebar.heroes.agents.is_empty() {
        return v_flex()
            .size_full()
            .items_center()
            .justify_center()
            .child("加载中…")
            .into_any_element();
    }

    if matches!(sidebar.heroes.mode, HeroesMode::Browse) {
        render_browse(sidebar, cx)
    } else {
        render_edit(sidebar, cx)
    }
}

// ── Browse：Agent 卡片网格 ──

fn render_browse(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) -> AnyElement {
    let agents = sidebar.heroes.agents.clone();
    let snaps_map = sidebar.heroes.snapshots.clone();

    v_flex()
        .size_full()
        .overflow_hidden()
        .child(
            h_flex()
                .items_center()
                .justify_between()
                .p_4()
                .child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .child(IconName::LayoutDashboard)
                        .child(div().font_bold().text_lg().child("我的选手"))
                        .child(
                            div()
                                .text_sm()
                                .text_color(cx.theme().muted_foreground)
                                .child(agents.len().to_string()),
                        ),
                )
                .child(
                    h_flex().gap_2().child(
                        Button::new("new-hero-btn")
                            .primary()
                            .icon(IconName::Plus)
                            .label("新建选手")
                            .on_click(cx.listener(|this, _, _, cx| {
                                start_new(this, cx);
                            })),
                    ),
                ),
        )
        .child(Separator::horizontal())
        .child(
            div()
                .flex_1()
                .w_full()
                .overflow_y_scrollbar()
                .child(if agents.is_empty() {
                    v_flex()
                        .items_center()
                        .justify_center()
                        .py_24()
                        .gap_4()
                        .child(
                            div()
                                .text_color(cx.theme().muted_foreground)
                                .text_sm()
                                .child("还没有选手，先建一个吧"),
                        )
                        .child(
                            Button::new("new-hero-empty-btn")
                                .outline()
                                .icon(IconName::Plus)
                                .label("新建选手")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    start_new(this, cx);
                                })),
                        )
                        .into_any_element()
                } else {
                    div()
                        .grid()
                        .grid_cols(3)
                        .gap_4()
                        .children(agents.iter().map(|a| {
                            let snaps = snaps_map.get(&a.id).cloned().unwrap_or_default();
                            render_agent_card(a, &snaps, sidebar, cx)
                        }))
                        .into_any_element()
                }),
        )
        .into_any_element()
}

fn render_agent_card(
    agent: &Agent,
    snaps: &[AgentSnapshot],
    _sidebar: &mut AppSidebar,
    cx: &mut Context<AppSidebar>,
) -> AnyElement {
    let agent_clone = agent.clone();
    let name = agent.name.clone();
    let champion = agent.champion.clone();
    let agent_type = agent.agent_type;
    let visibility = agent.visibility;
    let dirty = has_unpublished_changes(agent, snaps);
    let snap_label = latest_snapshot_label(snaps);

    div()
        .rounded_lg()
        .border_1()
        .border_color(cx.theme().border)
        .p_4()
        .cursor_pointer()
        .id(format!("agent-card-{}", agent.id))
        .on_click(cx.listener(move |this, _, _, cx| {
            enter_edit(this, cx, &agent_clone);
        }))
        .child(
            v_flex()
                .gap_2()
                .child(
                    h_flex()
                        .justify_between()
                        .items_center()
                        .child(div().font_bold().text_sm().child(name.clone()))
                        .child(
                            div()
                                .px_2()
                                .py_0p5()
                                .rounded_md()
                                .bg(cx.theme().muted)
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child(agent_type.as_str().to_uppercase()),
                        ),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child(champion_display(&champion)),
                )
                .child(
                    h_flex()
                        .justify_between()
                        .items_center()
                        .child(
                            div()
                                .px_2()
                                .py_0p5()
                                .rounded_md()
                                .bg(cx.theme().muted)
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child(visibility_label(visibility)),
                        )
                        .child(
                            h_flex()
                                .gap_2()
                                .items_center()
                                .child(if dirty {
                                    div()
                                        .text_xs()
                                        .text_color(rgb(0xd97706))
                                        .child("未发布改动")
                                        .into_any_element()
                                } else {
                                    div().into_any_element()
                                })
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground)
                                        .child(snap_label),
                                ),
                        ),
                ),
        )
        .into_any_element()
}

// ── Edit：编辑视图 ──

fn render_edit(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) -> AnyElement {
    let editing = matches!(
        sidebar.heroes.mode,
        HeroesMode::Edit {
            editing_id: Some(_)
        }
    );
    let draft_name = sidebar.heroes.draft_name.clone();
    let selected_tab = sidebar.heroes.selected_tab;

    let tab_config = make_tab_btn(
        "tab-config",
        "配置",
        selected_tab == HeroesTab::Config,
        cx,
        |this, cx| {
            this.heroes.selected_tab = HeroesTab::Config;
            cx.notify();
        },
    );
    let tab_publish = make_tab_btn(
        "tab-publish",
        "发布与快照",
        selected_tab == HeroesTab::Publish,
        cx,
        |this, cx| {
            this.heroes.selected_tab = HeroesTab::Publish;
            cx.notify();
        },
    );

    let tab_content = if selected_tab == HeroesTab::Config {
        render_config_tab(sidebar, cx)
    } else {
        render_publish_tab(sidebar, cx)
    };

    let main = v_flex()
        .size_full()
        .overflow_hidden()
        .child(
            h_flex()
                .items_center()
                .justify_between()
                .p_4()
                .child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .child(
                            Button::new("back-btn")
                                .ghost()
                                .icon(IconName::ChevronLeft)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.heroes.mode = HeroesMode::Browse;
                                    this.heroes.error_msg.clear();
                                    this.heroes.success_msg.clear();
                                    cx.notify();
                                })),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(cx.theme().muted_foreground)
                                .child(if editing {
                                    "编辑选手"
                                } else {
                                    "新建选手"
                                }),
                        )
                        .child(div().text_lg().font_bold().child(if draft_name.is_empty() {
                            "未命名选手".to_string()
                        } else {
                            draft_name.clone()
                        })),
                )
                .child(
                    h_flex().gap_2().child(
                        Button::new("save-btn")
                            .primary()
                            .label("保存配置")
                            .on_click(cx.listener(|this, _, _, cx| {
                                handle_save(this, cx);
                            })),
                    ),
                ),
        )
        .child(Separator::horizontal())
        .child(
            h_flex()
                .gap_1()
                .px_4()
                .py_2()
                .child(tab_config)
                .child(tab_publish),
        )
        .child(Separator::horizontal())
        .child(
            div()
                .flex_1()
                .w_full()
                .overflow_y_scrollbar()
                .px_6()
                .py_4()
                .child(tab_content),
        )
        .child(render_status_bar(sidebar, cx));

    if sidebar.heroes.show_delete_confirm {
        v_flex()
            .size_full()
            .relative()
            .child(main)
            .child(render_delete_modal(sidebar, cx))
            .into_any_element()
    } else {
        main.into_any_element()
    }
}

fn make_tab_btn(
    id: &str,
    label: impl Into<SharedString>,
    active: bool,
    cx: &mut Context<AppSidebar>,
    on_click: impl Fn(&mut AppSidebar, &mut Context<AppSidebar>) + 'static,
) -> AnyElement {
    let el_id: SharedString = id.to_string().into();
    let btn = Button::new(el_id).label(label);
    let btn = if active { btn.primary() } else { btn.ghost() };
    btn.on_click(cx.listener(move |this, _, _, cx| on_click(this, cx)))
        .into_any_element()
}

// ── Config Tab ──

fn render_config_tab(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) -> AnyElement {
    ensure_providers_loaded(sidebar, cx);

    let agent_type = sidebar.heroes.draft_agent_type;

    let type_buttons: Vec<AnyElement> = [AgentType::Llm, AgentType::Rl, AgentType::Script]
        .iter()
        .map(|&at| {
            let active = agent_type == at;
            let label = match at {
                AgentType::Llm => "LLM（语言模型）",
                AgentType::Rl => "RL（强化学习）",
                AgentType::Script => "Script（脚本）",
            };
            let btn = Button::new(format!("type-{:?}", at)).label(label);
            let btn = if active { btn.primary() } else { btn.outline() };
            btn.on_click(cx.listener(move |this, _, _, cx| {
                this.heroes.draft_agent_type = at;
                cx.notify();
            }))
            .into_any_element()
        })
        .collect();

    let type_specific = match agent_type {
        AgentType::Llm => v_flex()
            .gap_4()
            .child(edit_field(
                "系统提示词",
                render_edit_input(
                    sidebar,
                    cx,
                    "heroes-prompt",
                    "描述该选手的行为策略、对线风格、连招意图…",
                    true,
                    |s| s.heroes.draft_prompt.clone(),
                    |s, v| s.heroes.draft_prompt = v,
                ),
            ))
            .child(
                h_flex()
                    .gap_4()
                    .child(div().flex_1().child(edit_field("模型供应商", render_provider_select(sidebar, cx))))
                    .child(div().flex_1().child(edit_field("模型（留空用全局默认）", render_model_select(sidebar, cx)))),
            )
            .child(render_think_depth(sidebar, cx))
            .into_any_element(),
        AgentType::Rl => v_flex()
            .gap_4()
            .child(edit_field(
                "模型权重路径 (.pth)",
                render_edit_input(
                    sidebar,
                    cx,
                    "heroes-rl-path",
                    "如 checkpoints/ppo_riven.pth",
                    false,
                    |s| s.heroes.draft_rl_model_path.clone(),
                    |s, v| s.heroes.draft_rl_model_path = v,
                ),
            ))
            .child(edit_field(
                "推理端点 URL（可选）",
                render_edit_input(
                    sidebar,
                    cx,
                    "heroes-rl-endpoint",
                    "如 ws://127.0.0.1:8765",
                    false,
                    |s| s.heroes.draft_rl_endpoint.clone(),
                    |s, v| s.heroes.draft_rl_endpoint = v,
                ),
            ))
            .child(reward_grid(sidebar, cx))
            .into_any_element(),
        AgentType::Script => v_flex()
            .gap_4()
            .child(edit_field(
                "脚本",
                render_edit_input(
                    sidebar,
                    cx,
                    "heroes-script",
                    "// 在此编写宿主 API 脚本…",
                    true,
                    |s| s.heroes.draft_script.clone(),
                    |s, v| s.heroes.draft_script = v,
                ),
            ))
            .child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child("简化版脚本编辑器（无语法高亮 / 断点 / 热重载），保存时写入 config_json.script。"),
            )
            .into_any_element(),
    };

    v_flex()
        .gap_5()
        .child(edit_field(
            "选手名称",
            render_edit_input(
                sidebar,
                cx,
                "heroes-name",
                "如：锐雯 · 激进压制",
                false,
                |s| s.heroes.draft_name.clone(),
                |s, v| s.heroes.draft_name = v,
            ),
        ))
        .child(
            h_flex()
                .gap_4()
                .child(
                    div()
                        .flex_1()
                        .child(edit_field("英雄", render_champion_select(sidebar, cx))),
                )
                .child(div().flex_1().child(edit_field(
                    "决策驱动类型",
                    h_flex().gap_2().children(type_buttons).into_any_element(),
                ))),
        )
        .child(type_specific)
        .child(Separator::horizontal())
        .child(render_json_section(sidebar, cx))
        .into_any_element()
}

/// LLM 思考深度滑块（1-5，每格可点击）。
fn render_think_depth(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) -> AnyElement {
    let td = sidebar.heroes.draft_thinking_depth as i32;
    let pips: Vec<AnyElement> = (1..=5)
        .map(|d| {
            let active = d <= td;
            let target = d as u32;
            div()
                .w(px(40.))
                .h(px(16.))
                .rounded_sm()
                .bg(if active {
                    cx.theme().primary
                } else {
                    cx.theme().muted
                })
                .cursor_pointer()
                .id(format!("depth-pip-{}", d))
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.heroes.draft_thinking_depth = target;
                    cx.notify();
                }))
                .into_any_element()
        })
        .collect();

    v_flex()
        .gap_2()
        .child(
            h_flex()
                .justify_between()
                .child(div().text_sm().font_bold().child("思考深度"))
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child(format!("{} / 5", td)),
                ),
        )
        .child(
            div()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child(
                    "更高的思考深度让 LLM 在决策前展开更长的推理链，更稳但更慢、消耗更多 token。",
                ),
        )
        .child(h_flex().gap_2().children(pips))
        .into_any_element()
}

/// 英雄下拉（从 sidebar.champions_list 选择）。
fn render_champion_select(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) -> AnyElement {
    let list: Vec<String> = if sidebar.champions_list.is_empty() {
        vec!["Riven".to_string(), "Fiora".to_string()]
    } else {
        sidebar.champions_list.clone()
    };
    let current = sidebar.heroes.draft_champion.clone();
    let weak = cx.entity().downgrade();

    Button::new("heroes-champion")
        .outline()
        .w_full()
        .label(champion_display(&current))
        .dropdown_menu(move |menu, _w, _cx| {
            let mut m = menu;
            for c in &list {
                let cname = c.clone();
                let weak = weak.clone();
                let checked = current == cname;
                m = m.item(
                    PopupMenuItem::new(champion_display(&cname))
                        .checked(checked)
                        .on_click(move |_, _, cx| {
                            if let Some(app) = weak.upgrade() {
                                let _ = app.update(cx, |this, cx| {
                                    this.heroes.draft_champion = cname.clone();
                                    cx.notify();
                                });
                            }
                        }),
                );
            }
            m
        })
        .into_any_element()
}

/// provider 下拉：平台模型 + 启用的自带供应商（BYO）。
fn render_provider_select(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) -> AnyElement {
    let providers = sidebar.heroes.model_providers.clone();
    let current = sidebar.heroes.draft_provider_id.clone();
    let enabled: Vec<ModelProvider> = providers.iter().filter(|p| p.enabled).cloned().collect();
    let current_label = if current.is_empty() || current == PLATFORM_PROVIDER_ID {
        "平台模型".to_string()
    } else {
        providers
            .iter()
            .find(|p| p.id.to_string() == current)
            .map_or("未知供应商".to_string(), |p| p.name.clone())
    };
    let weak = cx.entity().downgrade();

    Button::new("heroes-provider")
        .outline()
        .w_full()
        .label(current_label)
        .dropdown_menu(move |menu, _w, _cx| {
            let mut m = menu;
            let checked_platform = current == PLATFORM_PROVIDER_ID;
            let weak_platform = weak.clone();
            m = m.item(
                PopupMenuItem::new("平台模型")
                    .checked(checked_platform)
                    .on_click(move |_, _, cx| {
                        if let Some(app) = weak_platform.upgrade() {
                            let _ = app.update(cx, |this, cx| {
                                this.heroes.draft_provider_id = PLATFORM_PROVIDER_ID.to_string();
                                this.heroes.draft_manual_model = false;
                                cx.notify();
                            });
                        }
                    }),
            );
            if enabled.is_empty() {
                m = m.item(PopupMenuItem::new("暂无启用供应商").disabled(true));
            }
            for p in &enabled {
                let pid = p.id.to_string();
                let pname = p.name.clone();
                let weak = weak.clone();
                let checked = current == pid;
                m = m.item(
                    PopupMenuItem::new(pname)
                        .checked(checked)
                        .on_click(move |_, _, cx| {
                            if let Some(app) = weak.upgrade() {
                                let _ = app.update(cx, |this, cx| {
                                    this.heroes.draft_provider_id = pid.clone();
                                    this.heroes.draft_manual_model = false;
                                    cx.notify();
                                });
                            }
                        }),
                );
            }
            m
        })
        .into_any_element()
}

/// 模型选择：平台模型走管理员清单；BYO 供应商走其 models 列表；可切「手填」。
fn render_model_select(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) -> AnyElement {
    let is_platform = sidebar.heroes.draft_provider_id.is_empty()
        || sidebar.heroes.draft_provider_id == PLATFORM_PROVIDER_ID;
    let manual = sidebar.heroes.draft_manual_model;
    let model = sidebar.heroes.draft_model.clone();
    let platform_models = sidebar.heroes.platform_models.clone();
    let providers = sidebar.heroes.model_providers.clone();
    let pid = sidebar.heroes.draft_provider_id.clone();

    let provider_models: Vec<String> = providers
        .iter()
        .find(|p| p.id.to_string() == pid)
        .map(|p| p.models.iter().map(|m| m.name.clone()).collect())
        .unwrap_or_default();
    let model_options: Vec<String> = if is_platform {
        platform_models.clone()
    } else {
        provider_models
    };
    let model_not_in_list = !model.is_empty() && !model_options.contains(&model);
    let show_manual = !is_platform && (manual || model_not_in_list || model_options.is_empty());

    let weak = cx.entity().downgrade();

    let selector: AnyElement = if show_manual {
        render_edit_input(
            sidebar,
            cx,
            "heroes-model-manual",
            "手动输入模型名…",
            false,
            |s| s.heroes.draft_model.clone(),
            |s, v| s.heroes.draft_model = v,
        )
    } else {
        let current_label = if model.is_empty() {
            "选择模型…".to_string()
        } else {
            model.clone()
        };
        let options = model_options.clone();
        let current = model.clone();
        Button::new("heroes-model")
            .outline()
            .w_full()
            .label(current_label)
            .dropdown_menu(move |menu, _w, _cx| {
                let mut m = menu;
                if options.is_empty() {
                    m = m.item(PopupMenuItem::new("暂无可选模型").disabled(true));
                }
                for opt in &options {
                    let opt = opt.clone();
                    let weak = weak.clone();
                    let checked = current == opt;
                    m = m.item(PopupMenuItem::new(opt.clone()).checked(checked).on_click(
                        move |_, _, cx| {
                            if let Some(app) = weak.upgrade() {
                                let _ = app.update(cx, |this, cx| {
                                    this.heroes.draft_model = opt.clone();
                                    this.heroes.draft_manual_model = false;
                                    cx.notify();
                                });
                            }
                        },
                    ));
                }
                m
            })
            .into_any_element()
    };

    v_flex()
        .gap_2()
        .child(selector)
        .child(
            h_flex()
                .gap_2()
                .items_center()
                .child(if !is_platform {
                    Button::new("heroes-model-manual-toggle")
                        .ghost()
                        .xsmall()
                        .label(if manual { "使用列表" } else { "手填" })
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.heroes.draft_manual_model = !this.heroes.draft_manual_model;
                            cx.notify();
                        }))
                        .into_any_element()
                } else {
                    div().into_any_element()
                })
                .child(if is_platform && platform_models.is_empty() {
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child("管理员暂未配置平台模型，可改选自带供应商。")
                        .into_any_element()
                } else {
                    div().into_any_element()
                }),
        )
        .into_any_element()
}

/// RL Reward Shaper 权重输入网格。
fn reward_grid(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) -> AnyElement {
    v_flex()
        .gap_2()
        .child(div().text_sm().font_bold().child("Reward Shaper 权重"))
        .child(
            div()
                .grid()
                .grid_cols(3)
                .gap_2()
                .children(RL_REWARD_KEYS.iter().map(|key| {
                    let kid = format!("heroes-rl-{}", key);
                    v_flex()
                        .gap_1()
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child(key.to_string()),
                        )
                        .child(render_edit_input(
                            sidebar,
                            cx,
                            &kid,
                            "0",
                            false,
                            move |s: &AppSidebar| {
                                s.heroes
                                    .draft_rl_rewards
                                    .get(*key)
                                    .map_or_else(|| "0".to_string(), |v| format!("{}", v))
                            },
                            move |s: &mut AppSidebar, v: String| {
                                let parsed = v.trim().parse::<f64>().unwrap_or(0.0);
                                s.heroes.draft_rl_rewards.insert(key.to_string(), parsed);
                            },
                        ))
                        .into_any_element()
                })),
        )
        .into_any_element()
}

/// JSON 导入 / 导出区块。
fn render_json_section(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) -> AnyElement {
    v_flex()
        .gap_2()
        .child(
            h_flex()
                .justify_between()
                .items_center()
                .child(
                    div()
                        .text_sm()
                        .font_bold()
                        .child("配置 JSON（导入 / 导出）"),
                )
                .child(
                    h_flex()
                        .gap_2()
                        .child(
                            Button::new("heroes-json-export")
                                .outline()
                                .label("导出")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    handle_export_json(this, cx);
                                })),
                        )
                        .child(
                            Button::new("heroes-json-import")
                                .outline()
                                .label("应用")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    handle_import_json(this, cx);
                                })),
                        ),
                ),
        )
        .child(render_edit_input(
            sidebar,
            cx,
            "heroes-json",
            "{}",
            true,
            |s| s.heroes.draft_config_json_str.clone(),
            |s, v| s.heroes.draft_config_json_str = v,
        ))
        .child(
            div()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child(
                    "「导出」序列化当前表单并复制到剪贴板；粘贴 JSON 到上方后点「应用」填充表单。",
                ),
        )
        .into_any_element()
}

// ── Publish Tab ──

fn render_publish_tab(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) -> AnyElement {
    let editing_id = match &sidebar.heroes.mode {
        HeroesMode::Edit { editing_id } => *editing_id,
        _ => None,
    };
    let current_agent = editing_id
        .and_then(|id| sidebar.heroes.agents.iter().find(|a| a.id == id))
        .cloned();
    let snaps = editing_id
        .map(|id| {
            sidebar
                .heroes
                .snapshots
                .get(&id)
                .cloned()
                .unwrap_or_default()
        })
        .unwrap_or_default();
    let upstream_id = current_agent
        .as_ref()
        .and_then(|a| a.upstream_agent_id.or(a.forked_from));

    v_flex()
        .gap_6()
        .child(if upstream_id.is_some() {
            render_upstream_sync(sidebar, cx, &current_agent)
        } else {
            div().into_any_element()
        })
        .child(render_visibility_section(sidebar, cx))
        .child(Separator::horizontal())
        .child(render_publish_section(sidebar, cx, &snaps))
        .child(render_snapshot_list(sidebar, cx, &snaps))
        .into_any_element()
}

/// 上游同步 + Fork diff 两栏对照 + 应用上游。
fn render_upstream_sync(
    sidebar: &mut AppSidebar,
    cx: &mut Context<AppSidebar>,
    current_agent: &Option<Agent>,
) -> AnyElement {
    let up = sidebar.heroes.upstream_agent.clone();
    let owner_id = up.as_ref().map_or(0, |a| a.owner_id);
    let up_name = up.as_ref().map_or("…".to_string(), |a| a.name.clone());
    let show_diff = up.is_some();
    let current_text = current_agent.as_ref().map(pretty_agent).unwrap_or_default();
    let upstream_text = up.as_ref().map(pretty_agent).unwrap_or_default();

    v_flex()
        .gap_3()
        .child(div().text_sm().font_bold().child("上游同步"))
        .child(
            div()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child("该选手 Fork 自上游公开选手。可对比差异并拉取上游最新策略覆盖当前编辑态。"),
        )
        .child(
            h_flex()
                .items_center()
                .justify_between()
                .rounded_md()
                .border_1()
                .border_color(cx.theme().border)
                .px_4()
                .py_3()
                .child(
                    div()
                        .text_sm()
                        .child(format!("Fork 自「{}」· 经理 #{}", up_name, owner_id)),
                )
                .child(if show_diff {
                    Button::new("pull-btn")
                        .outline()
                        .label("应用上游（覆盖当前）")
                        .on_click(cx.listener(|this, _, _, cx| {
                            handle_pull_upstream(this, cx);
                        }))
                        .into_any_element()
                } else {
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child("加载上游中…")
                        .into_any_element()
                }),
        )
        .child(if show_diff {
            h_flex()
                .gap_3()
                .items_start()
                .child(diff_column("当前", &current_text, cx))
                .child(diff_column("上游", &upstream_text, cx))
                .into_any_element()
        } else {
            div().into_any_element()
        })
        .into_any_element()
}

fn diff_column(title: &str, text: &str, cx: &mut Context<AppSidebar>) -> AnyElement {
    v_flex()
        .flex_1()
        .gap_1()
        .child(
            div()
                .text_xs()
                .font_bold()
                .text_color(cx.theme().muted_foreground)
                .child(title.to_string()),
        )
        .child(
            div()
                .h(px(260.))
                .overflow_y_scrollbar()
                .rounded_md()
                .border_1()
                .border_color(cx.theme().border)
                .px_3()
                .py_2()
                .text_xs()
                .child(text.to_string()),
        )
        .into_any_element()
}

fn render_visibility_section(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) -> AnyElement {
    let current = sidebar.heroes.draft_visibility;
    let vis_btns: Vec<AnyElement> = [Visibility::Private, Visibility::Friends, Visibility::Public]
        .iter()
        .map(|&v| {
            let active = current == v;
            let btn = Button::new(format!("vis-{:?}", v)).label(visibility_label(v));
            let btn = if active { btn.primary() } else { btn.outline() };
            btn.on_click(cx.listener(move |this, _, _, cx| {
                handle_visibility_change(this, v, cx);
            }))
            .into_any_element()
        })
        .collect();

    v_flex()
        .gap_2()
        .child(div().text_sm().font_bold().child("可见性"))
        .child(
            div()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child("公开后他人可在社区 Fork；提示词与模型配置等敏感字段是否暴露由可见性决定。"),
        )
        .child(h_flex().gap_2().children(vis_btns))
        .into_any_element()
}

fn render_publish_section(
    sidebar: &mut AppSidebar,
    cx: &mut Context<AppSidebar>,
    snaps: &[AgentSnapshot],
) -> AnyElement {
    let editing = matches!(
        sidebar.heroes.mode,
        HeroesMode::Edit {
            editing_id: Some(_)
        }
    );
    let editing_id = match &sidebar.heroes.mode {
        HeroesMode::Edit { editing_id } => *editing_id,
        _ => None,
    };
    let dirty = editing_id
        .and_then(|id| sidebar.heroes.agents.iter().find(|a| a.id == id))
        .map(|a| has_unpublished_changes(a, snaps))
        .unwrap_or(false);
    let publishing = sidebar.heroes.publishing;

    v_flex()
        .gap_3()
        .child(div().text_sm().font_bold().child("发布参赛快照"))
        .child(
            div()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child("Rank 队列始终取用该选手最新发布的快照。改完配置后需要再发布一次才会进入下一局；进行中的对局不受影响。"),
        )
        .child(if dirty {
            div()
                .text_xs()
                .text_color(rgb(0xd97706))
                .child("当前配置晚于最新快照，需重新发布才会在 Rank 生效。")
                .into_any_element()
        } else {
            div().into_any_element()
        })
        .child(if !editing {
            div()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child("该选手尚未在云端注册，无法发布快照。请先保存完成同步。")
                .into_any_element()
        } else {
            div().into_any_element()
        })
        .child(
            Button::new("publish-btn")
                .primary()
                .icon(IconName::Play)
                .label(if publishing { "发布中…" } else { "发布快照" })
                .on_click(cx.listener(|this, _, _, cx| {
                    handle_publish(this, cx);
                })),
        )
        .into_any_element()
}

fn render_snapshot_list(
    _sidebar: &mut AppSidebar,
    cx: &mut Context<AppSidebar>,
    snaps: &[AgentSnapshot],
) -> AnyElement {
    v_flex()
        .gap_2()
        .child(
            div()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .font_bold()
                .child("历史快照"),
        )
        .child(if snaps.is_empty() {
            div()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child("尚无历史快照")
                .into_any_element()
        } else {
            v_flex()
                .rounded_md()
                .border_1()
                .border_color(cx.theme().border)
                .children(snaps.iter().enumerate().map(|(i, s)| {
                    let row = v_flex().px_4().py_2();
                    let row = if i > 0 {
                        row.border_t_1().border_color(cx.theme().border)
                    } else {
                        row
                    };
                    row.child(
                        h_flex()
                            .justify_between()
                            .items_center()
                            .child(
                                h_flex()
                                    .gap_2()
                                    .items_center()
                                    .child(div().text_sm().child(format!("v{}", s.version)))
                                    .child(if i == 0 {
                                        div()
                                            .px_2()
                                            .py_0p5()
                                            .rounded_md()
                                            .bg(cx.theme().muted)
                                            .text_xs()
                                            .child("当前最新")
                                            .into_any_element()
                                    } else {
                                        div().into_any_element()
                                    }),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(format!("{} 前", ago(&s.created_at))),
                            ),
                    )
                    .into_any_element()
                }))
                .into_any_element()
        })
        .into_any_element()
}

fn render_status_bar(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) -> AnyElement {
    let editing = matches!(
        sidebar.heroes.mode,
        HeroesMode::Edit {
            editing_id: Some(_)
        }
    );
    let error = sidebar.heroes.error_msg.clone();
    let success = sidebar.heroes.success_msg.clone();

    h_flex()
        .px_4()
        .py_2()
        .gap_2()
        .items_center()
        .child(if !error.is_empty() {
            div()
                .text_sm()
                .text_color(cx.theme().danger)
                .child(error)
                .into_any_element()
        } else if !success.is_empty() {
            div().text_sm().child(success).into_any_element()
        } else {
            div().into_any_element()
        })
        .child(if editing {
            Button::new("delete-btn")
                .ghost()
                .label("删除")
                .on_click(cx.listener(|this, _, _, cx| {
                    this.heroes.show_delete_confirm = true;
                    cx.notify();
                }))
                .into_any_element()
        } else {
            div().into_any_element()
        })
        .into_any_element()
}

// ── 删除确认弹窗 ──

fn render_delete_modal(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) -> AnyElement {
    let name = sidebar.heroes.draft_name.clone();
    let deleting = sidebar.heroes.deleting;

    div()
        .absolute()
        .top_0()
        .bottom_0()
        .left_0()
        .right_0()
        .bg(rgba(0x00000073))
        .flex()
        .items_center()
        .justify_center()
        .child(
            v_flex()
                .w(px(380.))
                .rounded_lg()
                .border_1()
                .border_color(cx.theme().border)
                .bg(cx.theme().background)
                .p_6()
                .gap_4()
                .child(
                    div()
                        .text_lg()
                        .font_bold()
                        .child(format!("删除选手「{}」？", name)),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child("该操作不可撤销。引用此选手的场景槽位需手动重新选择。"),
                )
                .child(
                    h_flex()
                        .justify_end()
                        .gap_2()
                        .child(
                            Button::new("delete-cancel")
                                .outline()
                                .label("取消")
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.heroes.show_delete_confirm = false;
                                    cx.notify();
                                })),
                        )
                        .child(
                            Button::new("delete-confirm")
                                .danger()
                                .label(if deleting { "删除中…" } else { "删除" })
                                .disabled(deleting)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    handle_delete(this, cx);
                                })),
                        ),
                ),
        )
        .into_any_element()
}

// ── 数据加载 ──

/// 首次进入编辑态时懒加载 model providers 与平台模型清单。
fn ensure_providers_loaded(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) {
    if sidebar.heroes.providers_loaded {
        return;
    }
    sidebar.heroes.providers_loaded = true;
    let cloud = sidebar.cloud.clone();
    cx.spawn(
        move |this: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
            let this = this.clone();
            let mut cx = cx.clone();
            async move {
                let providers = cloud.list_model_providers().await.unwrap_or_default();
                let platform = cloud.list_platform_models().await.unwrap_or_default();
                this.update(&mut cx, |this, cx| {
                    this.heroes.model_providers = providers;
                    this.heroes.platform_models = platform;
                    cx.notify();
                })
                .ok();
            }
        },
    )
    .detach();
}

/// 加载当前编辑 agent 的上游 Agent（用于 diff 对照）。
fn spawn_load_upstream(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) {
    let editing_id = match &sidebar.heroes.mode {
        HeroesMode::Edit {
            editing_id: Some(id),
        } => *id,
        _ => return,
    };
    let upstream_id = sidebar
        .heroes
        .agents
        .iter()
        .find(|a| a.id == editing_id)
        .and_then(|a| a.upstream_agent_id.or(a.forked_from));
    let Some(uid) = upstream_id else {
        return;
    };
    let cloud = sidebar.cloud.clone();
    cx.spawn(
        move |this: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
            let this = this.clone();
            let mut cx = cx.clone();
            async move {
                let up = cloud.get_agent(&uid.to_string()).await.ok();
                this.update(&mut cx, |this, cx| {
                    this.heroes.upstream_agent = up;
                    cx.notify();
                })
                .ok();
            }
        },
    )
    .detach();
}

// ── 操作逻辑 ──

fn start_new(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) {
    sidebar.heroes = HeroesState {
        mode: HeroesMode::Edit { editing_id: None },
        ..Default::default()
    };
    ensure_providers_loaded(sidebar, cx);
    cx.notify();
}

/// 进入编辑态：用云端 Agent 填充草稿字段，并加载上游与 providers。
fn enter_edit(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>, agent: &Agent) {
    let cfg = agent.config_json.clone();
    let td = cfg
        .as_ref()
        .and_then(|v| v.get("thinking_depth"))
        .and_then(|v| v.as_u64())
        .map_or(2, |n| n as u32);
    let provider_id = cfg
        .as_ref()
        .and_then(|v| v.get("provider_id"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| PLATFORM_PROVIDER_ID.to_string());

    let mut rewards = default_rewards();
    if let Some(rs) = cfg
        .as_ref()
        .and_then(|v| v.get("reward_shaper"))
        .and_then(|v| v.as_object())
    {
        for (k, v) in rs {
            if let Some(n) = v.as_f64() {
                rewards.insert(k.clone(), n);
            }
        }
    }

    sidebar.heroes.mode = HeroesMode::Edit {
        editing_id: Some(agent.id),
    };
    sidebar.heroes.draft_name = agent.name.clone();
    sidebar.heroes.draft_champion = agent.champion.clone();
    sidebar.heroes.draft_agent_type = agent.agent_type;
    sidebar.heroes.draft_prompt = agent.prompt.clone();
    sidebar.heroes.draft_model = agent.model.clone().unwrap_or_default();
    sidebar.heroes.draft_visibility = agent.visibility;
    sidebar.heroes.draft_thinking_depth = td;
    sidebar.heroes.draft_provider_id = provider_id;
    sidebar.heroes.draft_manual_model = false;
    sidebar.heroes.draft_rl_model_path = cfg_str(&cfg, "model_path");
    sidebar.heroes.draft_rl_endpoint = cfg_str(&cfg, "inference_endpoint");
    sidebar.heroes.draft_rl_rewards = rewards;
    sidebar.heroes.draft_script = cfg_str(&cfg, "script");
    sidebar.heroes.draft_config_json_str = pretty_config(&agent.config_json);
    sidebar.heroes.upstream_agent = None;
    sidebar.heroes.show_delete_confirm = false;
    sidebar.heroes.deleting = false;
    sidebar.heroes.error_msg.clear();
    sidebar.heroes.success_msg.clear();
    sidebar.heroes.selected_tab = HeroesTab::Config;
    ensure_providers_loaded(sidebar, cx);
    spawn_load_upstream(sidebar, cx);
    cx.notify();
}

fn handle_save(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) {
    let name = sidebar.heroes.draft_name.trim().to_string();
    if name.is_empty() {
        sidebar.heroes.error_msg = "请填写选手名称".to_string();
        cx.notify();
        return;
    }

    let champion = sidebar.heroes.draft_champion.clone();
    let prompt = sidebar.heroes.draft_prompt.clone();
    let model = sidebar.heroes.draft_model.clone();
    let agent_type = sidebar.heroes.draft_agent_type;
    let visibility = sidebar.heroes.draft_visibility;
    let config_json = Some(draft_config(&sidebar.heroes));
    let editing_id = match &sidebar.heroes.mode {
        HeroesMode::Edit { editing_id } => *editing_id,
        _ => None,
    };

    let cloud = sidebar.cloud.clone();
    cx.spawn(
        move |this: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
            let this = this.clone();
            let mut cx = cx.clone();
            async move {
                let result = if let Some(id) = editing_id {
                    let dto = UpdateAgentDto {
                        name: Some(name),
                        champion: Some(champion),
                        agent_type: Some(agent_type),
                        prompt: Some(prompt),
                        model: if model.is_empty() { None } else { Some(model) },
                        config_json,
                        visibility: Some(visibility),
                    };
                    cloud.update_agent(&id.to_string(), &dto).await
                } else {
                    let dto = CreateAgentDto {
                        name,
                        champion,
                        agent_type,
                        prompt,
                        model: if model.is_empty() { None } else { Some(model) },
                        config_json,
                        visibility: Some(visibility),
                    };
                    cloud.create_agent(&dto).await
                };

                this.update(&mut cx, |this, cx| {
                    match result {
                        Ok(agent) => {
                            // 新建成功后进入该 agent 的编辑态（enter_edit 会清空提示）
                            if matches!(this.heroes.mode, HeroesMode::Edit { editing_id: None }) {
                                enter_edit(this, cx, &agent);
                            }
                            this.heroes.success_msg = "已保存".to_string();
                        }
                        Err(e) => {
                            this.heroes.error_msg = format!("保存失败：{}", e);
                        }
                    }
                    cx.notify();
                })
                .ok();

                // 重新加载列表
                let cloud2 = match this.update(&mut cx, |this, _| this.cloud.clone()) {
                    Ok(c) => c,
                    Err(_) => return,
                };
                let agents = cloud2.list_agents().await.unwrap_or_default();
                let mut snaps = std::collections::HashMap::new();
                for a in &agents {
                    if let Ok(s) = cloud2.list_snapshots(&a.id.to_string()).await {
                        snaps.insert(a.id, s);
                    }
                }
                this.update(&mut cx, |this, cx| {
                    this.heroes.agents = agents;
                    this.heroes.snapshots = snaps;
                    cx.notify();
                })
                .ok();
            }
        },
    )
    .detach();
}

fn handle_publish(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) {
    let editing_id = match &sidebar.heroes.mode {
        HeroesMode::Edit {
            editing_id: Some(id),
        } => *id,
        _ => return,
    };
    sidebar.heroes.publishing = true;
    let cloud = sidebar.cloud.clone();
    let id_str = editing_id.to_string();
    cx.spawn(
        move |this: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
            let this = this.clone();
            let mut cx = cx.clone();
            async move {
                let result = cloud.publish_snapshot(&id_str).await;
                this.update(&mut cx, |this, cx| {
                    this.heroes.publishing = false;
                    match result {
                        Ok(snap) => {
                            this.heroes
                                .snapshots
                                .entry(editing_id)
                                .or_default()
                                .insert(0, snap.clone());
                            this.heroes.success_msg = format!("已发布 v{}", snap.version);
                        }
                        Err(e) => {
                            this.heroes.error_msg = format!("发布失败：{}", e);
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

fn handle_visibility_change(sidebar: &mut AppSidebar, v: Visibility, cx: &mut Context<AppSidebar>) {
    let editing_id = match &sidebar.heroes.mode {
        HeroesMode::Edit {
            editing_id: Some(id),
        } => *id,
        _ => return,
    };
    let cloud = sidebar.cloud.clone();
    let id_str = editing_id.to_string();
    cx.spawn(
        move |this: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
            let this = this.clone();
            let mut cx = cx.clone();
            async move {
                let result = cloud.update_agent_visibility(&id_str, v).await;
                this.update(&mut cx, |this, cx| {
                    match result {
                        Ok(()) => {
                            this.heroes.draft_visibility = v;
                            if let Some(a) =
                                this.heroes.agents.iter_mut().find(|a| a.id == editing_id)
                            {
                                a.visibility = v;
                            }
                            this.heroes.success_msg = "可见性已更新".to_string();
                        }
                        Err(e) => {
                            this.heroes.error_msg = format!("更新可见性失败：{}", e);
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

fn handle_pull_upstream(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) {
    let editing_id = match &sidebar.heroes.mode {
        HeroesMode::Edit {
            editing_id: Some(id),
        } => *id,
        _ => return,
    };
    let cloud = sidebar.cloud.clone();
    let id_str = editing_id.to_string();
    cx.spawn(
        move |this: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
            let this = this.clone();
            let mut cx = cx.clone();
            async move {
                let result = cloud.pull_upstream(&id_str).await;

                // 重新加载列表
                let cloud2 = match this.update(&mut cx, |this, _| this.cloud.clone()) {
                    Ok(c) => c,
                    Err(_) => return,
                };
                let agents = cloud2.list_agents().await.unwrap_or_default();
                let mut snaps = std::collections::HashMap::new();
                for a in &agents {
                    if let Ok(s) = cloud2.list_snapshots(&a.id.to_string()).await {
                        snaps.insert(a.id, s);
                    }
                }
                let agent = agents.iter().find(|a| a.id == editing_id).cloned();
                this.update(&mut cx, |this, cx| {
                    this.heroes.agents = agents;
                    this.heroes.snapshots = snaps;
                    // 用拉取后的最新数据刷新编辑态（enter_edit 会清空提示）
                    if let Some(a) = agent {
                        enter_edit(this, cx, &a);
                    }
                    match result {
                        Ok(_) => {
                            this.heroes.success_msg =
                                "已拉取上游策略，请重新发布快照使其在 Rank 生效".to_string();
                        }
                        Err(e) => {
                            this.heroes.error_msg = format!("拉取上游失败：{}", e);
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

fn handle_delete(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) {
    let editing_id = match &sidebar.heroes.mode {
        HeroesMode::Edit {
            editing_id: Some(id),
        } => *id,
        _ => return,
    };
    sidebar.heroes.deleting = true;
    let cloud = sidebar.cloud.clone();
    let id_str = editing_id.to_string();
    cx.spawn(
        move |this: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
            let this = this.clone();
            let mut cx = cx.clone();
            async move {
                let result = cloud.delete_agent(&id_str).await;
                this.update(&mut cx, |this, cx| {
                    this.heroes.deleting = false;
                    this.heroes.show_delete_confirm = false;
                    match result {
                        Ok(()) => {
                            this.heroes.mode = HeroesMode::Browse;
                            this.heroes.success_msg = "已删除".to_string();
                        }
                        Err(e) => {
                            this.heroes.error_msg = format!("删除失败：{}", e);
                        }
                    }
                    cx.notify();
                })
                .ok();

                // 重新加载列表
                let cloud2 = match this.update(&mut cx, |this, _| this.cloud.clone()) {
                    Ok(c) => c,
                    Err(_) => return,
                };
                let agents = cloud2.list_agents().await.unwrap_or_default();
                let mut snaps = std::collections::HashMap::new();
                for a in &agents {
                    if let Ok(s) = cloud2.list_snapshots(&a.id.to_string()).await {
                        snaps.insert(a.id, s);
                    }
                }
                this.update(&mut cx, |this, cx| {
                    this.heroes.agents = agents;
                    this.heroes.snapshots = snaps;
                    cx.notify();
                })
                .ok();
            }
        },
    )
    .detach();
}

fn handle_export_json(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) {
    let json = export_json(&sidebar.heroes);
    sidebar.heroes.draft_config_json_str = json.clone();
    cx.write_to_clipboard(ClipboardItem::new_string(json));
    sidebar.heroes.success_msg = "已导出当前配置到下方 JSON 框并复制到剪贴板".to_string();
    cx.notify();
}

fn handle_import_json(sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) {
    let s = sidebar.heroes.draft_config_json_str.clone();
    match apply_import_json(&mut sidebar.heroes, &s) {
        Ok(()) => {
            sidebar.heroes.success_msg = "已应用导入的 JSON 配置".to_string();
        }
        Err(e) => {
            sidebar.heroes.error_msg = format!("导入失败：{}", e);
        }
    }
    cx.notify();
}
